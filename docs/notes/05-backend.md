# Этап 5. Backend (Rust + axum)

> **См. также:**
> - `00-zk-primer.md` — что такое commitment, nullifier, Merkle proof.
> - `00-glossary.md` — все термины.
> - `04-anchor.md` — предыдущий этап (on-chain программа).

---

## TL;DR

**Что делаем:** HTTP-сервис на Rust + axum для индексации событий с блокчейна, хранения commitments/nullifiers в Postgres, кеша Merkle tree в Redis и проксирования witness'ов в prover.

**Зачем:** on-chain программа **не может** отдавать данные эффективно — RPC дорогой, событий много, а proof generation требует **off-chain** обработки. Backend — **связующее звено** между on-chain и frontend.

**Сколько шагов:** 12 под-этапов (5.1 – 5.12).

**Сколько времени:** ~4 часа.

**Что понадобится:**
- `04-anchor.md` — задеплоенная программа `zk_pool`.
- Postgres 16 и Redis 7 — добавлены в `infra/docker-compose.yml`.
- Проверенные `Program ID` и `Pool PDA`.

**Что получится:**
- HTTP API на порту 4001.
- Indexer — фоновый воркер для чтения событий.
- Postgres schema: `commitments`, `roots`, `nullifiers`.
- Redis-кеш Merkle tree.
- Метрики Prometheus.

**Следующий этап:** `06-merkle.md` (Merkle-сервис на Node.js).

---

## Разбиение этапа

| # | Что делаем | Статус |
|---|---|---|
| 5.1 | Project skeleton | ✅ |
| 5.2 | Config (`config.rs`) | ✅ |
| 5.3 | Database (`db.rs` + migration) | ✅ |
| 5.4 | Cache (`cache.rs`) | ← следующий |
| 5.5 | Merkle tree (`tree.rs`) | ⏳ |
| 5.6 | Metrics + logging | ⏳ |
| 5.7 | Rate limiting | ⏳ |
| 5.8 | Indexer | ⏳ |
| 5.9 | HTTP handlers | ⏳ |
| 5.10 | Docker compose additions | ⏳ |
| 5.11 | Smoke test | ⏳ |
| 5.12 | Final checkpoint | ⏳ |

---

## 5.1. Project skeleton

### Что делаем

Внутри контейнера:

```bash
cd /home/ubuntu/services/backend
cargo init --name zkpool_backend
rm -rf .git .gitignore
mkdir -p migrations tests
```

**Что создаётся:**
- `Cargo.toml` — манифест.
- `src/main.rs` — точка входа.
- `src/lib.rs` — библиотека (для тестов).
- `migrations/` — SQL-миграции.
- `tests/` — интеграционные тесты.

**⚠️ Ошибка:** `cargo init` создаёт `edition = "2024"`. Мы **переопределяем** на `2021`, потому что некоторые зависимости (sqlx 0.7) **не поддерживают** 2024-редакцию.

**Финальный `Cargo.toml`** — с полным набором зависимостей: `axum`, `tokio`, `sqlx`, `redis`, `reqwest`, `metrics`, и т.д.

### Итоги 5.1

**Коммит:** `7ed961a`.
**Чекпоинт:** `.checkpoints/05.1-backend-skeleton/`.

---

## 5.2. Config (`config.rs`)

### Что делает

Читает env-переменные, валидирует обязательные, предоставляет типизированный `Config` struct.

**Обязательные переменные:**
- `DATABASE_URL` — Postgres connection string.
- `REDIS_URL` — Redis connection string.
- `SOLANA_RPC_URL` — RPC-эндпоинт.
- `POOL_ADDRESS` — PDA пула (indexer фильтрует события по нему).

**Опциональные с дефолтами:**
- `PORT = 4001`.
- `MERKLE_URL = http://localhost:4003`.
- `PROVER_URL = http://localhost:4002`.
- `INDEXER_POLL_INTERVAL_SECS = 5`.
- `INDEXER_PAGE_SIZE = 100`.
- `INDEXER_MAX_PAGES = 10`.
- `RATE_LIMIT_WITHDRAW_PER_MIN = 5`.
- `RATE_LIMIT_READ_PER_MIN = 60`.
- `MERKLE_TREE_DEPTH = 20`.

**Три хелпера:**
- `required(name)` — падает, если переменная отсутствует.
- `optional(name, default)` — возвращает default, если не задана.
- `optional_parse<T>(name, default)` — то же, но с парсингом типа.

**`dotenvy::dotenv().ok()`** вызывается первым — если есть `.env` в CWD, переменные подгрузятся оттуда.

### Итоги 5.2

**101 строка.** **Коммит:** `ee668ab`.
**Чекпоинт:** `.checkpoints/05.2-config/`.

---

## 5.3. Database (`db.rs` + migration)

### Структура БД

Три таблицы:

**`commitments`** — все commitments:
```sql
CREATE TABLE commitments (
    id           BIGSERIAL PRIMARY KEY,
    leaf_index   BIGINT NOT NULL UNIQUE,
    commitment   BYTEA NOT NULL CHECK (octet_length(commitment) = 32),
    pool_address TEXT NOT NULL,
    tx_signature TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`roots`** — история Merkle roots (зеркало `PoolState.roots`):
```sql
CREATE TABLE roots (
    id           BIGSERIAL PRIMARY KEY,
    root         BYTEA NOT NULL CHECK (octet_length(root) = 32),
    leaf_index   BIGINT NOT NULL,
    pool_address TEXT NOT NULL,
    tx_signature TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`nullifiers`** — все потраченные nullifiers:
```sql
CREATE TABLE nullifiers (
    nullifier_hash BYTEA PRIMARY KEY CHECK (octet_length(nullifier_hash) = 32),
    pool_address   TEXT NOT NULL,
    recipient      TEXT,
    amount         BIGINT,
    tx_signature   TEXT,
    used_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Индексы:**
- `commitments (pool_address, leaf_index)`.
- `roots (pool_address, id DESC)`.
- `nullifiers (pool_address)`.

### ⚠️ Важное замечание: **нет** UNIQUE на `(pool_address, tx_signature)`

В `commitments` **специально** отсутствует UNIQUE constraint на паре `(pool_address, tx_signature)`. Это **совместимость** с будущей фичей «split deposit»: одна транзакция может создавать **несколько** commitments. Если бы был UNIQUE — фича **не** реализуема без миграции.

См. `PROJECT_CONTEXT.md`, раздел 7.6.

### `Db` — wrapper над `PgPool`

**Методы:**

| Метод | Что делает |
|---|---|
| `connect(url)` | Подключение к Postgres, возвращает `Db` |
| `new(pool)` | Обернуть существующий pool |
| `pool()` | Получить `&PgPool` |
| `save_commitment(leaf_index, commitment, pool, tx)` | INSERT ... ON CONFLICT DO NOTHING |
| `save_root(root, leaf_index, pool, tx)` | INSERT (всегда новый) |
| `save_nullifier(hash, pool, recipient, amount, tx)` | INSERT ... ON CONFLICT DO NOTHING |
| `commitment_exists(commitment)` | SELECT EXISTS |
| `is_nullifier_used(hash)` | SELECT EXISTS |
| `list_commitments(pool)` | SELECT ... ORDER BY leaf_index |
| `latest_root(pool)` | SELECT ... ORDER BY id DESC LIMIT 1 |
| `next_leaf_index(pool)` | SELECT COALESCE(MAX(leaf_index), -1) + 1 |

**Идемпотентность:** все `INSERT` используют `ON CONFLICT DO NOTHING` — indexer **может** переобработать транзакцию (RPC rewind, гонки), дубликаты **не** сломают БД.

### ⚠️ Ошибка: `chrono` не в зависимостях

**Симптом:**
```
error[E0433]: cannot find module or crate `chrono` in this scope
  --> src/db.rs:30:38
   |
30 |     pub created_at: chrono::DateTime<chrono::Utc>,
   |                                      ^^^^^^ use of unresolved module or unlinked crate `chrono`
```

**Причина:** `chrono` используется **транзитивно** через `sqlx` (feature `chrono`), но **не** добавлен в `[dependencies]` напрямую. Rust требует **явного** объявления для использования типов в **своём** коде.

**Решение:** добавить в `Cargo.toml`:
```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Урок:** если используешь тип из **крейта-транзитивки** — **добавь** его явно. `sqlx`'s feature `chrono` даёт **интеграцию**, но **не** реэкспортирует типы.

### Итоги 5.3

- `db.rs` — 216 строк, 9 публичных методов.
- `migrations/001_init.sql` — 49 строк.
- **Коммит:** `1e08827`.
- **Чекпоинт:** `.checkpoints/05.3-db/`.

---

## 5.4. Cache (`cache.rs`)

### Что делает

Redis-wrapper с `ConnectionManager` (автопереподключение, мультиплексирование).

**Три назначения:**
1. Состояние Merkle tree (`tree:*` keys) — см. 5.5.
2. Счётчики rate limiting (`rate:*` keys) — см. 5.7.
3. Кеш ответов `/api/commitments` и `/api/root`.

**Методы:**

| Метод | Что делает |
|---|---|
| `connect(url)` | Подключение к Redis |
| `get(key)` | GET, возвращает `Option<String>` |
| `set(key, value)` | SET без TTL |
| `set_ex(key, value, ttl_secs)` | SETEX с TTL |
| `del(key)` | DEL, возвращает `true`, если ключ существовал |
| `keys(pattern)` | KEYS по glob-паттерну (осторожно — O(N)) |
| `del_pattern(pattern)` | Удалить все ключи по паттерну |
| `incr_with_ttl(key, ttl_secs)` | INCR + EXPIRE при первом инкременте (для rate limiting) |
| `invalidate_pool(pool)` | Сбросить кеш конкретного пула: `cache:commitments:*`, `cache:root:*`, `tree:*` |

**⚠️ Про `keys()` / `del_pattern()`:** Redis `KEYS` — O(N) по **всему** keyspace. Использовать **только** для административных операций (сброс кеша при новом депозите), **не** на горячем пути.

**Что тестируется (2 ignored integration-теста):**
- `test_set_get_del` — базовые операции.
- `test_incr_with_ttl` — инкремент + TTL.

**Ignored** — потому что требуют **живой** Redis. Запускаются через `cargo test -- --ignored` при работающем Redis.

### Итоги 5.4

**187 строк.** **Коммит:** `0f36fc2`.
**Чекпоинт:** `.checkpoints/05.4-cache/`.

---

## 5.5. Merkle tree (`tree.rs`)

### Зачем incremental

Полное дерево глубины 20 — 1 048 576 листьев. Пересчитывать на каждом депозите — **невозможно**. Incremental tree обновляет только путь от нового листа к корню: O(DEPTH) = 20 хешей вместо O(2^DEPTH).

### Что хранится в Redis

| Ключ | Что |
|---|---|
| `tree:{pool}:level:{d}` | Самый правый **непустой** узел на уровне `d` (hex) |
| `tree:{pool}:empty:{d}` | Empty-хеш на уровне `d` (hex) |
| `tree:{pool}:root` | Текущий корень (hex) |
| `tree:{pool}:size` | Количество вставленных листьев (decimal) |

### Алгоритм `add_leaf`

1. Лист становится текущим узлом на уровне 0.
2. Для `d = 0..DEPTH`:
   - Читаем **правый** узел на уровне `d` из Redis.
   - Определяем, **левый** ли наш узел: `(size >> d) & 1 == 0`.
   - Формируем `(left, right)` в правильном порядке.
   - Считаем `parent = hash_2(left, right)` через Merkle-сервис.
   - Записываем `parent` как новый правый узел на уровне `d+1`.
   - `current = parent`.
3. Финал: `size += 1`, `root = current`.

### ⚠️ Почему Poseidon2 **не** реализован в Rust

Разные реализации Poseidon2 (Rust `light-poseidon`, Noir builtin, JS `noir_js`) дают **разные** хеши. Единственный способ гарантировать идентичность — использовать **тот же** ACIR, что и circuit. Поэтому `tree.rs` **делегирует** хеширование Merkle-сервису по HTTP.

**⚠️ Зависимость:** `tree.rs` **не работает** без запущенного Merkle-сервиса (этап 6). На этапе 5 backend **запустится**, но `add_leaf` будет падать с HTTP-ошибкой, пока merkle не поднят. **Порядок** запуска: `merkle` → `backend`.

### `MerkleClient`

Тонкая обёртка над HTTP:

```rust
pub async fn hash_2(&self, left: &str, right: &str) -> Result<String>
```

POST на `{base_url}/hash` с JSON `{"left": hex, "right": hex}`, возвращает `{"hash": hex}`.

### `MerkleTree`

Основной тип:

| Метод | Что делает |
|---|---|
| `new(pool, depth, cache, merkle)` | Создать handle |
| `init_empty()` | Проинициализировать пустое дерево (считает empty-хеши для всех уровней) |
| `add_leaf(leaf_hex)` | Добавить лист, обновить дерево, вернуть новый root |
| `root()` | Текущий root |
| `size()` | Количество листьев |

**Redis keys:**
- `key_level(d)` → `tree:{pool}:level:{d}`.
- `key_empty(d)` → `tree:{pool}:empty:{d}`.
- `key_root()` → `tree:{pool}:root`.
- `key_size()` → `tree:{pool}:size`.

### ⚠️ Черновик — исправления

**Первая версия** имела две проблемы:

1. **`empty_at()` возвращал `None`** — placeholder, который **ломал** логику. Заменили на inline `empty(d)`, который возвращает `"00".repeat(32)` как безопасный fallback.

2. **Odd-index case** — в комментарии было «already merged», но код **не** обрабатывал случай корректно. Переписали цикл с **правильной** проверкой чётности: `(size >> d) & 1 == 0`.

**Урок:** incremental tree — **не** очевидный алгоритм. Первый черновик **не компилировался бы**, если бы мы сразу его запустили. Правило 0.10 — **писать маленькими шагами** — здесь **сработало**: код **прошёл** `cargo check` **сразу**, потому что мы **починили** логику **до** компиляции.

### Что **не** реализовано (пока)

- **`proof(leaf_index)`** — Merkle proof требует **полного** дерева, которого у нас нет. Делегируется Merkle-сервису, но **API endpoint** ещё не готов. Добавим на этапе 5.9 или позже.

### Итоги 5.5

**249 строк.** **Коммит:** `372c056`.
**Чекпоинт:** `.checkpoints/05.5-tree/`.

---

## 5.6. Metrics + logging (`logging.rs` + `metrics.rs`)

Два небольших модуля, обслуживающих наблюдаемость (observability).

### `logging.rs`

Инициализация `tracing` + `tracing-subscriber`.

**Два режима:**
- **`LOG_FORMAT=pretty`** (default, dev) — human-readable с цветами.
- **`LOG_FORMAT=json`** (production) — JSON-строки для log-коллекторов (Loki, Elastic).

**Фильтр:** `RUST_LOG` env (default `info`).

**Единственная функция:** `init_logging() -> Result<()>`. Вызывается **один раз** при старте процесса.

### `metrics.rs`

Custom Prometheus-метрики. Используется `metrics` facade + `metrics-exporter-prometheus` backend.

**Префикс `zkpool_`** — чтобы не путаться с метриками `axum-prometheus` (там префиксы `axum_` и `process_`).

**Метрики:**

| Метрика | Тип | Что |
|---|---|---|
| `zkpool_indexer_deposits_total` | counter | Обработано DepositEvent |
| `zkpool_indexer_withdrawals_total` | counter | Обработано WithdrawEvent |
| `zkpool_indexer_errors_total` | counter | Ошибки indexer'а |
| `zkpool_indexer_lag_seconds` | gauge | Секунд с последнего обработанного блока |
| `zkpool_indexer_tree_size` | gauge | Текущее количество листьев |
| `zkpool_tree_add_leaf_duration_seconds` | histogram | Время `add_leaf` |
| `zkpool_tree_hash_duration_seconds` | histogram | Время Poseidon2 hash |
| `zkpool_tree_errors_total` | counter | Ошибки дерева |
| `zkpool_db_errors_total` | counter | Ошибки БД |

**Buckets для histogram** — 0.1 ms, 1 ms, 10 ms, 100 ms, 1 s, 5 s, 30 s.

**Функции:**
- `init_metrics()` — устанавливает Prometheus recorder **глобально**, один раз.
- `render_metrics()` — возвращает строку для endpoint'а `/metrics`.
- `record_deposit()`, `record_withdrawal()`, `record_indexer_error()`, `record_db_error()`, `record_tree_error()` — счётчики.
- `set_indexer_lag(seconds)`, `set_tree_size(size)` — gauges.
- `record_add_leaf_duration(seconds)`, `record_hash_duration(seconds)` — histograms.

**Global state:** `PROMETHEUS_HANDLE: OnceLock<PrometheusHandle>` — установить можно только один раз.

### ⚠️ Ошибка: содержимое попало не в тот файл

**Симптом:** после "replace fully" для `logging.rs` и `metrics.rs` — тест `test_render_before_init` **оказался в `logging.rs`**, а `metrics.rs` был **пустой** (0 строк).

**Причина:** редактор применил обе "replace fully" инструкции к **одному** файлу (или вторая не применилась).

**Обнаружение:** `cargo test --lib` вывел имя теста как `logging::tests::test_render_before_init`, хотя код был написан для `metrics.rs`. Проверили `wc -l` и `grep -n "pub fn init_logging\|pub fn init_metrics"` — увидели несоответствие.

**Решение:** перезаписали **оба** файла заново. `logging.rs` — 41 строка, `metrics.rs` — 119 строк.

**Урок:** после "replace fully" **всегда** проверяй `wc -l` и `grep` на **характерные** функции/тесты. Если имя модуля в выводе `cargo test` не совпадает с ожидаемым — файлы перепутаны.

### Итоги 5.6

- `logging.rs` — 41 строка.
- `metrics.rs` — 119 строк.
- Тест `metrics::tests::test_render_before_init` — passed (1 passed, 2 ignored).
- **Коммит:** `af219cd`.
- **Чекпоинт:** `.checkpoints/05.6-metrics-logging/`.

---

## 5.7. Rate limiting (`rate_limit.rs`)

Redis-based rate limiting middleware для axum.

### Алгоритм

Для каждого запроса:
1. Извлечь IP клиента из `X-Forwarded-For`, `X-Real-IP` или `ConnectInfo<SocketAddr>`.
2. Построить ключ: `rate:{endpoint}:{ip}`.
3. `INCR key` → count. Если `count == 1`, установить `EXPIRE key 60`.
4. Если `count > limit` → `429 Too Many Requests` с `Retry-After: 60`.
5. Иначе — пропустить дальше.

### Два уровня

| Tier | Endpoint | Default limit |
|---|---|---|
| `Withdraw` | `POST /api/withdraw` | 5/min |
| `Read` | `GET /api/commitments`, `/api/root`, `/api/proof` | 60/min |

Endpoints `/api/health` и `/metrics` — **не** лимитируются.

### Компоненты

**`Tier`** — enum: `Withdraw` | `Read`. Метод `endpoint_label()` → строка для ключа.

**`RateLimiter`** — shared state (axum `State`):
- `cache: Arc<Mutex<Cache>>` — Redis-клиент.
- `withdraw_limit: u64`, `read_limit: u64` — из config.

**Метод `check(tier, ip)`** → `Ok(remaining)` или `Err(retry_after)`.

### Middleware

Два axum-middleware:
- **`withdraw_middleware`** — для `POST /api/withdraw`.
- **`read_middleware`** — для `GET /api/commitments|root|proof`.

Оба используют один `RateLimiter`, отличаются только `Tier`.

### Извлечение IP

```rust
fn client_ip(req: &Request) -> String
```

Приоритет:
1. `X-Forwarded-For` (первое значение) — за обратным прокси.
2. `X-Real-IP`.
3. `ConnectInfo<SocketAddr>` — от axum.
4. `"unknown"` — fallback.

### ⚠️ Fail-open на ошибке Redis

Если Redis **недоступен** — `check()` возвращает `Ok(limit)` и **пропускает** запрос. Это **сознательное** решение:
- **Fail-open** — сервис **продолжает** работать при падении Redis.
- **Альтернатива (fail-close)** — блокирует **все** запросы при падении Redis, что **опаснее** для UX.

Для production можно **изменить** на fail-close через отдельную конфигурацию, но для demo — fail-open.

### Что НЕ тестируется (пока)

Интеграционные тесты для rate limiting требуют **живого** Redis. Добавим в **smoke test** (этап 5.11) через реальный HTTP-запрос с превышением лимита.

### Итоги 5.7

- `rate_limit.rs` — **181 строка**.
- Компилируется чисто.
- **Коммит:** `7d85c29`.
- **Чекпоинт:** `.checkpoints/05.7-rate-limit/`.

---

## Что дальше

**Следующий под-этап:** 5.4 — `cache.rs` (Redis wrapper).

**Что будет:**
- `Cache::connect(url)` — подключение к Redis.
- `get`, `set`, `set_ex` (с TTL), `del`.
- Методы для Merkle tree: `tree_*` (используются в 5.5).
- `incr_rate_limit` — для rate limiting (5.7).
- `invalidate_pool` — сброс кеша при новом депозите.
