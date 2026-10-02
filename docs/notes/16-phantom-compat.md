# 16. Phantom-совместимость signAndSendTransaction и legacy single-notes

> **Внеплановая заметка** (не входит в Stage 15).
> Диагностика и фикс ошибки `Cannot use 'in' operator to search for 'version' in AQA…`
> при работе с UI-кошельком Phantom.

---

## TL;DR

При вызове `provider.signAndSendTransaction(wireBase64)` в браузере Phantom бросает
`TypeError: Cannot use 'in' operator to search for 'version' in AQAAAA…`.
Причина: Phantom ожидает **объект** с `serialize()` и `message.version`, а не base64-строку.
Фикс — передавать `{ serialize: () => bytes, message: { version: 0 } }`.

Плюс: single-ноты (legacy) с `splits.length === 1` не вписывались в схему,
которая жёстко требует `splits: [Field; 3]`. Фикс — на клиенте padding до `[amount, 0, 0]`
и `note_index = 0`. `total_amount = amount`. Constraint'ы C4 и C5 выполняются.

---

## 1. Симптом

### 1.1. Ошибка в UI

- Браузер: Chrome, страница `http://localhost:5173/`.
- Действие: клик **Deposit** (Single note) или **Withdraw**.
- Результат: красный текст в UI:
  ```
  Cannot use 'in' operator to search for 'version' in AQAAAAAAAA…
  ```

Строка `AQAAAA…` — длинная (сотни символов), начинается с `AQ`. Похожа на base64.

### 1.2. Stack trace (DevTools → Sources → Pause on uncaught exceptions)

```
at Mu (chrome-extension://bfnaelmomeimhlpmgjnjophhpkkoljpa/solana.js:13:32497)
at Tv (chrome-extension://bfnaelmomeimhlpmgjnjophhpkkoljpa/solana.js:13:32512)
at bi (chrome-extension://bfnaelmomeimhlpmgjnjophhpkkoljpa/solana.js:13:32638)
at Proxy.signAndSendTransaction (chrome-extension://bfnaelmomeimhlpmgjnjophhpkkoljpa/solana.js:13:44907)
at sendInstruction (http://localhost:5173/src/composables/useDeposit.ts:130:40)
at async Object.deposit (http://localhost:5173/src/composables/useDeposit.ts:51:25)
at async onDeposit (http://localhost:5173/src/components/DepositForm.vue:58:9)
```

`bfnaelmomeimhlpmgjnjophhpkkoljpa` — это extension ID кошелька **Phantom**.
Ошибка бросается **внутри Phantom**, а не в Noir или Solana kit.

---

## 2. Причина

Мы передавали в `provider.signAndSendTransaction()` **base64-строку**:

```ts
const wireBase64 = getBase64EncodedWireTransaction(compiled);
const { signature } = await provider.signAndSendTransaction(wireBase64);
```

Phantom (расширение, `solana.js`) принимает от dApp **объект**, у которого есть:
- метод `serialize()` → `Uint8Array` (wire-формат),
- поле `message` с полем `version` (0 для v0, `"legacy"` для legacy).

Когда Phantom пытается сделать `'version' in input.message`, а `input` — строка,
JS бросает `TypeError: Cannot use 'in' operator…`. В сообщении — сама строка
(наша base64 wire-транзакция, начинается с `AQ`, потому что первый байт v0-транзакции
с сигнатурами first — это `0x01 0x00 …` → base64 `AQAA…`).

---

## 3. Фикс

### 3.1. `web/src/composables/useDeposit.ts`

Было:

```ts
const { signature } = await provider.signAndSendTransaction(wireBase64);
```

Стало:

```ts
const signAndSend = provider.signAndSendTransaction;
if (!signAndSend) {
  throw new Error("wallet does not support signAndSendTransaction");
}
const { signature } = await signAndSend.call(provider, {
  serialize: () => Uint8Array.from(atob(wireBase64), (c) => c.charCodeAt(0)),
  message: { version: 0 },
} as never);
```

### 3.2. `web/src/composables/useWithdraw.ts`

То же самое — тот же вызов, тот же фикс.

### 3.3. Почему `as never`

TypeScript-тип `WalletProvider` (наш собственный, в `web/src/wallet/types.ts`)
объявляет `signAndSendTransaction` с сигнатурой, которая ожидает `string`.
Phantom, судя по поведению, ожидает объект. Мы приводим тип через `as never`,
чтобы обойти проверку TS, но **не** менять сам `WalletProvider` — на других
кошельках (Solflare, Backpack) поведение может отличаться, и мы это ещё не проверяли.

---

## 4. Вторая проблема: single-ноты

### 4.1. Симптом

После фикса Phantom withdraw выдал:

```
splits must have 3 elements, got 1
```

Это — валидация на фронте (`parseNote.ts`), потом на бэкенде (`api_types.rs`),
потом на prover'е (`witness.rs`), а затем сама схема Noir
(`circuits/withdrawal/src/main.nr`, `splits: [Field; SPLIT_COUNT]` где `SPLIT_COUNT = 3`).

### 4.2. Причина

Stage 15 ввёл `splits` длиной ровно 3 в схему circuit'а. Single-депозит
(старый, до Stage 15) сохраняет одну ноту с `splits: [amount]` — массив из 1.

### 4.3. Архитектурное решение (вариант B, выбран)

**Не менять схему.** Single-нота рассматривается как **вырожденный split**:
`[amount, 0, 0]`, `note_index = 0`, `total_amount = amount`.

- C4: `Σ splits[i] == total_amount` → `amount + 0 + 0 == amount` ✓
- C5: `splits[note_index] == amount` → `splits[0] == amount` ✓

Это не «обходной путь» — это корректная интерпретация single-ноты в терминах схемы.
Ничего в circuit'е, on-chain программе и verifier'е менять не пришлось.

### 4.4. Что изменено

**`web/src/withdraw/parseNote.ts`** — принимает `splits.length` ∈ `{1, SPLIT_COUNT}`.

**`web/src/withdraw/buildWitness.ts`** — если `note.splits.length === 1`,
то в API отправляется padded-вектор `[amount, "0".repeat(64), "0".repeat(64)]`,
`note_index = 0`, `total_amount = amount`.

**`services/backend/src/api_types.rs`** — `splits` длиной 1 **или** `SPLIT_COUNT`.

**`services/prover/src/witness.rs`** — то же самое.

Никаких изменений в circuit'е, в verifier'е, в on-chain программе.

---

## 5. Проверка

### 5.1. Single deposit через UI

- Phantom, `0.01 SOL`, кнопка **Deposit** (Single note).
- Транзакция `2TtfJ7mFumF4u8JRoyVNKL7Ux44bdrg5zQP44qLBaXqx3fTrYZ9f7vSqacLUHikTqTy8iyWyqDV9Ku6LC1kx6KXM`.
- Commitment `279acb455432d0506c91e6ecf3ad517a2a82758213cda05b6240084cc047f122`.

### 5.2. Single withdraw через UI

- Тот же note, recipient `3LChuQNFEYz8kTVrVPuAsbeyZxNpt8HKTsUGcRHnRgjP`.
- Транзакция `2dDW1Yzzr…J4uUJoZq` (полная подпись — в UI).
- Root `12853f0991bce4ce2ade6f20d4e1eae64e0efc784e21a95d8c4213bfe8de8516`.
- Nullifier hash `29cc89b74466d53400475ba73c8a0d7395a7fb7af8ef009a621a21c42003e956`.

### 5.3. Баланс получателя

```bash
solana balance 3LChuQNFEYz8kTVrVPuAsbeyZxNpt8HKTsUGcRHnRgjP --url devnet
```

До withdraw: `1.001 SOL`. После withdraw: `1.011 SOL`. Разница `0.01 SOL` — ровно сумма вывода.

### 5.4. Split-депозит через UI (0.5 + 0.3 + 0.2)

- Split deposit tx: `22eEBTUsW2gjZ3pU4JVqM9hJyn9YETLtRRYgVZjoKHirzNYh2LHTyjnSFcT4qBGUN9XxyMDTiJ6fSuUozENt3qkc`.
- Final root: `1eea0b726d54ece4738f1e89ce0f6c691ca8d6f02a00842a5577a303362b3e20`.
- Три ноты:
  - Note 1: `note_index = 0`, `amount = 0.5 SOL`, `nullifier_hash = 07a87eed4472…`.
  - Note 2: `note_index = 1`, `amount = 0.3 SOL`, `nullifier_hash = 0da7261d0deb…`.
  - Note 3: `note_index = 2`, `amount = 0.2 SOL`, `nullifier_hash = 1f277be2871c…`.

### 5.5. Три split-вывода через UI

Все три ноты выведены на `3LChuQNFEYz8kTVrVPuAsbeyZxNpt8HKTsUGcRHnRgjP`.

- Note 1 (0.5 SOL): tx `3fhhjWvzN…oVhB3vf7`.
- Note 2 (0.3 SOL): tx `2PEAPHTD…xryjyGdX`.
- Note 3 (0.2 SOL): tx `5cwkk8kF…aj4bbzyE`.

Root на момент всех трёх выводов: `1eea0b726d54ece4738f1e89ce0f6c691ca8d6f02a00842a5577a303362b3e20`.

### 5.6. Баланс после split-цикла

```
$ solana balance 3LChuQNFEYz8kTVrVPuAsbeyZxNpt8HKTsUGcRHnRgjP --url devnet
2.011 SOL
```

До split-цикла: `1.011 SOL`. После трёх выводов: `2.011 SOL`. Разница `1.0 SOL` — ровно сумма split-депозита (0.5 + 0.3 + 0.2).

### 5.7. Double-spend через UI

Повторный withdraw Note 1 (уже выведенной) был отклонён **до подписи**:

- Phantom: `This transaction reverted during simulation. Funds may be lost if submitted.`
- UI: `Unexpected error`.

Это ожидаемое поведение — on-chain `init` на `NullifierRecord` PDA не даёт создать аккаунт повторно.

## 6. Что не изменено

- `circuits/withdrawal/src/main.nr` — `splits: [Field; 3]`, `SPLIT_COUNT = 3`.
- `circuits/withdrawal/src/test_witness.nr` — без изменений.
- ACIR, verifier `.so`, `Program ID` — без изменений.
- On-chain программа `zk_pool` — без изменений.
- `encode_public_inputs` — без изменений.

Это **самый важный** пункт: фикс **не** требует пересборки verifier'а
и **не** ломает уже задеплоенные на devnet программы. Только клиент
и два Rust-сервиса (валидация).

---

## 7. Уроки

1. **Phantom требует объект, а не base64-строку.**
   `provider.signAndSendTransaction({ serialize, message: { version: 0 } })`.
   Не `signAndSendTransaction(base64)`.

2. **`getBase64EncodedWireTransaction` — правильный формат для RPC,
   но не для Phantom.** Это разные протоколы.

3. **Stack trace в браузере — единственный способ отличить ошибку
   Phantom от ошибки нашего кода.** Без `chrome-extension://…/solana.js`
   в стеке я бы и дальше искал причину в `noir_js`.

4. **Circuit со строгой длиной массива ломает любые legacy-ноты.**
   Если вводится `[Field; N]` как параметр, все существующие ноты с
   `[Field; 1]` становятся невыводимыми. Решение: рассматривать `[Field; 1]`
   как вырожденный случай `[Field; 3]` (padding нулями).

5. **Валидация должна быть на всех слоях, но с одинаковыми правилами.**
   Мы обновили 4 места (parseNote, buildWitness, api_types, witness), чтобы
   принимать length=1. Если бы хоть один слой остался строгим — вывод упал бы.

6. **Stale бинарь.** После правки `witness.rs` и `api_types.rs` надо
   `cargo build --release` **и** перезапустить процесс. Первый тест упал
   именно потому, что prover остался старым.

---

## 8. Файлы, которых касается заметка

- `web/src/composables/useDeposit.ts`
- `web/src/composables/useWithdraw.ts`
- `web/src/withdraw/buildWitness.ts`
- `web/src/withdraw/parseNote.ts`
- `services/backend/src/api_types.rs`
- `services/prover/src/witness.rs`
- `web/vite.config.ts` (отдельный фикс `process.env.NODE_ENV`, вошёл в тот же коммит)

**Commit:** `1aa9ffe`.
**Checkpoint:** `.checkpoints/fix-phantom-wallet/`.

---

## 9. Что дальше

Split-депозит (3 ноты) → 3 вывода через UI **проверены** (см. §5.4–5.7).
Padding length-1 вектора до `[amount, 0, 0]` **не мешает** нормальному split-пути.

- **Solflare** — `signAndSendTransaction` **не принимает** самодельный объект `{ serialize, message: { version: 0 } }`. Внутри Solflare он проходит проверку на инстанс `Transaction`/`VersionedTransaction` из `@solana/web3.js@1.x`. Наш объект недостаточно «транзакция». Симптом: `JsonRpcError: Internal error at chrome-extension://bhhhlbepdkbapadjdnnojkbgioiodbic/inpage.js`. Окно подписи Solflare **открывается** (devnet blockhash проходит проверку сети), но при нажатии «Одобрить» возвращается `Internal error` до отправки в RPC. В истории Solflare записи о попытке нет. **Поддерживается только Phantom.** Поддержка Solflare потребует web3.js-совместимого инстанса `Transaction`, а не `@solana/kit`-wire. Отложено.
- **Backpack** — не проверялся.
