# Этап 4. Anchor-программа `zk_pool`

> **См. также:**
> - `00-zk-primer.md` — что такое CPI, PDA, Solana-программы.
> - `00-glossary.md` — все термины.
> - `03-sunspot.md` — предыдущий этап (verifier program).

---

## TL;DR

**Что делаем:** пишем on-chain программу `zk_pool` на Rust + Anchor. Три инструкции: `pool` (инициализация пула), `deposit` (внести SOL), `withdraw` (вывести SOL с ZK-proof).

**Зачем:** центральная часть проекта — связывает circuit (этап 2), verifier (этап 3) и клиент (этап 8) через **единый формат** публичных входов.

**Сколько шагов:** 9 под-этапов (4.1.1 – 4.1.9).

**Что понадобится:** `01-setup.md`, `02-circuits.md`, `03-sunspot.md`, devnet-кошелёк.

**Что получится:** программа `zk_pool` на devnet, IDL для Codama, LiteSVM E2E-тест (4.5).

**Следующий этап:** `05-backend.md`.

---

## Разбиение этапа

| # | Что делаем | Статус |
|---|---|---|
| 4.1.1 | Anchor-workspace | ✅ |
| 4.1.2 | `constants.rs` | ✅ |
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | ✅ |
| 4.1.4 | `encode_public_inputs` | ✅ |
| 4.1.5 | Инструкция `pool` | ✅ |
| 4.1.6 | Инструкция `deposit` | ✅ |
| 4.1.7 | Инструкция `withdraw` | ✅ |
| 4.1.8 | Тесты | ✅ |
| 4.1.9 | Деплой на devnet | ← следующий |

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

Cluster = devnet, `[programs.devnet]` блок.

**5. Настроить `programs/zk_pool/Cargo.toml`.**

**⚠️ Ошибка №2:** `invalid --check-cfg argument`.

**Что не так:** нельзя «разрешить» `cfg(anchor-debug)` через `[lints.rust]`. Rust ожидает **или** `cfg(name)`, **или** `cfg(name, values("v1"))` — произвольные строки не работают.

**Решение:** **объявить** `anchor-debug`, `custom-heap`, `custom-panic` как **features**:

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

**Что не так:** `#[derive(Accounts)]` **не работает** с пустыми структурами. Anchor генерирует код с lifetime, а пустая структура его **не имеет**.

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

**Program ID:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
**Артефакты:** `.so` 57 480 B, keypair 292 B, IDL 356 B.
**Коммит:** `b8f6fdf`.
**Чекпоинт:** `.checkpoints/04.1.1-anchor-init/`.

---

## 4.1.2. `constants.rs`

### Зачем

**Все** константы программы в одном месте. Многие из них **должны совпадать** с circuit/spec/verifier.

### Что делает

**Seeds для PDA:**

| Seed | Значение |
|---|---|
| `POOL_SEED` | `b"pool3"` |
| `VAULT_SEED` | `b"vault3"` |
| `NULLIFIER_RECORD_SEED` | `b"nullifier_record"` |

**Почему `pool3`:** суффикс `3` — версия (v3). Позволяет разным версиям **сосуществовать** на одном блокчейне.

**Merkle tree:**

| Константа | Значение | Совпадает с |
|---|---|---|
| `TREE_DEPTH` | 20 | `spec.json` → `circuit.tree_depth`, `withdrawal/src/main.nr` → `global TREE_DEPTH` |
| `MAX_LEAVES` | 2^20 = 1 048 576 | — |
| `ROOT_HISTORY_SIZE` | 10 | — |
| `EMPTY_ROOT` | `[0u8; 32]` | — |

**`ROOT_HISTORY_SIZE = 10`** — окно для подтверждения proof'а: root меняется на каждом депозите, храним 10 последних. Если пользователь подготовил proof по старому root'у, а за это время прошёл **новый** депозит — proof бы **не прошёл**. 10 последних — **достаточное** окно.

**ZK proof:**

| Константа | Значение | Совпадает с |
|---|---|---|
| `NR_PUBLIC_INPUTS` | 5 | `spec.json` → `circuit.nr_public_inputs` |
| `PUBLIC_INPUTS_BYTES` | 172 | `spec.json` → `witness_layout.total_bytes` |
| `PROOF_LEN` | 324 | Groth16 (фиксировано) |
| `VERIFIER_PROGRAM_ID` | `5t51iu6a...` | verifier, задеплоенный на 3.4 |

**`VERIFIER_PROGRAM_ID`** — если circuit изменится, verifier надо **пересобрать**, Program ID **изменится**, константу **обновить**.

**Экономика:**

| Константа | Значение |
|---|---|
| `MIN_DEPOSIT_AMOUNT` | 1 000 000 lamports (0.001 SOL) |

**Зачем:** предотвратить спам-депозиты, которые заполнят Merkle tree **мусором**.

### Итоги 4.1.2

**88 строк**, ноль warnings. **Коммит:** `23c4dd9`.

---

## 4.1.3. `error.rs`, `events.rs`, `state.rs`

### Зачем

Определить:
- **Ошибки** — понятные сообщения при провалах.
- **События** — indexer (этап 5) читает их из логов.
- **Аккаунты** — `PoolState` и `NullifierRecord`.

### `error.rs` — 13 ошибок

| Категория | Ошибки |
|---|---|
| Pool | `PoolAlreadyInitialized`, `PoolNotInitialized` |
| Deposit | `DepositBelowMinimum`, `TreeFull`, `RootUnchanged` |
| Withdraw | `UnknownRoot`, `NullifierAlreadyUsed`, `RecipientMismatch`, `AmountMismatch`, `InsufficientVaultBalance` |
| ZK proof | `ProofVerificationFailed`, `InvalidProofLength`, `InvalidPublicInputsLength` |

### `events.rs` — 2 события

**`DepositEvent`** — payload **80 байт**:
- `commitment [u8; 32]` (32)
- `leaf_index u64` (8)
- `new_root [u8; 32]` (32)
- `timestamp i64` (8)

**`WithdrawEvent`** — payload **80 байт**:
- `nullifier_hash [u8; 32]` (32)
- `recipient Pubkey` (32)
- `amount u64` (8)
- `timestamp i64` (8)

**Важно:** порядок и типы полей **нельзя менять** после деплоя — indexer парсит по **бинарному** layout.

### `state.rs` — 2 аккаунта

**`PoolState`** — PDA `[POOL_SEED]`, `INIT_SPACE = 376` (без дискриминатора):

```rust
pub struct PoolState {
    pub authority: Pubkey,                            // 32
    pub next_leaf_index: u64,                         // 8
    pub total_deposits: u64,                          // 8
    pub current_root_index: u64,                      // 8
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],         // 10 × 32 = 320
}
```

**Методы:**
- `is_known_root(&root)` — линейный поиск, `[0u8; 32]` **отвергается**.
- `add_root(new_root)` — кольцевой буфер.
- `current_root()` — текущий root.
- `has_room()` — есть ли место в дереве.

**`NullifierRecord`** — PDA `[NULLIFIER_RECORD_SEED, pool, nullifier_hash]`, `INIT_SPACE = 112`:

```rust
pub struct NullifierRecord {
    pub pool: Pubkey,             // 32
    pub nullifier_hash: [u8; 32], // 32
    pub recipient: Pubkey,        // 32
    pub amount: u64,              // 8
    pub timestamp: i64,           // 8
}
```

**Если аккаунт существует** → nullifier использован. `init` constraint в `withdraw` **упадёт** при попытке создать второй раз — **защита от double-spend**.

### Итоги 4.1.3

**Коммит:** `c45cb75`. **Чекпоинт:** `.checkpoints/04.1.3-types/`.

---

## 4.1.4. `encode_public_inputs`

### Зачем

**Самая ответственная функция.** Именно здесь в v2 **сломалось**: байты программы **не совпали** с байтами, ожидаемыми verifier'ом.

**Что делает:** из **5 публичных входов** собирает **172-байтный** блоб в **точно том же формате**, что:
- `withdrawal.pw` (из `sunspot prove`, этап 3.5),
- `spec.json` → `witness_layout`,
- будет генерировать frontend (этап 8).

### Формат

```
[12-byte header]
  NR_PUBLIC_INPUTS (u32 BE) = 5
  0                 (u32 BE) = 0
  NR_PUBLIC_INPUTS (u32 BE) = 5
[5 × 32 bytes]
  root
  nullifier_hash
  recipient (reduced to BN254)
  recipient_binding
  amount (right-aligned u64)
```

**Итого: 12 + 5 × 32 = 172 байта.**

### Проблема: BN254 reduction

**Solana Pubkey = 32 байта = 256 бит.** **BN254 prime ≈ 2^254.** Pubkey **может быть** больше модуля — тогда он **не помещается** в `Field`.

**Решение:** `reduce_to_field(pubkey) = pubkey mod BN254_prime`.

**Алгоритм:** повторное **вычитание** prime из input'а, пока результат **не станет** `< prime`.

### ⚠️ Ошибка: цикл `for _ in 0..4`

**Симптом:** тест `test_reduce_to_field_max` **падал** с `assertion failed: !is_ge(&reduced, &BN254_PRIME_BE)`.

**Причина:** максимум `[0xff; 32]` = `2^256 - 1` ≈ **4.006 × p**. Четырёх итераций **не всегда достаточно** — если результат близок к `5 × p`, нужно **5** вычитаний.

**Решение:** `for _ in 0..5`.

**Урок:** прежде чем писать цикл `reduce`, **посчитай** worst case. `max_input / p` — сколько итераций **максимум** нужно.

### 9 unit-тестов

`test_encode_length_is_172`, `test_header`, `test_public_inputs_positions`, `test_reduce_to_field_small_value_unchanged`, `test_reduce_to_field_prime_is_zero`, `test_reduce_to_field_max`, `test_is_ge_equal`, `test_sub_be_simple`, `test_sub_be_borrow`.

**Результат:** 9 / 9 passed.

### Итоги 4.1.4

**293 строки.** **Коммит:** `dc0fb5e`. **Чекпоинт:** `.checkpoints/04.1.4-encoding/`.

---

## 4.1.5. Инструкция `pool`

### Зачем

Инициализация пула. Создаются **два** PDA:
1. **`PoolState`** — метаданные пула.
2. **`vault`** — аккаунт, хранящий SOL.

### Структура accounts

```rust
#[derive(Accounts)]
pub struct Pool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + PoolState::INIT_SPACE,
        seeds = [POOL_SEED],
        bump,
    )]
    pub pool: Account<'info, PoolState>,

    #[account(
        init,
        payer = authority,
        space = 0,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump,
    )]
    /// CHECK: this account holds SOL only; it has no data and is never deserialized.
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
```

### Разбор

| Что | Зачем |
|---|---|
| `Signer<'info>` для `authority` | Плательщик, должен подписать |
| `init` для `pool` | Создаёт аккаунт, **падает**, если уже существует |
| `space = 8 + PoolState::INIT_SPACE` | 8 — Anchor discriminator, `INIT_SPACE` — размер структуры |
| `seeds = [POOL_SEED]` | Детерминированный PDA |
| `init` для `vault` со `space = 0` | Пустой аккаунт — **только** для хранения SOL |
| `UncheckedAccount<'info>` | Мы **не читаем** данные — только SOL |
| `Program<'info, System>` | Anchor требует для `init` |

### Handler

```rust
pub fn handler_pool(ctx: Context<Pool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.next_leaf_index = 0;
    pool.total_deposits = 0;
    pool.current_root_index = 0;
    pool.roots = [[0u8; 32]; ROOT_HISTORY_SIZE];
    msg!("Pool initialized: ...");
    Ok(())
}
```

### ⚠️ Проблема: имя handler'а

Первая версия: `pub fn handler(...)` в обоих модулях (`pool`, `deposit`). В `instructions.rs` было `pub use pool::*;` и `pub use deposit::*;`. Компилятор выдал:

```
warning: ambiguous glob re-exports
```

**Решение:** переименовать функции в **уникальные** имена:
- `pool::handler` → `pool::handler_pool`.
- `deposit::handler` → `deposit::handler_deposit`.
- `withdraw::handler` → `withdraw::handler_withdraw`.

**Урок:** когда два модуля экспортируются через glob и имеют **одинаковые** имена — компилятор ругается. Уникальные имена решают проблему.

### Итоги 4.1.5

**69 строк.** Discriminator: `[134, 215, 119, 168, 28, 199, 193, 127]`.
**Коммит:** `553634e`. **Чекпоинт:** `.checkpoints/04.1.5-pool/`.

---

## 4.1.6. Инструкция `deposit`

### Зачем

Принять SOL в vault. Клиент передаёт **commitment** и **new_root** — новый Merkle root после добавления commitment'а в дерево.

### ⚠️ Trust model — важное замечание

**Инструкция НЕ проверяет** on-chain, что `new_root` — правильный результат вставки `commitment` в дерево. **Злоумышленник** может прислать **мусорный** `new_root` и **сломать** дерево для всех.

**Это — известное ограничение**, унаследованное из v2. Исправление требует **полного** Merkle tree on-chain или **дополнительного** ZK-proof корректности root'а — и то, и другое **вне** scope v3.

**Для production** — см. `docs/DEMO-NOTICE.md`.

### Структура accounts

```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)] pub depositor: Signer<'info>,
    #[account(mut, seeds = [POOL_SEED], bump)] pub pool: Account<'info, PoolState>,
    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    /// CHECK: ... 
    pub vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
```

### Handler

```rust
pub fn handler_deposit(
    ctx: Context<Deposit>,
    commitment: [u8; 32],
    new_root: [u8; 32],
    amount: u64,
) -> Result<()> {
    // 1. Validate amount
    require!(amount >= MIN_DEPOSIT_AMOUNT, ZkPoolError::DepositBelowMinimum);
    // 2. Validate tree has room
    require!(pool.has_room(), ZkPoolError::TreeFull);
    // 3. Validate new_root differs from current
    require!(new_root != pool.current_root(), ZkPoolError::RootUnchanged);
    // 4. Transfer SOL (CPI)
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);
    transfer(cpi_ctx, amount)?;
    // 5. Update pool state
    pool.next_leaf_index = leaf_index.checked_add(1).ok_or(...)?;
    pool.total_deposits = pool.total_deposits.checked_add(1).unwrap();
    pool.add_root(new_root);
    // 6. Emit event
    emit!(DepositEvent { commitment, leaf_index, new_root, timestamp: ... });
    Ok(())
}
```

### ⚠️ Ошибка: `E0308: mismatched types` в `CpiContext::new`

**Симптом:**

```
error[E0308]: mismatched types
  --> programs/zk_pool/src/instructions/deposit.rs:91:35
   |
91 |     let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);
   |                   --------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Pubkey`, found `AccountInfo`
```

**Причина:** в **Anchor 1.2.0** сигнатура изменилась:

```rust
pub fn new(program_id: Pubkey, accounts: T) -> Self
```

**Первый аргумент — `Pubkey`, не `AccountInfo`.** В **старых** версиях Anchor было `AccountInfo`. Мы писали по памяти из v2 — а там была **другая** версия.

**Решение:** использовать `.key()`:

```rust
CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts)
```

**Урок:** **всегда** смотри сигнатуры **своей** версии крейта. Путь: `~/.cargo/registry/src/index.crates.io-*/anchor-lang-1.2.0/src/context.rs`.

### ⚠️ Ошибка: `E0432: unresolved import crate`

**Симптом:** после замены glob re-exports на явные (`pub use deposit::Deposit;`) — компилятор выдал `unresolved import crate` в `#[program]`.

**Причина:** Anchor-макрос **требует** видимости `__client_accounts_*` — генерируемых структур для CPI. Их экспортирует **glob** `pub use deposit::*;`.

**Решение:** **вернуть** glob re-exports. Проблема `ambiguous` решается **переименованием** handler'ов.

**Урок:** Anchor-макрос **неявно** использует glob-reexport'ы. Не «оптимизируй» их.

### Итоги 4.1.6

**119 строк.** **Коммит:** `5455d04`. **Чекпоинт:** `.checkpoints/04.1.6-deposit/`.

---

## 4.1.7. Инструкция `withdraw`

### Зачем

**Главный под-этап.** Именно здесь в v2 сломалось (`InvalidInstructionData`).

### Что делает

1. Проверяет длину proof.
2. Проверяет, что `recipient` в инструкции **совпадает** с аккаунтом `to`.
3. Проверяет, что `root` **известен**.
4. **Кодирует** 172-байтный блоб из публичных входов.
5. **Вызывает verifier** через CPI с **правильным** порядком.
6. Создаёт `NullifierRecord` PDA.
7. Переводит SOL из vault в recipient.
8. Эмитит `WithdrawEvent`.

### Структура accounts (7)

```rust
#[derive(Accounts)]
#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32])]
pub struct Withdraw<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    #[account(mut, seeds = [POOL_SEED], bump)] pub pool: Account<'info, PoolState>,
    #[account(init, payer = payer, space = 8 + NullifierRecord::INIT_SPACE,
              seeds = [NULLIFIER_RECORD_SEED, pool.key().as_ref(),
                       nullifier_hash.as_ref()], bump)]
    pub nullifier_record: Account<'info, NullifierRecord>,
    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    /// CHECK: ...
    pub vault: UncheckedAccount<'info>,
    #[account(mut)] /// CHECK: ...
    pub to: UncheckedAccount<'info>,
    #[account(address = VERIFIER_PROGRAM_ID)] /// CHECK: ...
    pub verifier_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
```

### ⚠️ Критично: порядок CPI

**Смотрим исходник verifier'а** (`~/sunspot/gnark-solana/crates/verifier-bin/src/lib.rs`):

```rust
let proof_len = instruction_data.len() - (12 + NR_INPUTS * 32);
let proof_bytes = &instruction_data[..proof_len];
let public_witness_bytes = &instruction_data[proof_len..];
```

**Verifier ожидает:**
```
[proof: 324 bytes][public_witness: 12 + 5*32 = 172 bytes]
```

**Proof ПЕРВЫМ**, public witness **ВТОРЫМ**.

**Мой черновик был неверным** — я написал `[public_inputs || proof]`. **Переписал** после просмотра исходника.

### Правильный CPI

```rust
let mut data = Vec::with_capacity(PROOF_LEN + PUBLIC_INPUTS_BYTES);
data.extend_from_slice(&proof);           // 324 bytes
data.extend_from_slice(&public_inputs);   // 172 bytes

let ix = Instruction {
    program_id: VERIFIER_PROGRAM_ID,
    accounts: vec![],
    data,
};

invoke_signed(&ix, &[ctx.accounts.verifier_program.to_account_info()], &[])?;
```

**Никаких accounts у verifier'а** — он самодостаточен, читает только `instruction_data`.

### ⚠️ Ошибка: `E0277: #[instruction] type mismatch`

**Симптом:**

```
error[E0277]: instruction handler argument type `Vec<u8>` does not match
              `#[instruction(...)]` attribute type `[u8; 32]`
```

**Причина:** атрибут `#[instruction(nullifier_hash: [u8; 32])]` — **перечисляет** аргументы **в порядке**, в котором они идут в handler'е. Но **первый** аргумент — `proof: Vec<u8>`.

**Решение:** указать **все** аргументы **в правильном порядке**:

```rust
#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32])]
```

**Урок:** `#[instruction(...)]` **не** выбирает «нужный» аргумент из списка. Он **сверяет** **весь** список по порядку.

### Перевод SOL из vault

Vault — PDA, без приватного ключа. **Нельзя** использовать `system_program::transfer`. Используем **прямую** манипуляцию lamports:

```rust
**ctx.accounts.vault.try_borrow_mut_lamports()? = vault_lamports - amount;
**ctx.accounts.to.try_borrow_mut_lamports()? = to_lamports + amount;
```

**Почему безопасно:** vault принадлежит **нашей** программе, seeds валидированы, `require!(vault_lamports >= amount)`.

### Итоги 4.1.7

**216 строк.** Discriminator: `[183, 18, 70, 156, 148, 109, 161, 34]`.
**IDL args:** `proof: bytes`, `nullifier_hash: [u8; 32]`, `root: [u8; 32]`, `recipient: pubkey`, `amount: u64`, `recipient_binding: [u8; 32]`.
**Коммит:** `ce71a47`. **Чекпоинт:** `.checkpoints/04.1.7-withdraw/`.

---

## 4.1.8. Тесты

### Зачем

Юнит-тесты для on-chain программы. Проверяют то, что **не** зависит от сети: значения констант, размеры аккаунтов и событий, логика ring-buffer'а, дискриминаторы.

**Что НЕ тестируется:** полный E2E deposit → withdraw — задача **4.5** (LiteSVM).

### Что тестируется

**`constants.rs` — 10 тестов:**

| Тест | Проверяет |
|---|---|
| `test_tree_depth` | `TREE_DEPTH == 20` |
| `test_max_leaves` | `MAX_LEAVES == 2^20 == 1 048 576` |
| `test_root_history_size` | `ROOT_HISTORY_SIZE == 10` |
| `test_nr_public_inputs` | `NR_PUBLIC_INPUTS == 5` |
| `test_public_inputs_bytes` | `PUBLIC_INPUTS_BYTES == 12 + 5×32 == 172` |
| `test_proof_len` | `PROOF_LEN == 324` |
| `test_min_deposit_amount` | `MIN_DEPOSIT_AMOUNT == 1_000_000` |
| `test_empty_root` | `EMPTY_ROOT == [0; 32]` |
| `test_seeds_are_distinct` | Три seed'а **разные** |
| `test_verifier_program_id_parses` | Program ID парсится в строку |

**`state.rs` — 10 тестов:**

| Тест | Проверяет |
|---|---|
| `test_pool_state_init_space` | `INIT_SPACE == 376` |
| `test_nullifier_record_init_space` | `INIT_SPACE == 112` |
| `test_is_known_root_empty` | Пустой root не найден |
| `test_add_root_and_is_known` | После `add_root` root известен |
| `test_add_multiple_roots_ring_buffer` | После 15 добавлений — только 10 последних |
| `test_current_root_after_add` | `current_root` обновляется |
| `test_has_room_empty` | Есть место |
| `test_has_room_at_limit` | При `MAX_LEAVES` — нет места |
| `test_has_room_just_below_limit` | При `MAX_LEAVES - 1` — есть место |
| `test_is_known_root_rejects_zero` | Zero-root **не** считается известным |

**`events.rs` — 4 теста:**

| Тест | Проверяет |
|---|---|
| `test_deposit_event_size` | Payload `DepositEvent == 80` байт |
| `test_withdraw_event_size` | Payload `WithdrawEvent == 80` байт |
| `test_deposit_event_constructible` | Событие создаётся с полями |
| `test_withdraw_event_constructible` | Событие создаётся с полями |

**`lib.rs` — 4 теста:**

| Тест | Проверяет |
|---|---|
| `test_program_id_parses` | Program ID парсится |
| `test_pool_discriminator` | `[134, 215, 119, 168, 28, 199, 193, 127]` |
| `test_withdraw_discriminator` | `[183, 18, 70, 156, 148, 109, 161, 34]` |
| `test_verifier_program_id_matches_constant` | `VERIFIER_PROGRAM_ID` совпадает |

### Итого

**37 unit-тестов** в `zk_pool`:
- 9 (`encoding.rs`, stage 4.1.4)
- 10 (`constants.rs`, stage 4.1.8)
- 10 (`state.rs`, stage 4.1.8)
- 4 (`events.rs`, stage 4.1.8)
- 4 (`lib.rs`, stage 4.1.8)

**Результат:** 37 / 37 passed.

### Как запускать```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/onchain
  cargo test -p zk_pool --lib
'
```

**Ожидаемый результат:** `test result: ok. 37 passed; 0 failed`.

### ⚠️ Замечание про размеры

В комментариях к `PoolState` в `state.rs` написано «384 bytes» — но `INIT_SPACE` = **376**. Разница — **8-байтный** Anchor discriminator. Тест проверяет **`INIT_SPACE`**, не общий размер аккаунта.

**Для читателя:** `space = 8 + PoolState::INIT_SPACE` в `pool.rs` — 8 байт под discriminator, 376 под данные. **Итого 384**.

### Коммит

`8261530` — test(onchain): add 28 unit tests for constants, state, events, discriminators.

**Чекпоинт:** `.checkpoints/04.1.8-tests/`.

---

## Ошибки Stage 4.1 — сводка

| # | Симптом | Под-этап | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | 4.1.1 | Инициализировать во временной папке |
| 2 | `invalid --check-cfg argument` | 4.1.1 | Объявить `anchor-debug`, `custom-heap`, `custom-panic` как features |
| 3 | `E0107: struct takes 0 lifetime arguments` | 4.1.1 | Убрать заглушку с `#[derive(Accounts)]` |
| 4 | `unused import: super::*` | 4.1.1 | Убрать строку |
| 5 | `test_reduce_to_field_max` FAILED | 4.1.4 | Цикл `0..4` → `0..5` |
| 6 | `ambiguous glob re-exports` | 4.1.5 | Переименовать `handler` → `handler_pool`, `handler_deposit`, `handler_withdraw` |
| 7 | `E0308: mismatched types` в `CpiContext::new` | 4.1.6 | `Pubkey` в 1.2.0, а не `AccountInfo` |
| 8 | `E0432: unresolved import crate` | 4.1.6 | Вернуть glob re-exports |
| 9 | `E0277: #[instruction] type mismatch` | 4.1.7 | `#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32])]` |
| 10 | **Неправильный порядок CPI** | 4.1.7 | `[proof \|\| public_witness]`, а не наоборот |

**Ошибка №10 — критичная.** Именно она привела бы к `InvalidInstructionData` в v2. Поймали на этапе **проектирования**, посмотрев исходник verifier'а.

---

## Что дальше

**Следующий под-этап:** 4.1.9 — деплой на devnet.

**Что будет:**
- `anchor deploy --provider.cluster devnet`.
- Проверка `solana program show` для `zk_pool`.
- Создание `.secrets/zk_pool-keypair.json` для стабильного Program ID.
- Обновление `PROJECT_CONTEXT.md` с задеплоенной программой.
