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
| 4.1.2 | `constants.rs` | ✅ |
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | ← следующий |
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

Убрать `tests` из members **временно** (вернём на 4.5).

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

Добавить `devnet` cluster и `programs.devnet` блок. Program ID пока шаблонный (`EDzVvsts...`) — **изменится** после `anchor build`.

**5. Настроить `programs/zk_pool/Cargo.toml`.**

**Ошибка №2 — правильный набор features.**

Первая попытка:

```toml
[lints.rust]
unexpected_cfgs = { level = "allow", check-cfg = [
    'cfg(anchor-debug)',
    'cfg(custom-heap)',
    'cfg(custom-panic)',
] }
```

**Результат:** `error: invalid --check-cfg argument`.

**Что не так:** `check-cfg` **не принимает** такие строки. Синтаксис другой.

**Вторая попытка:** `'anchor_debug'`.

**Результат:** `error: invalid --check-cfg argument: anchor_debug`.

**Правильное решение (из v2):** не пытаться «разрешить» через `[lints]`, а **объявить** `anchor-debug`, `custom-heap`, `custom-panic` как **features**:

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

**Результат:** ноль warnings.

**Урок:** если макрос обращается к `cfg(feature = "X")` — объяви `X` в `[features]`. Не борись с линтером.

**6. Упростить `lib.rs`.**

Попытка с заглушкой:

```rust
#[program]
pub mod zk_pool {
    use super::*;
    pub fn noop(_ctx: Context<Noop>) -> Result<()> { Ok(()) }
}

#[derive(Accounts)]
pub struct Noop {}
```

**⚠️ Ошибка №3:** `E0107` — `struct takes 0 lifetime arguments but 1 was given`.

**Что не так:** `#[derive(Accounts)]` **не работает** с пустыми структурами.

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

**Результат:** сборка проходит, IDL пустой (356 байт).

**⚠️ Ошибка №4:** `unused import: super::*`.

**Решение:** убрать строку.

### Итоги 4.1.1

**Program ID (настоящий):** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.

**Артефакты:**
- `onchain/target/deploy/zk_pool.so` — 57 480 байт.
- `onchain/target/deploy/zk_pool-keypair.json` — 292 байта.
- `onchain/target/idl/zk_pool.json` — 356 байт (пустой IDL).

**Коммит:** `b8f6fdf`.
**Чекпоинт:** `.checkpoints/04.1.1-anchor-init/`.

### Ошибки 4.1.1

| # | Симптом | Причина | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | `anchor init .` использует имя директории | Инициализировать во временной папке |
| 2 | `invalid --check-cfg argument` | Неправильный синтаксис lints | Объявить `anchor-debug`, `custom-heap`, `custom-panic` как features |
| 3 | `E0107: struct takes 0 lifetime arguments` | `#[derive(Accounts)]` не работает с пустыми структурами | Убрать заглушку |
| 4 | `unused import: super::*` | Пустой `#[program]` не использует | Убрать строку |

---

## 4.1.2. `constants.rs`

### Зачем

Определить **все** константы программы в **одном** месте. Многие из них **должны совпадать** с другими слоями (circuit, spec, verifier) — если разойдутся, on-chain `withdraw` не сработает.

### Что делает

**Seeds для PDA:**

| Seed | Значение | Где используется |
|---|---|---|
| `POOL_SEED` | `b"pool3"` | Адрес `PoolState` |
| `VAULT_SEED` | `b"vault3"` | Адрес vault (хранение SOL) |
| `NULLIFIER_RECORD_SEED` | `b"nullifier_record"` | Адрес записи о nullifier'е |

**Почему `pool3` / `vault3`, а не `pool` / `vault`:** суффикс `3` — версия. В v2 был `pool2` / `vault2`. В v3 — `pool3` / `vault3`. Это **позволяет** разным версиям **сосуществовать** на одном блокчейне.

**Merkle tree:**

| Константа | Значение | Что значит |
|---|---|---|
| `TREE_DEPTH` | 20 | Глубина дерева |
| `MAX_LEAVES` | 2^20 = 1 048 576 | Максимум листьев |
| `ROOT_HISTORY_SIZE` | 10 | Сколько последних root'ов хранить |
| `EMPTY_ROOT` | `[0u8; 32]` | Root пустого дерева |

**`TREE_DEPTH = 20`** должен совпадать:
- `spec.json` → `circuit.tree_depth = 20`.
- `circuits/withdrawal/src/main.nr` → `global TREE_DEPTH = 20`.

**`ROOT_HISTORY_SIZE = 10`** — зачем: root меняется на каждом депозите. Если пользователь подготовил proof по старому root'у, а за это время прошёл **новый** депозит — proof бы **не прошёл**. Храним 10 последних root'ов, чтобы дать **окно** для подтверждения.

**ZK proof:**

| Константа | Значение | Что значит |
|---|---|---|
| `NR_PUBLIC_INPUTS` | 5 | Сколько публичных входов |
| `PUBLIC_INPUTS_BYTES` | 172 | Размер кодированных входов |
| `PROOF_LEN` | 324 | Размер Groth16-proof |
| `VERIFIER_PROGRAM_ID` | `5t51iu6a...` | Program ID verifier'а |

**`NR_PUBLIC_INPUTS = 5`** должен совпадать с `spec.json` → `circuit.nr_public_inputs = 5`.

**`PUBLIC_INPUTS_BYTES = 172`** — 12-byte header + 5 × 32 = **172**.

**`PROOF_LEN = 324`** — Groth16 фиксирован.

**`VERIFIER_PROGRAM_ID`** — verifier, задеплоенный на этапе 3.4:
`5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.

**⚠️ Если circuit изменится** → verifier надо **пересобрать** → Program ID **изменится** → эту константу надо **обновить**. Это **известное ограничение**, описанное в `PROJECT_CONTEXT.md`, раздел 7.4.

**Экономика:**

| Константа | Значение | Что значит |
|---|---|---|
| `MIN_DEPOSIT_AMOUNT` | 1 000 000 lamports (0.001 SOL) | Минимальный депозит |

**Зачем:** предотвратить спам-депозиты, которые заполнят Merkle tree **мусором**. 1 млн lamports — относительно небольшая сумма, но **достаточная**, чтобы спам был **дорогим**.

### Что делаем

Создать `onchain/programs/zk_pool/src/constants.rs` — см. `PROJECT_CONTEXT.md`, раздел 6. Ключевые моменты:
- `use anchor_lang::prelude::*;` — для `pubkey!` макроса.
- Все константы `pub`.
- `pubkey!` макрос для `VERIFIER_PROGRAM_ID`.

### Ожидаемый результат

```bash
cat onchain/programs/zk_pool/src/constants.rs
```

88 строк. `anchor build` — **ноль** warnings, **ноль** ошибок.

### Коммит

`23c4dd9` — feat(onchain): add program constants (stage 4.1.2).

### Чекпоинт

`.checkpoints/04.1.2-constants/`:
- `constants.rs` — копия.
- `constants.rs.sha256` — `14e60b2cf58c156f1be28bae6283accad0b7994cde7ceb810841526ed624330c`.
- `commit.txt` — `23c4dd9c9657794663c23bb2623c679e7a15b02e`.

---

## Ошибки Stage 4.1 — сводка

| # | Симптом | Под-этап | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | 4.1.1 | Инициализировать во временной папке |
| 2 | `invalid --check-cfg argument` | 4.1.1 | Объявить `anchor-debug`, `custom-heap`, `custom-panic` как features |
| 3 | `E0107: struct takes 0 lifetime arguments` | 4.1.1 | Убрать заглушку с `#[derive(Accounts)]` |
| 4 | `unused import: super::*` | 4.1.1 | Убрать строку |

---

## Что дальше

**Следующий под-этап:** 4.1.3 — `error.rs`, `events.rs`, `state.rs`.

**Что будет:**
- **`ZkPoolError`** — 10+ вариантов ошибок (недостаточный депозит, неизвестный root, использованный nullifier и т.д.).
- **`DepositEvent`** и **`WithdrawEvent`** — Anchor events для indexer'а.
- **`PoolState`** — аккаунт пула: authority, current_root_index, roots[10], next_leaf_index, total_deposits.
- **`NullifierRecord`** — аккаунт-пометка «nullifier использован».
