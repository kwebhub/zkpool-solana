# Этап 2. ZK-схемы на Noir

## Разбиение этапа

Этап 2 разбит на **микро-этапы** — в v3 мы не торопимся, каждый шаг чекпоинтится:

- **2.0** — `spec.json` (единый источник истины). ✅
- **2.1** — `validate-spec` (Rust CLI, валидирует spec и потребителей). 🚧
  - **2.1.1** — скелет. ✅
  - **2.1.2** — правила для `spec.json`. ✅
  - **2.1.3** — правила для `.nr` (после 2.2).
  - **2.1.4** — правила для Rust (после 4.1).
  - **2.1.5** — правила для TS (после 8).
- **2.2** — circuit'ы (poseidon, hash2, hashes, withdrawal).
- **2.3** — `sync-circuits` с проверкой SHA-256.
- **2.4** — чекпоинт 2.

---

## Этап 2.0. `spec.json` — единый источник истины

### Зачем

В v2 порядок публичных входов задавался **в трёх местах**:
1. `circuits/withdrawal/src/main.nr` — `pub` параметры `main()`.
2. `onchain/.../instructions.rs::encode_public_inputs` — порядок в witness.
3. `web/.../useWithdraw.ts` — порядок в instruction data.

Каждое изменение требовало **трёх правок**. Рассинхрон между ними — прямой путь к `InvalidInstructionData`.

**v3 fix:** один файл `circuits/withdrawal/spec.json` описывает всё. Остальные слои **валидируются против него**.

### Что описано в `spec.json`

| Раздел | Что описывает |
|---|---|
| `circuit` | Имя, entrypoint, `TREE_DEPTH=20`, `NR_PUBLIC_INPUTS=5`. |
| `public_inputs` | 5 входов с типами, размерами, порядком, кодировкой. |
| `private_inputs` | 5 входов. |
| `constraints` | C1, C2, C3. |
| `hash_functions` | `hash_1`, `hash_2`, `hash_3`. `hash_4` — **forbidden**. |
| `witness_layout` | 12-байтный заголовок + 160 байт публичных входов = 172 байта. |
| `artifacts` | JSON-артефакты и их потребители. |
| `consumers_of_public_layout` | Список файлов, обязанных соответствовать порядку. |
| `checkpoints` | Список артефактов для чекпоинтов 2 и 3. |

### Публичные входы

| # | Имя | Тип | `bytes` | `padded_to` | Кодировка |
|---|---|---|---|---|---|
| 0 | `root` | field | 32 | — | BN254 big-endian |
| 1 | `nullifier_hash` | field | 32 | — | BN254 big-endian |
| 2 | `recipient` | pubkey | 32 | — | Solana Pubkey, редуцированный к BN254 |
| 3 | `recipient_binding` | field | 32 | — | BN254 big-endian |
| 4 | `amount` | u64 | **8** | **32** | u64 big-endian, right-aligned |

**Сумма `bytes` (сырых данных):** 32+32+32+32+8 = **136**.
**Сумма слотов в witness (`padded_to.unwrap_or(bytes)`):** 32+32+32+32+**32** = **160**.

Это **не одно и то же**. `bytes` — размер данных; `padded_to` — размер слота в witness.

### Приватные входы

| Имя | Тип | Размер |
|---|---|---|
| `nullifier` | field | — |
| `secret` | field | — |
| `note_secret` | field | — |
| `merkle_proof` | `[Field; 20]` | 20 |
| `is_even` | `[bool; 20]` | 20 |

### Constraints

| # | Формула |
|---|---|
| C1 | `hash_1(nullifier) == nullifier_hash` |
| C2 | `hash_2(note_secret, recipient) == recipient_binding` |
| C3 | `compute_merkle_root(hash_3(nullifier, secret, amount), merkle_proof, is_even) == root` |

### `hash_4` — forbidden

Poseidon2 с `t=4`, `rate=3`, `capacity=1`. Использовать все 4 слота как вход — capacity=0, криптостойкость теряется.

- `hash_1(in) = poseidon2_permutation([in, 0, 0, 0])[0]`
- `hash_2(in1, in2) = poseidon2_permutation([in1, in2, 0, 0])[0]`
- `hash_3(in1, in2, in3) = poseidon2_permutation([in1, in2, in3, 0])[0]`

### Byte layout witness

```
[12-byte header]
  NR_PUBLIC_INPUTS (u32 BE) = 5
  0                 (u32 BE) = 0
  NR_PUBLIC_INPUTS (u32 BE) = 5
[5 × 32 bytes public inputs]
  root
  nullifier_hash
  recipient
  recipient_binding
  amount (right-aligned u64 in 32-byte word)
```

**Итого: 12 + 5×32 = 172 байта.**

### Consumers — кто обязан соответствовать spec

| Слой | Файл | Обязанность |
|---|---|---|
| Noir | `circuits/withdrawal/src/main.nr` | `pub` параметры `main()` в порядке `public_inputs[]` |
| Anchor | `onchain/.../instructions.rs::encode_public_inputs` | Байты совпадают с `witness_layout` |
| Frontend | `web/src/services/poseidon.ts::generateWithdrawalWitness` | Witness-блоб совпадает с `witness_layout` |
| Frontend | `web/src/composables/useWithdraw.ts` | Порядок публичных входов в instruction data совпадает с `public_inputs[]` |

### Artifacts — JSON circuit'ов

| Circuit | JSON | Prover | Merkle | Web |
|---|---|---|---|---|
| `hash2` | `hash2.json` | — | copy | copy |
| `hashes` | `hashes.json` | — | copy | copy |
| `withdrawal` | `withdrawal.json` | **direct** | copy | copy |

**`prover` — direct:** читает JSON напрямую из `circuits/withdrawal/target/`.
**`merkle` / `web` — copy:** копируются через `sync-circuits`.

---

## Этап 2.1. `validate-spec` — Rust CLI

### 2.1.1. Скелет

Создан Rust-крейт `scripts/validate-spec/`:
- `Cargo.toml` — зависимости `anyhow`, `serde`, `serde_json`.
- `src/main.rs` — точка входа.
- `src/project.rs` — `find_project_root()`.
- `src/spec.rs` — типизированные структуры `Spec`, `Circuit`, `PublicInput`, и т.д.
- `src/rules.rs` — правила валидации.

**Скелет** (`2.1.1`) просто:
1. Находит project root (по наличию `circuits/` и `onchain/`).
2. Читает и парсит `spec.json` в типизированный `Spec`.
3. Печатает summary.
4. Говорит `✅ Spec loaded successfully (validation rules not yet implemented)`.

**Проверено:** запуск внутри контейнера, summary корректное.

**Коммит:** `e479137`.

### 2.1.2. Правила валидации spec

`rules::validate_all()` содержит **15 правил**:

| # | Правило | Что проверяет |
|---|---|---|
| 1 | `rule_version` | `version == 1` |
| 2 | `rule_circuit_name` | `circuit.name == "withdrawal"` |
| 3 | `rule_tree_depth` | `circuit.tree_depth == 20` |
| 4 | `rule_nr_public_inputs` | `circuit.nr_public_inputs == public_inputs.len() == 5` |
| 5 | `rule_public_inputs_unique_names` | `public_inputs[].name` уникальны |
| 6 | `rule_public_inputs_bytes_sum` | raw sum == 136, witness slot sum == 160 |
| 7 | `rule_public_inputs_bytes_match_type` | `bytes` соответствует типу (field=32, pubkey=32, u64=8), для u64 — `padded_to==32` и `padding=="right_aligned"` |
| 8 | `rule_witness_total_bytes` | `total_bytes == 172` и `header + public_section == 172` |
| 9 | `rule_witness_header_bytes` | `header.bytes == 12` |
| 10 | `rule_witness_public_section_bytes` | `public_inputs_section.bytes == 160` |
| 11 | `rule_constraints` | 3 constraints с id `C1`, `C2`, `C3` |
| 12 | `rule_hash_functions` | `hash_1/2/3` с правильными сигнатурами; `hash_4.status == "forbidden"` |
| 13 | `rule_artifacts_json` | `artifacts.json == "withdrawal.json"` |
| 14 | `rule_consumers_count` | 4 потребителя |
| 15 | `rule_checkpoints_non_empty` | `checkpoints.stage_2` и `stage_3` непустые |

**Первый прогон обнаружил реальное расхождение.** Правило `rule_public_inputs_bytes_sum` суммировало `bytes` и получало 136 вместо ожидаемых 160. Причина: `amount` имеет `bytes: 8`, но `padded_to: 32`. **Валидатор сам поймал ошибку в собственном правиле** — это ровно то, для чего мы его строим.

**Фикс:** правило теперь проверяет **две суммы**:
- **raw sum** = 136 (`bytes` — сырые данные).
- **witness slot sum** = 160 (`padded_to.unwrap_or(bytes)` — слоты в witness).

**Проверено:** все 15 правил прошли.

**Коммит:** `dbb4365`.

### Как запускать

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'
```

**Ожидаемый вывод:**
```
=== Spec summary ===
  version:            1
  circuit name:       withdrawal
  tree depth:         20
  nr public inputs:   5
  public inputs:      5 entries
  private inputs:     5 entries
  witness total:      172 bytes
  ...

=== Running validation rules ===
  ✓ version == 1
  ...
  ✓ checkpoints: stage_2 (4 entries), stage_3 (5 entries)
✅ All 15 rules passed.

✅ Validation passed.
```

### Артефакты чекпоинта 2.1

- `.checkpoints/02.1-validate-spec/commit.txt` — `dbb4365b2990e9f17ca957671157fdf386d17226`.
- `.checkpoints/02.1-validate-spec/validate-output.txt` — вывод всех 15 правил.

---

## Ошибки, которые встретил

1. **`jsonls`: `Unable to load schema from 'internal://zkpool-solana/withdrawal-spec-v1'`**
   LazyVim / `jsonls` пытается загрузить JSON Schema по URI `internal://...`. Решение: убрать `$schema` из `spec.json`.

2. **`unused_import: Path`** в `main.rs`.
   Убрал из `use std::path::{Path, PathBuf}`.

3. **`cannot find function find_project_root`** и др.
   Ошибки компиляции после замены `main.rs` — ссылается на модули, которых ещё нет. **Урок:** порядок — сначала модули, потом `main.rs`.

4. **`expected a type, found a trait`** — оставил мёртвый хелпер с `Context::new` в `rules.rs`. Убрал.

5. **`rule_public_inputs_bytes_sum` ожидало 160, получило 136.**
   Реальное расхождение. Причина: `amount.bytes == 8`, но `padded_to == 32`. Правило суммировало `bytes` вместо `padded_to.unwrap_or(bytes)`. Исправлено: теперь проверяются **обе** суммы.

## Коммиты

- `6df5fa5` — feat(circuits): add withdrawal circuit spec.json as single source of truth
- `e479137` — feat(scripts): add validate-spec skeleton (loads and summarizes spec.json)
- `dbb4365` — feat(scripts): add spec validation rules (15 rules for spec.json)
