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

## Что дальше

- **12.2** — Docker build workflow (публикация образов в GHCR).
- **12.3** — Release workflow (тегированные релизы).
- **12.4** — Security workflow (`cargo audit`, `pnpm audit`).
- **12.5** — Шаблоны: PR, issues, dependabot.
- **12.6** — финальный чекпоинт.
