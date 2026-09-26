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

## 6.3. `src/merkle.js` — построение дерева, root, proof

**Дата:** 2026-09-25
**Commit:** (будет после коммита)

### Зачем

Три функции:
- `computeEmptyHashes()` — каскад пустых поддеревьев.
- `computeRoot(commitments)` — корень по списку листьев.
- `computeProof(commitments, leafIndex)` — Merkle-пруф для листа.

### Соглашения (должны совпадать с `services/backend/src/tree.rs`)

- `empty[0] = 0x00…00` (32 нулевых байта)
- `empty[d+1] = hash_2(empty[d], empty[d])`
- Лист с индексом `i` на уровне `d` — ЛЕВЫЙ ребёнок, если бит `d` числа `i` равен 0.
- Корень — на уровне `DEPTH` (20).

### Ключевой момент индексации (главная ошибка)

**Первая версия кода:** использовала `empty[TREE_DEPTH - d]` как хеш пустого сиблинга на уровне `d`.

**Это было неверно.** Симптомы на `check_merkle.js`:

```
computeRoot([commitment]) = 0x2f06bf9c…5316   ← не совпадает
manual fold                = 0x266ab002…4967

proof[0]  = 0x3039bcb2…e387  (= empty[20])   expected empty[19]  ← off by one
proof[19] = 0x18dfb8dc…732e  (= empty[1])    expected empty[0]   ← off by one
```

**Причина:** `empty[d]` — это хеш пустого поддерева с `2^d` листьями. На уровне дерева `d` (листья на `d=0`) пустой сиблинг покрывает `2^(DEPTH - d - 1)` листьев, значит его хеш — `empty[DEPTH - d - 1]`, а не `empty[DEPTH - d]`.

**Исправление:** заменить `empty[TREE_DEPTH - d]` → `empty[TREE_DEPTH - d - 1]` во всех местах.

**После исправления:**

```
computeRoot([commitment]) = 0x266ab002…4967
manual fold                = 0x266ab002…4967   ✅

proof[0]  = 0x276ff13f…1a38  = empty[19]  ✅
proof[19] = 0x0000…0000      = empty[0]   ✅
isEven all true            = true         ✅
```

### Кросс-проверки

**1. Одно-листовое дерево — replay через логику схемы:**

```js
// Воспроизводим compute_merkle_root(leaf, proof, isEven) из Noir-схемы.
let cur = commitment;
for (let i = 0; i < 20; i++) {
  const sibling = proof[i];
  const [left, right] = isEven[i] ? [cur, sibling] : [sibling, cur];
  cur = await poseidon2Hash(left, right);
}
```

Результат: `cur === computeRoot([commitment])` → `true`. ✅

**2. Много-листовое дерево (3 листа) — replay для каждого листа:**

```
root: 0x113d9c40938e1c247472568eae2fdc884f534edf0e8ec10a1659f5724bec093a
leaf 0 replay root: …093a OK
leaf 1 replay root: …093a OK
leaf 2 replay root: …093a OK
```

Все три листа воспроизводят один и тот же корень. ✅

Это упражняет:
- уровень 0: 3 листа → 2 родителя (одна реальная пара + один с пустым сиблингом);
- верхние уровни: одиночный узел парится с каскадными пустыми сиблингами;
- чередование `isEven` по уровням.

### Временный файл `check_merkle.js`

Проверочный скрипт был написан как **временный** (вне `test/`, без коммита) — из-за проблемы с длинными многострочными `node -e` в интерактивном шелле контейнера (см. 6.2). После прохождения проверок удалён.

**Урок:** для нетривиальной логики (индексация, off-by-one) — писать проверочный скрипт в файл, а не в `node -e`. Результаты видны целиком, ошибки не «съедаются» терминалом.

### Уроки

1. **Проверять формулы пустых поддеревьев на бумаге до кода.** `empty[d]` = 2^d листьев, а не «уровень d». Off-by-one здесь ломает всю цепочку.
2. **Одно-листовое дерево — плохой тест.** Оно вырождено (все сиблинги пустые), но не проверяет случай «левый/правый ребёнок».
3. **Много-листовое дерево — обязательный тест.** Только оно упражняет `isEven[i] == false` для правого ребёнка.
4. **Replay через логику схемы — сильнейшая проверка.** Она не зависит от нашей реализации `computeRoot`: мы буквально повторяем то, что делает Noir `compute_merkle_root`.

---

## Чекпоинты 6.1 – 6.3 (backfill)

**Дата:** 2026-09-25

### Что было не так

Прошли под-этапы 6.1, 6.2, 6.3, коммитили и пушили каждый — но **забыли создать чекпоинты** сразу после коммитов. Обнаружено при обсуждении Stage 6.4, когда пользователь спросил: «where checkpoints?».

Правило 0.5 требует чекпоинт **после каждого этапа**. Конвенция проекта (видно в `.checkpoints/`) — **чекпоинт на каждый под-этап**, не только на финальный:
- `05.1-backend-skeleton`, `05.2-config`, …, `05.11-smoke-test`, `05.12-stage-5-final`
- `04.1.1-anchor-init`, …

### Что сделано

Созданы три чекпоинта с опозданием:

| Чекпоинт | Артефакты | Commit |
|---|---|---|
| `06.1-package` | `package.json`, `pnpm-lock.yaml` | `4142789` |
| `06.2-poseidon` | `poseidon.js` | `d021210` |
| `06.3-merkle` | `merkle.js` | `0e9d766` |

Формат — как в `05.5-tree`:
- артефакты **скопированы** в директорию чекпоинта;
- `manifest.txt` — строки `<sha256>  <filename>`;
- `commit.txt` — хеш коммита.

Хеши артефактов:

```
58224dec1d074977de929b8588807335e4eaef8398da5f8e83380c05868383ea  package.json
35ab54d50ba1faa885bc35d006674e76ef5d809910934e46b207567bcfc53cc0  pnpm-lock.yaml
91f36b41d801c339e8c6ad56a7a6035e1de56c14fc3de0972abf0537fffdbc53  poseidon.js
ef5a48d610f96f6e24355d61d152f244ae3a3eb8ea15bdc3a776e3c9bdf3cf98  merkle.js
```

### Урок

**Создавать чекпоинт сразу после `git commit && git push` под-этапа, не откладывая.** Backfill — это восстановление, не норма. Чекпоинт — не «награда в конце», а страховка на случай регрессии между этапами.

### Заметка про heredoc

Первая попытка создать `manifest.txt` через heredoc на этапе 5.12 зависла (делимитер отображался как текст). При backfill 6.1–6.3 тот же heredoc с `<<'ENDOFFILE'` отработал без проблем.

**Вывод:** вероятно, это был транзиентный сбой терминала (накопленные Ctrl+C, «грязное» состояние строки ввода), а не системная проблема heredoc. Если зависает — `Ctrl+C`, `rm -f` частичного файла, повторить. Если снова зависает — писать файл в редакторе.

---

## 6.4. `src/server.js` — Fastify HTTP server

**Дата:** 2026-09-26

### Формат данных — ключевое решение

**Проблема:** у слоёв разные конвенции hex-строк:

| Слой | Конвенция |
|---|---|
| Noir-схема / `noir_js` | `0x`-prefixed |
| `services/backend/src/tree.rs` (HTTP + Redis) | **без `0x`** (bare hex) |

**Решение (выбрано A1):** **bare hex везде внутри `services/merkle/`.** `poseidon.js` добавляет `0x` только при вызове `noir.execute`, снимает `0x` с `returnValue`. `merkle.js` работает с bare hex. `server.js` принимает и отдаёт bare hex.

**Почему не A2 (0x внутри, трансляция на границе HTTP):** больше мест для ошибки, две конвенции в одном сервисе. Одна конвенция — понятнее.

**Почему не B (0x везде, менять tree.rs):** `tree.rs` уже написан, протестирован, отлажен (Stage 5). Не трогать работающий код.

**Почему не C (принимать оба, отдавать bare):** лишняя сложность валидации; backend всё равно шлёт один формат.

### Рефакторинг `poseidon.js` и `merkle.js`

Оба модуля переведены на bare hex:
- `poseidon2Hash(left, right)` — принимает bare hex, добавляет `0x` для `noir_js`, снимает `0x` с результата.
- `ZERO_LEAF = "00".repeat(32)` (было `"0x" + "00".repeat(32)`).
- Все возвращаемые значения — bare hex.

Кросс-проверка после рефакторинга: тот же корень `266ab002…4967`, что и раньше, но без `0x`.

### Endpoints

| Метод | Путь | Тело | Ответ |
|---|---|---|---|
| GET | `/health` | — | `{status: "ok"}` |
| POST | `/hash` | `{left, right}` | `{hash}` |
| POST | `/root` | `{commitments: [...]}` | `{root}` |
| POST | `/proof` | `{commitments: [...], leaf_index}` | `{proof: [...], is_even: [...]}` |

Формат `/hash`: `{"hash": "<64 hex chars>"}` — **точно совпадает** с тем, что читает `tree.rs` (`resp.get("hash").and_then(|v| v.as_str())`).

### Валидация

Все входы проверяются регуляркой `/^[0-9a-f]{64}$/`. Невалидный ввод → 400 с понятным сообщением.

Проверено:
- `/hash` с отсутствующим `right` → 400.
- `/hash` с `0x`-префиксом → 400 (корректно отклонено).
- `/proof` с `leaf_index` вне диапазона → 400.

### Smoke test

```
GET  /health → {"status":"ok"}
POST /hash   → {"hash":"299bfccd…a636"}   ← совпадает с эталоном
POST /root   → {"root":"266ab002…4967"}   ← совпадает
POST /proof  → {"proof":[20 items], "is_even":[20 items]}
```

`proof.length = 20`, `is_even.length = 20`, `is_even` все `true` (один лист — всегда левый ребёнок).

### Уроки

1. **Формат на границе сервиса — часть контракта.** `tree.rs` написан до Merkle-сервиса; сервис должен адаптироваться к нему, а не наоборот. Выяснять формат **до** написания кода.
2. **Одна конвенция внутри сервиса.** Смешение `0x` и bare hex в одном модуле — источник off-by-one строковых ошибок (`.slice(2)` не там, где нужно).
3. **Валидация входа — не «на будущее».** Именно она поймает случайный `0x`-префикс от фронтенда в Stage 8.

## 6.5. Тесты (`node --test`)

**Дата:** 2026-09-26

### Рефакторинг `server.js` → `app.js` + `server.js`

**Зачем:** `node --test` не может тестировать файл, который вызывает `app.listen()` при импорте. `listen()` биндит порт, тесты конфликтуют друг с другом.

**Решение:** разделить на два модуля:
- `src/app.js` — `buildApp()` возвращает Fastify-инстанс **без** `listen()`. Тестируется через `app.inject()` (встроенный в Fastify, без реального порта).
- `src/server.js` — тонкая обёртка: импортирует `buildApp()`, вызывает `listen()`.

Стандартный паттерн Fastify. Тесты стали быстрыми (375 мс на 23 теста) и без порт-конфликтов.

### Три файла тестов

| Файл | Тестов | Что проверяет |
|---|---|---|
| `test/poseidon.test.js` | 4 | known value, determinism, non-commutativity, формат |
| `test/merkle.test.js` | 10 | empty cascade, root, proof, circuit replay, ошибки |
| `test/server.test.js` | 9 | все endpoints + валидация через `app.inject()` |

**Итого: 23 теста, все проходят.**

### Грабли с `node --test <dir>`

**Симптом:**

```
node --test test/
Error: Cannot find module '/home/ubuntu/services/merkle/test'
```

**Причина:** в Node 24 `node --test <path>` трактует `<path>` как **модуль**, а не как директорию для сканирования. Директория `test/` не является модулем.

**Решение:** передать glob, который разворачивает шелл:

```bash
node --test test/*.test.js
```

**Обновлён `package.json`:**

```json
"test": "node --test test/*.test.js"
```

Без кавычек вокруг glob — чтобы `sh -c` (через который pnpm запускает скрипт) развернул `*` в список файлов.

### Что тесты гарантируют

1. **Poseidon2 выдаёт известное значение.** `poseidon2Hash(1, 2) == 299bfccd…a636`. Если эталон изменится — тест упадёт, и мы узнаем, что схема или noir_js изменились.
2. **Merkle root воспроизводится.** Single leaf — matches manual fold. Multi-leaf — каждый лист replay-ится в тот же корень через логику схемы `compute_merkle_root`.
3. **HTTP-контракт с `tree.rs` соблюдён.** `POST /hash` возвращает `{hash: "<bare hex>"}`. `0x`-префикс отклоняется с 400.
4. **Валидация не пропускает мусор.** Short hex, `0x`-prefix, out-of-range index, non-integer index — все 400.

### Уроки

1. **`app.inject()` — лучший способ тестировать Fastify.** Не биндит порт, работает in-process, быстрее и надёжнее HTTP-запросов.
2. **Разделять построение и запуск.** `buildApp()` / `listen()` — всегда, если планируются тесты.
3. **`node --test` в Node 24 требует glob или явный список файлов.** Директория не работает как аргумент.
4. **Известные значения в тестах — это регрессионная защита.** Тот же `299bfccd…a636` теперь проверяется в 2 местах (poseidon.test.js + server.test.js), плюс `266ab002…4967` для root.

---

## 6.6. Smoke test + интеграция с бэкендом

**Дата:** 2026-09-26

### Запуск сервера

```bash
nohup node src/server.js > /tmp/merkle.log 2>&1 &
```

Сервер слушает `http://127.0.0.1:4003` и `http://172.18.0.3:4003` (внутрисетевой адрес контейнера).

### `smoke.sh` — 8 проверок

Скрипт `services/merkle/smoke.sh` прогоняет curl-запросы и печатает PASS/FAIL по каждому кейсу:

| Проверка | Ожидание | Результат |
|---|---|---|
| `GET /health` | `{"status":"ok"}` | ✅ |
| `POST /hash(1,2)` | `{"hash":"299bfccd…a636"}` | ✅ |
| `POST /root([commitment])` | `{"root":"266ab002…4967"}` | ✅ |
| `POST /proof` → `proof.length` | `20` | ✅ |
| `POST /proof` → `is_even.length` | `20` | ✅ |
| `POST /hash` короткий hex | `400` | ✅ |
| `POST /hash` с `0x` | `400` | ✅ |
| `POST /proof` вне диапазона | `400` | ✅ |

**Итог: 8/8 PASS.**

### Интеграция с бэкендом

**Ключевое наблюдение:** `services/backend/src/config.rs` по умолчанию использует `MERKLE_URL=http://localhost:4003` — **тот же порт, тот же хост** (контейнер `solana`). Никакой конфигурации менять не нужно.

**Проверка:** написан пример `services/backend/examples/merkle_smoke.rs`, который:
1. Строит `MerkleClient::new("http://localhost:4003")`.
2. Вызывает `hash_2(one, two)` — **реальный HTTP-клиент из `tree.rs`**.
3. Сверяет результат с эталоном `299bfccd…a636`.

Запуск:

```bash
cargo run --release --example merkle_smoke
```

Вывод:

```
hash_2(1, 2) = 299bfccd7daf3c917e51291383929049ec0eaed800af245056cbf135f7dea636
OK
```

**Это сильнейшая проверка короткого цикла:** реальный Rust-код бэкенда общается с реальным Node.js-сервисом, контракт соблюдён.

### Предупреждение сборки

```
warning: the following packages contain code that will be rejected by a future version of Rust: sqlx-postgres v0.7.4
```

Не блокирует. Записать для Stage 11 (безопасность) / Stage 12 (финализация).

### Уроки

1. **`MerkleClient` — публичный и легко тестируется.** `pub struct MerkleClient`, `pub fn new(base_url)`, `pub fn hash_2(...)`. Пример в `examples/` — минимальный способ интеграционно проверить HTTP-код без поднятия всего бэкенда.
2. **Совпадение портов по умолчанию — везение, но не случайность.** `MERKLE_URL` дефолтит на `localhost:4003`, потому что оба сервиса живут в одном контейнере. Если в Stage 9 порты будут мапиться наружу, дефолт может измениться на имя сервиса в docker-compose сети.
3. **`smoke.sh` как отдельный артефакт.** Воспроизводимый набор curl-проверок полезен как для CI, так и для отладки после рестарта.

---

## 6.7. Финальный чекпоинт Stage 6

**Дата:** 2026-09-26
**Commit:** `99a228a`

### Что сделано

Stage 6 полностью завершён. Все 7 под-этапов (6.1 – 6.7) пройдены.

### Итог Stage 6

**Сервис:** `services/merkle/`, порт 4003, Node.js 24.21.0 + Fastify 5.12.5.

**Модули:**
- `src/poseidon.js` — `poseidon2Hash(left, right)`, обёртка над `noir_js`.
- `src/merkle.js` — `computeEmptyHashes`, `computeRoot`, `computeProof`.
- `src/app.js` — `buildApp()`, Fastify routes (тестируемый).
- `src/server.js` — `listen()`-обёртка.

**Endpoints:**
| Метод | Путь | Тело | Ответ |
|---|---|---|---|
| GET | `/health` | — | `{status: "ok"}` |
| POST | `/hash` | `{left, right}` | `{hash}` |
| POST | `/root` | `{commitments: [...]}` | `{root}` |
| POST | `/proof` | `{commitments: [...], leaf_index}` | `{proof: [...], is_even: [...]}` |

**Конвенция:** bare hex (без `0x`) на границе HTTP — совпадает с `tree.rs`.

**Тесты:** 23 (`node --test`), все проходят.
**Smoke:** 8/8 через `smoke.sh`.
**Интеграция:** `MerkleClient` из бэкенда успешно вызывает сервис.

### Артефакты чекпоинта

10 файлов (4 модуля, 3 теста, `package.json`, `pnpm-lock.yaml`, `smoke.sh`) — SHA-256 зафиксированы в `manifest.txt`.

### Что дальше

- **Stage 7** — Prover (Rust + Sunspot), порт 4002.
- **Stage 8** — Frontend (Vue 3).
- **Stage 9** — Инфраструктура (Makefile, Prometheus, Grafana, порт-маппинг).
- **Stage 10** — Инженерные процессы (CI/CD).
- **Stage 11** — Безопасность (threat model, расширенные тесты).
- **Stage 12** — Финализация.
- **Stage 13** — Отложенный split deposit (после v0.1.0).
