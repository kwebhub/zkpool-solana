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

## Что дальше

- **9.2** — Solana wallet adapter (Phantom / Solflare).
- **9.3** — Codama-клиент для `zk_pool`.
- **9.4** — `@noir-lang/noir_js` — commitments и nullifier_hash в браузере.
- **9.5** — API-клиент (типизированные обёртки над бэкендом).
- **9.6** — UI: депозит.
- **9.7** — UI: вывод.
- **9.8** — Финальный чекпоинт.
