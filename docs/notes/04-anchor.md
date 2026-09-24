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
| 4.1.5 | Инструкция `pool` | ← следующий |
| 4.1.6 | Инструкция `deposit` | ⏳ |
| 4.1.7 | Инструкция `withdraw` | ⏳ |
| 4.1.8 | Тесты | ⏳ |
| 4.1.9 | Деплой на devnet | ⏳ |

---

## 4.1.1. Anchor-workspace

### Зачем

Создать **каркас** Anchor-программы. На этом под-этапе — **никакой** логики, только структура.

### Что делаем

**1. Создать workspace через `anchor init`.**

**⚠️ Ошибка №1:** `anchor init .` **не работает**.

```
Error: Anchor workspace name must be a valid Rust identifier.
```

**Решение:** создавать во **временной** папке.

```bash
cd /tmp
anchor init zk_pool --no-git --test-template rust
```

Затем скопировать в `onchain/`:
```bash
cp Anchor.toml Cargo.toml rust-toolchain.toml /home/ubuntu/onchain/
cp -r programs tests /home/ubuntu/onchain/
```

**Что НЕ копируем:** `app/`, `migrations/`, `target/`, `.gitignore`, `.prettierignore`.

**2. Настроить workspace `Cargo.toml`.**

Убрать `tests` из members **временно** (вернём на 4.5).

**3. Настроить `Anchor.toml`.** Cluster = devnet, `[programs.devnet]`.

**4. Настроить `programs/zk_pool/Cargo.toml`.**

**⚠️ Ошибка №2:** `invalid --check-cfg argument`.

**Что не так:** нельзя «разрешить» `cfg(anchor-debug)` через `[lints.rust]`.

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

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = [
    'cfg(target_os, values("solana"))',
] }
```

**5. Упростить `lib.rs`.**

**⚠️ Ошибка №3:** `E0107: struct takes 0 lifetime arguments but 1 was given` — `#[derive(Accounts)]` **не работает** с пустыми структурами.

**Решение:** убрать заглушку. Anchor **позволяет** программу **без** инструкций.

**⚠️ Ошибка №4:** `unused import: super::*`.

**Решение:** убрать строку.

### Итоги

**Program ID:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.

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

**Почему `pool3`:** суффикс `3` — версия (v3). Позволяет разным версиям **сосуществовать**.

**Merkle tree:**

| Константа | Значение | Совпадает с |
|---|---|---|
| `TREE_DEPTH` | 20 | `spec.json` → `circuit.tree_depth`, `withdrawal/src/main.nr` → `global TREE_DEPTH` |
| `MAX_LEAVES` | 2^20 = 1 048 576 | — |
| `ROOT_HISTORY_SIZE` | 10 | — |
| `EMPTY_ROOT` | `[0u8; 32]` | — |

**`ROOT_HISTORY_SIZE = 10`** — окно для подтверждения proof'а: root меняется на каждом депозите, храним 10 последних.

**ZK proof:**

| Константа | Значение | Совпадает с |
|---|---|---|
| `NR_PUBLIC_INPUTS` | 5 | `spec.json` → `circuit.nr_public_inputs` |
| `PUBLIC_INPUTS_BYTES` | 172 | `spec.json` → `witness_layout.total_bytes` |
| `PROOF_LEN` | 324 | Groth16 (фиксировано) |
| `VERIFIER_PROGRAM_ID` | `5t51iu6a...` | verifier, задеплоенный на 3.4 |

**`VERIFIER_PROGRAM_ID`** — **если circuit изменится**, verifier надо **пересобрать**, Program ID **изменится**, константу **обновить**.

**Экономика:**

| Константа | Значение |
|---|---|
| `MIN_DEPOSIT_AMOUNT` | 1 000 000 lamports (0.001 SOL) |

**Зачем:** предотвратить спам-депозиты.

### Итоги

**88 строк**, ноль warnings.

**Коммит:** `23c4dd9`.
**Чекпоинт:** `.checkpoints/04.1.2-constants/`.

---

## 4.1.3. `error.rs`, `events.rs`, `state.rs`

### Зачем

Определить:
- **Ошибки** — понятные сообщения при провалах (недостаточный депозит, использованный nullifier).
- **События** — indexer (этап 5) читает их из логов транзакции.
- **Аккаунты** — `PoolState` и `NullifierRecord`.

### `error.rs` — 13 ошибок

| Категория | Ошибки |
|---|---|
| Pool | `PoolAlreadyInitialized`, `PoolNotInitialized` |
| Deposit | `DepositBelowMinimum`, `TreeFull`, `RootUnchanged` |
| Withdraw | `UnknownRoot`, `NullifierAlreadyUsed`, `RecipientMismatch`, `AmountMismatch`, `InsufficientVaultBalance` |
| ZK proof | `ProofVerificationFailed`, `InvalidProofLength`, `InvalidPublicInputsLength` |

**Коммит:** `c45cb75`.

### `events.rs` — 2 события

**`DepositEvent`** — payload **88 байт**:
- `commitment [u8; 32]`
- `leaf_index u64`
- `new_root [u8; 32]`
- `timestamp i64`

**`WithdrawEvent`** — payload **80 байт**:
- `nullifier_hash [u8; 32]`
- `recipient Pubkey`
- `amount u64`
- `timestamp i64`

**Важно:** порядок и типы полей **нельзя менять** после деплоя — indexer парсит по **бинарному** layout.

### `state.rs` — 2 аккаунта

**`PoolState`** — PDA `[POOL_SEED]`, размер **384 байта**:

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
- `is_known_root(&root)` — линейный поиск.
- `add_root(new_root)` — кольцевой буфер.
- `current_root()` — текущий root.
- `has_room()` — есть ли место в дереве.

**`NullifierRecord`** — PDA `[NULLIFIER_RECORD_SEED, pool, nullifier_hash]`, размер **88 байт**:

```rust
pub struct NullifierRecord {
    pub pool: Pubkey,
    pub nullifier_hash: [u8; 32],
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
```

**Если аккаунт существует** → nullifier использован. `init` constraint в `withdraw` **упадёт** при попытке создать второй раз — это **защита от double-spend**.

**Коммит:** `c45cb75`.

**Чекпоинт:** `.checkpoints/04.1.3-types/`.

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

### Реализация

```rust
pub fn reduce_to_field(input: &[u8; 32]) -> [u8; 32] {
    let mut value = *input;
    for _ in 0..5 {                    // ← 5, не 4!
        if !is_ge(&value, &BN254_PRIME_BE) {
            break;
        }
        value = sub_be(&value, &BN254_PRIME_BE);
    }
    value
}
```

`is_ge` и `sub_be` — big-endian сравнение и вычитание.

### ⚠️ Ошибка: цикл `for _ in 0..4`

**Симптом:** тест `test_reduce_to_field_max` **падал** с `assertion failed: !is_ge(&reduced, &BN254_PRIME_BE)`.

**Причина:** максимум `[0xff; 32]` = `2^256 - 1` ≈ **4.006 × p**. Четырёх итераций **не всегда достаточно** — если результат близок к `5 × p`, нужно **5** вычитаний.

**Решение:** `for _ in 0..5`.

**Урок:** прежде чем писать цикл `reduce`, **посчитай** worst case. `max_input / p` — сколько итераций **максимум** нужно.

### `encode_public_inputs`

```rust
pub fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    recipient_binding: &[u8; 32],
    amount: u64,
) -> [u8; PUBLIC_INPUTS_BYTES] {
    let mut out = [0u8; PUBLIC_INPUTS_BYTES];

    // 12-byte header
    out[0..4].copy_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());
    out[4..8].copy_from_slice(&0u32.to_be_bytes());
    out[8..12].copy_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());

    // 5 × 32 bytes
    out[12..44].copy_from_slice(root);
    out[44..76].copy_from_slice(nullifier_hash);
    out[76..108].copy_from_slice(&reduce_to_field(&recipient.to_bytes()));
    out[108..140].copy_from_slice(recipient_binding);
    out[140..164].copy_from_slice(&[0u8; 24]);           // 24 zero bytes
    out[164..172].copy_from_slice(&amount.to_be_bytes()); // u64 BE

    out
}
```

### 9 unit-тестов

| Тест | Что проверяет |
|---|---|
| `test_encode_length_is_172` | Размер — 172 |
| `test_header` | `[0,0,0,5, 0,0,0,0, 0,0,0,5]` |
| `test_public_inputs_positions` | Каждое поле на своём месте |
| `test_reduce_to_field_small_value_unchanged` | Маленькие значения не меняются |
| `test_reduce_to_field_prime_is_zero` | `p mod p = 0` |
| `test_reduce_to_field_max` | `(2^256 - 1) mod p < p` |
| `test_is_ge_equal` | `a >= a` |
| `test_sub_be_simple` | Простое вычитание |
| `test_sub_be_borrow` | Вычитание с borrow |

**Результат:** 9 / 9 passed.

### Итоги

**293 строки**, ноль warnings, 9 unit-тестов.

**Коммит:** `dc0fb5e`.
**Чекпоинт:** `.checkpoints/04.1.4-encoding/`.

---

## Ошибки Stage 4.1 — сводка

| # | Симптом | Под-этап | Решение |
|---|---|---|---|
| 1 | `Anchor workspace name must be a valid Rust identifier` | 4.1.1 | Инициализировать во временной папке |
| 2 | `invalid --check-cfg argument` | 4.1.1 | Объявить `anchor-debug`, `custom-heap`, `custom-panic` как features |
| 3 | `E0107: struct takes 0 lifetime arguments` | 4.1.1 | Убрать заглушку с `#[derive(Accounts)]` |
| 4 | `unused import: super::*` | 4.1.1 | Убрать строку |
| 5 | `test_reduce_to_field_max` FAILED | 4.1.4 | Цикл `0..4` → `0..5` |

---

## Что дальше

**Следующий под-этап:** 4.1.5 — инструкция `pool`.

**Что будет:**
- Структура `Pool` (accounts).
- Инициализация `PoolState` PDA.
- Инициализация `vault` PDA.
- Anchor handler `handler_pool`.
- Регистрация в `lib.rs`.
