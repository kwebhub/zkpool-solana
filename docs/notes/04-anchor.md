# Этап 4. Anchor-программа `zk_pool`

> **См. также:**
> - `00-zk-primer.md` — что такое CPI, PDA, Solana-программы.
> - `00-glossary.md` — все термины.
> - `03-sunspot.md` — предыдущий этап (verifier program).

---

## TL;DR

**Что делаем:** пишем on-chain программу `zk_pool` на Rust + Anchor. Три инструкции: `pool`, `deposit`, `withdraw`.

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
| 4.1.8 | Тесты | ← следующий |
| 4.1.9 | Деплой на devnet | ⏳ |

---

## 4.1.1 – 4.1.4 (краткая сводка)

**4.1.1 — Anchor-workspace.** Коммит `b8f6fdf`.
**4.1.2 — `constants.rs`.** Коммит `23c4dd9`.
**4.1.3 — `error.rs`, `events.rs`, `state.rs`.** Коммит `c45cb75`.
**4.1.4 — `encode_public_inputs`.** Коммит `dc0fb5e`.

Полное описание — в `PROJECT_CONTEXT.md`, раздел 6.

---

## 4.1.5. Инструкция `pool`

### Зачем

Инициализация пула. Создаются **два** PDA: `PoolState` и `vault`.

### Структура

```rust
#[derive(Accounts)]
pub struct Pool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + PoolState::INIT_SPACE,
              seeds = [POOL_SEED], bump)]
    pub pool: Account<'info, PoolState>,
    #[account(init, payer = authority, space = 0,
              seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    /// CHECK: ...
    pub vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
```

**Ключевое:** `space = 0` для vault — аккаунт **только** для хранения SOL, без данных.

**Коммит:** `553634e`.

---

## 4.1.6. Инструкция `deposit`

### Зачем

Принять SOL в vault.

### ⚠️ Trust model

**Инструкция НЕ проверяет** on-chain, что `new_root` — результат вставки `commitment`. Известное ограничение из v2.

### Структура + handler

См. `PROJECT_CONTEXT.md`. Ключевое:
- Валидация `amount >= MIN_DEPOSIT_AMOUNT`.
- Проверка `has_room()`.
- Проверка `new_root != current_root`.
- **CPI-перевод SOL** через `transfer`.
- Обновление `PoolState`.
- `emit!(DepositEvent)`.

**Коммит:** `5455d04`.

### ⚠️ Ошибка `E0308`: `CpiContext::new` в Anchor 1.2.0

**Симптом:** `expected Pubkey, found AccountInfo`.

**Причина:** в Anchor **1.2.0** сигнатура изменилась:
```rust
pub fn new(program_id: Pubkey, accounts: T) -> Self
```
**Не** `AccountInfo`, а **`Pubkey`**.

**Решение:**
```rust
CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts)
```

**Урок:** всегда смотри сигнатуры **своей** версии крейта. Путь: `~/.cargo/registry/src/index.crates.io-*/anchor-lang-1.2.0/src/context.rs`.

### ⚠️ Ошибка `E0432`: `unresolved import crate`

**Симптом:** после замены glob re-exports на явные — `unresolved import crate` в `#[program]`.

**Причина:** Anchor-макрос **требует** видимости `__client_accounts_*` — генерируемых структур для CPI. Их экспортирует **glob** `pub use pool::*;`.

**Решение:** **вернуть** glob re-exports. Проблема `ambiguous` решается **переименованием** handler'ов.

**Урок:** Anchor-макрос **неявно** использует glob-reexport'ы. Не «оптимизируй» их.

---

## 4.1.7. Инструкция `withdraw`

### Зачем

**Главный под-этап.** Именно здесь в v2 сломалось (`InvalidInstructionData`).

### Что делает

1. Проверяет длину proof.
2. Проверяет, что `recipient` в инструкции **совпадает** с аккаунтом `to`.
3. Проверяет, что `root` **известен**.
4. **Кодирует** 172-байтный блоб из публичных входов.
5. **Вызывает verifier** через CPI с **правильным** порядком `[proof || public_witness]`.
6. Создаёт `NullifierRecord` PDA — защита от double-spend.
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

**Ключевое:**
- `#[instruction(proof, nullifier_hash)]` — **оба** аргумента **в правильном порядке**.
- `init` для `nullifier_record` — **падает**, если уже существует (double-spend).
- `address = VERIFIER_PROGRAM_ID` — **проверка** адреса verifier'а.

### ⚠️ Критично: порядок CPI

**Смотрим исходник verifier'а** (`~/sunspot/gnark-solana/crates/verifier-bin/src/lib.rs`):

```rust
let proof_len = instruction_data.len() - (12 + NR_INPUTS * 32);
let proof_bytes = &instruction_data[..proof_len];
let public_witness_bytes = &instruction_data[proof_len..];
```

**Verifier ожидает:**
```
[proof: PROOF_LEN bytes][public_witness: 12 + 5*32 = 172 bytes]
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

### ⚠️ Ошибка `E0277`: `#[instruction(...)]` type mismatch

**Симптом:**

```
error[E0277]: instruction handler argument type `Vec<u8>` does not match
              `#[instruction(...)]` attribute type `[u8; 32]`
```

**Причина:** атрибут `#[instruction(nullifier_hash: [u8; 32])]` — **перечисляет** аргументы **в порядке**, в котором они идут в handler'е. Но **первый** аргумент — `proof: Vec<u8>`, а не `nullifier_hash`.

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

**216 строк**, ноль warnings.

**IDL:**
- Discriminator: `[183, 18, 70, 156, 148, 109, 161, 34]`.
- 7 accounts: `payer`, `pool`, `nullifier_record`, `vault`, `to`, `verifier_program`, `system_program`.
- 6 args: `proof`, `nullifier_hash`, `root`, `recipient`, `amount`, `recipient_binding`.

**Коммит:** `ce71a47`.
**Чекпоинт:** `.checkpoints/04.1.7-withdraw/`.

---

## Ошибки Stage 4.1 — сводка

| # | Симптом | Под-этап | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | 4.1.1 | Инициализировать во временной папке |
| 2 | `invalid --check-cfg argument` | 4.1.1 | Объявить `anchor-debug`, `custom-heap`, `custom-panic` как features |
| 3 | `E0107: struct takes 0 lifetime arguments` | 4.1.1 | Убрать заглушку с `#[derive(Accounts)]` |
| 4 | `unused import: super::*` | 4.1.1 | Убрать строку |
| 5 | `test_reduce_to_field_max` FAILED | 4.1.4 | Цикл `0..4` → `0..5` |
| 6 | `ambiguous glob re-exports` | 4.1.5 | Переименовать `handler` → `handler_pool`, `handler_deposit` |
| 7 | `E0308: mismatched types` в `CpiContext::new` | 4.1.6 | `Pubkey` в 1.2.0, а не `AccountInfo` |
| 8 | `E0432: unresolved import crate` | 4.1.6 | Вернуть glob re-exports |
| 9 | `E0277: #[instruction] type mismatch` | 4.1.7 | `#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32])]` |
| 10 | **Неправильный порядок CPI** | 4.1.7 | `[proof \|\| public_witness]`, а не наоборот |

**Ошибка №10 — критичная.** Именно она привела бы к `InvalidInstructionData` в v2. Мы её **поймали** на этапе **проектирования**, посмотрев исходник verifier'а.

---

## Что дальше

**Следующий под-этап:** 4.1.8 — тесты.

**Что будет:**
- Проверка Program ID и `VERIFIER_PROGRAM_ID` — parsing.
- Проверка детерминизма PDA.
- Проверка констант (`TREE_DEPTH`, `NR_PUBLIC_INPUTS`, `MIN_DEPOSIT_AMOUNT`).
- Проверка размеров аккаунтов (`PoolState` 384, `NullifierRecord` 88).
- Проверка размеров событий (88, 80).
- Проверка `encode_public_inputs` (уже есть 9 тестов в `encoding.rs`).
