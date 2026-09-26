# 07. Prover (Rust + Sunspot)

> **Этап 7** проекта zkpool-solana.
> Сервис на порту 4002, генерирующий Groth16-пруфы для вывода средств через CLI Sunspot.

---

## TL;DR

Prover-сервис — тонкая Rust+axum обёртка над двумя CLI: `nargo execute` (считает witness) и `sunspot prove` (генерирует Groth16-пруф). Sunspot — Go-бинарь без Rust-библиотеки, поэтому вызываем через `std::process::Command`. Из-за фиксированных имён выходных файлов (`.proof`, `.pw` всегда называются по ACIR, не по witness) — все запросы сериализуются через async mutex.

**Порт:** 4002
**Стек:** Rust 1.89.0 + axum 0.7 + tokio

---

## 7.1. Скелет проекта

**Дата:** 2026-09-26
**Commit:** `ff31292`

### Зачем

Минимальный собираемый crate: `Cargo.toml`, `lib.rs` (для тестов), `main.rs` (запуск), четыре placeholder-модуля под будущие под-этапы.

### Структура

```
services/prover/
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── lib.rs        ← pub mod config/prover/server/witness
    ├── main.rs       ← binary entry point
    ├── config.rs     ← (placeholder)
    ├── witness.rs    ← (placeholder)
    ├── prover.rs     ← (placeholder)
    └── server.rs     ← (placeholder)
```

### `Cargo.toml`

```toml
[package]
name = "zkpool_prover"
version = "0.1.0"
edition = "2021"
rust-version = "1.89.0"
license = "MIT"
description = "Prover for zkpool-solana: generates Groth16 proofs via Sunspot"

[lib]
name = "zkpool_prover"
path = "src/lib.rs"

[[bin]]
name = "zkpool-prover"
path = "src/main.rs"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dotenvy = "0.15"
anyhow = "1.0"
uuid = { version = "1", features = ["v4"] }
hex = "0.4"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
thiserror = "2"

[workspace]
```

**Зависимости:** подмножество бэкенда. **Не нужны** sqlx, redis, reqwest, metrics — prover stateless.

**[workspace] в конце** — как у бэкенда. Явно помечает crate как самостоятельный workspace root.

**Cargo.lock коммитится** — это binary crate, воспроизводимость сборки важна.

### Проверка

```
cargo build --release → Finished in 20.10s
./target/release/zkpool-prover → zkpool-prover placeholder
```

### Уроки

1. **Собирается с первого раза** — потому что `Cargo.toml` скопирован из проверенного бэкенда, только с урезанным списком зависимостей.
2. **Placeholder-модули с одинаковым содержимым** (`//! Placeholder`) дают одинаковый SHA-256. Нормально — чекпоинт зафиксирует это.
3. **`.gitignore` уже покрывает `**/target/`** — build-артефакты не утекают в коммит.

---

## 7.2. `config.rs` — пути к артефактам и бинарям

**Дата:** 2026-09-26
**Commit:** `4678050`

### Зачем

Все пути — в одном месте. Env vars с дефолтами под контейнер `solana`.

### Поля `Config`

| Поле | Env var | Default |
|---|---|---|
| `port` | `PORT` | `4002` |
| `circuit_dir` | `CIRCUIT_DIR` | `/home/ubuntu/circuits/withdrawal` |
| `nargo_bin` | `NARGO_BIN` | `nargo` |
| `sunspot_bin` | `SUNSPOT_BIN` | `sunspot` |
| `nargo_timeout_secs` | `NARGO_TIMEOUT_SECS` | `30` |
| `sunspot_timeout_secs` | `SUNSPOT_TIMEOUT_SECS` | `30` |

### Хелперы для путей

Методы `Config`, а не отдельные поля — чтобы не дублировать формат:

- `acir_path()` → `<circuit_dir>/target/withdrawal.json`
- `ccs_path()` → `<circuit_dir>/target/withdrawal.ccs`
- `pk_path()` → `<circuit_dir>/target/withdrawal.pk`
- `proof_path()` → `<circuit_dir>/target/withdrawal.proof`
- `pw_path()` → `<circuit_dir>/target/withdrawal.pw`

### Паттерн из бэкенда

Хелперы `optional` / `optional_parse` скопированы из `backend/src/config.rs` без изменений. Согласованность важнее микрооптимизаций.

### Тесты

2 теста:
- `test_defaults` — `from_env()` возвращает дефолты.
- `test_artifact_paths` — все `*_path()` собираются правильно.

cargo test --lib → 2 passed

### Уроки

1. **Методы вместо полей для производных путей.** `acir_path()` собирается на лету — если завтра структура `target/` изменится, правим одно место.
2. **Хелперы из бэкенда скопированы дословно.** Позже, если появится третий сервис, их можно вынести в общий crate. Пока — дублирование дешевле преждевременной абстракции.
3. **Env vars с дефолтами, а не required.** В контейнере `solana` всё уже на месте; в CI можно переопределить.

---

## Что дальше

- **7.2** — `config.rs`: пути к `circuits/withdrawal/` и бинарям `nargo`/`sunspot`.
- **7.3** — `witness.rs`: сериализация `Prover.toml`.
- **7.4** — `prover.rs`: `nargo execute` + `sunspot prove` под mutex.
- **7.5** — `server.rs`: axum `POST /prove`, `GET /health`.
- **7.6** — unit-тесты.
- **7.7** — smoke test: реальный witness → реальный пруф → `sunspot verify`.
- **7.8** — финальный чекпоинт.

**Ключевое ограничение:** `sunspot prove` всегда пишет `withdrawal.proof` / `withdrawal.pw` (по имени ACIR, не witness). Параллельные запросы без mutex затр
