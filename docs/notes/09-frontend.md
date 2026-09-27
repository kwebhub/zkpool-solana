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

## Что дальше

- **9.5** — API-клиент (типизированные обёртки над бэкендом).
- **9.6** — UI: депозит.
- **9.7** — UI: вывод.
- **9.8** — Финальный чекпоинт.
