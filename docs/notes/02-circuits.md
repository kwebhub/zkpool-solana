# Этап 2. ZK-схемы на Noir

## Разбиение этапа

- **2.0** — `spec.json` (единый источник истины). ✅
- **2.1** — `validate-spec` (Rust CLI).
  - **2.1.1** — скелет. ✅
  - **2.1.2** — правила для `spec.json` (15 правил). ✅
  - **2.1.3** — правила для `.nr` (после 2.2.3).
  - **2.1.4** — правила для Rust (после 4.1).
  - **2.1.5** — правила для TS (после 8).
- **2.2** — circuit'ы.
  - **2.2.1** — `poseidon` library. ✅
  - **2.2.2** — `hash2` и `hashes` circuit'ы.
  - **2.2.3** — `withdrawal` circuit.
- **2.3** — `sync-circuits` с проверкой SHA-256.
- **2.4** — чекпоинт 2.

---

## Этап 2.0. `spec.json` — единый источник истины

Описан полностью в разделе 2.0 ниже (см. предыдущую версию заметок). Ключевое:

- 5 публичных входов: `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`.
- 5 приватных: `nullifier`, `secret`, `note_secret`, `merkle_proof[20]`, `is_even[20]`.
- 3 constraints: C1, C2, C3.
- Witness: 12-байтный header + 160 байт = **172 байта**.
- `amount` имеет `bytes: 8`, `padded_to: 32`, `padding: "right_aligned"`.
- Сумма сырых `bytes` = **136**, сумма слотов witness = **160**.

---

## Этап 2.1. `validate-spec` — Rust CLI

### 2.1.1. Скелет

- `scripts/validate-spec/` — Rust CLI, зависимости `anyhow`, `serde`, `serde_json`.
- Модули: `main.rs`, `project.rs`, `spec.rs`, `rules.rs`.
- `find_project_root` — ищет по наличию `circuits/` и `onchain/`.
- `Spec::load` — типизированный парсинг JSON.
- `print_summary` — вывод краткой сводки.
- **Коммит:** `e479137`.

### 2.1.2. 15 правил для spec

| # | Правило | Что проверяет |
|---|---|---|
| 1 | `rule_version` | `version == 1` |
| 2 | `rule_circuit_name` | `circuit.name == "withdrawal"` |
| 3 | `rule_tree_depth` | `tree_depth == 20` |
| 4 | `rule_nr_public_inputs` | `nr_public_inputs == public_inputs.len() == 5` |
| 5 | `rule_public_inputs_unique_names` | уникальность имён |
| 6 | `rule_public_inputs_bytes_sum` | raw sum == 136, witness slot sum == 160 |
| 7 | `rule_public_inputs_bytes_match_type` | `bytes` по типу; для u64 — `padded_to==32`, `padding=="right_aligned"` |
| 8 | `rule_witness_total_bytes` | `total_bytes == 172` и `header + public_section == 172` |
| 9 | `rule_witness_header_bytes` | `header.bytes == 12` |
| 10 | `rule_witness_public_section_bytes` | `public_section.bytes == 160` |
| 11 | `rule_constraints` | 3 constraints с id C1, C2, C3 |
| 12 | `rule_hash_functions` | сигнатуры hash_1/2/3; hash_4 forbidden |
| 13 | `rule_artifacts_json` | `artifacts.json == "withdrawal.json"` |
| 14 | `rule_consumers_count` | 4 потребителя |
| 15 | `rule_checkpoints_non_empty` | stage_2 и stage_3 непустые |

**Первый прогон выявил ошибку в самом правиле `rule_public_inputs_bytes_sum`** — оно суммировало `bytes` вместо `padded_to.unwrap_or(bytes)` и получало 136 вместо 160. Это ровно то, для чего строится валидатор: он сам ловит расхождения.

**Коммит:** `dbb4365`.

### Как запускать

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'
```

**Ожидаемый вывод:**

```
=== Running validation rules ===
  ✓ version == 1
  ...
  ✓ checkpoints: stage_2 (4 entries), stage_3 (5 entries)
✅ All 15 rules passed.

✅ Validation passed.
```

---

## Этап 2.2.1. `poseidon` library

### Что сделано

Создана Noir-библиотека `circuits/poseidon/` (`type = "lib"`):

- `Nargo.toml` — пакет `poseidon`, `type = "lib"`, `compiler_version = ">=1.0.0"`.
- `src/lib.nr` — 151 строка, три публичные функции + 11 тестов.

### Функции

```rust
pub fn hash_1(input: Field) -> Field {
    let state = [input, 0, 0, 0];
    poseidon2_permutation(state)[0]
}

pub fn hash_2(input1: Field, input2: Field) -> Field {
    let state = [input1, input2, 0, 0];
    poseidon2_permutation(state)[0]
}

pub fn hash_3(input1: Field, input2: Field, input3: Field) -> Field {
    let state = [input1, input2, input3, 0];
    poseidon2_permutation(state)[0]
}
```

### Сигнатура `poseidon2_permutation` в nargo 1.0.0-rc.2

**Один** аргумент — массив `[Field; 4]`. Размер состояния выводится из размера массива. **Не** `poseidon2_permutation(state, 4)`, как было в некоторых примерах.

### Zero-padding

Poseidon2 с t=4 имеет rate r=3, capacity c=1. Заполнять все 4 слота **нельзя** — capacity обнулится, криптостойкость потеряется. Поэтому zero-padding: 1, 2 или 3 нуля в зависимости от арности.

### Domain separation — известное свойство

Zero-padding приводит к **коллизиям между разными арностями**:

```
hash_1(x)    == hash_2(x, 0)
hash_2(x, y) == hash_3(x, y, 0)
```

Это **не баг** — прямое следствие zero-padding.

**Почему это безопасно для zkpool-solana:**

Мы **никогда** не хешируем одно и то же значение разными функциями в одном контексте:

- `hash_1` — только для `nullifier_hash`.
- `hash_2` — только для `recipient_binding` и узлов Merkle tree.
- `hash_3` — только для `commitment`.

Входные данные **разные** (nullifier ≠ note_secret ≠ commitment), пересечений нет.

**Что это значит на будущее:**

Если в новой фиче понадобится **domain separation** (например, подписывать данные, где `hash_2(x, 0)` может спутаться с `hash_1(x)`), нужно:

1. Изменить формулы в `poseidon/src/lib.nr` — добавить **уникальный тег** в состояние:
   - `hash_1(in)` → `[in, 1, 0, 0]`
   - `hash_2(in1, in2)` → `[in1, in2, 2, 0]`
   - `hash_3(in1, in2, in3)` → `[in1, in2, in3, 3]`
2. Обновить `spec.json` (раздел `hash_functions`).
3. Перекомпилировать **все** circuit'ы, зависящие от `poseidon`.
4. Пересобрать Sunspot-артефакты (ACIR, CCS, PK, VK).
5. Обновить Sunspot verifier program (ID **изменится**).
6. Обновить `web/.env` через `sync-program-id`.
7. Пересчитать все чекпоинты.

**Это большая работа.** Делать её только при явной необходимости.

### Тесты (11 штук)

| Тест | Что проверяет |
|---|---|
| `test_hash_1_deterministic` | `hash_1(42)` дважды даёт одно и то же |
| `test_hash_2_deterministic` | `hash_2(1, 2)` дважды |
| `test_hash_3_deterministic` | `hash_3(1, 2, 3)` дважды |
| `test_hash_1_differs_for_different_inputs` | `hash_1(1) != hash_1(2)` |
| `test_hash_2_differs_for_different_inputs` | `hash_2(1, 2) != hash_2(2, 1)` |
| `test_hash_2_not_commutative` | Poseidon2 **не** коммутативен |
| `test_hash_3_not_commutative_with_first_two` | `hash_3(1,2,3) != hash_3(2,1,3)` |
| `test_hash_3_not_commutative_with_last_two` | `hash_3(1,2,3) != hash_3(1,3,2)` |
| `test_zero_padding_collision_hash_1_and_hash_2` | **документирует** `hash_1(x) == hash_2(x, 0)` |
| `test_zero_padding_collision_hash_2_and_hash_3` | **документирует** `hash_2(x,y) == hash_3(x,y,0)` |
| `test_hash_1_nonzero_differs_from_hash_2_nonzero` | `hash_1(9) != hash_2(9, 1)` |

**Результат:** 11 / 11 passed.

### Запуск

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/poseidon && nargo test'
```

### Коммит

- `baade07` — feat(circuits): add poseidon library with hash_1, hash_2, hash_3 (11 tests)

### Артефакты чекпоинта 2.2.1

- `.checkpoints/02.2.1-poseidon/commit.txt` — `baade071d1eca4d16648bc81d5fb5280fdf793fd`.
- `.checkpoints/02.2.1-poseidon/nargo-test-output.txt` — вывод `nargo test`.

---

## Ошибки, которые встретил

1. **`jsonls`: `Unable to load schema from 'internal://zkpool-solana/withdrawal-spec-v1'`**
   LazyVim / `jsonls` пытается загрузить JSON Schema по URI `internal://...`. Решение: убрать `$schema` из `spec.json`.

2. **`unused_import: Path`** в `main.rs`.
   Убрал из `use std::path::{Path, PathBuf}`.

3. **`cannot find function find_project_root`** и др.
   Ошибки компиляции после замены `main.rs` — ссылается на модули, которых ещё нет. **Урок:** порядок — сначала модули, потом `main.rs`.

4. **`expected a type, found a trait`** — мёртвый хелпер с `Context::new` в `rules.rs`. Убрал.

5. **`rule_public_inputs_bytes_sum` ожидало 160, получило 136.**
   Реальное расхождение. Причина: `amount.bytes == 8`, но `padded_to == 32`. Правило суммировало `bytes` вместо `padded_to.unwrap_or(bytes)`. Исправлено: теперь проверяются **обе** суммы.

6. **`Function expects 1 parameter but 2 were given`** — `poseidon2_permutation(state, 4)`.
   В nargo 1.0.0-rc.2 у `poseidon2_permutation` **один** аргумент — массив `[Field; 4]`. Размер выводится из массива. Исправлено.

7. **2 теста `*_differ` падали** — `hash_1(x) == hash_2(x, 0)`.
   Это **не баг**, а следствие zero-padding. Тесты переписаны на **документирующие** (assert `==`, а не `!=`). Добавлен тест `test_hash_1_nonzero_differs_from_hash_2_nonzero` — то свойство, на которое мы **действительно** полагаемся.

---

## Коммиты

- `6df5fa5` — feat(circuits): add withdrawal circuit spec.json as single source of truth
- `e479137` — feat(scripts): add validate-spec skeleton (loads and summarizes spec.json)
- `dbb4365` — feat(scripts): add spec validation rules (15 rules for spec.json)
- `3222231` — docs: update stage 2 notes with validate-spec (2.1.1 and 2.1.2)
- `baade07` — feat(circuits): add poseidon library with hash_1, hash_2, hash_3 (11 tests)
