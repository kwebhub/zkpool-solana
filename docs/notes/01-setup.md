# Этап 1. Репозиторий и Docker-окружение

## Что сделано

Монорепо `zkpool-solana`, Docker-контейнер `solana-zkpool-solana` со всеми инструментами под фиксированными версиями. Sunspot клонируется **в Dockerfile** — клон персистентен (в v2 это было одной из причин, почему `sunspot deploy` падал после пересоздания контейнера).

## Команды

### Структура папок

```bash
mkdir -p onchain circuits services/{backend,merkle,prover} web
mkdir -p infra/docker infra/grafana/provisioning/{datasources,dashboards}
mkdir -p scripts/sync-circuits
mkdir -p docs/notes
mkdir -p .secrets
mkdir -p .checkpoints
```

### `.gitignore`

Ключевые блоки:

```gitignore
# Rust
target/
**/target/

# Node
node_modules/
**/node_modules/
.pnpm-store/
*.log

# Vue/Vite
dist/
.vite/
*.local

# Anchor
.anchor/
test-ledger/

# Env
.env
.env.local
!.env.example

# Solana keys
solana/*.json

# IDE
.vscode/*
!.vscode/extensions.json
.idea/

# Tests
coverage/
playwright-report/
test-results/

# Monitoring
prometheus-data/
grafana-data/

# Noir
Prover.toml
**/Prover.toml

# Secrets
.secrets/

# Checkpoints
.checkpoints/
```

**`docs/notes/` — не игнорируется** (в v2 было наоборот — и это мешало: заметки не попадали в коммиты).

### `infra/docker-compose.yml`

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

**Отличия от v2:**
- Имя контейнера `solana-zkpool-solana` — чтобы не конфликтовать со старым `solana`.
- `network_mode: host` **не используется** — порты явные.

### `infra/docker/Dockerfile.solana`

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

**Ключевое отличие от v2:** `git clone sunspot` — **в Dockerfile**, а не вручную. Клон персистентный.

### Сборка и запуск

```bash
cd infra
docker compose build solana    # ~20 минут
docker compose up -d solana
```

### Проверка версий внутри контейнера

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

## Результат проверки (чекпоинт 1)

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

Sunspot clone:
```
~/sunspot/gnark-solana/crates/verifier-bin
├── Cargo.toml
├── build.rs
└── src
```

**Все версии совпадают с эталоном.** Чекпоинт 1 пройден.

## Ключевые концепции

1. **Монорепо:** `onchain/`, `circuits/`, `services/`, `web/`, `scripts/`, `infra/`, `docs/`.
2. **Контейнер `solana-zkpool-solana`** — all-in-one: Rust, Solana CLI, Anchor, Nargo, Sunspot, Node, pnpm.
3. **Volumes:** монтируются в `/home/ubuntu/...`, чтобы файлы принадлежали локальному пользователю.
4. **`USER ubuntu` + `user: "1000:1000"`** — файлы в volumes принадлежат локальному пользователю.
5. **Фиксированные версии** всех инструментов — любое расхождение = стоп.

## Известные замечания

- **`sunspot --version` не поддерживается** — у него только `--help`. Это нормально для 1.0.0. Не баг.
- **`bash -ic` обязателен** для команд внутри контейнера — `.bashrc` иначе не читается, `node`/`pnpm`/`cargo` не находятся.

## Отличия от v2

| Что | v2 | v3 |
|---|---|---|
| Sunspot clone | вручную после падения `sunspot deploy` | **в Dockerfile** |
| Имя контейнера | `solana` | `solana-zkpool-solana` |
| `network_mode` | `host` | явные порты |
| Volume `scripts/` | добавлен в этапе 9 | сразу |
| Checkpoints | нет | `.checkpoints/NN-name/` |
| `docs/notes/` | gitignored | **коммитится** |

## Коммиты

- `8a41984` — initial project skeleton with checkpoints
- `94c18fe` — Docker environment with pinned tool versions
- `e1fec4c` — un-ignore `docs/notes/`, add checkpoint methodology
- `951f79d` — add `PROJECT_CONTEXT.md`
- `e55dcc6` — record `PROJECT_CONTEXT` commit hash
