# 09. Frontend (Vue 3 + TypeScript + Pug + SCSS)

> **Этап 9** проекта zkpool-solana.
> Веб-интерфейс: подключение кошелька, депозит, вывод. Порт 5173.

---

## TL;DR

Vue 3 SPA. Стек: Vite 5, TypeScript 5, Pinia 2, Pug, SCSS. Общается с бэкендом (4001) и кошельком (browser wallet adapter). Генерирует commitments локально через `@noir-lang/noir_js`, собирает witness, шлёт на `/api/withdraw`, получает base64-пруф, отправляет в on-chain программу через сгенерированный Codama-клиент.

**Порт:** 5173 (dev), статика (prod).
**Стек:** Vue 3.5 + Vite 5.4 + Pinia 2.2 + TypeScript 5.9 + Pug 3.0 + SCSS 1.105.
**Node:** 24.21.0.
**Пакетный менеджер:** pnpm 12.5.1.

---

## 9.1. Скелет `web/`

**Дата:** 2026-09-27
**Commit:** `45b887e`

### Зачем TypeScript + Pug + SCSS

Изначально в `PROJECT_CONTEXT.md` §4 было просто "Vue 3 + Vite + Pinia". Пользователь указал добавить TS + Pug + SCSS. Обновлена таблица технологий:

| Frontend | Vue 3 + Vite + Pinia + TypeScript + Pug + SCSS | 3.5+ / 5.4+ / 2.2+ / 5.6+ / 3.0+ / 1.79+ |

**TS** — типы для witness'а, ответов бэкенда, Solana-объектов. Без типов ZK-код превращается в кашу.
**Pug** — компактный шаблон. Меньше строк, чем HTML, читаемее для форм с 10+ полями.
**SCSS** — вложенность, переменные для цветов/типографики, миксины для кнопок.

### Структура

```
web/
├── index.html
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── tsconfig.json
├── vite.config.ts
├── public/circuits/        ← ACIR-ы (gitignored, sync-circuits)
└── src/
    ├── main.ts
    ├── App.vue
    └── env.d.ts
```

### Стек

**Runtime deps:**
- `vue@^3.5`
- `vue-router@^4.4`
- `pinia@^2.2`

**Dev deps:**
- `@vitejs/plugin-vue@^5.1`
- `pug@^3.0.3`
- `sass@^1.79`
- `typescript@^5.6`
- `vite@^5.4`
- `vue-tsc@^2.1`

### `vite.config.ts`

```typescript
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  css: {
    preprocessorOptions: {
      scss: { api: "modern-compiler" },
    },
  },
  server: {
    host: "0.0.0.0",
    port: 5173,
    proxy: { "/api": "http://localhost:4001" },
  },
});
```

**`host: "0.0.0.0"`** — чтобы Vite был доступен не только с localhost внутри контейнера, но и с хоста (после маппинга портов в Stage 11).
**`proxy /api → localhost:4001`** — dev-режим, чтобы фронт делал запросы на тот же origin и не упирался в CORS.

### `pnpm-workspace.yaml`

```yaml
allowBuilds:
  "@parcel/watcher": true
  esbuild: true
  vue-demi: true
```

**Проблема:** pnpm 12 блокирует postinstall-скрипты зависимостей по умолчанию. У `esbuild`, `@parcel/watcher`, `vue-demi` есть postinstall — без них сборка не работает.

**Попытки:**
1. `pnpm.onlyBuiltDependencies` в `package.json` — **не работает.** pnpm 12 больше не читает поле `pnpm` в `package.json`.
2. `onlyBuiltDependencies` в `pnpm-workspace.yaml` — **не работает.** Переименовано в `allowBuilds`.
3. `allowBuilds` в `pnpm-workspace.yaml` — **работает.**

**`pnpm approve-builds`** в неинтерактивном режиме падает: `IO error: not a terminal`.

### Скрипты `package.json`

```json
"scripts": {
  "dev": "vite",
  "build": "vue-tsc --noEmit && vite build",
  "preview": "vite preview",
  "typecheck": "vue-tsc --noEmit"
}
```

`build` включает typecheck перед Vite-сборкой — ошибки типов не пропускаются.

### Проверка

```
pnpm typecheck → OK (0 output)
pnpm build → ✓ 26 modules, dist/assets/index-*.js 63.57 kB (gzip 25.29 kB)
```

### Грабли

1. **Stale `vite.config.js`.** Первоначально создал `.js`, потом переключился на `.ts`, но `.js` не удалил. Оба файла уживаются; Vite предпочтёт `.ts`. Удалить лишний.
2. **Sass legacy JS API deprecation.** По умолчанию Vite использует старый API. Ворнинг — не ошибка, но в Dart Sass 2.0 уберут. Фикс: `css.preprocessorOptions.scss.api = "modern-compiler"`.
3. **`pnpm-workspace.yaml` нужен даже для одного пакета.** `allowBuilds` без workspace-конфига не читается. Файл создан, хотя `web/` — не монорепа.

### Уроки

1. **pnpm 12 ужесточил политику postinstall.** Whitelist через `allowBuilds` в `pnpm-workspace.yaml` — новый способ. Старые примеры (`pnpm.onlyBuiltDependencies` в `package.json`) не работают.
2. **TypeScript + Pug + SCSS работают из коробки** с `@vitejs/plugin-vue` — никаких дополнительных плагинов для Pug/SCSS не нужно.
3. **`pnpm typecheck` — отдельный скрипт.** Полезно для CI: `pnpm typecheck && pnpm build`.
4. **Проверка `ls` перед удалением — обязательна.** После переключения `.js` → `.ts` легко забыть про старый файл.

---

## 9.2. Minimal wallet connect

**Дата:** 2026-09-27
**Commit:** `d06a931`

### Решение: без `@solana/wallet-adapter-vue`

Стандартный пакет `@solana/wallet-adapter-vue@0.4.5` требует `@solana/web3.js@^1.99` как peer dependency. Но наш проект строит инструкции через **`@solana/kit@8.3.0`** (современный SDK). Смешивать legacy `web3.js` и `@solana/kit` в одном приложении — конверсия `PublicKey ↔ Address` на каждой границе, дополнительные 200+ KB bundle.

**Решение:** минимальный собственный слой поверх инжектированных провайдеров (`window.phantom.solana`, `window.solflare`, `window.solana`).

**Стандартный connect-флоу** любой Solana wallet provider:
- `provider.connect()` → `{ publicKey: { toBase58() } }`
- `provider.disconnect()`
- `provider.signTransaction(tx)` / `provider.signAndSendTransaction(tx)`
- `provider.on("connect" | "disconnect" | "accountChanged", handler)`

Ничего специфичного для Vue не нужно — обычный observable через Pinia `ref`.

### Модули

**`src/wallet/types.ts`** — `WalletProvider` интерфейс (то, что торчит из `window`) и `DetectedWallet` (обёртка для UI).

**`src/wallet/detect.ts`** — `detectWallets()`:
- Проверяет `window.phantom?.solana`, `window.solflare`, `window.backpack`, `window.solana`.
- Дедуплицирует (Phantom может торчать и как `window.phantom.solana`, и как `window.solana`).
- Возвращает массив с именем и провайдером.

**`src/stores/wallet.ts`** — Pinia store:
- `available` — найденные кошельки.
- `provider` — выбранный (после connect).
- `addr` — `Address | null` (тип из `@solana/kit`).
- `connect(wallet)` / `disconnect()` / `tryEagerConnect()`.
- `connected`, `walletName`, `connecting`, `error`.

### `tryEagerConnect`

При загрузке страницы пробуем `provider.connect({ onlyIfTrusted: true })` для каждого доступного кошелька. Если пользователь уже авторизовал этот dapp раньше — подключение мгновенное, без попапа. Если нет — silent fail, ждём клика.

**Стандартный паттерн.** Phantom, Solflare его поддерживают.

### UI

`App.vue` — хедер с двумя состояниями:
- **Подключено:** имя кошелька + сокращённый адрес (`5iM6…AKGc`) + кнопка Disconnect.
- **Не подключено:** кнопка Connect {Phantom|Solflare|Backpack} для каждого найденного. Если ни одного — сообщение "Install Phantom or Solflare".

Pug-шаблон, SCSS стили, scoped.

### Стек

```json
"@solana/addresses": "^8.3.0",
"@solana/kit": "^8.3.0",
"pinia": "^2.2.0",
"vue": "^3.5.0",
"vue-router": "^4.4.0"
```

### Проверка

```bash
pnpm typecheck → OK
pnpm build → ✓ 68 modules, index-*.js 73.39 kB (gzip 29.32 kB)
```

Bundle вырос с 63 KB до 73 KB (gzip: 25 → 29 KB) — приемлемо.

### Грабли

1. **`window.solana` и `window.phantom.solana` — один и тот же объект.** Проверяем дедупликацию через `Set<WalletProvider>`. Иначе в UI будет две кнопки "Phantom".
2. **`Address` из `@solana/kit` vs `PublicKey` из `web3.js`.** Не путать. Наш `address(pk)` конвертирует base58-строку в `Address`. Внутри kit это брендированный тип.
3. **`connect({ onlyIfTrusted: true })` кидает исключение, если пользователь не авторизован.** Обязательно оборачивать в `try/catch` и продолжать цикл по другим кошелькам.

### Уроки

1. **Не тащить библиотеку ради 50 строк кода.** wallet-adapter-vue тянет web3.js, который конфликтует с нашим kit-стеком.
2. **`window.phantom?.solana ?? window.solana` — правильный fallback.** Phantom выставляет оба, Solflare — только `window.solflare`, Backpack — `window.backpack`.
3. **`onlyIfTrusted` — тихий auto-connect.** Без него при каждой загрузке страницы появляется попап кошелька.

---

## 9.3. Codama-клиент для `zk_pool`

**Дата:** 2026-09-27
**Commit:** `70c1b6d`

### Зачем

Фронтенду нужны инструкции `pool`, `deposit`, `withdraw` и PDAs (`pool`, `vault`, `nullifierRecord`). Писать вручную аккаунт-меты, дискриминаторы, borsh-сериализацию — источник ошибок того же класса, что убил v2. **Codama** генерирует TypeScript-клиент из Anchor IDL — единый источник правды.

### Пакеты

```json
"devDependencies": {
  "@codama/nodes-from-anchor": "1.5.6",
  "@codama/renderers-js": "2.5.0",
  "codama": "1.11.0"
}
```

**Точные версии** (без `^`), как в правиле для `@solana/kit` и `@codama/*` (см. `PROJECT_CONTEXT.md` §8).

**Runtime:**
```json
"dependencies": {
  "@solana/program-client-core": "8.3.0",
  "@solana/addresses": "8.3.0",
  "@solana/kit": "8.3.0"
}
```

### Скрипт генерации

`web/scripts/generate-client.mjs`:

```javascript
import { readFileSync } from "node:fs";
import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor } from "@codama/renderers-js";

const idl = JSON.parse(readFileSync("../onchain/target/idl/zk_pool.json", "utf8"));
const codama = createFromRoot(rootNodeFromAnchor(idl));
codama.accept(renderVisitor("./src/generated/zk_pool"));
```

**Запуск:** `pnpm generate:client`

**Перегенерировать нужно** после любого изменения Anchor-программы (IDL меняется).

### Что сгенерировано

20 файлов в `web/src/generated/zk_pool/src/generated/`:

- `instructions/{pool,deposit,withdraw}.ts` — конструкторы инструкций.
- `pdas/{pool,vault,nullifierRecord}.ts` — derivation PDAs.
- `accounts/{poolState,nullifierRecord}.ts` — десериализаторы.
- `events/{depositEvent,withdrawEvent}.ts` — типы событий.
- `errors/zkPool.ts` — типы ошибок.
- `programs/zkPool.ts` — корневой program-объект.
- `index.ts` — агрегатор.

### Реэкспорт

`web/src/client.ts`:

```typescript
export * from "./generated/zk_pool/src/generated";
```

Единая точка импорта — если структура папок изменится, правим одно место.

### Грабли

1. **`@solana/program-client-core` не устанавливается автоматически.** Codama-рендерер генерирует `import { ... } from "@solana/program-client-core"`, но добавлять его в `package.json` должен пользователь. Ошибка: `Cannot find module '@solana/program-client-core'`.
   - Fix: `pnpm add @solana/program-client-core`.

2. **`@types/node` нужен.** В сгенерированных `errors/zkPool.ts` используется `process.env.NODE_ENV`. Без `@types/node` — `error TS2591: Cannot find name 'process'`.
   - Fix: `pnpm add -D @types/node`, добавить `"node"` в `tsconfig.json` → `compilerOptions.types`.

3. **Bundle size не изменился после генерации.** Потому что сгенерированный код ещё не импортируется в `App.vue`. Дерево встряхнётся только когда `client.ts` реально кто-то использует.

### Уроки

1. **Codama 1.11.0 + Anchor 1.2.0 работают.** IDL из `onchain/target/idl/zk_pool.json` совместим.
2. **`generate:client` — идемпотентный.** Запуск несколько раз даёт одинаковый вывод. Хорошо для CI.
3. **Проверять актуальность IDL.** Если Anchor-программа изменилась, а клиент не перегенерировали — фронтенд будет собирать невалидные инструкции. Возможный шаг для CI: `pnpm generate:client && git diff --exit-code`.

---

## 9.4. `@noir-lang/noir_js` в браузере

**Дата:** 2026-09-27
**Commit:** `c3537c4`

### Зачем

Депозит требует commitment = `hash_3(nullifier, secret, amount)`, который должен совпадать байт-в-байт с тем, что схема withdrawal вычислит на стороне verifier'а. Единственный способ — использовать **тот же ACIR** (`hashes.json`), что и withdrawal-схема, через `noir_js` в браузере.

**Это та же логика, что в Merkle-сервисе (Stage 6.2):** Rust и JS Poseidon2 реализации дают **разные** хеши, чем Noir builtin. Только Noir ACIR даёт идентичность.

### Пакет

```
"@noir-lang/noir_js": "1.0.0-rc.2"
```

**Точная версия** — совпадает с nargo 1.0.0-rc.2, что критично.

### Два модуля

**`src/noir/poseidon.ts`** — `poseidon2Hash(left, right)`:

```typescript
let cachedNoir: Noir | null = null;

async function loadHash2(): Promise<Noir> {
  if (cachedNoir) return cachedNoir;
  const resp = await fetch("/circuits/hash2.json");
  if (!resp.ok) throw new Error(`failed to load hash2.json: HTTP ${resp.status}`);
  const circuit = await resp.json();
  cachedNoir = new Noir(circuit);
  return cachedNoir;
}

export async function poseidon2Hash(left: string, right: string): Promise<string> {
  const noir = await loadHash2();
  const result = await noir.execute({ left: "0x" + left, right: "0x" + right });
  return (result.returnValue as string).slice(2);
}
```

**`src/noir/hashes.ts`** — `computeHashes(nullifier, secret, amount)`:

```typescript
export async function computeHashes(nullifier, secret, amount): Promise<CommitmentResult> {
  const noir = await loadHashes();
  const result = await noir.execute({
    nullifier: "0x" + nullifier,
    secret: "0x" + secret,
    amount: "0x" + amount,
  });
  const [commitment, nullifierHash] = result.returnValue as [string, string];
  return {
    commitment: commitment.slice(2),
    nullifierHash: nullifierHash.slice(2),
  };
}
```

Возвращает `{ commitment, nullifierHash }` — **bare hex**.

### Конвенция hex

- **Bare hex** (без `0x`) на границе модуля — совпадает с бэкендом, Merkle-сервисом, prover'ом.
- `0x` добавляется **только** при вызове `noir.execute(...)`, снимается с `returnValue`.

### Кэширование

`cachedNoir` — `Noir` инстанцируется **один раз** на модуль. Повторные вызовы переиспользуют circuit, не парсят JSON заново. Значимо для интерактивных форм: каждое нажатие клавиши в поле `amount` может вызвать `computeHashes` — без кэша парсили бы 30 KB JSON каждый раз.

### ACIR-ы в `web/public/circuits/`

| Файл | SHA-256 | Размер |
|---|---|---|
| `hash2.json` | `27c1937b…37c6` | 30 208 B |
| `hashes.json` | `ca81b137…be9` | 31 403 B |
| `withdrawal.json` | `29ac2e67…91db` | 41 414 B |

Синхронизируются через `sync-circuits --apply`. Папка gitignored.

**Проверено:** все три хеша совпадают с ожидаемыми перед Stage 9.4.

### Грабли

**`fetch("/circuits/hash2.json")` зависит от Vite dev-server.** В production-сборке Vite копирует `public/*` в `dist/*`. Путь `/circuits/hash2.json` работает и там. **Не** нужно перемещать в `src/`.

### Уроки

1. **Один ACIR — два потребителя.** Merkle-сервис и браузер используют **тот же** `hash2.json`. Расхождение невозможно по построению.
2. **`returnValue` — tuple для `hashes`, строка для `hash2`.** Типизация через TypeScript cast — `as [string, string]` и `as string`. Проверено в Stage 6.2 inline-тестом.
3. **Кэш `Noir` инстанса обязателен.** `new Noir(circuit)` дорого (парсинг ACIR, init ACVM). Один раз — на всё время жизни страницы.

---

## 9.5. Типизированный API-клиент

**Дата:** 2026-09-27
**Commit:** `d28f530`

### Зачем

Фронтенд общается с бэкендом (4001) через пять эндпоинтов. Без типов легко ошибиться в именах полей (`public_witness` vs `publicWitness`), забыть про `null` в `root`, перепутать форматы hex/base64.

**Решение:** два файла — `types.ts` (интерфейсы) и `client.ts` (обёртки над fetch).

### `types.ts` — контракты

**Базовые типы:**
- `Hex = string` — bare hex, без `0x`.
- `Base64 = string` — стандартный алфавит с padding.

**Response-типы:**

```typescript
HealthResponse      // {status, db, version}
CommitmentsResponse // {pool_address, count, commitments: CommitmentRecord[]}
RootResponse        // {pool_address, root: Hex | null}
ProofResponse       // {pool_address, leaf_index, proof: Hex[20], is_even: bool[20]}
WithdrawResponse    // {proof: Base64, public_witness: Base64}
```

**Request-тип:**

```typescript
WithdrawRequest     // 10 полей, все Hex (кроме is_even: bool[])
```

**Ошибка:**

```typescript
ApiError            // {error: string, [key: string]: unknown}
```

Индексная сигнатура `[key: string]: unknown` — потому что бэкенд добавляет контекстные поля (`prover_status`, `prover_body`) к базовому `{error}`.

### `client.ts` — обёртки

**Общий хелпер `request<T>(path, init?)`:**

1. `fetch` с JSON-заголовком.
2. Читает тело как текст (не `.json()`, чтобы корректно обработать пустой ответ).
3. Если тело не пустое — `JSON.parse`, иначе `body = null`.
4. Если `!resp.ok` — бросает `ApiClientError` с телом и статусом.
5. Возвращает `T`.

**`ApiClientError`** — свой класс ошибки с `status` и `body`, чтобы вызывающий код мог различать 400 / 502 / 500.

**Экспортируемые функции:**

```typescript
getHealth()
getCommitments(poolAddress?)
getRoot(poolAddress?)
getProof(leafIndex, poolAddress?)
postWithdraw(req)
```

`poolAddress` опционален во всех: если не указан, бэкенд использует свой `POOL_ADDRESS` из env.

### BASE = ""

**Пустая строка.** В dev-режиме Vite проксирует `/api/*` на `localhost:4001` (`vite.config.ts`). В production тот же origin — reverse proxy должен сделать то же самое. **Никаких абсолютных URL.**

### Обработка ошибок

**Три класса ошибок:**

| Класс | Когда | Что в теле |
|---|---|---|
| `ApiClientError` (400) | `WithdrawRequest` невалиден | `{error: "<validation msg>"}` |
| `ApiClientError` (502) | prover/merkle недоступен | `{error: "prover unreachable"}` |
| `ApiClientError` (500) | DB error, internal | `{error: "db query failed"}` |

Все — через единый `ApiClientError`. Вызывающий код может смотреть на `e.status` для деталей.

### Грабли

1. **`resp.json()` не работает для пустого тела.** `Content-Length: 0` → исключение. Поэтому читаем через `resp.text()` и парсим вручную.

2. **`root: null` в `RootResponse`.** TypeScript строгий — `Hex | null`. Клиент должен проверять.

3. **Никаких `throw` для HTTP-ошибок, кроме `!resp.ok`.** 200 с `{"error": "..."}` в теле — невозможно; бэкенд всегда возвращает 200 только с валидными данными.

### Уроки

1. **Один хелпер — все эндпоинты.** Никаких пяти почти одинаковых `fetch`-обёрток.
2. **`ApiClientError` — класс, не просто `Error`.** Позволяет `catch (e) { if (e instanceof ApiClientError && e.status === 400) ... }`.
3. **Явные Response-типы** — самодокументируемый контракт между фронтом и бэком. Если бэкенд меняет поле — TypeScript укажет на все места использования.

---

## 9.6a. Бэкенд: `POST /api/root-preview`

**Дата:** 2026-09-27
**Commit:** `dc0a59b`

### Зачем

Инструкция `deposit` на Solana принимает **три аргумента**: `commitment`, `new_root`, `amount`. `new_root` — Merkle-корень **после** добавления нового commitment'а. On-chain инструкция **не проверяет** корректность `new_root` (см. `deposit.rs`, раздел "Trust model") — это ответственность клиента.

**Проблема:** если фронтенд вычислит `new_root` неправильно (несовпадающий алгоритм, off-by-one, stale commitment list) — корень уйдёт в `PoolState.roots[]`, и все последующие withdraw-пруфы будут валидны только относительно этого (возможно, неверного) корня. Это ровно тот класс баги, что убил v2.

### Варианты

- **(a)** TS-порт `merkle.js` во фронтенде. Дублирование логики — риск расхождения.
- **(b)** Новый endpoint `POST /api/root-preview` на бэкенде, проксирующий Merkle-сервис `/root`.
- **(c)** Фронтенд напрямую дёргает Merkle-сервис на 4003. Нужен второй Vite proxy.

**Выбрали (b).** Ноль дублирования — Merkle-сервис уже содержит проверенную реализацию (Stage 6.3, кросс-проверки пройдены). Бэкенд уже проксирует `/api/proof` → Merkle `/proof` тем же паттерном.

### Реализация

**`services/backend/src/main.rs`:**

- Новый маршрут в `read_routes`: `.route("/api/root-preview", post(post_root_preview))`.
- Хендлер `post_root_preview`:
  1. Принимает `{commitments: [hex]}`.
  2. `POST {MERKLE_URL}/root` с тем же телом.
  3. Возвращает `{root: "<hex>"}` (или ошибку с 502).

### Формат

**Request:**
```json
{"commitments": ["09d9d188...", "…"]}
```

**Response:**
```json
{"root": "266ab002e3bea99195ef395e33dee02a83674155eaee9d1dc84019af70714967"}
```

### Проверка

```
POST /api/root-preview {"commitments":["09d9d188784ab20199a5eb7267ce27765a374ce6bc672deee9eeac9ba90b80fc"]}
→ {"root":"266ab002e3bea99195ef395e33dee02a83674155eaee9d1dc84019af70714967"}
```

Совпадает с эталоном из Stage 6.3. Транспорт бэкенд → Merkle корректен.

### Rate limiting

Попадает под **read** rate-limit (60/min), т.к. зарегистрирован в `read_routes`. Правильно: preview — read-only операция, не создаёт side-effects.

### Уроки

1. **Не портировать логику через язык без нужды.** Merkle-сервис уже написан на JS и проверен. TS-порт мог бы дать расхождение.
2. **Проксирование вместо дублирования — архитектурный принцип.** Бэкенд уже проксирует `/api/proof` → Merkle, `/api/withdraw` → Prover. `/api/root-preview` — третий случай.
3. **`new_root` в `deposit` — открытая поверхность для ошибок.** On-chain не проверяется (документировано в `deposit.rs`). Единственная защита — клиент использует проверенную реализацию. Ещё один аргумент за (b).

---

## 9.6b. Логика депозита

**Дата:** 2026-09-27
**Commit:** `0a086a5`

### Что сделано

Пять файлов:

| Файл | Роль |
|---|---|
| `src/constants.ts` | Program ID, Pool PDA, MIN_DEPOSIT_AMOUNT, TREE_DEPTH |
| `src/api/client.ts` | + `postRootPreview` (проксирует Merkle `/root`) |
| `src/deposit/generateNote.ts` | Генерация 4 секретов + commitment |
| `src/wallet/kitSigner.ts` | `makeNoopSigner(address)` — заглушка для Codama |
| `src/deposit/useDeposit.ts` | Композабл: полный flow |

### `constants.ts`

```typescript
ZK_POOL_PROGRAM_ID  = "8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm"
VERIFIER_PROGRAM_ID = "5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ"
POOL_PDA            = "B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf"
TREE_DEPTH          = 20
MIN_DEPOSIT_AMOUNT  = 1_000_000n  // 0.001 SOL
LAMPORTS_PER_SOL    = 1_000_000_000n
```

**Источник значений:** `onchain/programs/zk_pool/src/constants.rs`. Синхронизируется в CI (Stage 12).

### `generateNote.ts`

**`DepositNote`:**
```typescript
{
  nullifier:      string;  // bare hex, 64 chars
  secret:         string;  // bare hex, 64 chars
  noteSecret:     string;  // bare hex, 64 chars
  amount:         string;  // bare hex, 64 chars (lamports, padded)
  commitment:     string;  // derived: hash_3(nullifier, secret, amount)
  nullifierHash:  string;  // derived: hash_1(nullifier)
}
```

**`randomFieldHex()`** — Web Crypto API, 32 байта, маска верхних 2 бит (`bytes[0] &= 0x3f`) — значение остаётся ниже простого числа BN254 (2^254).

**`generateNote(amountLamports)`** — генерит три секрета + гексовую сумму, вызывает `computeHashes()` из `noir/hashes.ts` (Stage 9.4).

**Критично:** потеря note = потеря средств. On-chain восстановление невозможно. UI должен заставлять пользователя сохранять note.

### `kitSigner.ts` — почему заглушка

Codama-инструкция `getDepositInstructionAsync` требует `InstructionSignerInput` для аккаунта `depositor` — то есть `TransactionSigner` или `AccountSignerMeta { address, role, signer }`. У нас есть **инжектированный** wallet provider с методом `signAndSendTransaction(wire_tx)`, но не `TransactionSigner` в смысле `@solana/kit`.

**Два подхода:**

- **(A)** Построить полный `TransactionSigner`-adapter вокруг wallet provider'а (методы `signTransactions`, `address`, …).
- **(B)** Отдать `makeNoopSigner(address)` в Codama (только для построения instruction'а), а **подписание** сделать через `provider.signAndSendTransaction(base64_wire_tx)`.

**Выбрали (B).** Меньше кода, соответствует тому, как реально работает Phantom/Solflare: они подписывают **сериализованную** транзакцию, а не kit-объект.

**`makeNoopSigner`** возвращает `{ address, signTransactions: () => throw }` — type-safe заглушка. Если её метод вызовется, ошибка будет громкой.

### `useDeposit.ts` — pipeline

```
1. generateNote(amountLamports)
   └─> computeHashes() — noir_js в браузере
2. getCommitments() → список текущих commitments
3. Список + note.commitment → postRootPreview()
   └─> backend → Merkle /root → новый корень
4. getDepositInstructionAsync({depositor: noopSigner, commitment, newRoot, amount})
5. rpc.getLatestBlockhash()
6. createTransactionMessage + setLifetime + appendInstruction
7. compileTransaction → getBase64EncodedWireTransaction
8. provider.signAndSendTransaction(base64)
9. (fone-and-forget) confirmSignature()
```

**Composable API:**
```typescript
const { loading, error, result, deposit } = useDeposit();
await deposit(1_000_000n);
```

`loading` / `error` / `result` — реактивные refs.

### Критично: где вычисляется `new_root`

**Не во фронтенде.** Вызов `postRootPreview()` → backend → Merkle-сервис `/root`. Это **единственный** вычислитель корней во всей системе.

On-chain инструкция `deposit` **не проверяет** корректность `new_root` (см. `deposit.rs`, "Trust model"). Единственная защита — клиент использует проверенную реализацию. Мы это делаем.

### Грабли

1. **Codama требует `TransactionSigner` для signer-аккаунтов.** `{address, role}` — недостаточно, `AccountSignerMeta` требует поле `signer`. Обход — `makeNoopSigner`.
2. **`amountLamports` в `DepositNote` — уже hex**, не decimal. `generateNote` конвертирует при создании. Дальше по пайплайну всё hex.
3. **`postRootPreview` не был в `api/client.ts`** при первом прогоне — забыли при рефакторинге. Добавлено по месту.

### Уроки

1. **Один вычислитель корней — Merkle-сервис.** Никаких TS-портов `merkle.js`.
2. **Codama-инструкции не подписывают.** Они только **конструируют** instruction. Подпись — отдельный шаг, всегда через wallet provider.
3. **Noop signer — рабочий паттерн** для "build без подписи". Используется везде, где нужно собрать instruction, но подпись отложена.

---

## 9.6c. UI экран депозита

**Дата:** 2026-09-27
**Commit:** `fae22c4`

### Что сделано

- `src/components/DepositForm.vue` — форма ввода суммы + результат.
- `src/App.vue` — обновлён: показывает `DepositForm` при подключённом кошельке.

### `DepositForm.vue`

**Поля:**
- `Amount (SOL)` — text input, дефолт `0.01`, минимум из `MIN_DEPOSIT_AMOUNT`.
- Кнопка `Deposit` (disabled, пока amount < min или кошелёк не подключён).

**После успеха:**
- Signature с ссылкой на Explorer (devnet).
- Commitment, new_root — полные hex.
- **Отдельный блок "⚠️ Save this note":** все 4 секрета (`nullifier`, `secret`, `note_secret`, `amount` в hex).
- Кнопка "Copy note as JSON" — копирует всё, включая `tx_signature` и `pool_pda`.

**Критично:** потеря note = потеря средств. Визуально это выделено жёлтым блоком с красным предупреждением.

### Валидация

```typescript
canDeposit = wallet.connected
          && !deposit.loading
          && amount > 0
          && amountLamports >= MIN_DEPOSIT_AMOUNT
```

**Проверяется на фронте до вызова** — быстрый отказ, экономия RPC-вызовов и попапа кошелька.

### `App.vue`

**Три состояния:**
1. Кошелёк не подключён → "Connect a wallet to deposit."
2. Кошелёк подключён → `DepositForm`.
3. Ошибка кошелька → сообщение под main.

### Bundle size после 9.6c

```
noirc_abi_wasm_bg.wasm      788.51 kB
acvm_js_bg.wasm           3,047.92 kB
index.js                    165.86 kB (gzip 58.93 kB)
```

**3.8 MB WASM** — цена `noir_js` в браузере. Это `acvm_js` — ACIR Virtual Machine, компилируемая из Rust в WASM. Критично для byte-identical Poseidon2 хешей.

**Дальнейшие оптимизации** (не сейчас):
- Lazy load WASM только когда нужен депозит/вывод.
- Предзагрузка через `<link rel="preload">`.

### Уроки

1. **Форма должна явно говорить о важности note.** Просто показать hex-строки недостаточно. `⚠️ Loss of this note means loss of funds.`
2. **`copy as JSON`** — практично: пользователь сохраняет в файл одной кнопкой, а не собирает поля руками.
3. **`shortSig`** в UI — `abc…xyz` вместо 88 символов base58. Полный signature — в Explorer-ссылке.
4. **Проверка min amount на фронте** — UX + экономия. Но **on-chain проверка тоже есть** (`DepositBelowMinimum`), это последняя линия защиты.

---

## 9.7a. BN254 reduction (recipient)

**Дата:** 2026-09-27
**Commit:** `c8ef710`

### Зачем

Solana pubkey — 32 случайных байта. BN254 поле имеет порядок ≈ 2^254, а pubkey — 256 бит. Значит, ~75% pubkey'ев **больше** модуля. Схема withdrawal ожидает `recipient` как **элемент поля** — значение должно быть < prime.

On-chain модуль `encoding.rs::encode_public_inputs` уже делает reduction при кодировании публичных входов:

```rust
let recipient_reduced = reduce_to_field(&recipient.to_bytes());
```

Но `recipient_binding = hash_2(note_secret, recipient)` вычисляется **в браузере** (Stage 9.4). Значит, фронтенд должен использовать **тот же** reduced-`recipient`, что и on-chain — иначе proof не пройдёт верификацию.

### Проверка API `noir_js`

Тест перед написанием кода:

```
noir.execute({left: BN254_PRIME + 1, right: 0})
→ Error: The value passed for parameter `left` is invalid:
  Expected witness values to be integers, but `30644e72...00000002` failed with `invalid digit found in string`
```

**Вывод:** `noir_js` **отбрасывает** значения ≥ prime. Reduction обязателен на фронтенде.

### `reduce.ts` — TS-порт `encoding.rs::reduce_to_field`

**Алгоритм:** повторное вычитание модуля.

- `isGe(a, b)` — сравнение 32-байтовых BE-чисел.
- `subBe(a, b)` — вычитание с borrow.
- `reduceToField(input)` — 5 итераций вычитания (максимум нужно 4, т.к. 2^256 / p ≈ 4.006; пятая — safety margin, как в Rust).
- `reduceToFieldHex` — hex-обёртка.
- `bytesToHex`, `hexToBytes` — утилиты.

**Константа `BN254_PRIME_BE`** — скопирована из `encoding.rs`, 32 байта.

### Тесты

4 теста, все проходят:

```
✔ zero stays zero
✔ prime reduces to zero
✔ max value reduces below prime
✔ small value unchanged
```

**Те же кейсы, что в Rust-тестах `encoding.rs`.** Это кросс-языковая проверка — если TS-порт разойдётся с Rust, CI (Stage 12) поймает.

### Запуск TS-тестов

```
node --experimental-strip-types --test src/withdraw/reduce.test.ts
```

Node 24 умеет исполнять `.ts` без сборки (type stripping). **Требует** `allowImportingTsExtensions: true` в `tsconfig.json` и явного `.ts` в импортах:

```typescript
import { reduceToField } from "./reduce.ts";
```

**Конфликт:** `vue-tsc` ругается на `.ts` без флага. Флаг добавлен в `tsconfig.json`.

### Грабли

1. **`noir_js` не делает implicit reduction.** Ошибка `invalid digit found in string` — невнятная, но означает "значение не является валидным Field".
2. **TypeScript 5.9 ужесточил `Uint8Array` generic.** `Uint8Array<ArrayBufferLike>` не присваивается `Uint8Array<ArrayBuffer>`. Фикс: явные аннотации `Uint8Array<ArrayBuffer>`.
3. **`--experimental-strip-types` требует `.ts` в импортах.** Node не делает auto-extension resolution как bundler.

### Уроки

1. **Кросс-языковые порты требуют кросс-языковых тестов.** TS-порт `reduce_to_field` проверяется теми же кейсами, что Rust. Если разойдётся — упадёт тест, а не верификация on-chain.
2. **Один алгоритм — два применения.** Rust использует `reduce_to_field` внутри CPI-вызова, TS — при вычислении `recipient_binding`. Оба обязаны давать одинаковый результат.
3. **`--experimental-strip-types` — практичный способ тестировать TS.** Без Vitest/Jest, только встроенный Node test runner.

---

## Что дальше

- **9.7** — UI: вывод.
- **9.8** — Финальный чекпоинт.
