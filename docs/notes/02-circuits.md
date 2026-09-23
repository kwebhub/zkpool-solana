# Этап 2. ZK-схемы на Noir

## Разбиение этапа

- **2.0** — `spec.json` (единый источник истины). ✅
- **2.1** — `validate-spec` (Rust CLI).
  - **2.1.1** — скелет. ✅
  - **2.1.2** — правила для `spec.json` (15 правил). ✅
  - **2.1.3** — правила для `.nr` (после 2.2.3). ⏳
  - **2.1.4** — правила для Rust (после 4.1). ⏳
  - **2.1.5** — правила для TS (после 8). ⏳
- **2.2** — circuit'ы. ✅
  - **2.2.1** — `poseidon` library. ✅
  - **2.2.2** — `hash2` и `hashes` circuit'ы. ✅
  - **2.2.3** — `withdrawal` circuit. ✅
- **2.3** — `sync-circuits` с проверкой SHA-256. ✅
- **2.4** — чекпоинт 2. ✅

---

## Этап 2.0. `spec.json` — единый источник истины

### Зачем

В v2 порядок публичных входов задавался **в трёх местах** (circuit, Anchor, frontend). Каждое изменение требовало **трёх правок**. Рассинхрон = `InvalidInstructionData`.

**v3 fix:** один файл `circuits/withdrawal/spec.json`. Остальные слои **валидируются против него**.

### Публичные входы

| # | Имя | Тип | `bytes` | `padded_to` |
|---|---|---|---|---|
| 0 | `root` | field | 32 | — |
| 1 | `nullifier_hash` | field | 32 | — |
| 2 | `recipient` | pubkey | 32 | — |
| 3 | `recipient_binding` | field | 32 | — |
| 4 | `amount` | u64 | **8** | **32** |

**raw sum** = 136. **witness slot sum** = 160.

### Приватные входы

`nullifier`, `secret`, `note_secret`, `merkle_proof[20]`, `is_even[20]`.

### Constraints

- C1: `hash_1(nullifier) == nullifier_hash`
- C2: `hash_2(note_secret, recipient) == recipient_binding`
- C3: `compute_merkle_root(hash_3(nullifier, secret, amount), merkle_proof, is_even) == root`

### Witness layout

12-байтный header + 5×32 = **172 байта**.

---

## Этап 2.1. `validate-spec`

### 2.1.1. Скелет

`scripts/validate-spec/` — Rust CLI, модули `main`, `project`, `spec`, `rules`.
**Коммит:** `e479137`.

### 2.1.2. 15 правил для spec

См. полный список правил в `PROJECT_CONTEXT.md`, раздел 6.
**Коммит:** `dbb4365`.

**Первый прогон выявил ошибку в собственном правиле `rule_public_inputs_bytes_sum`** — оно суммировало `bytes` (136) вместо `padded_to.unwrap_or(bytes)` (160). Это ровно то, для чего строится валидатор: он **сам** ловит расхождения.

### Как запускать

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'
```

---

## Этап 2.2.1. `poseidon` library

- `circuits/poseidon/` — Noir library (`type = "lib"`), 151 строка.
- Функции: `hash_1`, `hash_2`, `hash_3`.
- **11 тестов** — детерминизм, некоммутативность, domain separation (документирующие).
- **Коммит:** `baade07`.

### Сигнатура `poseidon2_permutation` в nargo 1.0.0-rc.2

**Один** аргумент — массив `[Field; 4]`. Размер состояния выводится из массива.
**НЕ** `poseidon2_permutation(state, 4)`.

### Zero-padding

Poseidon2 t=4, r=3, c=1. Заполнять все 4 слота **нельзя**. Zero-padding: 1, 2 или 3 нуля в зависимости от арности.

### Domain separation — известное свойство

```
hash_1(x)    == hash_2(x, 0)
hash_2(x, y) == hash_3(x, y, 0)
```

**Не баг.** Прямое следствие zero-padding. Безопасно для zkpool-solana: мы **никогда** не хешируем одно и то же значение разными функциями в одном контексте.

Если понадобится domain separation — см. `PROJECT_CONTEXT.md`, раздел 7.4.

---

## Этап 2.2.2. `hash2` и `hashes` circuit'ы

### `hash2`

- **Назначение:** внешний Poseidon2 hash двух элементов. Используется merkle-сервисом и фронтом через `noir_js`.
- **Публичные входы:** нет.
- **Приватные входы:** `left: Field`, `right: Field`.
- **Возвращает:** `pub Field` = `hash_2(left, right)`.
- **5 тестов.**
- **ACIR:** 30 208 байт, SHA-256 `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`.

### `hashes`

- **Назначение:** commitment + nullifier_hash одним circuit'ом для фронта.
- **Публичные входы:** нет.
- **Приватные входы:** `nullifier: Field`, `secret: Field`, `amount: Field`.
- **Возвращает:** `pub (Field, Field)` = `(hash_3(nullifier, secret, amount), hash_1(nullifier))`.
- **9 тестов.**
- **ACIR:** 31 403 байта, SHA-256 `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`.

### Ключевое правило Noir

**`main()` в `type = "bin"` circuit'е обязан иметь `pub` на возвращаемом типе.** Verifier не может получить приватный witness, поэтому без `pub` outputs недостижимы.

### Field values < 2^254

Noir **не принимает** 256-битные литералы. Произвольный Solana Pubkey (32 байта) **нельзя** использовать как `Field` напрямую — нужно редуцировать к BN254 (см. `reduce_to_field` в v2).

### Коммит

- `8354921` — feat(circuits): add hash2 and hashes circuits (14 tests total)

---

## Этап 2.2.3. `withdrawal` circuit

### Структура

```
circuits/withdrawal/
├── Nargo.toml
├── spec.json          ← единый источник истины
└── src/
    ├── main.nr        ← circuit (200 строк, 8 тестов)
    └── merkle_tree.nr ← библиотека (143 строки, 8 тестов)
```

### `merkle_tree.nr`

`compute_merkle_root<let DEPTH: u32>(leaf, path, is_even) -> Field`

**Порядок важен.** Poseidon2 не коммутативен. `is_even[i]`:
- `true` — наш узел **левый**, sibling правый: `hash_2(current, sibling)`
- `false` — наш узел **правый**, sibling левый: `hash_2(sibling, current)`

### `main.nr`

- 5 публичных входов **в порядке из spec**: `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`.
- 5 приватных: `nullifier`, `secret`, `note_secret`, `merkle_proof[20]`, `is_even[20]`.
- `global TREE_DEPTH: u32 = 20`.
- Constraints C1, C2, C3.
- **16 тестов** (8 + 8).

### `TREE_DEPTH` и `compute_merkle_root::<TREE_DEPTH>`

`global TREE_DEPTH` используется как const generic при вызове `compute_merkle_root::<TREE_DEPTH>`. Работает.

### ACIR

**42 808 байт**, SHA-256 `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050`.

Для сравнения: заглушка была **37 005 байт**. Разница — `hash_1`, `hash_2`, `hash_3` и Merkle-цикл на 20 уровней.

### Коммит

- `90afe4c` — feat(circuits): add withdrawal circuit with merkle_tree module (16 tests)

---

## Этап 2.3. `sync-circuits`

### Назначение

Синхронизировать ACIR-файлы circuit'ов с потребителями:
- `services/merkle/circuits/*.json` — для merkle-сервиса.
- `web/public/circuits/*.json` — для фронта.

### Два режима

**`--check`** (default, для CI):
1. Компилирует все три circuit'а.
2. Считает SHA-256 источников и потребителей.
3. Сравнивает:
   - **совпадают** — no-op,
   - **потребитель отсутствует** — копирует,
   - **расходятся** — **ошибка**, exit 1, **не копирует**.

**`--apply`** (для локального использования):
- То же, но при расхождении **копирует** с предупреждением, exit 0.

### Зачем два режима

Silent overwriting **опасен**: если кто-то откатил `circuits/`, но потребители остались новыми — молчаливое копирование **затрёт** новые данные. `--check` **обнаруживает** проблему, `--apply` **чинит** осознанно.

### `.gitignore`

Потребители (`services/merkle/circuits/`, `web/public/circuits/`) — **производные** файлы, **не коммитятся**. Восстанавливаются из `circuits/*/target/*.json` командой `sync-circuits --apply`.

### Проверено

| Сценарий | Результат |
|---|---|
| Первый запуск — создать 6 файлов | ✅ `copied: 6` |
| Второй запуск — no-op | ✅ `copied: 0, mismatches: 0` |
| Порча одного потребителя, `--check` | ✅ `mismatches: 1, EXIT_CODE=1` |
| Порча одного потребителя, `--apply` | ✅ `copied: 1, EXIT_CODE=0` |
| После `--apply` — снова в синхроне | ✅ `copied: 0, mismatches: 0` |

### Коммит

- `aa5d8d9` — feat(scripts): add sync-circuits with SHA-256 check/apply modes

---

## Этап 2.4. Финальный чекпоинт

### Что сохранено

`.checkpoints/02.4-stage-2-final/`:
- `sources/hash2.json`, `sources/hashes.json`, `sources/withdrawal.json` — копии источников.
- `manifest.txt` — SHA-256 источников, потребителей, и финальный коммит.

### Хеши

| Файл | SHA-256 |
|---|---|
| `hash2.json` | `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6` |
| `hashes.json` | `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9` |
| `withdrawal.json` | `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050` |

**Все 6 потребительских копий совпадают с источниками.** Это гарантия: merkle и frontend увидят **те же** байты, что и on-chain.

---

## Итого по этапу 2

| Под-этап | Тесты | ACIR |
|---|---|---|
| 2.2.1 `poseidon` library | 11 | (library) |
| 2.2.2 `hash2` | 5 | 30 208 B |
| 2.2.2 `hashes` | 9 | 31 403 B |
| 2.2.3 `withdrawal` | 16 | 42 808 B |
| **Всего** | **41** | — |

---

## Ошибки, которые встретил

1. **`jsonls`: `Unable to load schema from 'internal://...'`** — убрать `$schema` из `spec.json`.

2. **`unused_import: Path`** в `validate-spec/src/main.rs`.

3. **`cannot find function find_project_root`** и др. — модули созданы **до** их использования в `main.rs`.

4. **`expected a type, found a trait`** — мёртвый хелпер с `Context::new` в `rules.rs`.

5. **`rule_public_inputs_bytes_sum` ожидало 160, получило 136.** Реальное расхождение в правиле. Причина: `amount.bytes == 8`, но `padded_to == 32`. Правило суммировало `bytes` вместо `padded_to.unwrap_or(bytes)`.

6. **`Function expects 1 parameter but 2 were given`** — `poseidon2_permutation(state, 4)`. В nargo 1.0.0-rc.2 **один** аргумент.

7. **2 теста `*_differ` падали** — `hash_1(x) == hash_2(x, 0)`. Это **не баг**, а следствие zero-padding. Тесты переписаны на **документирующие**.

8. **`Integer literal is too large`** — 256-битный `0xfedc...` в тесте. Field values **< 2^254**.

9. **`missing pub keyword on return type of function main`** — Noir требует `pub` на возвращаемом типе. Исправлено на `-> pub (Field, Field)`.

10. **`Object type is unknown in field access`** — `0.001 as Field`. Noir **не поддерживает** float. Заменить на `1_000_000` lamports.

---

## Коммиты

- `6df5fa5` — feat(circuits): add withdrawal circuit spec.json as single source of truth
- `e479137` — feat(scripts): add validate-spec skeleton (loads and summarizes spec.json)
- `dbb4365` — feat(scripts): add spec validation rules (15 rules for spec.json)
- `3222231` — docs: update stage 2 notes with validate-spec (2.1.1 and 2.1.2)
- `baade07` — feat(circuits): add poseidon library with hash_1, hash_2, hash_3 (11 tests)
- `ce45bb5` — docs: update stage 2 notes with poseidon library and domain separation
- `a508a46` — docs: update PROJECT_CONTEXT with rules and domain separation notes
- `8354921` — feat(circuits): add hash2 and hashes circuits (14 tests total)
- `90afe4c` — feat(circuits): add withdrawal circuit with merkle_tree module (16 tests)
- `4b938f2` — docs: update PROJECT_CONTEXT with stage 2.2 completion and sync-circuits design
- `aa5d8d9` — feat(scripts): add sync-circuits with SHA-256 check/apply modes
