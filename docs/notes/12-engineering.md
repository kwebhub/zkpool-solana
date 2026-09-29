# 12. Engineering processes

> **Этап 12** проекта zkpool-solana.
> CI/CD, GitHub Actions, шаблоны PR/issues.

---

## TL;DR

Проект работает, но все проверки — ручные. Этап 12 — автоматизация: тесты на каждый push, сборка Docker-образов, релизы, шаблоны для контрибьюторов.

**Компоненты:**
- `.github/workflows/ci.yml` — тесты на push/PR.
- `.github/workflows/docker.yml` — сборка образов (planned).
- `.github/workflows/release.yml` — релизы (planned).
- `.github/workflows/security.yml` — `cargo audit`, `pnpm audit` (planned).
- `.github/PULL_REQUEST_TEMPLATE.md`, `ISSUE_TEMPLATE/`.
- `dependabot.yml`.

---

## 12.1. CI workflow

**Дата:** 2026-09-29
**Commit:** `f61b865`

### Зачем

`make up` + прогон тестов вручную на каждой итерации — медленно и забывается. GitHub Actions запускает тесты автоматически на push в `main` и на каждый PR.

### Jobs (7)

| Job | Что делает | Toolchain |
|---|---|---|
| `validate-spec` | `spec.json` vs `.nr` + Rust + TS | stable |
| `onchain` | `cargo check` + `cargo test -p zk_pool --lib` | 1.89.0 |
| `litesvm` | `cargo test` в `tests/` | 1.98.1 |
| `backend` | `cargo test --lib` + `cargo build --release` | stable |
| `prover` | `cargo test --lib` + `cargo build --release` | stable |
| `merkle` | `pnpm test` (23 tests) | Node 24 + pnpm 12.5.1 |
| `web` | `pnpm typecheck` + `pnpm build` | Node 24 + pnpm 12.5.1 |

**Разные toolchain** — намеренно. `onchain` пинится на 1.89.0 (SBF), `litesvm` — на 1.98.1 (Agave 4.2), остальное — stable.

### Кэш

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      services/backend/target
    key: ${{ runner.os }}-backend-${{ hashFiles('services/backend/Cargo.lock') }}
```

**Ключ по хэшу `Cargo.lock`** — кэш инвалидируется только при изменении зависимостей.

**Кэш для `web/`** — встроен в `pnpm/action-setup`. Не требует отдельной настройки.

### Known gap: merkle job

`services/merkle/` требует `hash2.json`, `hashes.json`, `withdrawal.json` в `circuits/`. В CI их нет (папка gitignored). Нужно собирать через `nargo compile` перед тестами.

**Placeholder сейчас:**
```yaml
- name: Generate test circuits
  run: |
    echo "TODO: build circuits via nargo — see Stage 12.5"
    exit 0
```

**Настоящее решение (Stage 12.5 или раньше):**
1. Установить `nargo 1.0.0-rc.2` через `noirup` или скачать бинарь.
2. `cd circuits/{hash2,hashes,withdrawal} && nargo compile`.
3. `cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- apply`.

Сложность: `nargo` в CI не тривиально (нет apt-пакета). Варианты:
- `curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash` + `noirup -v 1.0.0-rc.2`.
- Докер-образ `noir-lang/noir:1.0.0-rc.2`.
- Кэшировать уже скомпилированные ACIRs как artifact.

**Пока отложено** — остальные 6 jobs работают без этих ACIRs.

### Грабли

1. **Rust toolchain per job — критично.** `dtolnay/rust-toolchain@stable` без явной версии тянет latest. Для `onchain` это ломает сборку (SBF нужен 1.89.0).
2. **`pnpm/action-setup@v4` + `setup-node@v4`** — порядок важен. pnpm ставится после Node.
3. **Кэш `target/` может быть большим.** Для Rust-проектов с большим деревом (backend ~150 MB после release build) — GitHub Actions лимитирует 10 GB на репозиторий.
4. **`--frozen-lockfile`** обязателен для `pnpm install` в CI — гарантирует, что lock соответствует package.json.

### Уроки

1. **Отдельный job на каждый сервис — параллельно.** 7 jobs запускаются одновременно. Общее время CI ≈ время самого медленного (web build или backend build).
2. **`working-directory: services/backend`** — вместо `cd services/backend && ...`. Чище, работает в CI.
3. **Placeholder `exit 0` лучше красного CI.** Позволяет мержить остальные изменения, пока доделывается одна часть.
4. **CI.yml как первая часть — не последняя.** Все последующие workflows (docker, release, security) переиспользуют паттерны кэша и toolchain из этого.

---

## 12.2. Docker build workflow

**Дата:** 2026-09-29
**Commit:** `bb5830b`

### Зачем

`solana` образ содержит всё: Rust, Anchor, Solana CLI, nargo, Sunspot, Node.js, pnpm. Сборка занимает 5–10 минут локально. В CI этот образ нужно публиковать, чтобы другие разработчики (или CI-джобы) могли его переиспользовать без пересборки.

### Триггеры

```yaml
on:
  push:
    branches: [main]
    paths:
      - "infra/docker/**"
      - "infra/docker-compose.yml"
      - ".github/workflows/docker.yml"
  pull_request:
    paths: ...
  workflow_dispatch:
```

**`paths:` фильтр** — сборка только при изменениях в docker-related файлах. Не на каждый PR (экономия CI-минут).

**`workflow_dispatch`** — можно запустить вручную через UI.

### Публикация

```yaml
env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}-solana
```

**Итоговое имя:** `ghcr.io/kwebhub/zkpool-solana-solana` (owner + repo + `-solana` суффикс для disambiguation).

### Metadata action

```yaml
- uses: docker/metadata-action@v5
  with:
    images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
    tags: |
      type=ref,event=branch
      type=ref,event=pr
      type=sha,prefix=sha-,format=short
      type=raw,value=latest,enable={{is_default_branch}}
```

**Теги:**
- `main` (branch name)
- `pr-123` (на PR)
- `sha-abc1234` (короткий SHA)
- `latest` (только на main)

### Push policy

```yaml
push: ${{ github.event_name != 'pull_request' }}
```

**На PR** — только build (проверить, что Dockerfile рабочий). **На main** — build + push.

### Кэш

```yaml
cache-from: type=gha
cache-to: type=gha,mode=max
```

**`type=gha`** — использует GitHub Actions Cache (не локальный Docker cache). Слои образа переиспользуются между сборками. Sunspot clone + nargo build — самые тяжёлые слои; после первого билда они кэшируются.

### Permissions

```yaml
permissions:
  contents: read
  packages: write
```

**`packages: write`** — обязательно для публикации в GHCR. По умолчанию `GITHUB_TOKEN` имеет только `read`.

### Dockerfile context

```yaml
context: .
file: infra/docker/Dockerfile.solana
```

**Context = repo root**, не `infra/`. Dockerfile копирует файлы из корня репозитория (см. Stage 1). Если поставить `context: infra/`, COPY не найдёт нужные пути.

### Грабли

1. **`permissions: packages: write` обязателен.** Без него `docker login` проходит, но push падает с `denied`.
2. **`context: .` не `context: infra/`.** Dockerfile использует пути от корня репозитория.
3. **Sunspot clone — самый долгий слой.** При первом билде ~2–3 минуты. С gha cache — секунды.

### Уроки

1. **Paths filter — экономия.** Не пересобирать образ, если изменили только Rust-код или docs.
2. **`metadata-action` генерирует несколько тегов сразу.** Каждый push в main → 4 тега.
3. **`docker/login-action@v3`** + `secrets.GITHUB_TOKEN` — без явных секретов. GHCR использует токен actions.
4. **`sha-` префикс** — сокращённый SHA. В `docker pull ghcr.io/...:sha-abc1234` — точно известная ревизия.

---

## Что дальше

- **12.3** — Release workflow (тегированные релизы).
- **12.4** — Security workflow (`cargo audit`, `pnpm audit`).
- **12.5** — Шаблоны: PR, issues, dependabot.
- **12.6** — финальный чекпоинт.
