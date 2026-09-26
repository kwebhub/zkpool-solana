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

## 7.3. `witness.rs` — сериализация `Prover.toml`

**Дата:** 2026-09-26
**Commit:** `95052dd`

### Что сделано

- `WitnessInputs` — struct с 10 полями (5 публичных, 5 приватных).
- `validate()` — проверка длины `merkle_proof` и `is_even` (20 элементов), непустые хеши.
- `to_toml()` — рендер в TOML для `nargo execute -p <name>`.
- `with_0x()` — идемпотентно добавляет префикс `0x` (если его нет).

### Критическое правило: `0x` обязателен

Проверено экспериментально:

| Поле | Bare hex | `0x`-prefixed |
|---|---|---|
| `root` | ✅ | ✅ |
| `nullifier_hash` | ✅ | ✅ |
| `recipient` | ✅ | ✅ |
| `recipient_binding` | ✅ | ✅ |
| `amount` | ❌ | ✅ |
| `merkle_proof[i]` | ❌ | ✅ |

Ошибка bare hex для `amount`:

Failed to deserialize inputs: The value passed for parameter amount is invalid:
Expected witness values to be integers, but 00f4240 failed with invalid digit found in string

**Вывод:** все hex в `Prover.toml` пишем с `0x`. Метод `with_0x()` идемпотентен — не превращает `0x…` в `0x0x…`.

**Следствие для формата HTTP:** Merkle-сервис отдаёт **bare hex** (для `tree.rs`), prover принимает **bare hex** от бэкенда и **добавляет `0x`** только при записи TOML. Трансляция — на границе prover'а.

### Критическое правило: имя Prover-файла без точек

`nargo execute -p Prover-7.3-test` → `Cannot find input file '…/Prover-7.toml'`. Nargo обрезает имя на первой `.`.

**Решение:** имена без точек, например `Prover-<uuid>` где дефисы допустимы, а точки — нет. В коде (7.4) будем формировать `Prover-{uuid_simple}`, где `uuid_simple` — UUID без дефисов (или только с дефисами, без точек).

### Тесты (6)

- `test_validate_ok` — валидный вход проходит.
- `test_validate_wrong_merkle_len` — 19 элементов вместо 20 → ошибка.
- `test_validate_wrong_is_even_len` — 19 вместо 20 → ошибка.
- `test_to_toml_has_0x_prefix` — все hex-поля начинаются с `0x`.
- `test_to_toml_idempotent_0x` — повторный вызов `with_0x` не портит уже префиксованный вход.
- `test_to_toml_lists_lengths` — `merkle_proof` содержит 20 строк, `is_even` содержит 20 булевых.

cargo test --lib witness → 6 passed

### End-to-end проверка

Пример `examples/dump_toml.rs` выводил TOML для синтетического witness'а (значения из Stage 3.5 `Prover.toml`). Результат скормлен `nargo execute`:

[withdrawal] Circuit witness successfully solved
[withdrawal] Witness saved to target/w73.gz

Witness успешно посчитан. Пример удалён после проверки (временный артефакт).

### Уроки

1. **Nargo сериализует поля по-разному.** `Field` принимает `0x`-prefix и bare hex; `u64`-подобные (`amount` — фактически `Field`, но путь парсинга другой) требуют именно `0x`. Единая стратегия — всегда `0x` — устраняет класс ошибок.
2. **Имя `-p` не должно содержать точку.** Неочевидно; зафиксировать в `PROJECT_CONTEXT.md` §8.
3. **Idempotent-prefix.** `with_0x()` сначала проверяет, потом добавляет — можно безопасно вызывать повторно.

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
