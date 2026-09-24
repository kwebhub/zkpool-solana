# Этап 4. Anchor-программа `zk_pool`

> **См. также:**
> - `00-zk-primer.md` — что такое CPI, PDA, Solana-программы.
> - `00-glossary.md` — все термины.
> - `03-sunspot.md` — предыдущий этап (verifier program).

---

## TL;DR

**Что делаем:** пишем on-chain программу `zk_pool` на Rust + Anchor. Три инструкции: `pool` (инициализация пула), `deposit` (внести SOL), `withdraw` (вывести SOL с ZK-proof).

**Зачем:** это **центральная** часть проекта — она связывает circuit (этап 2), verifier (этап 3) и клиент (этап 8) через **единый формат** публичных входов.

**Сколько шагов:** 9 под-этапов (4.1.1 – 4.1.9).

**Сколько времени:** ~3 часа.

**Что понадобится:**
- `01-setup.md` — Docker.
- `02-circuits.md` — ACIR `withdrawal.json`.
- `03-sunspot.md` — verifier Program ID.
- Devnet-кошелёк с SOL.

**Что получится:**
- Программа `zk_pool` на devnet.
- IDL для Codama.
- LiteSVM E2E-тест (4.5).

**Следующий этап:** `05-backend.md`.

---

## Разбиение этапа

| # | Что делаем | Статус |
|---|---|---|
| 4.1.1 | Anchor-workspace | ✅ |
| 4.1.2 | `constants.rs` | ← следующий |
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | ⏳ |
| 4.1.4 | `encode_public_inputs` | ⏳ |
| 4.1.5 | Инструкция `pool` | ⏳ |
| 4.1.6 | Инструкция `deposit` | ⏳ |
| 4.1.7 | Инструкция `withdraw` | ⏳ |
| 4.1.8 | Тесты | ⏳ |
| 4.1.9 | Деплой на devnet | ⏳ |

---

## 4.1.1. Anchor-workspace

### Зачем

Создать **каркас** Anchor-программы. На этом под-этапе — **никакой** логики, только структура. Цель — убедиться, что `anchor build` **работает** в нашем окружении, до того как писать код.

### Что делаем

**1. Создать workspace через `anchor init`.**

Попытка напрямую:

```bash
cd onchain
anchor init . --no-git
```

**⚠️ Ошибка №1:** `anchor init .` **не работает**.

```
Error: Anchor workspace name must be a valid Rust identifier.
```

**Причина:** Anchor использует **имя текущей директории** (`onchain`) как имя workspace. Имя должно быть валидным Rust-идентификатором.

**Решение:** создавать во **временной** папке с правильным именем, потом копировать.

```bash
cd /tmp
anchor init zk_pool --no-git --test-template rust
```

**Флаг `--test-template rust`** — генерирует **Rust-тесты** вместо Mocha/Jest. Нам нужен Rust: LiteSVM-тесты на TypeScript **нельзя** написать.

**Ожидаемый результат:** `/tmp/zk_pool/` с файлами:

```
zk_pool/
├── Anchor.toml
├── Cargo.toml
├── rust-toolchain.toml
├── app/                    ← пустая
├── migrations/             ← пустая
├── programs/zk_pool/       ← программа
├── tests/                  ← Rust-тесты
└── target/                 ← скомпилированное
```

**2. Скопировать нужное в `onchain/`.**

```bash
cd /tmp/zk_pool
cp Anchor.toml /home/ubuntu/onchain/
cp Cargo.toml /home/ubuntu/onchain/
cp rust-toolchain.toml /home/ubuntu/onchain/
cp -r programs /home/ubuntu/onchain/
cp -r tests /home/ubuntu/onchain/
```

**Что НЕ копируем:**
- `app/` — пустая.
- `migrations/` — не нужна.
- `target/` — скомпилированное.
- `.gitignore`, `.prettierignore` — у нас **свой** корневой.

**3. Настроить workspace `Cargo.toml`.**

Шаблон содержит `tests` в members:

```toml
members = ["programs/*", "tests"]
```

**⚠️ Проблема:** `tests` использует зависимости (LiteSVM), которых у нас **пока нет**. Если оставить — `anchor build` **упадёт**.

**Решение:** убрать `tests` из members **временно**. Вернём на 4.5, когда добавим LiteSVM.

```toml
[workspace]
members = ["programs/*"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.89.0"

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1

[profile.release.build-override]
opt-level = 3
incremental = false
codegen-units = 1
```

**4. Настроить `Anchor.toml`.**

Добавляем `devnet` cluster и `programs.devnet` блок. Программный ID **пока** — шаблонный (`EDzVvsts...`), он **изменится** после `anchor build`.

**5. Настроить `programs/zk_pool/Cargo.toml`.**

**Ошибка №2 — правильный набор features.** Первая попытка:

```toml
[features]
default = []
cpi = ["no-entrypoint"]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
idl-build = ["anchor-lang/idl-build"]
```

и

```toml
[lints.rust]
unexpected_cfgs = { level = "allow", check-cfg = [
    'cfg(anchor-debug)',
    'cfg(custom-heap)',
    'cfg(custom-panic)',
] }
```

**Результат:** `error: invalid --check-cfg argument`.

**Что не так:** Rust ожидает **простые** строки (`'anchor-debug'`), а не `cfg(...)`. Но даже простые строки **не работают** — синтаксис `check-cfg` **другой**.

**Вторая попытка:** `'anchor_debug'` (с подчёркиванием).

**Результат:** `error: invalid --check-cfg argument: anchor_debug`.

**Что не так:** `check-cfg` **не принимает** произвольные значения. Он ожидает **или** `cfg(name)`, **или** `cfg(name, values("v1", "v2"))`.

**Правильное решение (из v2):**

Не пытаться «разрешить» `anchor-debug`, `custom-heap`, `custom-panic` через `[lints]`. Вместо этого **объявить** их как **features**:

```toml
[features]
default = []
cpi = ["no-entrypoint"]
no-entrypoint = []
no-log-ix-name = []
idl-build = ["anchor-lang/idl-build"]
anchor-debug = []
custom-heap = []
custom-panic = []

[dependencies]
anchor-lang = "1.1.2"

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = [
    'cfg(target_os, values("solana"))',
] }
```

**Что здесь:**
- Три **пустых** features: `anchor-debug`, `custom-heap`, `custom-panic`. Rust видит `cfg(feature = "anchor-debug")` — и **не выдаёт** warning.
- `[lints.rust]` — **только** для `cfg(target_os, values("solana"))`. Это **отдельная** проблема Anchor-макроса.

**Результат:** ноль warnings, ноль ошибок.

**Урок:** если макрос обращается к `cfg(feature = "X")` — объяви `X` в `[features]`. Не борись с линтером.

**6. Упростить `lib.rs`.**

Первая попытка включала заглушку:

```rust
#[program]
pub mod zk_pool {
    use super::*;

    pub fn noop(_ctx: Context<Noop>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Noop {}
```

**⚠️ Ошибка №3:** `E0107` — `struct takes 0 lifetime arguments but 1 was given`.

**Что не так:** `#[derive(Accounts)]` **не работает** с пустыми структурами. Anchor генерирует код, требующий **lifetime**, а пустая структура его **не имеет**.

**Решение:** **убрать** заглушку. Anchor **позволяет** программу **без** инструкций.

```rust
use anchor_lang::prelude::*;

declare_id!("EDzVvstsabPPHYx6QLgw1J8ZRFhJrGv2fiz2a6o9Kym9");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

#[program]
pub mod zk_pool {
    // No instructions yet — added in stages 4.1.5 – 4.1.7.
}
```

**Результат:** сборка **проходит**, IDL **пустой** (356 байт).

**⚠️ Ошибка №4:** `unused import: super::*`.

**Что не так:** `use super::*;` в пустом модуле **не используется**.

**Решение:** убрать строку. В пустом `#[program]` она не нужна.

### Итоги

**Файлы:**
- `onchain/Anchor.toml` — cluster devnet, Program ID.
- `onchain/Cargo.toml` — workspace без `tests`.
- `onchain/rust-toolchain.toml` — Rust 1.89.0.
- `onchain/programs/zk_pool/Cargo.toml` — features + lints.
- `onchain/programs/zk_pool/src/lib.rs` — пустой `#[program]`.
- `onchain/programs/zk_pool/src/{constants,error,instructions,state}.rs` — заглушки.
- `onchain/tests/` — скопирован из шаблона, но **не в members**.

**Program ID (настоящий):** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.

**Артефакты:**
- `onchain/target/deploy/zk_pool.so` — 57 480 байт.
- `onchain/target/deploy/zk_pool-keypair.json` — 292 байта.
- `onchain/target/idl/zk_pool.json` — 356 байт (пустой IDL).

### Коммиты

- `b8f6fdf` — feat(onchain): initialize Anchor workspace with empty zk_pool program (stage 4.1.1).

### Чекпоинт

`.checkpoints/04.1.1-anchor-init/`:
- `program-id.txt` — `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
- `manifest.txt` — SHA-256 пяти файлов.
- `commit.txt` — `b8f6fdfb9f0c8873a7535c26e843b11900eaa12a`.

---

## Ошибки Stage 4.1.1 — сводка

| # | Симптом | Причина | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | `anchor init .` использует имя директории | Инициализировать во **временной** папке с правильным именем, потом копировать |
| 2 | `invalid --check-cfg argument` | Неправильный синтаксис `[lints.rust]` | Объявить `anchor-debug`, `custom-heap`, `custom-panic` как **features**, а не через lints |
| 3 | `E0107: struct takes 0 lifetime arguments` | `#[derive(Accounts)]` **не работает** с пустыми структурами | Убрать заглушку `Noop` |
| 4 | `unused import: super::*` | Пустой `#[program]` не использует `super::*` | Убрать строку |

### Уроки

1. **`anchor init` требует валидный Rust-идентификатор** — а `onchain` (имя папки) им **не является**, если что-то в нём смущает Anchor. Решение — временная папка.
2. **Не бороться с линтером через `[lints]`** — объявить feature **легитимно**.
3. **Anchor позволяет программу без инструкций** — можно идти **пошагово**, не выдумывая заглушки.
4. **Пустые структуры с `#[derive(Accounts)]` не компилируются** — Anchor генерирует код с lifetime.

---

## Что дальше

**Следующий под-этап:** 4.1.2 — `constants.rs`.

**Что будет:**
- Seeds для PDA: `POOL_SEED`, `VAULT_SEED`, `NULLIFIER_RECORD_SEED`.
- `TREE_DEPTH = 20`, `MAX_LEAVES`, `ROOT_HISTORY_SIZE = 10`.
- `MIN_DEPOSIT_AMOUNT = 1_000_000` (0.001 SOL).
- `NR_PUBLIC_INPUTS = 5`.
- `VERIFIER_PROGRAM_ID = pubkey!("5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ")`.
