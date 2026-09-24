# Этап 4. Anchor-программа `zk_pool`

> **См. также:**
> - `00-zk-primer.md` — что такое CPI, PDA, Solana-программы.
> - `00-glossary.md` — все термины.
> - `03-sunspot.md` — предыдущий этап (verifier program).

---

## TL;DR

**Что делаем:** пишем on-chain программу `zk_pool` на Rust + Anchor. Три инструкции: `pool`, `deposit`, `withdraw`. Плюс — LiteSVM интеграционные тесты.

**Зачем:** центральная часть проекта — связывает circuit (этап 2), verifier (этап 3) и клиент (этап 8) через **единый формат** публичных входов.

**Сколько шагов:** 9 под-этапов (4.1.1 – 4.1.9) + 7 под-этапов (4.5.1 – 4.5.7).

**Что понадобится:** `01-setup.md`, `02-circuits.md`, `03-sunspot.md`, devnet-кошелёк.

**Что получится:** программа `zk_pool` на devnet, IDL для Codama, LiteSVM E2E-тест.

**Следующий этап:** `05-backend.md`.

---

## Разбиение этапа

### Stage 4.1 — Anchor program

| # | Что делаем | Статус |
|---|---|---|
| 4.1.1 | Anchor-workspace | ✅ |
| 4.1.2 | `constants.rs` | ✅ |
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | ✅ |
| 4.1.4 | `encode_public_inputs` | ✅ |
| 4.1.5 | Инструкция `pool` | ✅ |
| 4.1.6 | Инструкция `deposit` | ✅ |
| 4.1.7 | Инструкция `withdraw` | ✅ |
| 4.1.8 | Unit-тесты (37) | ✅ |
| 4.1.9 | Деплой на devnet | ✅ |

### Stage 4.5 — LiteSVM E2E test

| # | Что делаем | Статус |
|---|---|---|
| 4.5.1 | Реорганизация `tests/` (вынос из `onchain/`) | ✅ |
| 4.5.2 | `helpers.rs` | ← следующий |
| 4.5.3 | `test_pool.rs` | ⏳ |
| 4.5.4 | `test_deposit.rs` | ⏳ |
| 4.5.5 | `test_withdraw.rs` | ⏳ |
| 4.5.6 | `test_double_spend.rs` | ⏳ |
| 4.5.7 | Финальный чекпоинт | ⏳ |

---

## 4.1.1. Anchor-workspace

### Зачем

Создать **каркас** Anchor-программы. На этом под-этапе — **никакой** логики, только структура. Цель — убедиться, что `anchor build` **работает** в нашем окружении.

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

**Флаг `--test-template rust`** — генерирует **Rust-тесты** вместо Mocha/Jest.

**2. Скопировать нужное в `onchain/`.**

```bash
cd /tmp/zk_pool
cp Anchor.toml /home/ubuntu/onchain/
cp Cargo.toml /home/ubuntu/onchain/
cp rust-toolchain.toml /home/ubuntu/onchain/
cp -r programs /home/ubuntu/onchain/
cp -r tests /home/ubuntu/onchain/
```

**Что НЕ копируем:** `app/`, `migrations/`, `target/`, `.gitignore`, `.prettierignore`.

**3. Настроить workspace `Cargo.toml`.**

Убрать `tests` из members **временно** (вернём на 4.5).

**4. Настроить `Anchor.toml`.** Cluster = devnet, `[programs.devnet]`.

**5. Настроить `programs/zk_pool/Cargo.toml`.**

**⚠️ Ошибка №2:** `invalid --check-cfg argument`.

**Что не так:** нельзя «разрешить» `cfg(anchor-debug)` через `[lints.rust]`. Rust ожидает **или** `cfg(name)`, **или** `cfg(name, values("v1"))`.

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

**6. Упростить `lib.rs`.**

**⚠️ Ошибка №3:** `E0107` — `struct takes 0 lifetime arguments but 1 was given`. `#[derive(Accounts)]` **не работает** с пустыми структурами.

**Решение:** убрать заглушку. Anchor **позволяет** программу **без** инструкций.

**⚠️ Ошибка №4:** `unused import: super::*`. Убрать строку.

### Итоги 4.1.1

**Program ID:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
**Артефакты:** `.so` 57 480 B, keypair 292 B, IDL 356 B.
**Коммит:** `b8f6fdf`. **Чекпоинт:** `.checkpoints/04.1.1-anchor-init/`.

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
| `TREE_DEPTH` | 20 | `spec.json` → `circuit.tree_depth` |
| `MAX_LEAVES` | 2^20 = 1 048 576 | — |
| `ROOT_HISTORY_SIZE` | 10 | — |
| `EMPTY_ROOT` | `[0u8; 32]` | — |

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

**Зачем:** предотвратить спам-депозиты.

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

**Что делает:** из **5 публичных входов** собирает **172-байтный** блоб в **точно том же формате**, что `withdrawal.pw`, `spec.json`, frontend.

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

Инициализация пула. Создаются **два** PDA: `PoolState` и `vault`.

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

### Итоги 4.1.5

**69 строк.** Discriminator: `[134, 215, 119, 168, 28, 199, 193, 127]`.
**Коммит:** `553634e`. **Чекпоинт:** `.checkpoints/04.1.5-pool/`.

---

## 4.1.6. Инструкция `deposit`

### Зачем

Принять SOL в vault. Клиент передаёт **commitment** и **new_root**.

### ⚠️ Trust model — важное замечание

**Инструкция НЕ проверяет** on-chain, что `new_root` — правильный результат вставки `commitment` в дерево. **Злоумышленник** может прислать **мусорный** `new_root` и **сломать** дерево.

**Известное ограничение**, унаследованное из v2. См. `docs/DEMO-NOTICE.md`.

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
    require!(amount >= MIN_DEPOSIT_AMOUNT, ZkPoolError::DepositBelowMinimum);
    require!(pool.has_room(), ZkPoolError::TreeFull);
    require!(new_root != pool.current_root(), ZkPoolError::RootUnchanged);
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);
    transfer(cpi_ctx, amount)?;
    pool.next_leaf_index = leaf_index.checked_add(1).ok_or(...)?;
    pool.total_deposits = pool.total_deposits.checked_add(1).unwrap();
    pool.add_root(new_root);
    emit!(DepositEvent { commitment, leaf_index, new_root, timestamp: ... });
    Ok(())
}
```

### ⚠️ Ошибка: `E0308` в `CpiContext::new`

**Симптом:**

```
error[E0308]: mismatched types
   |
91 |     let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);
   |                   --------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Pubkey`, found `AccountInfo`
```

**Причина:** в **Anchor 1.2.0** сигнатура изменилась:

```rust
pub fn new(program_id: Pubkey, accounts: T) -> Self
```

**Первый аргумент — `Pubkey`, не `AccountInfo`.** В **старых** версиях Anchor было `AccountInfo`.

**Решение:** использовать `.key()`:

```rust
CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts)
```

**Урок:** **всегда** смотри сигнатуры **своей** версии крейта. Путь: `~/.cargo/registry/src/index.crates.io-*/anchor-lang-1.2.0/src/context.rs`.

### ⚠️ Ошибка: `E0432: unresolved import crate`

**Симптом:** после замены glob re-exports на явные — компилятор выдал `unresolved import crate` в `#[program]`.

**Причина:** Anchor-макрос **требует** видимости `__client_accounts_*` — генерируемых структур для CPI. Их экспортирует **glob** `pub use deposit::*;`.

**Решение:** **вернуть** glob re-exports.

**Урок:** Anchor-макрос **неявно** использует glob-reexport'ы. Не «оптимизируй» их.

### Итоги 4.1.6

**119 строк.** **Коммит:** `5455d04`. **Чекпоинт:** `.checkpoints/04.1.6-deposit/`.

---

## 4.1.7. Инструкция `withdraw`

### Зачем

**Главный под-этап.** Именно здесь в v2 сломалось (`InvalidInstructionData`).

### Что делает

1. Проверяет длину proof.
2. Проверяет, что `recipient` совпадает с аккаунтом `to`.
3. Проверяет, что `root` известен.
4. Кодирует 172-байтный блоб из публичных входов.
5. Вызывает verifier через CPI с **правильным** порядком.
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

### ⚠️ Ошибка: `E0277: #[instruction] type mismatch`

**Симптом:**

```
error[E0277]: instruction handler argument type `Vec<u8>` does not match
              `#[instruction(...)]` attribute type `[u8; 32]`
```

**Причина:** атрибут `#[instruction(nullifier_hash: [u8; 32])]` **не совпадает** с первым аргументом handler'а (`proof: Vec<u8>`).

**Решение:**

```rust
#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32])]
```

**Урок:** `#[instruction(...)]` **не** выбирает «нужный» аргумент. Он **сверяет** **весь** список по порядку.

### Перевод SOL из vault

Vault — PDA, без приватного ключа. **Прямая** манипуляция lamports:

```rust
**ctx.accounts.vault.try_borrow_mut_lamports()? = vault_lamports - amount;
**ctx.accounts.to.try_borrow_mut_lamports()? = to_lamports + amount;
```

### Итоги 4.1.7

**216 строк.** Discriminator: `[183, 18, 70, 156, 148, 109, 161, 34]`.
**Коммит:** `ce71a47`. **Чекпоинт:** `.checkpoints/04.1.7-withdraw/`.

---

## 4.1.8. Unit-тесты

**37 unit-тестов** в `zk_pool`. Коммит `8261530`.

Разбито по файлам:
- `constants.rs` — 10 тестов.
- `state.rs` — 10 тестов.
- `events.rs` — 4 теста.
- `lib.rs` — 4 теста.
- `encoding.rs` — 9 тестов.

**Результат:** 37 / 37 passed.

---

## 4.1.9. Деплой на devnet

Коммит `3f6dcd6`.

**Program Id:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
**ProgramData:** `FaLqLdL1FVLcwZPpJTw2ugG67KKnEbZRuUJmqvyNtCeA`.
**Data Length:** 210 000 B.
**Rent:** 1.068 SOL.
**IDL metadata:** `C931NVVbVKu4mjh1wjgh7bmFx6j6TfML89ut1GsRQHXk`.

### ⚠️ Замечания

- `anchor deploy` — **deprecated**. Использовать `anchor program deploy`.
- `.so` **вырос** с 57 KB (пустая) до 210 KB (3 инструкции).

---

## 4.5. LiteSVM E2E test

### Зачем LiteSVM

- **Быстро** — миллисекунды вместо секунд.
- **Детерминированно** — никаких сетевых проблем.
- **Без SOL** — не нужен balance.
- **Ловит регрессии** — если что-то сломается, узнаём **сразу**.
- **Полный E2E** — реальный proof, реальный CPI в verifier.

### 4.5.1. Реорганизация `tests/` — вынос из `onchain/`

**Начальная попытка:** держать `tests/` **внутри** `onchain/tests/`, как это сделал `anchor init`.

**⚠️ Первая проблема:** `cargo fetch` в `onchain/tests/` падает с:

```
error: current package believes it's in a workspace when it's not:
current:   /home/ubuntu/onchain/tests/Cargo.toml
workspace: /home/ubuntu/onchain/Cargo.toml
```

**Причина:** `onchain/Cargo.toml` — workspace root. `members = ["programs/*"]` **не покрывает** `tests/`.

**Первое решение:** добавить `tests` в `workspace.exclude`:

```toml
[workspace]
members = ["programs/*"]
exclude = ["tests"]
```

**Помогло:** `cargo fetch` **запустился**.

**⚠️ Вторая проблема:** `cargo check` в `onchain/tests/` упал с:

```
error[E0658]: use of unstable library feature `maybe_uninit_write_slice`
  --> solana-syscalls-4.2.2/src/lib.rs:2531
```

**Причина:** `litesvm 0.16` тянет **Agave 4.2**, где `solana-syscalls 4.2.2` использует **нестабильную** функцию Rust. Требуется Rust ≥ 1.90.

**Но** `onchain/rust-toolchain.toml` фиксирует **1.89.0**, и `tests/` **подчиняется** ему, потому что **внутри** `onchain/`.

**Третья попытка:** откатить `litesvm` до **0.14.0**.

**Результат:** `litesvm 0.14.0` тянет **Agave 4.3.0**, требующий **Rust 1.97.1**. **Хуже.**

**Итог:** проблема **не** в версии litesvm, а в **подчинении** `tests/` toolchain'у от `onchain/`.

**Проверка:**

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu && rustc --version       # 1.98.1
  cd /home/ubuntu/onchain && rustc --version # 1.89.0
'
```

**Финальное решение — вынести `tests/` из `onchain/` в корень проекта.**

```
zkpool-solana/
├── onchain/                 ← Rust 1.89.0 (rust-toolchain.toml)
│   └── programs/zk_pool/
└── tests/                   ← Rust 1.98.1 (свой rust-toolchain.toml)
    └── Cargo.toml           ← standalone crate
```

**Что сделали:**
1. Удалили `onchain/tests/`.
2. Убрали `exclude = ["tests"]` из `onchain/Cargo.toml`.
3. Создали `tests/` в корне проекта.
4. Добавили `tests/rust-toolchain.toml` с `channel = "1.98.1"`.
5. Создали `tests/Cargo.toml` с `zk_pool = { path = "../onchain/programs/zk_pool" }`.
6. Добавили volume `../tests:/home/ubuntu/tests` в `infra/docker-compose.yml`.
7. Пересоздали контейнер: `docker compose up -d --force-recreate solana`.

**Результат:** `cargo check` в `tests/` **прошёл** за 1m 15s, **без** `E0658`. `cargo test --no-run` упал **только** на отсутствующих модулях (`E0583`) — ожидаемо.

**Урок:** когда крейт требует **более новый** Rust, чем workspace — **выносите** его **из** workspace. Не боритесь с `rust-toolchain.toml` — **изолируйте**.

### Процесс подбора зависимостей

**Проблема:** разные версии `solana-*` крейтов **несовместимы**. `litesvm 0.16` требует **конкретные** версии — с **тильдой** (`~4.5.0`), что означает «**только** 4.5.x».

**Пример ошибки:**

```
error: failed to select a version for `solana-hash`.
  ... required by package `solana-message v5.0.0`
  versions that meet the requirements `^4.6.0` are: 4.7.0, 4.6.0

  previously selected package `solana-hash v4.5.0`
  ... required by `litesvm v0.16.0`
```

`solana-message 5.0.0` требует `solana-hash >= 4.6.0`, а `litesvm 0.16.0` — **только** 4.5.0. **Совместимых** версий **нет**.

**Процесс подбора:**

**1. Проверить, какие версии требует litesvm.**

```bash
find ~/.cargo/registry/src -type d -name "litesvm-0.16.0"
grep -A 2 "^\[dependencies.solana-" \
  ~/.cargo/registry/src/index.crates.io-*/litesvm-0.16.0/Cargo.toml \
  | grep -E "dependencies.solana-|version"
```

**2. Использовать ровно эти версии.**

| Крейт | Версия (для litesvm 0.16) |
|---|---|
| `solana-account` | `4.3.0` |
| `solana-hash` | `~4.5.0` → используем `4.5.0` |
| `solana-instruction` | `~3.4.0` → используем `3.4.0` |
| `solana-keypair` | `3.1.2` |
| `solana-message` | `4.2.4` |
| `solana-sdk-ids` | `3.1.0` |
| `solana-signer` | `3.0.1` |
| `solana-transaction` | `4.1.5` |
| `solana-transaction-error` | `3.3.1` |

**3. Не пытаться «угадать» версии.**

**Не работает:**
```toml
solana-message = "5"
solana-hash = "4"
```

**Работает:**
```toml
solana-message = "4.2.4"
solana-hash = "4.5.0"
```

**4. Проверять `cargo tree` перед написанием кода.**

```bash
cd tests
cargo tree -p litesvm 2>&1 | head -20
```

Если `cargo tree` **прошёл** — зависимости **разрешились**.

**5. Проверять, какой Rust требуется.**

```bash
grep "rust-version" ~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/Cargo.toml
```

Если требуется **новее**, чем в текущем toolchain — **изолировать** крейт.

**Урок:** при работе с `solana-*` крейтами **всегда** смотрите **точные** версии в исходниках `litesvm`. **Не** полагайтесь на semver.

**6. Итоговая рабочая комбинация для нашего проекта.**

```toml
[dependencies]
litesvm = "0.16"
solana-account = "4.3.0"
solana-hash = "4.5.0"
solana-instruction = "3.4.0"
solana-keypair = "3.1.2"
solana-message = "4.2.4"
solana-sdk-ids = "3.1.0"
solana-signer = "3.0.1"
solana-transaction = "4.1.5"
solana-transaction-error = "3.3.1"
zk_pool = { path = "../onchain/programs/zk_pool" }
```

**Rust toolchain:** `1.98.1`.

### Коммиты

- `bf04b7d` — set up workspace config; document `workspace.exclude` pitfall.
- Восстановление `tests/` в корне — будет в следующем коммите.

---

## Ошибки Stage 4.1 — сводка

| # | Симптом | Под-этап | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | 4.1.1 | Temp dir. |
| 2 | `invalid --check-cfg argument` | 4.1.1 | Объявить features. |
| 3 | `E0107: struct takes 0 lifetime arguments` | 4.1.1 | Убрать заглушку. |
| 4 | `unused import: super::*` | 4.1.1 | Убрать строку. |
| 5 | `test_reduce_to_field_max` FAILED | 4.1.4 | `0..4` → `0..5`. |
| 6 | `ambiguous glob re-exports` | 4.1.5 | Переименовать handlers. |
| 7 | `E0308` в `CpiContext::new` | 4.1.6 | `Pubkey`, не `AccountInfo`. |
| 8 | `E0432: unresolved import crate` | 4.1.6 | Вернуть glob re-exports. |
| 9 | `E0277: #[instruction] type mismatch` | 4.1.7 | Все аргументы в порядке. |
| 10 | **Неправильный порядок CPI** | 4.1.7 | `[proof \|\| public_witness]`. |

**Ошибка №10 — критичная.**

---

## Ошибки Stage 4.5 — сводка

| # | Симптом | Решение |
|---|---|---|
| 1 | `current package believes it's in a workspace when it's not` | Вынести `tests/` из `onchain/`. |
| 2 | `E0658: maybe_uninit_write_slice` | Rust 1.98.1, изолировать крейт. |
| 3 | `failed to select a version for solana-hash` | Точные версии из `Cargo.toml` litesvm. |
| 4 | `/home/ubuntu/tests: No such file or directory` | Volume в `docker-compose.yml`, пересоздать контейнер. |
| 5 | `E0583: file not found for module` (5 модулей) | Ожидаемо — модули не написаны. |

---

## Что дальше

**Следующий под-этап:** 4.5.2 — `helpers.rs`.

**Что будет:**
- Функции для **загрузки** `zk_pool.so` и `verifier.so` в LiteSVM.
- Функции для **создания** аккаунтов (payer, recipient).
- Функции для **вызова** инструкций.
- Функции для **чтения** состояния.
