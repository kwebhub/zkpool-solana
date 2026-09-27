# 08. Deploy pool on-chain

> **Этап 8** проекта zkpool-solana.
> Инициализация пула на devnet: создание `PoolState` PDA и `vault` PDA через инструкцию `pool`.

---

## TL;DR

Anchor-программа `zk_pool` задеплоена в Stage 4.1, но её инструкция `pool` (Initialize Pool) ни разу не вызывалась. Без неё `PoolState` PDA не существует, пул не готов принимать депозиты, индексатор ничего не видит. Stage 8 — вызвать `pool` один раз, чтобы создать PDA.

**Инструмент:** `scripts/pool-init/` — Rust CLI, шлющий инструкцию `pool` через raw JSON-RPC к devnet. Без зависимости `solana-client` (конфликт версий), только модульные `solana-*` crates + `reqwest`.

**Кошелёк:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`
**Program ID:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`
**Pool PDA (будет создан):** `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`

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

Нужные вызовы:
- `getLatestBlockhash`
- `sendTransaction`
- `getSignatureStatuses`
- `getAccountInfo` (проверка, что PDA создан)

Все — обычные POST с JSON-телом.

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

solana-hash = "4.5.0"
solana-instruction = "3.4.0"
solana-keypair = "3.1.2"
solana-message = "4.2.4"
solana-pubkey = "3"
solana-sdk-ids = "3.1.0"
solana-signer = "3.0.1"
solana-transaction = "4.1.5"
```

**Версии `solana-*` скопированы из `tests/Cargo.toml`** — те, что уже проверены на совместимость.

### Что делает

1. Читает keypair из `~/.config/solana/id.json`.
2. Деривирует `PoolState` PDA: seeds `[POOL_SEED]`, program `zk_pool`.
3. Деривирует `vault` PDA: seeds `[VAULT_SEED, pool]`.
4. Строит инструкцию `pool`:
   - discriminator (8 байт): `[134, 215, 119, 168, 28, 199, 193, 127]`
   - accounts: `authority` (signer, writable), `pool` (writable), `vault` (writable), `system_program` (readonly)
   - data: только discriminator (нет аргументов)
5. Получает свежий blockhash через `getLatestBlockhash`.
6. Подписывает и отправляет транзакцию.
7. Ждёт подтверждения.

### Проверка

Планируется:
- PDA существует после вызова (`getAccountInfo` возвращает не-null).
- `PoolState` десериализуется: `authority == wallet`, `next_leaf_index == 0`, `total_deposits == 0`.

### Уроки

1. **`solana-client` — не единственный путь.** Raw JSON-RPC через `reqwest` даёт полный контроль и ноль конфликтов версий.
2. **Дискриминатор — часть контракта.** Он зафиксирован в `PROJECT_CONTEXT.md` §6 и не меняется, пока не изменится сигнатура инструкции.
3. **`solana-*` crates версионируются вместе.** Копирование набора из `tests/Cargo.toml` — надёжный способ избежать `failed to select a version`.
