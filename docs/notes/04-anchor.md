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
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | ✅ |
| 4.1.4 | `encode_public_inputs` | ✅ |
| 4.1.5 | Инструкция `pool` | ✅ |
| 4.1.6 | Инструкция `deposit` | ✅ |
| 4.1.7 | Инструкция `withdraw` | ← следующий |
| 4.1.8 | Тесты | ⏳ |
| 4.1.9 | Деплой на devnet | ⏳ |

---

## 4.1.1. Anchor-workspace

См. предыдущую версию заметки (4.1.1 не менялась).

**Program ID:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
**Коммит:** `b8f6fdf`.

---

## 4.1.2. `constants.rs`

См. предыдущую версию заметки (4.1.2 не менялась).

**Коммит:** `23c4dd9`.

---

## 4.1.3. `error.rs`, `events.rs`, `state.rs`

См. предыдущую версию заметки (4.1.3 не менялась).

**Коммит:** `c45cb75`.

---

## 4.1.4. `encode_public_inputs`

См. предыдущую версию заметки (4.1.4 не менялась).

**Коммит:** `dc0fb5e`.

---

## 4.1.5. Инструкция `pool`

### Зачем

Инициализировать пул. Создаются **два** PDA:
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

### Итоги 4.1.5

**69 строк**, ноль warnings.

**IDL содержит:**
- Инструкцию `pool` с discriminator `[134, 215, 119, 168, 28, 199, 193, 127]`.
- Accounts: `authority`, `pool`, `vault`.
- PDA seeds для `pool`: `b"pool3"`, для `vault`: `b"vault3"`.

**Коммит:** `553634e`.
**Чекпоинт:** `.checkpoints/04.1.5-pool/`.

### ⚠️ Проблема: имя handler'а

Первая версия: `pub fn handler(...)` в обоих модулях (`pool`, `deposit`). В `instructions.rs` было `pub use pool::*;` и `pub use deposit::*;`. Компилятор выдал:

```
warning: ambiguous glob re-exports
```

**Решение:** переименовать функции в **уникальные** имена:
- `pool::handler` → `pool::handler_pool`.
- `deposit::handler` → `deposit::handler_deposit`.

Это **то, что было в v2** — и именно поэтому.

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
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED],
        bump,
    )]
    pub pool: Account<'info, PoolState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump,
    )]
    /// CHECK: this account holds SOL only; ...
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
    let pool = &mut ctx.accounts.pool;

    // 1. Validate amount
    require!(amount >= MIN_DEPOSIT_AMOUNT, ZkPoolError::DepositBelowMinimum);

    // 2. Validate tree has room
    require!(pool.has_room(), ZkPoolError::TreeFull);

    // 3. Validate new_root differs from current
    require!(new_root != pool.current_root(), ZkPoolError::RootUnchanged);

    // 4. Transfer SOL (CPI)
    let cpi_accounts = Transfer {
        from: ctx.accounts.depositor.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);
    transfer(cpi_ctx, amount)?;

    // 5. Update pool state
    let leaf_index = pool.next_leaf_index;
    pool.next_leaf_index = leaf_index.checked_add(1).ok_or(ZkPoolError::TreeFull)?;
    pool.total_deposits = pool.total_deposits.checked_add(1).unwrap();
    pool.add_root(new_root);

    // 6. Emit event
    emit!(DepositEvent {
        commitment,
        leaf_index,
        new_root,
        timestamp: Clock::get()?.unix_timestamp,
    });

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

**Причина:** в **Anchor 1.2.0** сигнатура `CpiContext::new` **изменилась**:

```rust
pub fn new(program_id: Pubkey, accounts: T) -> Self
```

**Первый аргумент — `Pubkey`, не `AccountInfo`.** В **старых** версиях Anchor было `AccountInfo`. Мы писали код по памяти из v2 — а там была **другая** версия.

**Решение:** использовать `.key()` вместо `.to_account_info()`:

```rust
let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);
```

**Урок:** **всегда** смотри сигнатуры **своей** версии крейта, а не копируй из старых примеров. Anchor **меняет** API между минорными версиями.

**Как искать:** `~/.cargo/registry/src/index.crates.io-*/anchor-lang-1.2.0/src/context.rs`.

### ⚠️ Ошибка: `E0432: unresolved import crate` после уборки glob re-exports

**Симптом:**

```
error[E0432]: unresolved import `crate`
  --> programs/zk_pool/src/lib.rs:19:1
   |
19 | #[program]
   | ^^^^^^^^^^
   | unresolved import
   | help: a similar path exists: `deposit::__client_accounts_deposit`
```

**Причина:** мы **убрали** `pub use deposit::*;` и заменили на `pub use deposit::Deposit;`. Anchor-макрос `#[program]` **требует**, чтобы были **видны** сгенерированные структуры `__client_accounts_*` — они нужны для **CPI**.

**Решение:** **вернуть** glob re-exports. Проблема `ambiguous glob re-exports` решается **переименованием** handler'ов (см. 4.1.5).

**Урок:** Anchor-макрос **неявно** использует glob-reexport'ы. Не пытайся «оптимизировать» их в явные импорты — сломается.

### Итоги 4.1.6

**119 строк**, ноль warnings.

**IDL содержит:**
- Инструкцию `deposit` с accounts: `depositor`, `pool`, `vault`, `system_program`.
- Аргументы: `commitment`, `new_root`, `amount`.

**Коммит:** `5455d04`.
**Чекпоинт:** `.checkpoints/04.1.6-deposit/`.

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
| 7 | `E0308: mismatched types` в `CpiContext::new` | 4.1.6 | Первый аргумент — `Pubkey` (Anchor 1.2.0), не `AccountInfo` |
| 8 | `E0432: unresolved import crate` | 4.1.6 | Вернуть glob re-exports (Anchor-макрос требует `__client_accounts_*`) |

---

## Что дальше

**Следующий под-этап:** 4.1.7 — инструкция `withdraw`.

**Что будет:**
- Приём proof, nullifier_hash, root, recipient, amount, recipient_binding.
- Проверка, что `recipient` в инструкции совпадает с `recipient` в proof.
- Проверка `pool.is_known_root(&root)`.
- Создание `NullifierRecord` PDA (защита от double-spend).
- Вызов verifier через **CPI** с 172-байтным блобом из `encode_public_inputs`.
- Перевод SOL из vault в recipient.
- **Главный под-этап**: именно здесь в v2 сломалось.
