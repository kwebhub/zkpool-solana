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

## 7.4. `prover.rs` — nargo + sunspot под mutex

**Дата:** 2026-09-26
**Commit:** `8103dd1`

### Что сделано

- `Prover` — struct с `Config` + `Mutex<()>`.
- `Prover::prove(&self, inputs)` — async метод, возвращает `ProofResult { proof, public_witness }`.
- `run_pipeline` — оркестрация: write TOML → nargo → sunspot → read `.proof`/`.pw`.
- `run_nargo`, `run_sunspot` — отдельные методы для subprocess с таймаутом.

### Ключевое решение: mutex

`sunspot prove` всегда пишет `target/withdrawal.proof` и `target/withdrawal.pw`. Имена **не зависят** от имени witness'а — только от ACIR. Без сериализации два параллельных запроса:

1. A пишет `withdrawal.proof` (свой).
2. B перезаписывает `withdrawal.proof` (свой).
3. A читает файл — получает пруф B.

**Следствие:** результат A невалиден относительно его публичных входов. Data corruption без ошибки.

**Решение:** `Mutex<()>` в `Prover`. Все вызовы `prove()` сериализуются. `_guard = self.lock.lock().await` держится до конца `run_pipeline`.

### Cleanup

Файлы `Prover-<uuid>.toml` и `target/w-<uuid>.gz` удаляются **после** пайплайна, best-effort:

```rust
let result = self.run_pipeline(...).await;
let _ = tokio::fs::remove_file(&prover_toml_path).await;
let _ = tokio::fs::remove_file(&witness_gz_path).await;
result
```

Даже если пайплайн упал — cleanup всё равно выполнится.

НЕ удаляются withdrawal.proof и withdrawal.pw — они перезаписываются каждым запросом, и мы их читаем до релиза mutex'а.

### Имена файлов

- Prover-<uuid_simple>.toml — где uuid_simple — UUID без дефисов (.simple() в uuid crate). Без точек — критично, см. 7.3.
- w-<uuid_simple> — имя witness'а.

### Таймауты

- nargo execute — nargo_timeout_secs (default 30).
- sunspot prove — sunspot_timeout_secs (default 30).

tokio::time::timeout вокруг Command::output(). При превышении — ошибка, cleanup всё равно выполняется.

### Тесты

- Unit-тесты witness.rs и config.rs продолжают проходить (8 штук).
- test_prove_real — #[ignore], требует nargo + sunspot + артефакты. Запускается явно.

### End-to-end проверка (реальный пруф)

```bash
cargo test --lib test_prove_real -- --ignored --nocapture
→ test result: ok. 1 passed; finished in 0.31s
```

Пруф: 324 байта. Public witness: 172 байта. Оба совпадают с размерами из Stage 3.5.

Проверка валидности:
```bash
sunspot verify target/withdrawal.vk target/withdrawal.proof target/withdrawal.pw
→ ✅ Verification successful!
```

Полный цикл: witness.rs → nargo execute → sunspot prove → валидный Groth16-пруф.

### Уроки

1. Mutex через tokio::sync::Mutex, не std::sync::Mutex. Мы в async-контексте; std::sync::Mutex блокирует поток executor'а.
2. Cleanup в let _ = .... Best-effort удаление — не валить запрос, если temp-файл уже удалён кем-то другим.
3. #[ignore] для интеграционных тестов. test_prove_real требует внешних бинарей и артефактов — запускается только вручную или в CI со всеми зависимостями.
4. Фиксированные имена выходных файлов sunspot — архитектурное ограничение. Mutex — следствие. Альтернатива (per-request temp dir) отвергнута в дизайне (раздел 15 PROJECT_CONTEXT.md): проще mutex, пропускная способность ~2–5 пруфов/сек достаточна.

---

## 7.5. `server.rs` — axum HTTP

**Дата:** 2026-09-26
**Commit:** `6aa5fb4`

- `build_router(Arc<AppState>)` → `axum::Router`.
- `GET /health` → `{status: "ok"}`.
- `POST /prove` → `WitnessInputs` → `{proof, public_witness}` (bare hex).
- `main.rs` — `tracing_subscriber`, `Config::from_env()`, `TcpListener` на `0.0.0.0:4002`.

**Формат ответа:**
```json
{"proof": "03286e64...", "public_witness": "..."}
```
Bare hex. 324 B proof + 172 B public witness.

**Формат запроса:** `WitnessInputs` из `witness.rs`. Hex-поля — bare или `0x`-prefixed, нормализуются в `to_toml()`.

**Ошибки:** успех → 200; любая ошибка `prove()` → 500 `{error: "<chain>"}` (`format!("{:#}", e)`).

**Middleware:** `CorsLayer::permissive()` (для Stage 8), `TraceLayer` (логи).

**E2E через HTTP:**
```
POST /prove @/tmp/prove-req.json
→ proof len: 648 hex (324 B), pw len: 344 hex (172 B)
```
Пруф декодирован в бинарь и проверен `sunspot verify` → `✅ Verification successful!`.

**Уроки:**
1. Multi-line curl с `-d "{...}"` в контейнере ломается — писать payload в файл, использовать `-d @file`.
2. `jq` нет в контейнере — парсить JSON через `node -e`.
3. Bare hex на границе сервиса — согласовано с Merkle-сервисом и `tree.rs`.

---

## 7.6. HTTP-тесты

**Дата:** 2026-09-26
**Commit:** `7603672`

### Что сделано

`services/prover/tests/server_test.rs` — интеграционные тесты HTTP-слоя.

**Метод:** `tower::ServiceExt::oneshot` — in-process запросы, без биндинга порта.

### Тесты (4)

| Тест | Тип | Что проверяет |
|---|---|---|
| `test_health` | sync | `GET /health` → 200 `{"status":"ok"}` |
| `test_prove_bad_payload_returns_422` | sync | `POST /prove {}` → 422 (axum Json extractor) |
| `test_prove_wrong_merkle_len_returns_500` | sync | 19 элементов `merkle_proof` → 500 с сообщением |
| `test_prove_real` | `#[ignore]` | реальный пруф через HTTP → 648 hex proof + 344 hex pw |

### Dev-dependencies

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
serde_json = "1.0"
```

- `tower::util::ServiceExt` — `.oneshot()`.
- `http_body_util::BodyExt` — сборка тела ответа в `Bytes`.

### Результат

```
cargo test → 3 passed; 1 ignored
cargo test --test server_test -- --ignored test_prove_real → 1 passed (0.31s)
```

### Уроки

1. **`axum::Router` тестируется через `tower::ServiceExt::oneshot`.** Без биндинга порта, in-process, быстро.
2. **Невалидный JSON → 422, не 400.** Это делает `axum::Json` extractor. Валидация семантики (длина `merkle_proof`) — уже в нашем коде, → 500. Разные слои — разные коды.
3. **`#[ignore]` + явный запуск.** `test_prove_real` требует nargo + sunspot. Обычный `cargo test` быстрый, полный прогон — вручную.

---

## 7.7. Smoke test

**Дата:** 2026-09-26
**Commit:** `89ad83d`

### Что сделано

`services/prover/smoke.sh` — 6 curl-проверок против работающего сервиса.

### Проверки

| # | Проверка | Ожидание |
|---|---|---|
| 1 | `GET /health` | `{"status":"ok"}` |
| 2 | `POST /prove` → HTTP код | `200` |
| 3 | `POST /prove` → `proof` hex длина | `648` (324 B) |
| 4 | `POST /prove` → `public_witness` hex длина | `344` (172 B) |
| 5 | `merkle_proof` из 1 элемента | `500` |
| 6 | Пустой JSON `{}` | `422` |

### Результат

```
pass: 6
fail: 0
```

### Особенности

- Payload записывается в `/tmp/prover-smoke-req.json`, передаётся через `curl -d @file`. **Не** через inline `-d "{...}"` — многострочные inline ломаются в этом контейнере.
- `node -e` для парсинга JSON-ответа (`jq` в контейнере нет).
- Разные коды ошибок тестируются раздельно: 422 (невалидный JSON — extractor axum) и 500 (семантическая ошибка — наш код).

### Уроки

1. **Smoke-скрипт — воспроизводимый артефакт.** Не одноразовые команды в терминале: любой может запустить `./smoke.sh` после рестарта.
2. **Два класса ошибок, два кода.** 422 — «не могу распарсить», 500 — «понял, но не могу выполнить». Тестировать оба.

---

## 7.8. Финальный чекпоинт Stage 7

**Дата:** 2026-09-26
**Commit:** `f5a48bf`

### Итог Stage 7

**Сервис:** `services/prover/`, порт 4002, Rust 1.89.0 + axum 0.7.

**Модули:**
- `config.rs` — env-конфиг + методы `*_path()`.
- `witness.rs` — `WitnessInputs`, `validate()`, `to_toml()`.
- `prover.rs` — `Prover`, mutex, nargo + sunspot pipeline.
- `server.rs` — axum router, `/health`, `/prove`.
- `main.rs` — entry point.
- `tests/server_test.rs` — HTTP-тесты.

**Endpoints:**
| Метод | Путь | Тело | Ответ |
|---|---|---|---|
| GET | `/health` | — | `{status: "ok"}` |
| POST | `/prove` | `WitnessInputs` | `{proof, public_witness}` (bare hex) |

**Конвенция:** bare hex на границе HTTP. `0x` добавляется только при записи `Prover.toml`.

**Тесты:**
- Unit: 8 (`config` — 2, `witness` — 6).
- Integration: 3 + 1 ignored (`test_prove_real`).
- Smoke: 6/6 через `smoke.sh`.

**Итого: 17 активных тестов + 1 ignored.**

**Реальный пруф:** 324 B proof + 172 B public witness. Проверен `sunspot verify` → valid.

### Артефакты чекпоинта

10 файлов — SHA-256 в `manifest.txt`.

### Что дальше

- **Stage 8** — Frontend (Vue 3).
- **Stage 9** — Инфраструктура (Makefile, Prometheus, Grafana, порт-маппинг).
- **Stage 10** — CI/CD.
- **Stage 11** — Безопасность.
- **Stage 12** — Финализация.
- **Stage 13** — Split deposit (после v0.1.0).

### Связь с бэкендом

Бэкенд имеет `PROVER_URL` (default `http://localhost:4002`). Ожидающий эндпоинт `/api/proof` (сейчас `501 STUB`) должен вызвать `POST /prove`. Следующий шаг — реализация этой связки (Stage 8 или отдельный sub-stage Stage 7).

### Уроки

1. **Sunspot — CLI без API.** Мутекс — плата за фиксированные имена выходных файлов.
2. **Два формата hex в проекте:** bare hex на HTTP, `0x` в Noir TOML. Трансляция в `witness.rs::with_0x()`.
3. **Один subprocess-pipeline, сериализованный mutex'ом, даёт ~3 пруфа/сек.** Достаточно для демо.

## Что дальше

- **7.2** — `config.rs`: пути к `circuits/withdrawal/` и бинарям `nargo`/`sunspot`.
- **7.3** — `witness.rs`: сериализация `Prover.toml`.
- **7.4** — `prover.rs`: `nargo execute` + `sunspot prove` под mutex.
- **7.5** — `server.rs`: axum `POST /prove`, `GET /health`.
- **7.6** — unit-тесты.
- **7.7** — smoke test: реальный witness → реальный пруф → `sunspot verify`.
- **7.8** — финальный чекпоинт.

**Ключевое ограничение:** `sunspot prove` всегда пишет `withdrawal.proof` / `withdrawal.pw` (по имени ACIR, не witness). Параллельные запросы без mutex затр
