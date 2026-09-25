# 06. Merkle-сервис (Node.js + Fastify)

> **Этап 6** проекта zkpool-solana.
> Сервис на порту 4003, предоставляющий Poseidon2-хеширование и операции над Merkle-деревом бэкенду и фронтенду.

---

## TL;DR

Merkle-сервис — тонкая JS-обёртка над `@noir-lang/noir_js`. Он выполняет **тот же ACIR** (`hash2.json`), что и схема withdrawal, поэтому выдаёт **байт-в-байт** идентичные хеши. Rust и JS реализации Poseidon2 дают разные результаты — единственный способ гарантировать идентичность — использовать Noir-схему через JS.

**Порты:** 4003
**Стек:** Node.js 24.21.0 + Fastify 5.12.5 + `@noir-lang/noir_js@1.0.0-rc.2`

---

## 6.1. `package.json` + зависимости

**Дата:** 2026-09-25
**Commit:** `4142789`

### Зачем

Прежде чем писать код — зафиксировать точные версии зависимостей. Правило проекта (раздел 8 `PROJECT_CONTEXT.md`): для Solana/Noir JS-стека пинить **точные** версии, без `^` и `~`. Иначе semver-дрейф ломает воспроизводимость.

### Что сделано

Создан `services/merkle/package.json`:

```json
{
  "name": "zkpool-merkle",
  "version": "0.1.0",
  "private": true,
  "description": "Merkle service for zkpool-solana — Poseidon2 hashing via @noir-lang/noir_js",
  "type": "module",
  "engines": { "node": ">=24.0.0" },
  "scripts": {
    "start": "node src/server.js",
    "test": "node --test test/"
  },
  "dependencies": {
    "@fastify/cors": "11.3.0",
    "@noir-lang/noir_js": "1.0.0-rc.2",
    "fastify": "5.12.5"
  }
}
```

### Проверка версий через npm (до записи в файл)

Правило 0.10: не угадывать API/версии. Проверено командами:

```bash
npm view @noir-lang/noir_js@1.0.0-rc.2 version   # → 1.0.0-rc.2
npm view fastify version                          # → 5.12.5
npm view @fastify/cors version                    # → 11.3.0
npm view @noir-lang/noir_js@1.0.0-rc.2 dependencies --json
```

Транзитивные зависимости `noir_js@1.0.0-rc.2`:
- `@noir-lang/acvm_js@1.0.0-rc.2`
- `@noir-lang/types@1.0.0-rc.2`
- `@noir-lang/noirc_abi@1.0.0-rc.2`
- `pako@^3.0.1`

Все зафиксированы на rc.2 — semver-дрейфа не будет.

### Установка

```bash
pnpm install
```

Установлено 56 пакетов за 3.8 с.

### Зачем `type: "module"`

`@noir-lang/noir_js` — ESM-only (`"type": "module"`, `lib/*.mjs`). Без `"type": "module"` в нашем `package.json` Node попытается загрузить его как CommonJS и упадёт.

### Грабли

- **pnpm переформатирует `package.json` при `install`.** Ключ `engines` переехал в конец файла. Не баг — нормализация. Не перезаписывать вручную.

---

## 6.2. `src/poseidon.js`

**Дата:** 2026-09-25
**Commit:** `d021210`

### Зачем

Обёртка над `noir_js`, которая:
- Загружает `hash2.json` **один раз** при инициализации модуля (кэш).
- Экспортирует `poseidon2Hash(left, right)` → hex-строку.

### Проверка API перед написанием кода

Правило 0.10: маленькие шаги, читать исходники пакета.

**Что смотрели:**

```bash
ls node_modules/@noir-lang/noir_js/lib/
cat node_modules/@noir-lang/noir_js/lib/index.d.ts
cat node_modules/@noir-lang/noir_js/lib/program.d.ts
```

**Что выяснили:**

TypeScript-сигнатуры `Noir`:

```typescript
export declare class Noir {
    constructor(circuit: CompiledCircuit);
    init(): Promise<void>;
    execute(inputs: InputMap, foreignCallHandler?: ForeignCallHandler): Promise<{
        witness: Uint8Array;
        returnValue: InputValue;
    }>;
}
```

- `Noir.prototype` содержит ровно: `constructor`, `init`, `execute`.
- `returnValue` может быть **строкой** (одно поле) или **массивом строк** (кортеж). Проверено вживую.

### Кросс-проверка (ключевой момент)

Перед написанием модуля — проверено, что `noir_js` воспроизводит **реальные** значения из `Prover.toml` (Stage 3.5):

| Значение | JS (`noir_js`) | Эталон | Совпадение |
|---|---|---|---|
| `nullifier_hash` | `0x1412cc9d…e6e4` | `0x1412cc9d…e6e4` (Prover.toml) | ✅ |
| `commitment` | `0x09d9d188…80fc` | — (derived) | — |
| `root` | `0x1e8508c3…c6b2` | `0x1e8508c3…c6b2` (Prover.toml) | ✅ |

**Это доказывает основной принцип Stage 6:** JS-`noir_js` даёт байт-в-байт идентичные результаты со схемой Noir и с публичными входами Sunspot-пруфа.

### Код `src/poseidon.js`

```javascript
import { Noir } from "@noir-lang/noir_js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const CIRCUIT_PATH = join(__dirname, "..", "circuits", "hash2.json");

const circuit = JSON.parse(readFileSync(CIRCUIT_PATH, "utf8"));
const noir = new Noir(circuit);

export async function poseidon2Hash(left, right) {
  const result = await noir.execute({ left, right });
  return result.returnValue;
}
```

### Проверка

```bash
node --input-type=module -e "import { poseidon2Hash } from './src/poseidon.js'; console.log(await poseidon2Hash('0x1', '0x2'));"
```

Вывод:

```
0x299bfccd7daf3c917e51291383929049ec0eaed800af245056cbf135f7dea636
```

Совпадает с inline-тестом до написания модуля.

### Полная кросс-проверка через модуль

```bash
node --input-type=module -e "
import { poseidon2Hash } from './src/poseidon.js';
const commitment = '0x09d9d188784ab20199a5eb7267ce27765a374ce6bc672deee9eeac9ba90b80fc';
let current = commitment;
for (let i = 0; i < 20; i++) {
  const sibling = i === 0 ? '0x07b5bad595e238e3' : '0x00';
  current = await poseidon2Hash(current, sibling);
}
console.log('computed root:', current);
"
```

Вывод: `0x1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2` ✅

### Грабли

- **Длинные многострочные `node -e` скрипты ломаются в интерактивном шелле контейнера.** Терминал искажает переносы строк, склеивает команды, теряет кавычки. Симптом: видно на экране «мешанину», но программа всё равно отрабатывает.

  **Решение:** писать тестовые скрипты в файлы (`test/*.js`), запускать `node file.js`. Так и сделаем в 6.5 для `node --test`.

---

## Что дальше

- **6.3** — `src/merkle.js`: построение дерева, root, proof.
- **6.4** — `src/server.js`: Fastify routes.
- **6.5** — тесты (`node --test`).
- **6.6** — запуск + smoke test.
- **6.7** — финальный чекпоинт.

**Ключевое ограничение:** формат ответа должен совпадать с тем, что ожидает `tree.rs` — `{"hash": "<hex>"}`.
