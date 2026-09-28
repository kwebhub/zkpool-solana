# 11. Infrastructure

> **Этап 11** проекта zkpool-solana.
> Makefile, port mappings, Prometheus + Grafana.

---

## TL;DR

Всё работает, но запуск требует ручных команд. Этап 11 — сделать запуск одной командой (`make up`), открыть порты наружу, добавить мониторинг.

**Компоненты:**
- `Makefile` в корне — `up`/`down`/`reset`/`status`/`logs`/`web`/`build`/`clean`.
- Port mappings в `docker-compose.yml` — 4001–4003, 5173 доступны с хоста.
- Prometheus + Grafana — сбор метрик backend'а.

---

## 11.1. Makefile

**Дата:** 2026-09-28
**Commit:** `1b1f80d`

### Зачем

Запуск проекта требовал 3–4 ручных команд:

```bash
docker compose -f infra/docker-compose.yml up -d
docker compose exec solana bash -ic 'cd services/merkle && nohup node src/server.js > /tmp/merkle.log 2>&1 &'
docker compose exec solana bash -ic 'cd services/prover && nohup ./target/release/zkpool-prover > /tmp/prover.log 2>&1 &'
docker compose exec solana bash -ic 'cd services/backend && nohup ./target/release/zkpool-backend > /tmp/backend.log 2>&1 &'
```

Плюс health checks, restart после рестарта контейнера, очистка БД/Redis.

**Makefile — одно место для всех операций.**

### Target'ы

| Target | Что делает |
|---|---|
| `make up` | docker-compose up + merkle + prover + backend + health check |
| `make down` | stop all services + docker-compose down |
| `make reset` | down → up infra → clear DB + Redis → start services |
| `make status` | containers + процессы + HTTP health каждого сервиса |
| `make logs` | tail -20 всех логов (merkle, prover, backend) |
| `make web` | запуск Vite dev server на 5173 |
| `make build` | backend + prover + web |
| `make clean` | удалить target/, node_modules/, dist/ |
| `make help` | список всех target'ов |

### Ключевые технические детали

**`SHELL := /bin/bash`** — Makefile по умолчанию использует `/bin/sh`, где нет `source ~/.bashrc`.

**`exec_c := docker compose ... exec solana bash -ic`** — `-ic` обязателен. `bash -c` не читает `.bashrc`, и PATH для `cargo`/`node`/`pnpm` не настроен.

**`exec_d := docker compose ... exec -d solana bash -ic`** — `-d` для detach. Используется для старта сервисов в фоне.

**Backgrounded processes:**
```makefile
$(COMPOSE) exec -d $(CONTAINER) bash -ic 'pkill -f zkpool-prover || true; cd ... && nohup ./target/release/zkpool-prover > /tmp/prover.log 2>&1 &'
```

`pkill || true` — идемпотентно. Если уже что-то запущено — убиваем, потом стартуем заново.

### `make reset` — правильный порядок

**Первый вариант (сломанный):**
```makefile
reset: down
	docker exec zkpool-postgres psql ...  # ошибка — контейнер удалён
	docker exec zkpool-redis ...          # ошибка
	$(MAKE) up
```

**Фикс:** сначала `up infra`, потом `clear`, потом `start services`:

```makefile
reset: down
	$(COMPOSE) up -d
	sleep 3
	docker exec zkpool-postgres psql ...
	docker exec zkpool-redis ...
	$(MAKE) merkle prover backend wait-healthy
```

### Грабли

1. **`docker compose down` падает, если контейнеры уже остановлены.** Фикс: `2>/dev/null || true`.
2. **`docker exec` на несуществующем контейнере** — ошибка `No such container`. Поэтому нужен порядок "up → clear".
3. **`pkill` не убивает `bash -ic` обёртки.** Видно в `make status` — процессы `bash -ic pkill ...` остаются. Не мешает, но лишние строки в выводе.

### Уроки

1. **Makefile — это API проекта.** `make up` вместо четырёх строк — понятнее и воспроизводимее.
2. **Идемпотентность важнее элегантности.** `pkill || true` и `2>/dev/null || true` — некрасиво, но каждая команда безопасна для повторного запуска.
3. **`-ic` — обязательный флаг.** `bash -c` не читает `.bashrc`, а без него нет PATH для cargo/node/pnpm. Проверено в Stage 5.
4. **`make reset` = воспроизводимое "чистое состояние".** Полезно в CI и при отладке.

---

## 11.2. Port mappings

**Дата:** 2026-09-28
**Commit:** `6d2eb42`

### Зачем

До этого этапа сервисы были доступны **только изнутри** `solana` контейнера. Чтобы открыть `http://localhost:4001` в браузере или обратиться к `curl` с хоста — нужен port mapping.

### Изменения

**`infra/docker-compose.yml`**, сервис `solana`:

```yaml
ports:
  - "4001:4001"   # backend
  - "4002:4002"   # prover
  - "4003:4003"   # merkle
  - "5173:5173"   # vite dev server
```

### Перезапуск

```bash
docker compose -f infra/docker-compose.yml up -d --force-recreate solana
```

**Важно:** `--force-recreate` обязателен. Без него compose видит "контейнер запущен" и не применяет изменения в `ports`.

**Следствие:** все процессы внутри контейнера убиваются. Их надо перезапустить через `make up`.

### Проверка с хоста

```bash
$ docker compose -f infra/docker-compose.yml ps
solana-zkpool-solana  ...  0.0.0.0:4001-4003->4001-4003/tcp, 0.0.0.0:5173->5173/tcp

$ curl http://localhost:4001/api/health
{"db":true,"status":"ok","version":"0.1.0"}

$ curl http://localhost:4002/health
{"status":"ok"}

$ curl http://localhost:4003/health
{"status":"ok"}

$ curl http://localhost:5173/
HTTP 200
```

### Обновлён `make web`

**Было:**
```makefile
web:
	@$(COMPOSE) exec -d $(CONTAINER) bash -ic '...'
	@echo "   (not yet reachable from host — port mapping in Stage 11.2)"
```

**Стало:**
```makefile
web:
	@$(COMPOSE) exec -d $(CONTAINER) bash -ic 'pkill -f vite || true; cd /home/ubuntu/web && nohup pnpm dev > /tmp/web.log 2>&1 &'
	@sleep 3
	@curl -sf -m 5 http://localhost:5173/ >/dev/null && echo "   ✓ http://localhost:5173" || echo "   ✗ vite not reachable"
```

**Плюс:** `make web` теперь сам проверяет, что Vite поднялся.

### Забыли сохранить Makefile

**Симптом:** `git commit -m "port mappings"` ушёл с **старой** версией Makefile. `make web` всё ещё писал "(not yet reachable from host)".

**Причина:** отредактировали `Makefile`, но не сохранили в редакторе до `git add -A`.

**Фикс:** отдельный коммит `6d2eb42` с одной строкой (`1 file changed, 1 insertion(+), 1 deletion(-)`).

**Урок:** после редактирования файла — сохранить. Можно визуально проверить через `git diff` перед `git add`.

### Грабли

1. **`--force-recreate` обязателен для применения `ports`.** Без него compose не пересоздаёт контейнер.
2. **Рестарт контейнера убивает все процессы.** `make up` их восстанавливает.
3. **Порты не конфликтуют с Postgres (5432) / Redis (6379).** Разные диапазоны.

### Уроки

1. **Port mapping — последний шаг к полноценному dev-опыту.** До этого всё тестирование было через `docker compose exec`.
2. **`make web` сам себя проверяет.** `curl` после запуска — быстрый сигнал о проблеме.
3. **Редактор → сохранить → `git diff` → `git add`.** Порядок, который предотвращает "забыл сохранить".
4. **`--force-recreate` не пересоздаёт volumes.** Postgres/Redis данные остаются.

---

## Что дальше

- **11.2** — port mappings в `docker-compose.yml` (4001–4003, 5173).
- **11.3** — Prometheus + Grafana в compose, scrape config.
- **11.4** — Grafana dashboard для backend metrics.
- **11.5** — финальный чекпоинт.
