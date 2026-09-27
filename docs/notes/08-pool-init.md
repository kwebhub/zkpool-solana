# 08. Deploy pool on-chain

> **Этап 8** проекта zkpool-solana.
> Инициализация пула на devnet: создание `PoolState` PDA и `vault` PDA через инструкцию `pool`.

---

## TL;DR

Anchor-программа `zk_pool` задеплоена в Stage 4.1, но её инструкция `pool` (Initialize Pool) ни разу не вызывалась. Без неё `PoolState` PDA не существует, пул не готов принимать депозиты, индексатор ничего не видит. Stage 8 — вызвать `pool` один раз, чтобы создать PDA.

**Инструмент:** `scripts/pool-init/` — Rust CLI, шлющий инструкцию `pool` через raw JSON-RPC к devnet. Без зависимости `solana-client` (конфликт версий), только модульные `solana-*` crates + `reqwest`.

**Кошелёк:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`
**Program ID:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`
**Pool PDA:** `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`
**Vault PDA:** `HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq`
**Init tx:** `2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1Cfh4pWrTFC2TcnEEiKfjq6e6AeBvNixxGQcBYnQZnB2Y3TPGFQ98zSeC`

---

## 8.1. Скелет `scripts/pool-init/`

**Дата:** 2026-09-27
**Commit:** `b342417`

### Зачем CLI, а не фронтенд

Обсуждали три варианта:
- **A)** Rust CLI в `scripts/` — сейчас.
- **B)** Инициализация из фронтенда (Vue).
- **C)** Оба.

**Выбрали A.** Причины:
1. **Разблокирует Stage 9 (Frontend).** Без живого пула фронтенд нечего тестировать.
2. **Переиспользование в CI и E2E.** Stage 10 (полный E2E) и Stage 12 (CI/CD) будут создавать свежий пул с нуля — CLI для этого.
3. **Не требует кошелька в браузере.** Работает headless.
4. **Фронтенд всё равно получит свою кнопку "Initialize Pool"** в Stage 9 как UI-фичу (вариант C по факту).

### Почему без `solana-client`

`solana-client` на crates.io — либо `4.4.0-alpha.5` (pre-release), либо рассинхронизирован с модульными `solana-*` crates, которые мы уже используем в `tests/`. Микс грозит конфликтом версий (`failed to select a version for solana-hash` — мы это уже проходили в Stage 4.5).

**Решение:** вызывать RPC через `reqwest` напрямую. Solana JSON-RPC — простой HTTP.

### Стек `scripts/pool-init/Cargo.toml`

```toml
[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
bs58 = "0.5"
hex = "0.4"
bincode = "1.3"
base64 = "0.22"

solana-account = "4.3.0"
solana-address = { version = "~2.6.1", features = ["curve25519"] }
solana-hash = "4.5.0"
solana-instruction = "3.4.0"
solana-keypair = "3.1.2"
solana-message = "4.2.4"
solana-pubkey = "3"
solana-sdk-ids = "3.1.0"
solana-signer = "3.0.1"
```

**Версии `solana-*` скопированы из `tests/Cargo.toml`** — те, что уже проверены на совместимость.

**`solana-transaction` не используется** — её методы подписи (`sign`, `sign_partial`, `new_signed_with_payer`) все gated за фичей `wincode`, которая при включении тянет `wincode 0.5.5` и `wincode 0.6.1` одновременно → конфликт.

---

## 8.2. Инициализация пула на devnet

**Дата:** 2026-09-27
**Commit:** `a4d274a`

### Что делает

1. Читает keypair из `~/.config/solana/id.json`.
2. Деривирует PDA:
   - `PoolState`: seeds `[b"pool3"]`
   - `vault`: seeds `[b"vault3", pool_pda]`
3. Проверяет, что `PoolState` ещё не существует (`getAccountInfo`).
4. Строит инструкцию `pool`:
   - discriminator: `[134, 215, 119, 168, 28, 199, 193, 127]`
   - accounts: `authority` (signer, writable), `pool` (writable), `vault` (writable), `system_program` (readonly)
   - data: только discriminator
5. Получает blockhash (`getLatestBlockhash`).
6. Строит `Message` через `solana_message::Message::new_with_blockhash`.
7. **Сериализует message вручную** (`Message::serialize` gated за `wincode`).
8. Подписывает через `keypair.sign_message(&message_bytes)`.
9. Собирает wire-формат вручную: `compact_u16(1) || sig(64) || message`.
10. Отправляет через `sendTransaction`.
11. Ждёт подтверждения (`getSignatureStatuses`).
12. Читает `PoolState` из сети, печатает поля.

### Ручная сериализация message

Формат legacy-сообщения:
```
header: 3 × u8
compact-u16: num_account_keys
account_keys: 32 байта × N
blockhash: 32 байта
compact-u16: num_instructions
для каждой инструкции:
  program_id_index: u8
  compact-u16: num_accounts
  account_indices: u8 × N
  compact-u16: data_len
  data: байты
```

Хелпер `serialize_legacy_message(&Message, &Hash) -> Vec<u8>` — в `src/main.rs`.

### Ручная сборка транзакции

Wire-формат:
```
compact-u16(num_signatures)
signatures: 64 байта × num_signatures
message: байты
```

Для одной подписи: `0x01 || sig[64] || message`.

### Полная ручная сборка — почему

**`solana-transaction 4.1.5`** в нашей конфигурации предоставляет только `new_unsigned`, `new_with_payer`, `new_with_compiled_instructions`. Все методы подписи (`sign`, `partial_sign`, `try_partial_sign`, `new_signed_with_payer`) — под `#[cfg(feature = "wincode")]`.

**Включение `wincode`** ломает сборку: `wincode 0.5.5` (через `solana-transaction`) и `wincode 0.6.1` (через `solana-message`) одновременно в графе → `SchemaRead` trait не совпадает.

**Следствие:** полностью ручная сборка + подпись + wire-формат. Дополнительный код — плата за совместимость версий.

### Запуск

```
RPC:      https://api.devnet.solana.com
Authority: 5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc
Pool PDA:  B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf (bump 254)
Vault PDA: HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq (bump 254)

Sending transaction (277 bytes)...
Signature: 2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1Cfh4pWrTFC2TcnEEiKfjq6e6AeBvNixxGQcBYnQZnB2Y3TPGFQ98zSeC
Waiting for confirmation...
✅ Confirmed

✅ Pool account exists (384 bytes)
   authority:          5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc
   next_leaf_index:    0
   total_deposits:     0
   current_root_index: 0
```

**`PoolState` создан, 384 байта.** Layout:
- 8 байт: discriminator
- 32 байта: authority
- 8 байт: next_leaf_index
- 8 байт: total_deposits
- 1 байт: current_root_index
- `ROOT_HISTORY_SIZE` × 32 байта: roots
- (остальное: padding до 8-байтового выравнивания)

### Грабли

1. **`Pubkey::find_program_address` → `Pubkey::derive_program_address`.**
   Новый API; возвращает `Result<_, _>`, требует обработки.

2. **`solana-address` нужна фича `curve25519`.**
   Без неё `derive_program_address` паникует: `bytes_are_curve_point is only available with the curve25519 feature enabled`.

3. **Seed mismatch — `pool3`, не `pool`.**
   Первый запуск ушёл с `b"pool"` → программа вернула `ConstraintSeeds` (2006). Оказалось, в `constants.rs` константы `b"pool3"` / `b"vault3"` (versioned seeds, чтобы не конфликтовать с предыдущими деплоями). **Не копировать seed из документации — читать `constants.rs`.**

4. **Клиппи-предупреждение `int_plus_one`** — не ошибка, проигнорировано. `data.len() >= X + 1` можно переписать в `> X`, но читаемость важнее.

5. **Stale бинарь.** После `cargo build` в первый раз была запущена старая версия `main.rs` (placeholder). Симптом: `./pool-init` печатает `pool-init placeholder`. Лечение — `touch src/main.rs && cargo build --release`.

### Уроки

1. **`solana-*` crates мутируют быстрее, чем документация по ним.** API-методы появляются/уезжают под feature-flags. Всегда читать исходники установленной версии.

2. **`wincode` — опасная фича.** Тянет несколько версий в граф. Обходить — собирать wire-формат вручную.

3. **Seed-константы versioned.** `b"pool3"` вместо `b"pool"` — правильно для повторных деплоев. Но неочевидно. Всегда читать `constants.rs` перед деривацией PDA.

4. **Rust `Debug` для `Hash` показывает base58-хеш.** Удобно.

### Что дальше

- **Stage 8.3** — финальный чекпоинт.
- **Stage 9** — Frontend: подключение, депозит, вывод.
- **Stage 10** — Full E2E на devnet.

Индексатор бэкенда теперь видит пул. Первый депозит → первое реальное commitment в Postgres → первый реальный Merkle root. Всё, что было построено в Stages 5–7, начнёт работать с реальными данными.
