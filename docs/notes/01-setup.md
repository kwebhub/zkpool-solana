# Этап 1. Репозиторий и Docker-окружение

> **См. также:**
> - `00-zk-primer.md` — если не знаешь, что такое ZK-proof и circuit.
> - `00-glossary.md` — если встречаешь незнакомый термин.

---

## TL;DR

**Что делаем:** создаём монорепо `zkpool-solana` и Docker-контейнер со **всеми** инструментами: Rust, Solana CLI, Anchor, Nargo (Noir), Sunspot, Node.js, pnpm.

**Зачем:** чтобы **все** компоненты проекта (circuit, on-chain программа, backend, frontend) разрабатывались в **одном** окружении с **фиксированными** версиями. Это устраняет класс проблем «у меня работает, у тебя нет».

**Сколько шагов:** 7.

**Сколько времени:** ~30 минут (из них 20 — сборка Docker-образа).

**Что понадобится:**
- Docker Desktop или Docker Engine.
- Git.
- Пустой приватный репозиторий на GitHub.

**Что получится:**
- Рабочий каталог `~/Projects/Solana/zkpool-solana/`.
- Docker-контейнер `solana-zkpool-solana` со всеми инструментами.
- Проверенные версии всех инструментов (записаны в чекпоинт).

**Следующий этап:** `02-circuits.md`.

---

## 1. Почему monorepo

Проект состоит из **пяти** крупных частей:

1. **On-chain программа** (`zk_pool`) на Rust + Anchor.
2. **ZK-схемы** на Noir.
3. **Backend** на Rust + axum.
4. **Merkle-сервис** на Node.js + Fastify.
5. **Frontend** на Vue 3.

Можно было бы сделать **multi-repo** — по одному репозиторию на каждую часть. Мы выбрали **монорепо** по причинам:

| Причина | Объяснение |
|---|---|
| **Единый `spec.json`** | Публичные входы circuit'а описываются **один раз**. Валидатор проверяет, что все слои ему соответствуют. Multi-repo потребовал бы **дублирования** или **синхронизации между репозиториями**. |
| **Единый `sync-circuits`** | ACIR circuit'а копируется в **несколько** потребителей (merkle, frontend). Multi-repo потребовал бы **публикации** артефактов. |
| **Атомарные коммиты** | Изменение публичных входов circuit'а **одновременно** меняет circuit, Anchor-программу и frontend. В монорепо — **один** коммит. В multi-repo — **три**, с окном рассинхрона. |
| **Единый `make`** | Все команды — из корня. Не нужно прыгать по репозиториям. |
| **Единый CI** | Один workflow, одна матрица, один `ci-success`. |

**Плата за это:** размер репозитория (десятки МБ) и требование дисциплины (не коммитить `target/`).

**Итог:** монорепо **сильно** упрощает v3-цель — **исключить** рассинхрон между слоями.

---

## 2. Почему один Docker-контейнер

Проекту нужны **девять** инструментов:

| Инструмент | Версия | Зачем |
|---|---|---|
| Rust | 1.98.1 | On-chain, backend, prover, CLI |
| Cargo | 1.98.1 | Сборка Rust |
| Solana CLI | 3.1.10 | Работа с блокчейном, деплой |
| Anchor CLI | 1.1.2 | Фреймворк для Solana |
| Nargo | 1.0.0-rc.2 | Компиляция Noir |
| Sunspot | 1.0.0 | Groth16 verifier для Solana |
| Node.js | 24.21.0 | Merkle-сервис, frontend |
| pnpm | 12.5.1 | Менеджер пакетов Node |
| `git` | системный | Клонирование Sunspot |

Можно было бы:
- **A.** Установить всё на хост.
- **B.** Использовать **разные** контейнеры для разных сервисов.
- **C.** Один контейнер со **всем**.

**Почему C:**

| Причина | Объяснение |
|---|---|
| **Изоляция от хоста** | Версии инструментов **зафиксированы** в образе. Хост может иметь другие версии Rust/Node. |
| **Один `docker-compose up`** | Не нужно устанавливать 9 инструментов вручную. |
| **Воспроизводимость** | `Dockerfile.solana` — **точная** спецификация окружения. |
| **Упрощение CI** | CI использует **тот же** образ. Локально и в CI — идентичное окружение. |
| **Sunspot требует клон** | Sunspot-клон (`gnark-solana`) — сотни МБ. В контейнере он **изолирован** от хоста. |

**Плата:** образ собирается ~20 минут, весит ~3 ГБ. **Единожды** — потом только `docker run`.

**Итог:** один контейнер = воспроизводимость + отсутствие «у меня работает».

---

## 3. Почему именно эти версии

**Критично:** версии подобраны **не случайно**. Между ними **проверенная** совместимость.

| Инструмент | Версия | Почему именно эта |
|---|---|---|
| Rust | 1.98.1 | Anchor 1.1.2 требует Rust ≥ 1.89. Sunspot компилируется под 1.89-SBF. 1.98.1 — последняя стабильная. |
| Solana CLI | 3.1.10 | Совместима с Anchor 1.1.2. |
| Anchor CLI | 1.1.2 | Совместим с nargo 1.0.0-rc.2 (через `anchor build`). |
| Nargo | 1.0.0-rc.2 | Sunspot 1.0.0 **поддерживает только** эту версию Noir. |
| Sunspot | 1.0.0 | Последняя стабильная. |
| Node.js | 24.21.0 | Требуется `noir_js` 1.0.0-rc.2 (совместимость с nargo). |
| pnpm | 12.5.1 | Быстрее `npm`/`yarn`. Стабильная. |

**Ключевая связка:** `nargo 1.0.0-rc.2` ↔ `sunspot 1.0.0` ↔ `noir_js 1.0.0-rc.2`. Если **любая** из трёх версий отличается — circuit **не скомпилируется** или **не сгенерирует** proof.

**Правило проекта:** **не обновлять** версии без **полного** пересмотра совместимости. Любое обновление — новый этап с чекпоинтом.

---

## 4. Создание репозитория

### Зачем

Нам нужен **пустой** приватный репозиторий на GitHub — **без** README, `.gitignore`, лицензии. Всё создаём **вручную**, чтобы контролировать **каждый** файл.

### Шаги

1. Открыть https://github.com/new
2. **Repository name:** `zkpool-solana`
3. **Visibility:** Private
4. **НЕ** ставить галочки «Add a README file», «Add .gitignore», «Choose a license».
5. **Create repository**.

### Ожидаемый результат

Пустой репозиторий. GitHub показывает **инструкцию** для push, но мы сначала **клонируем** пустой репозиторий.

---

## 5. Клонирование и структура папок

### Зачем

Создаём **каркас** монорепозитория — папки для пяти частей проекта + инфраструктуры + документации.

### Шаги

```bash
cd ~/Projects/Solana
git clone git@github.com:kwebhub/zkpool-solana.git
cd zkpool-solana

mkdir -p onchain circuits services/{backend,merkle,prover} web
mkdir -p infra/docker infra/grafana/provisioning/{datasources,dashboards}
mkdir -p scripts/sync-circuits
mkdir -p docs/notes
mkdir -p .secrets
mkdir -p .checkpoints
```

### Ожидаемый результат

```
zkpool-solana/
├── onchain/          ← Anchor-программа
├── circuits/         ← Noir-схемы
├── services/
│   ├── backend/      ← Rust + axum
│   ├── merkle/       ← Node.js + Fastify
│   └── prover/       ← Rust + Sunspot
├── web/              ← Vue 3
├── infra/
│   ├── docker/       ← Dockerfile
│   └── grafana/      ← provisioning
├── scripts/          ← Rust CLI
├── docs/notes/       ← заметки
├── .secrets/         ← keypair-бэкапы (gitignored)
└── .checkpoints/     ← артефакты этапов (gitignored)
```

**Важно:** папки `.secrets/` и `.checkpoints/` — **gitignored** (см. ниже).

---

## 6. `.gitignore`

### Зачем

Мы **не коммитим**:
- `target/` — скомпилированные артефакты Rust (десятки МБ).
- `node_modules/` — зависимости Node (сотни МБ).
- `.env` — локальные переменные окружения.
- `solana/` — кошелёк (приватные ключи!).
- `.secrets/` — бэкапы keypair.
- `.checkpoints/` — артефакты этапов.
- `Prover.toml` — witness (содержит секреты!).
- `services/merkle/circuits/`, `web/public/circuits/` — копии ACIR (синхронизируются через `sync-circuits`).

**НЕ игнорируем** (важно):
- `docs/notes/` — заметки **часть** портфолио.
- `.env.example` — шаблон без секретов.

### Содержимое

Открой `.gitignore` и **замени полностью** на:

```gitignore
# ============================================================
# Rust
# ============================================================
target/
**/target/
Cargo.lock.bak

# ============================================================
# Node.js
# ============================================================
node_modules/
**/node_modules/
.pnpm-store/
*.log
npm-debug.log*

# ============================================================
# Vue / Vite
# ============================================================
dist/
.vite/
*.local

# ============================================================
# Anchor
# ============================================================
.anchor/
test-ledger/
**/test-ledger/

# ============================================================
# Env
# ============================================================
.env
**/.env
.env.local
.env.*.local
!.env.example

# ============================================================
# Solana (wallet, CLI config)
# ============================================================
solana/
**/solana/

# ============================================================
# IDE
# ============================================================
.idea/
.vscode/*
!.vscode/extensions.json

# ============================================================
# OS
# ============================================================
.DS_Store
Thumbs.db

# ============================================================
# Tests
# ============================================================
coverage/
.nyc_output/
playwright-report/
test-results/

# ============================================================
# Monitoring
# ============================================================
prometheus-data/
grafana-data/

# ============================================================
# Noir
# ============================================================
Prover.toml
**/Prover.toml

# ============================================================
# Secrets
# ============================================================
.secrets/

# ============================================================
# Checkpoints
# ============================================================
.checkpoints/

# ============================================================
# Circuit JSON consumers (generated by scripts/sync-circuits)
# ============================================================
services/merkle/circuits/
web/public/circuits/
```

### Ожидаемый результат

Файл `.gitignore` создан. `git status` **не** показывает `.secrets/`, `.checkpoints/`, `target/`, `node_modules/`.

### ⚠️ Ошибка: `solana/cli/config.yml` попал в git

**Симптом:** `git status` показывает `new file: solana/cli/config.yml`.

**Причина:** сначала в `.gitignore` было `solana/*.json` — не **вся** папка.

**Решение:** `solana/` целиком (см. выше). Плюс `git rm --cached solana/cli/config.yml`.

**Урок:** если папка содержит **и** приватное, **и** конфиги — игнорируй **всю** папку.

---

## 7. Docker-окружение

### Зачем

Создаём **один** контейнер `solana-zkpool-solana` со **всеми** инструментами. Один раз собрали — используем **всю** разработку.

### `infra/docker-compose.yml`

Открой файл и **замени полностью** на:

```yaml
services:
  solana:
    build:
      context: ..
      dockerfile: infra/docker/Dockerfile.solana
    container_name: solana-zkpool-solana
    stdin_open: true
    tty: true
    user: "1000:1000"
    working_dir: /home/ubuntu
    volumes:
      - ../onchain:/home/ubuntu/onchain
      - ../circuits:/home/ubuntu/circuits
      - ../services:/home/ubuntu/services
      - ../web:/home/ubuntu/web
      - ../scripts:/home/ubuntu/scripts
      - ../solana:/home/ubuntu/.config/solana
    environment:
      - CARGO_TERM_COLOR=always
    restart: unless-stopped
```

### Разбор

| Поле | Что делает | Почему |
|---|---|---|
| `container_name: solana-zkpool-solana` | Имя контейнера | Чтобы не конфликтовать со старым проектом |
| `user: "1000:1000"` | Пользователь внутри — UID/GID 1000 | Файлы в volumes принадлежат **локальному** пользователю |
| `working_dir: /home/ubuntu` | Рабочая директория | Все пути — относительно неё |
| `volumes` — 6 штук | Монтирование папок проекта | Изменения **на хосте** сразу видны в контейнере (и наоборот) |
| `restart: unless-stopped` | Авто-перезапуск | Если контейнер упадёт — Docker поднимет |

### ⚠️ Важно: `user: "1000:1000"`

**Проблема:** если контейнер **создаёт** папку `solana/` от root, она станет **root-owned** на хосте. Пользователь **не сможет** писать в неё.

**Симптом:** `Permission denied` при создании кошелька Solana.

**Решение:** `user: "1000:1000"` + **создать** папки от **своего** пользователя **до** первого запуска контейнера.

Если уже случилось:

```bash
sudo chown -R 1000:1000 solana/
```

**Урок:** volumes должны быть созданы **от пользователя** или приведены к UID 1000.

### `infra/docker/Dockerfile.solana`

Открой файл и **замени полностью** на:

```dockerfile
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    TZ=UTC

USER root

RUN apt-get update && \
    apt-get install -y curl git build-essential pkg-config libssl-dev \
        ca-certificates --no-install-recommends && \
    apt-get clean && rm -rf /var/lib/apt/lists/* && \
    curl -L -o /tmp/sunspot.deb \
      "https://github.com/reilabs/sunspot/releases/download/v1.0.0/sunspot_1.0.0_linux_amd64.deb" && \
    apt-get -y install /tmp/sunspot.deb && \
    rm /tmp/sunspot.deb

USER ubuntu
WORKDIR /home/ubuntu

ENV PATH="/home/ubuntu/.local/share/solana/install/active_release/bin:/home/ubuntu/.cargo/bin:/home/ubuntu/.avm/bin:/home/ubuntu/.noirup/bin:/home/ubuntu/.nargo/bin:/home/ubuntu/.local/share/pnpm:${PATH}"
ENV NVM_DIR="/home/ubuntu/.nvm"

RUN echo 'export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"' >> /home/ubuntu/.bashrc && \
    curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash && \
    curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash && \
    noirup --version v1.0.0-rc.2 && \
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.7/install.sh | METHOD=script bash && \
    bash -c "source $NVM_DIR/nvm.sh && nvm install 24 && nvm alias default 24" && \
    curl -fsSL https://get.pnpm.io/install.sh | env PNPM_VERSION=12.5.1 ENV="$HOME/.bashrc" SHELL="$(which bash)" bash - && \
    echo 'export NVM_DIR="$HOME/.nvm"' >> $HOME/.bashrc && \
    echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"' >> $HOME/.bashrc && \
    echo '[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"' >> $HOME/.bashrc && \
    git clone https://github.com/reilabs/sunspot.git /home/ubuntu/sunspot

CMD ["/bin/bash"]
```

### Разбор Dockerfile

**1. `FROM ubuntu:24.04`.** Базовый образ. Ubuntu 24.04 — свежий LTS.

**2. Установка системных пакетов + Sunspot `.deb`.**

```dockerfile
apt-get install -y curl git build-essential pkg-config libssl-dev ca-certificates
```

Нужны для сборки Rust и клонирования.

```dockerfile
curl -L -o /tmp/sunspot.deb "https://github.com/reilabs/sunspot/releases/download/v1.0.0/sunspot_1.0.0_linux_amd64.deb"
apt-get -y install /tmp/sunspot.deb
```

**Sunspot** устанавливаем из `.deb`. **НО:** `.deb` содержит **только** CLI-бинарь. Для `sunspot deploy` нужен **полный** клон репозитория (см. шаг 5 в разделе «Что улучшено»).

**3. `USER ubuntu` + `WORKDIR /home/ubuntu`.**

Дальнейшие команды выполняются от **не-root** пользователя. Это **важно** для прав на файлы.

**4. Установка Solana CLI, Noir, Node.js, pnpm.**

```dockerfile
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```
Официальный установщик Solana CLI.

```dockerfile
curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
noirup --version v1.0.0-rc.2
```
**noirup** — менеджер версий Noir. Ставим **строго** 1.0.0-rc.2.

```dockerfile
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.7/install.sh | METHOD=script bash
bash -c "source $NVM_DIR/nvm.sh && nvm install 24 && nvm alias default 24"
```
**nvm** — менеджер версий Node. Ставим Node 24.

```dockerfile
curl -fsSL https://get.pnpm.io/install.sh | env PNPM_VERSION=12.5.1 ...
```
**pnpm** — устанавливаем **строго** 12.5.1.

**5. `git clone sunspot` — критично.**

```dockerfile
git clone https://github.com/reilabs/sunspot.git /home/ubuntu/sunspot
```

**Зачем:** `sunspot deploy` компилирует verifier program из **Rust-крейта** `gnark-solana/crates/verifier-bin`, который **не входит** в `.deb`. Нужен **полный клон**.

**Что было в v2:** клон делался **вручную** после первого падения `sunspot deploy`. При пересоздании контейнера клон **терялся**, и деплой **снова** падал. В v3 — **сразу в Dockerfile**.

**6. `GNARK_VERIFIER_BIN`.**

```dockerfile
echo 'export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"' >> /home/ubuntu/.bashrc
```

Переменная окружения, которую **ожидает** `sunspot deploy`. Указывает на крейт verifier'а.

### Сборка образа

```bash
cd infra
docker compose build solana
```

**⏱ Займёт ~20 минут.** Большая часть — установка Solana CLI, Node.js, Rust-toolchain, компиляция SBF.

### Запуск контейнера

```bash
docker compose up -d solana
```

### Ожидаемый результат

```
[+] Running 1/1
 ✔ Container solana-zkpool-solana  Started
```

---

## 8. Проверка версий

### Зачем

**Критично:** убедиться, что **все** версии совпадают с ожидаемыми. Если хоть одна отличается — дальше идти **нельзя**.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "solana=$(solana --version)"
  echo "anchor=$(anchor --version)"
  echo "nargo=$(nargo --version | head -1)"
  echo "sunspot=1.0.0"
  echo "node=$(node --version)"
  echo "pnpm=$(pnpm --version)"
'
```

### Ожидаемый результат

```
rustc=rustc 1.98.1 (48a229cea 2026-09-01)
cargo=cargo 1.98.1 (797e8a9bc 2026-08-05)
solana=solana-cli 3.1.10 (src:7bc9c805; feat:1620780344, client:Agave)
anchor=anchor-cli 1.1.2
nargo=nargo version = 1.0.0-rc.2
sunspot=1.0.0
node=v24.21.0
pnpm=12.5.1
```

### ⚠️ Почему `bash -ic`, а не `bash -c`

**Проблема:** команды внутри контейнера **не находят** `node`, `pnpm`, `cargo` при использовании `bash -c`.

**Причина:** `nvm`, `cargo`, `pnpm` добавляют себя в `PATH` через **`.bashrc`**. А `.bashrc` **не читается** неинтерактивным shell'ом.

**Решение:** флаг `-i` (interactive) заставляет bash **читать** `.bashrc`.

**Правило проекта:** **всегда** использовать `bash -ic` для команд внутри контейнера.

### Проверка Sunspot-клона

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  ls -la ~/sunspot/gnark-solana/crates/verifier-bin
'
```

### Ожидаемый результат

```
total 20
drwxr-xr-x 3 ubuntu ubuntu 4096 Sep 22 16:00 .
drwxr-xr-x 4 ubuntu ubuntu 4096 Sep 22 16:00 ..
-rw-r--r-- 1 ubuntu ubuntu  577 Sep 22 16:00 Cargo.toml
-rw-r--r-- 1 ubuntu ubuntu  565 Sep 22 16:00 build.rs
drwxr-xr-x 2 ubuntu ubuntu 4096 Sep 22 16:00 src
```

Если папка **отсутствует** — `git clone` в Dockerfile **не сработал**. Нужно пересобрать образ.

---

## 9. Чекпоинт этапа

### Зачем

Зафиксировать **фактический** результат этапа: версии инструментов + коммит. При **следующих** этапах сверимся с ним.

### Команды

```bash
mkdir -p .checkpoints/01-setup

docker compose -f infra/docker-compose.yml exec solana bash -ic '
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "solana=$(solana --version)"
  echo "anchor=$(anchor --version)"
  echo "nargo=$(nargo --version | head -1)"
  echo "sunspot=1.0.0"
  echo "node=$(node --version)"
  echo "pnpm=$(pnpm --version)"
' > .checkpoints/01-setup/versions.txt

docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'ls -la ~/sunspot/gnark-solana/crates/verifier-bin' \
  > .checkpoints/01-setup/sunspot-clone.txt

git rev-parse HEAD > .checkpoints/01-setup/commit.txt
```

### Ожидаемый результат

Три файла в `.checkpoints/01-setup/`:
- `versions.txt` — 8 строк версий.
- `sunspot-clone.txt` — листинг `verifier-bin`.
- `commit.txt` — SHA коммита.

---

## 10. Первый коммит

### Команды

```bash
git add -A
git status
```

**Проверить:** в staging только `.gitignore`, `.env.example`, `infra/docker-compose.yml`, `infra/docker/Dockerfile.solana`.

**НЕ** должно быть: `.checkpoints/`, `.secrets/`, `solana/`.

```bash
git commit -m "chore: initial project skeleton with Docker environment"
git push -u origin main
```

### Ожидаемый результат

Коммит запушен в `origin/main`.

---

## 11. Что улучшено относительно v2

| Что | v2 | v3 | Почему |
|---|---|---|---|
| Sunspot clone | вручную после падения `sunspot deploy` | **в Dockerfile** | Клон персистентен, не теряется при пересоздании |
| Имя контейнера | `solana` | `solana-zkpool-solana` | Не конфликтует с v2 |
| `network_mode` | `host` | явные порты | Портативнее |
| Volume `scripts/` | добавлен в этапе 9 | сразу | Все папки монтируются с самого начала |
| Checkpoints | нет | `.checkpoints/NN-name/` | Воспроизводимость |
| `docs/notes/` | gitignored | **коммитится** | Портфолио-материал |
| `solana/` в `.gitignore` | `solana/*.json` | `solana/` целиком | Не пропустить `cli/config.yml` |

---

## 12. Частые ошибки

### `Permission denied` при создании кошелька

**Симптом:**

```
Error: Unable to write /home/ubuntu/.config/solana/id.json: Permission denied (os error 13)
```

**Причина:** папка `solana/` на хосте **root-owned**. Пользователь в контейнере (`uid=1000`) **не может** писать.

**Диагностика:**

```bash
ls -la solana/                                     # на хосте
docker compose -f infra/docker-compose.yml exec solana bash -ic 'id'
```

**Решение:**

```bash
sudo chown -R 1000:1000 solana/
```

### `node: command not found` в `bash -c`

**Причина:** `.bashrc` не читается неинтерактивным shell'ом.

**Решение:** `bash -ic` (флаг `-i`).

### `AddrInUse` для портов

**Причина:** старая версия сервиса не убита.

**Решение:**

```bash
sudo pkill -f zkpool-backend
sudo pkill -f zkpool-prover
sudo pkill -f "node src/server.js"
```

### Docker-образ не собирается

**Причина:** часто — сетевая ошибка при загрузке Sunspot `.deb` или Solana CLI.

**Решение:** повторить `docker compose build solana`. Если повторяется — проверить интернет.

---

## 13. Воспроизведение с нуля

Минимальный набор команд для **полного** повторения этапа:

```bash
# 1. Клонировать
cd ~/Projects/Solana
git clone git@github.com:kwebhub/zkpool-solana.git
cd zkpool-solana

# 2. Создать структуру (см. раздел 5)
# 3. Создать .gitignore (см. раздел 6)
# 4. Создать infra/docker-compose.yml (см. раздел 7)
# 5. Создать infra/docker/Dockerfile.solana (см. раздел 7)

# 6. Собрать образ (~20 минут)
cd infra
docker compose build solana
docker compose up -d solana
cd ..

# 7. Проверить версии (см. раздел 8)

# 8. Чекпоинт (см. раздел 9)

# 9. Первый коммит (см. раздел 10)
```

**Что должно быть готово до старта:**
- Docker установлен.
- Git настроен (SSH ключ на GitHub).
- Свободно ~5 ГБ на диске (образ ~3 ГБ).

---

## 14. Что дальше

**Следующий этап:** `02-circuits.md` — ZK-схемы на Noir.

**Что будет:**
- Создание `circuits/poseidon/` — общая библиотека хешей.
- `circuits/hash2/`, `circuits/hashes/`, `circuits/withdrawal/` — три circuit'а.
- Написание `withdrawal.nr` с тремя constraints (C1, C2, C3).
- 41 тест в `nargo test`.
- Компиляция в ACIR (`.json`).

**Перед прочтением** (если ещё не):
- `00-zk-primer.md` — что такое ZK-proof, circuit, witness.
- `00-glossary.md` — термины по мере необходимости.

---

## 15. Ссылки

- [Docker Compose documentation](https://docs.docker.com/compose/)
- [Solana install](https://docs.solana.com/cli/install-solana-cli-tools)
- [noirup](https://github.com/noir-lang/noirup)
- [nvm](https://github.com/nvm-sh/nvm)
- [pnpm](https://pnpm.io/)
- [Sunspot releases](https://github.com/reilabs/sunspot/releases)
- `docs/notes/00-glossary.md` — термины.
- `docs/notes/00-zk-primer.md` — введение в ZK.
- `docs/PROJECT_CONTEXT.md`, раздел 14 — политика документации.
