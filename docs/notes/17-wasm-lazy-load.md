# 17. Lazy-load `@noir-lang/noir_js` — снять 3.84 MB WASM с initial bundle

> **Stage 16.1.**
> Динамический `import()` для `noir_js` в `web/src/noir/{poseidon,hashes}.ts`.
> Первое сокращение начального бандла frontend'а.

---

## TL;DR

Раньше `@noir-lang/noir_js` импортировался статически в `noir/poseidon.ts` и
`noir/hashes.ts`. По цепочке `poseidon.ts → generateNote.ts → useDeposit.ts → App.vue`
два WASM-бинаря (`acvm_js_bg.wasm` 3.05 MB + `noirc_abi_wasm_bg.wasm` 789 KB)
попадали в initial bundle. Сайт не показывал первую страницу, пока браузер
не скачает ~3.84 MB WASM.

Fix: `import("@noir-lang/noir_js")` **внутри** `loadHash2()` / `loadHashes()`,
только при первом вызове `poseidon2Hash()` / `computeHashes()`.
Vite автоматически выделяет `noir_js` в отдельный chunk. WASM грузится
**только** при клике Deposit или Withdraw.

**Результат:** `dist/assets/index-*.js` — два чанка (40.69 KB + 149.73 KB).
WASM — отдельные файлы, запрашиваются по требованию.

---

## 1. Симптом

### 1.1. Размер бандла

```
dist/assets/noirc_abi_wasm_bg-DX-Hyssd.wasm    788.51 kB
dist/assets/acvm_js_bg-hM8J3cqa.wasm         3,047.92 kB
dist/assets/index-*.js                         177.46 kB (gzip 62.28 kB)
```

Итого ~3.84 MB WASM в `dist/assets/`. Пользователь качает их при **первом**
открытии страницы, ещё до клика на любую кнопку.

### 1.2. Причина

Импортная цепочка:

```
App.vue
  → DepositForm.vue
    → useDeposit.ts
      → generateNote.ts
        → noir/hashes.ts
          → import { Noir } from "@noir-lang/noir_js"   ← статический
```

Аналогично `WithdrawForm.vue` → `useWithdraw.ts` → `buildWitness.ts` → `noir/poseidon.ts`.

`@noir-lang/noir_js` при инициализации подтягивает `acvm_js` и `noirc_abi_wasm` —
wasm-pack-модули, у которых есть `.wasm`-ассеты. Vite включает их в initial
bundle, потому что не знает, что они нужны **только** при вызове функции.

---

## 2. Фикс

### 2.1. `web/src/noir/poseidon.ts`

Было:

```ts
import { Noir } from "@noir-lang/noir_js";

let cachedNoir: Noir | null = null;

async function loadHash2(): Promise<Noir> {
  if (cachedNoir) return cachedNoir;
  const resp = await fetch("/circuits/hash2.json");
  if (!resp.ok) throw new Error(...);
  const circuit = await resp.json();
  cachedNoir = new Noir(circuit);
  return cachedNoir;
}
```

Стало:

```ts
import type { Noir } from "@noir-lang/noir_js";   // ← type-only, ничего не тянет

let cachedNoir: Noir | null = null;

async function loadHash2(): Promise<Noir> {
  if (cachedNoir) return cachedNoir;
  const [noirMod, resp] = await Promise.all([
    import("@noir-lang/noir_js"),                 // ← dynamic
    fetch("/circuits/hash2.json"),
  ]);
  if (!resp.ok) throw new Error(...);
  const circuit = await resp.json();
  cachedNoir = new noirMod.Noir(circuit);
  return cachedNoir;
}
```

**Ключевое:**
- `import type` — стирается TypeScript'ом при компиляции, рантайм-импорта не остаётся.
- `import("@noir-lang/noir_js")` внутри `loadHash2()` — вызывается **только** при
  первом обращении к `poseidon2Hash()`.
- `Promise.all([import(...), fetch(...)])` — параллельно грузим WASM и ACIR JSON.

### 2.2. `web/src/noir/hashes.ts`

То же самое.

### 2.3. `web/vite.config.ts` — удаление `vite-plugin-top-level-await`

Было:

```ts
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [vue(), wasm(), topLevelAwait()],
  ...
});
```

Стало:

```ts
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [vue(), wasm()],
  ...
});
```

**Почему:** `vite-plugin-top-level-await` был нужен, когда `noir_js` попадал в
initial bundle и Vite не умел обработать TLA. Теперь `noir_js` — отдельный
динамически загружаемый chunk, TLA внутри него Vite обрабатывает нативно.

**Ошибка до удаления:**

```
[vite-plugin-top-level-await] missing field `type`
  at Compiler.printSync (@swc/core/index.js:257)
  ...
```

Плагин не может обработать код после того, как Vite разбил его на чанки.

---

## 3. Проверка

### 3.1. Build

```bash
cd /home/ubuntu/web && pnpm typecheck && pnpm build
```

```
✓ 116 modules transformed.
dist/assets/noirc_abi_wasm_bg-DX-Hyssd.wasm    788.51 kB
dist/assets/acvm_js_bg-hM8J3cqa.wasm         3,047.92 kB
dist/assets/index-D_TNDXqA.css                   5.75 kB │ gzip:  1.19 kB
dist/assets/index-BTSZRHO7.js                   40.69 kB │ gzip: 12.32 kB
dist/assets/index-BTa_IwNy.js                  149.73 kB │ gzip: 53.42 kB
✓ built in 1.24s
```

**Два JS-чанка:** 40.69 KB (main) + 149.73 KB (lazy chunk with noir_js).

### 3.2. Network при первом load-е (F5)

В DevTools → Network → All, клик 🚫, F5.

**Наблюдения:** нет запросов к `hash2.json`, `hashes.json`, `acvm_js_bg.wasm`,
`noirc_abi_wasm_bg.wasm`. Загружаются только `main.ts`, `App.vue`,
`wallet.ts`, `@solana_kit.js`, `detect.ts`, `DepositForm.vue`, `WithdrawForm.vue`,
`useDeposit.ts`, `useWithdraw.ts` и т.д.

### 3.3. Network при клике Deposit

Disable cache → 🚫 → клик Deposit.

Запросы, которых **не было** при первом load-е:
- `hashes.json` (`200`, 31.7 KB)
- `index.mjs` (noir_js, 3.7 KB)
- `acvm_js.js` (`200`, 190 KB)
- `noirc_abi_wasm.js` (`200`, 95.2 KB)
- `program.mjs` (`200`, 8.4 KB)
- `witness_generation.mjs` (`200`, 19.4 KB)
- `base64_decode.mjs`, `debug.mjs`, `pako.mjs`
- `acvm_js_bg.wasm` (`200`, **3 048 KB**)
- `noirc_abi_wasm_bg.wasm` (`200`, **789 KB**)

**Вывод:** ~3.84 MB WASM + ~5 KB ACIR + ~350 KB JS запрашиваются **только**
при клике Deposit. Это подтверждает работу lazy-import.

---

## 4. Что не изменено

- Логика `poseidon2Hash()` и `computeHashes()` — те же сигнатуры и тот же
  результат.
- Конвенция bare hex — без изменений.
- Все вызовы `computeHashes()` в `deposit/generateNote.ts` и `withdraw/buildWitness.ts`
  работают без изменений.
- Версии зависимостей — `@noir-lang/noir_js@1.0.0-rc.2`, `@noir-lang/acvm_js@1.0.0-rc.2`,
  `@noir-lang/noirc_abi@1.0.0-rc.2` — те же.

---

## 5. Уроки

1. **`import type` вместо `import` для типов.** TypeScript стирает `import type`
   при компиляции. Runtime-импорт не остаётся, WASM не подтягивается.

2. **`import()` внутри async-функции — единственный способ lazy-load в Vite.**
   Top-level `import()` всё равно попадёт в main chunk.

3. **`Promise.all([import(...), fetch(...)])` параллелит загрузку WASM и ACIR.**
   Без этого ACIR ждёт WASM, потом загружается сам.

4. **`vite-plugin-top-level-await` несовместим с lazy-chunk'ами в Vite 5.4.**
   Ошибка `missing field 'type'` из `@swc/core`. Удаление плагина не ломает
   сборку — Vite сам умеет TLA в отдельных чанках.

5. **Первый load и первый клик — разные измерения.** Проверять **оба**:
   - При F5 не должно быть `hash2.json` / `hashes.json` / `.wasm`.
   - При клике Deposit должны появиться **все** — ACIR, `.js`-обёртки, `.wasm`.

6. **`Disable cache` в DevTools обязателен.** Без него второй клик покажет
   `304 Not Modified` вместо `200` — размеры будут видны (`3 048 KB`), но
   статус введёт в заблуждение.

---

## 6. Файлы, которых касается заметка

- `web/src/noir/poseidon.ts`
- `web/src/noir/hashes.ts`
- `web/vite.config.ts`

**Commit:** TBD.
**Checkpoint:** `.checkpoints/16.1-wasm-lazy/` (TBD).

---

## 7. Что дальше

- **Stage 16.2** — Solflare / Backpack support. Проверить, тот же ли формат
  `signAndSendTransaction` они ожидают.
- **Stage 17** — по §0.

См. также:
- `docs/notes/16-phantom-compat.md` — fix 16 (Phantom wire format + legacy single-notes).
- `docs/PROJECT_CONTEXT.md` §8.12 — Known pitfalls.
