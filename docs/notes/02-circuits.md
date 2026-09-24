# Этап 2. ZK-схемы на Noir

> **См. также:**
> - `00-zk-primer.md` — что такое circuit, witness, commitment, nullifier.
> - `00-glossary.md` — все термины.
> - `01-setup.md` — Docker-окружение, если ещё не настроено.

---

## TL;DR

**Что делаем:** пишем четыре Noir-схемы — общую библиотеку `poseidon` и три circuit'а: `hash2`, `hashes`, `withdrawal`. Плюс `spec.json` — единый источник истины для публичных входов. Плюс два Rust-CLI: `validate-spec` (проверяет spec) и `sync-circuits` (синхронизирует ACIR между потребителями).

**Зачем:** в v2 (предыдущая версия проекта) `withdraw` падал с `InvalidInstructionData`. Причина — **рассинхрон** публичных входов между **тремя** слоями (circuit, Anchor, frontend). Этап 2 устраняет **этот** класс багов **по построению**: единый `spec.json` + валидатор + синхронизатор.

**Сколько шагов:** 6 под-этапов (2.0 – 2.4).

**Сколько времени:** ~2 часа (с учётом отладки).

**Что понадобится:**
- `01-setup.md` — Docker-окружение должно быть готово.
- Понимание ZK-терминов из `00-zk-primer.md`.

**Что получится:**
- Четыре circuit'а с **41 тестом**.
- Три ACIR-файла (`.json`).
- `spec.json` — 15 правил валидации.
- `sync-circuits` — CLI с двумя режимами.

**Следующий этап:** `03-sunspot.md`.

---

## 1. Почему Noir

ZK-circuit можно писать на разных языках:

| Язык | Плюсы | Минусы |
|---|---|---|
| **Circom** | Самый популярный. Много библиотек. | Низкоуровневый. Много boilerplate. |
| **Leo** | Высокоуровневый. Похож на Rust. | Меньше поддержки. |
| **Cairo** | Мощный. StarkNet. | Для нашей задачи — overkill. |
| **Noir** | Высокоуровневый. Rust-подобный. Хорошая поддержка Sunspot. | Молодой. |

**Почему мы выбрали Noir:**

1. **Rust-подобный синтаксис** — знаком читателям, которые уже знают Rust.
2. **Хорошая поддержка Sunspot** — единственный практичный путь к Groth16-verifier на Solana.
3. **Встроенные ZK-примитивы** — Poseidon2 уже есть (`poseidon2_permutation`). Не нужно реализовывать самому.
4. **`noir_js`** — тот же ACIR можно **запускать в браузере**, что критично для frontend.
5. **Активная разработка** — Aztec и сообщество.

**Ключевой момент:** ACIR-файл circuit'а можно использовать **и** для генерации proof (через Sunspot), **и** для вычисления хешей в браузере (через `noir_js`). Это **устраняет** рассинхрон хешей между разными языками.

---

## 2. Почему Poseidon2

В ZK-circuit'ах **нельзя** использовать произвольные хеш-функции. Точнее, **можно**, но это **дорого**.

**Проблема:** каждая операция в circuit'е превращается в **constraints**. Чем больше constraints — тем **дольше** proof, тем **больше** `.ccs`, `.pk`.

**SHA-256** в circuit'е — это **сотни** constraints на каждый вызов. Побитовые операции (`AND`, `XOR`, shift) **дорого** выражаются в арифметике поля.

**Poseidon2** — специально **спроектирован** для ZK:
- Операции: **сложение** и **умножение** в поле.
- Constraints: **десятки** на вызов.
- **В 100+ раз дешевле** SHA-256 в circuit.

**Компромисс:** Poseidon2 **менее изучен**, чем SHA-256. Меньше криптоанализа. Но для **нашей** задачи (devnet, demo) это **приемлемо**.

**Стандарт в ZK:** Poseidon, Poseidon2, Rescue, MiMC. Мы используем Poseidon2 — он **новее** и **быстрее**.

**Важно:** на **фронтенде** и в **merkle-сервисе** мы **не** реализуем Poseidon2 сами. Мы **загружаем ACIR** circuit'а `hash2.json` через `noir_js` и **вычисляем** хеши через него. Так гарантируется **идентичность** хешей между Noir, Rust и JS.

---

## 3. Что такое ACIR

**ACIR** (Abstract Circuit Intermediate Representation) — это **промежуточное** представление circuit'а после компиляции Noir.

**Аналогия:** как `.class` в Java или `.o` в C. Ты пишешь на высокоуровневом языке, компилятор превращает в **низкоуровневое** представление.

**Что содержит ACIR:**
- **Описание** публичных и приватных входов.
- **Constraints** (ограничения).
- **Байткод** для witness generation.

**Что можно делать с ACIR:**
- **Выполнить** (witness generation) — через `nargo execute` или `noir_js`.
- **Скомпилировать** в CCS (для Groth16) — через `sunspot compile`.

**Файл:** `withdrawal.json` (~41 КБ для нашего circuit'а).

**Ключевое:** ACIR — **один и тот же** файл для всех потребителей. Если он **изменяется**, все потребители **должны** получить новую версию. Для этого — `sync-circuits`.

### Поток данных

```
circuits/withdrawal/src/main.nr     ← рукописный Noir
        │ nargo compile
        ▼
circuits/withdrawal/target/withdrawal.json   ← ACIR
        │
        ├──► services/merkle/circuits/withdrawal.json   ← копия (sync-circuits)
        ├──► web/public/circuits/withdrawal.json        ← копия (sync-circuits)
        └──► (prover читает напрямую из circuits/withdrawal/target/)
```

---

## 4. Проблема v2 и решение через `spec.json`

### Проблема

В v2 порядок публичных входов задавался **в трёх местах**:

1. **`circuits/withdrawal/src/main.nr`** — `pub` параметры `main()`.
2. **`onchain/.../instructions.rs::encode_public_inputs`** — порядок полей в witness.
3. **`web/.../useWithdraw.ts`** — порядок в instruction data.

**При изменении** одного места **забывали** обновить остальные. Proof **генерировался** корректно, но on-chain verifier получал **другие** байты. Результат — `InvalidInstructionData`.

### Решение: `spec.json`

**Единый файл** `circuits/withdrawal/spec.json`, описывающий:
- 5 публичных входов (имя, тип, размер, порядок).
- 5 приватных входов.
- 3 constraints.
- Byte layout witness.
- Список потребителей, которые **обязаны** соответствовать.

**Остальные слои валидируются против него** через `validate-spec`. Если кто-то забыл обновить Rust или TS — `validate-spec` **упадёт**.

**Что в spec описано подробно:**

| Раздел | Что |
|---|---|
| `circuit` | Имя, entrypoint, `TREE_DEPTH=20`, `NR_PUBLIC_INPUTS=5` |
| `public_inputs[]` | 5 входов: `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount` |
| `private_inputs[]` | 5 входов: `nullifier`, `secret`, `note_secret`, `merkle_proof[20]`, `is_even[20]` |
| `constraints[]` | C1, C2, C3 |
| `hash_functions` | `hash_1`, `hash_2`, `hash_3`. `hash_4` — forbidden |
| `witness_layout` | 12-byte header + 5×32 = 172 байта |
| `artifacts` | JSON-файлы и потребители |
| `consumers_of_public_layout` | Файлы, обязанные соответствовать |
| `checkpoints` | Артефакты для чекпоинтов |

### Публичные входы

| # | Имя | Тип | `bytes` | `padded_to` | Кодировка |
|---|---|---|---|---|---|
| 0 | `root` | field | 32 | — | BN254 big-endian |
| 1 | `nullifier_hash` | field | 32 | — | BN254 big-endian |
| 2 | `recipient` | pubkey | 32 | — | Solana Pubkey, редуцированный к BN254 |
| 3 | `recipient_binding` | field | 32 | — | BN254 big-endian |
| 4 | `amount` | u64 | **8** | **32** | u64 BE, right-aligned |

**Две суммы:**
- **raw sum** = 32+32+32+32+8 = **136** (сырые данные).
- **witness slot sum** = 32+32+32+32+**32** = **160** (размер в witness).

**Это разные вещи.** `bytes` — размер **данных**, `padded_to` — размер **слота** в witness.

### Constraints

| # | Формула | Что доказывает |
|---|---|---|
| C1 | `hash_1(nullifier) == nullifier_hash` | Знаем nullifier |
| C2 | `hash_2(note_secret, recipient) == recipient_binding` | Знаем note_secret, привязали к recipient |
| C3 | `compute_merkle_root(hash_3(nullifier, secret, amount), merkle_proof, is_even) == root` | Commitment в дереве |

### Witness layout

```
[12-byte header]
  NR_PUBLIC_INPUTS (u32 BE) = 5
  0                 (u32 BE) = 0
  NR_PUBLIC_INPUTS (u32 BE) = 5
[5 × 32 bytes public inputs]
  root                (32 bytes)
  nullifier_hash      (32 bytes)
  recipient           (32 bytes)
  recipient_binding   (32 bytes)
  amount              (32 bytes, right-aligned u64)
```

**Итого: 12 + 5×32 = 172 байта.**

Этот формат **точно** совпадает с on-chain `encode_public_inputs` (этап 4). Мы **проверим** это на этапе 4 через чекпоинт.

---

## 5. Под-этап 2.0: `spec.json`

### Зачем

**Единый источник истины** для публичных входов. Без него — рассинхрон между слоями (как в v2).

### Создание

```bash
mkdir -p circuits/withdrawal/src
touch circuits/withdrawal/spec.json
```

**Полное содержимое** — в `PROJECT_CONTEXT.md`, раздел 6 (`Stage 2.0`). Ключевые моменты:
- 5 публичных, 5 приватных входов.
- 3 constraints.
- `hash_4: forbidden`.
- 12+160 = 172 байта.

### Проверка

```bash
python3 -m json.tool circuits/withdrawal/spec.json > /dev/null && echo "OK: valid JSON"
```

**Ожидаемый результат:** `OK: valid JSON`.

### ⚠️ Ошибка: `jsonls` не загружает `$schema`

**Симптом:** `Unable to load schema from 'internal://zkpool-solana/withdrawal-spec-v1'`.

**Причина:** `jsonls` (LSP для JSON) пытается загрузить JSON Schema по URI `internal://...`, которого **нет**.

**Решение:** убрать `$schema` из `spec.json`.

**Урок:** не все «стандартные» поля JSON работают в LSP.

### Коммит

```bash
git commit -m "feat(circuits): add withdrawal circuit spec.json as single source of truth"
```

**Коммит:** `6df5fa5`.

---

## 6. Под-этап 2.1: `validate-spec` CLI

### Зачем

Проверять, что `spec.json` **внутренне согласован**. Валидатор **сам** находит расхождения **до** того, как они попадут в прод.

**Пример из реальной практики:** при первом прогоне правило `rule_public_inputs_bytes_sum` **само** обнаружило ошибку в **себе** — суммировало `bytes` (136) вместо `padded_to.unwrap_or(bytes)` (160). Это **ровно** то, для чего строится валидатор.

### Структура

```
scripts/validate-spec/
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── main.rs        ← точка входа
    ├── project.rs     ← поиск project root
    ├── spec.rs        ← типизированные структуры
    └── rules.rs       ← 15 правил
```

**Стек:** Rust + `anyhow` + `serde` + `serde_json`.

### 15 правил

| # | Правило | Что проверяет |
|---|---|---|
| 1 | `version` | `== 1` |
| 2 | `circuit_name` | `== "withdrawal"` |
| 3 | `tree_depth` | `== 20` |
| 4 | `nr_public_inputs` | `circuit.nr_public_inputs == public_inputs.len() == 5` |
| 5 | `unique_names` | Имена публичных входов уникальны |
| 6 | `bytes_sum` | Raw sum = 136, witness slot sum = 160 |
| 7 | `bytes_match_type` | `bytes` соответствует типу; для u64 — `padded_to=32`, `padding="right_aligned"` |
| 8 | `witness_total` | `total_bytes == 172`, header + public_section == 172 |
| 9 | `header_bytes` | `== 12` |
| 10 | `public_section_bytes` | `== 160` |
| 11 | `constraints` | 3 constraints с id C1/C2/C3 |
| 12 | `hash_functions` | Сигнатуры hash_1/2/3, hash_4 forbidden |
| 13 | `artifacts_json` | `artifacts.json == "withdrawal.json"` |
| 14 | `consumers_count` | 4 потребителя |
| 15 | `checkpoints_non_empty` | stage_2 и stage_3 непустые |

### Запуск

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'
```

### Ожидаемый результат

```
📁 Project root: /home/ubuntu
📄 Spec: /home/ubuntu/circuits/withdrawal/spec.json

=== Spec summary ===
  version:            1
  circuit name:       withdrawal
  ...
=== Running validation rules ===
  ✓ version == 1
  ✓ circuit.name == "withdrawal"
  ...
  ✓ checkpoints: stage_2 (4 entries), stage_3 (5 entries)
✅ All 15 rules passed.

✅ Validation passed.
```

### ⚠️ Ошибка №1: `unused_import: Path`

**Симптом:** LazyVim показывает предупреждение `unused_import: Path`.

**Решение:** убрать `Path` из `use std::path::{Path, PathBuf}`.

### ⚠️ Ошибка №2: `cannot find function find_project_root`

**Симптом:** после замены `main.rs` на модульный вариант — **4** ошибки компиляции.

**Причина:** `main.rs` **ссылается** на модули (`project`, `spec`, `rules`), которых **ещё нет** или они **пустые**.

**Решение:** **порядок** — сначала заполнить **все** модули, потом менять `main.rs`.

**Урок:** при рефакторинге **сначала** создаются зависимости, **потом** — потребители.

### ⚠️ Ошибка №3: `expected a type, found a trait`

**Симптом:** в `rules.rs` — ошибка на `let _ = Context::new;`.

**Причина:** `Context` — это **trait**, а не тип. Оставил **мёртвый** код.

**Решение:** убрать мёртвый хелпер и `Context` из `use`.

### ⚠️ Ошибка №4: собственный баг валидатора

**Симптом:** правило `rule_public_inputs_bytes_sum` **падает** с `expected sum 160 bytes, got 136`.

**Причина:** правило суммировало `bytes` (сырые данные), а сравнивало с **witness slot sum** (160). Но `amount.bytes = 8`, а `amount.padded_to = 32`.

**Решение:** правило теперь проверяет **две суммы** — raw и witness.

**Урок:** **валидатор может содержать баги.** Первый прогон **всегда** — «калибровка».

### Коммиты

- `e479137` — skeleton.
- `dbb4365` — 15 rules.

---

## 7. Под-этап 2.2.1: `poseidon` library

### Зачем

**Общая библиотека** для хешей. Вместо дублирования `hash_1`, `hash_2`, `hash_3` в каждом circuit'е — **один** файл, подключаемый как зависимость.

### Структура

```
circuits/poseidon/
├── Nargo.toml       ← type = "lib"
└── src/
    └── lib.nr
```

### `Nargo.toml`

```toml
[package]
name = "poseidon"
type = "lib"
authors = [""]
compiler_version = ">=1.0.0"

[dependencies]
```

**Ключевое:** `type = "lib"` — это **библиотека**, не circuit. Не компилируется в ACIR сама по себе.

### `lib.nr` — три функции

```rust
use std::hash::poseidon2_permutation;

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

### Zero-padding

Poseidon2 с состоянием **t = 4** имеет **rate = 3** и **capacity = 1**. Использовать **все 4** слота как вход **нельзя** — capacity обнулится, криптостойкость потеряется.

**Поэтому:**
- `hash_1(in)` → `[in, 0, 0, 0]` (три нуля).
- `hash_2(in1, in2)` → `[in1, in2, 0, 0]` (два нуля).
- `hash_3(in1, in2, in3)` → `[in1, in2, in3, 0]` (один ноль).
- `hash_4` → **запрещён**.

### Domain separation — известное свойство

Zero-padding приводит к **коллизиям между разными арностями**:

```
hash_1(x)    == hash_2(x, 0)
hash_2(x, y) == hash_3(x, y, 0)
```

**Это не баг.** Прямое следствие zero-padding.

**Почему безопасно для нашего проекта:**

Мы **никогда** не хешируем одно и то же значение разными функциями в одном контексте:
- `hash_1` — только для `nullifier_hash`.
- `hash_2` — только для `recipient_binding` и узлов Merkle tree.
- `hash_3` — только для `commitment`.

Входные данные **разные** (nullifier ≠ note_secret ≠ commitment), пересечений **нет**.

**Если понадобится** domain separation в будущем (новая фича) — см. `PROJECT_CONTEXT.md`, раздел 7.4. Это **большая** работа: пересборка всех артефактов, смена Program ID verifier'а.

### 11 тестов

**Детерминизм:**
- `test_hash_1_deterministic`
- `test_hash_2_deterministic`
- `test_hash_3_deterministic`

**Разные входы → разные выходы:**
- `test_hash_1_differs_for_different_inputs`
- `test_hash_2_differs_for_different_inputs`

**Некоммутативность (важно для Merkle tree):**
- `test_hash_2_not_commutative`
- `test_hash_3_not_commutative_with_first_two`
- `test_hash_3_not_commutative_with_last_two`

**Документирующие коллизии:**
- `test_zero_padding_collision_hash_1_and_hash_2` — **assert** `hash_1(x) == hash_2(x, 0)`.
- `test_zero_padding_collision_hash_2_and_hash_3` — **assert** `hash_2(x, y) == hash_3(x, y, 0)`.

**Свойство, на которое мы действительно полагаемся:**
- `test_hash_1_nonzero_differs_from_hash_2_nonzero` — `hash_1(9) != hash_2(9, 1)`.

### Запуск

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/poseidon && nargo test'
```

**Ожидаемый результат:** `11 tests passed`.

### ⚠️ Ошибка: `Function expects 1 parameter but 2 were given`

**Симптом:** `poseidon2_permutation(state, 4)` → ошибка.

**Причина:** в nargo 1.0.0-rc.2 сигнатура `poseidon2_permutation` — **один** аргумент (массив `[Field; 4]`). Размер выводится **из размера массива**.

**Решение:** `poseidon2_permutation(state)` без `4`.

### ⚠️ Ошибка: 2 теста `*_differ` падали

**Симптом:** `test_hash_1_and_hash_2_differ` и `test_hash_2_and_hash_3_differ` — FAIL.

**Причина:** `hash_1(x) == hash_2(x, 0)` из-за zero-padding. **Не баг** — свойство.

**Решение:** переписать тесты на **документирующие** (assert `==`, не `!=`). Добавить `test_hash_1_nonzero_differs_from_hash_2_nonzero` — **то свойство**, на которое мы полагаемся.

### Коммит

`baade07` — feat(circuits): add poseidon library with hash_1, hash_2, hash_3 (11 tests).

---

## 8. Под-этап 2.2.2: `hash2` и `hashes` circuits

### Зачем два circuit'а

**`hash2`** — **примитив**. Один хеш `hash_2(left, right)`. Нужен merkle-сервису (при обновлении дерева) и frontend (при построении witness). **Вызывается часто.**

**`hashes`** — **композиция**. Два хеша: `commitment = hash_3(...)` и `nullifier_hash = hash_1(...)`. Нужен frontend для проверки commitment **до** депозита.

**Почему не один:**
- **№1:** разные потребители (merkle нужен только `hash_2`, frontend — оба).
- **№2:** разное назначение (примитив vs композиция).
- **№3:** эффективность — если объединить, каждый вызов `hash_2` в merkle будет **лишне** считать commitment.

### `circuits/hash2/`

**Структура:**
```
circuits/hash2/
├── Nargo.toml       ← type = "bin", poseidon dependency
└── src/
    └── main.nr
```

**`main.nr`:**

```rust
use poseidon::hash_2;

fn main(left: Field, right: Field) -> pub Field {
    hash_2(left, right)
}
```

**5 тестов:** детерминизм, некоммутативность, совпадение с библиотекой, типичный Merkle input, максимальный валидный `Field`.

**ACIR:** 30 208 байт, SHA-256 `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`.

### `circuits/hashes/`

**`main.nr`:**

```rust
use poseidon::{hash_1, hash_3};

fn main(nullifier: Field, secret: Field, amount: Field) -> pub (Field, Field) {
    let commitment = hash_3(nullifier, secret, amount);
    let nullifier_hash = hash_1(nullifier);
    (commitment, nullifier_hash)
}
```

**9 тестов:** детерминизм, commitment зависит от каждого входа, nullifier_hash только от nullifier, commitment ≠ nullifier_hash.

**ACIR:** 31 403 байта, SHA-256 `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`.

### ⚠️ Ошибка №1: `missing pub keyword on return type of function main`

**Симптом:** Noir требует `pub` на **возвращаемом типе** `main()`.

**Причина:** verifier не может получить приватный witness. Без `pub` outputs **недостижимы**.

**Решение:** `fn main(...) -> pub (Field, Field)`.

**Урок:** в `type = "bin"` circuit'ах `main()` **обязан** иметь `pub` на возврате.

### ⚠️ Ошибка №2: `Object type is unknown in field access`

**Симптом:** `0.001 as Field` → ошибка.

**Причина:** Noir **не поддерживает** литералы с плавающей точкой.

**Решение:** использовать **целые** числа. Для lamports: `1_000_000` (0.001 SOL).

**Урок:** ZK работает с **целыми** числами в поле. Дроби — **запрещены**.

### ⚠️ Ошибка №3: `Integer literal is too large`

**Симптом:** `0xfedcba98...3210` (256 бит) → ошибка.

**Причина:** BN254 prime ≈ **2^254**. Числа **≥ 2^254** не помещаются в `Field`.

**Решение:** использовать **маленькие** литералы. Для реалистичных тестов — максимум `0x30644e72...0000000` (BN254 - 1).

**Урок:** Solana Pubkey (32 байта, 256 бит) **не влезает** в Field. Нужна **редукция** через `reduce_to_field`.

### Коммит

`8354921` — feat(circuits): add hash2 and hashes circuits (14 tests total).

---

## 9. Под-этап 2.2.3: `withdrawal` circuit

**Самый важный circuit.** Реализует три constraints: C1, C2, C3.

### Структура

```
circuits/withdrawal/
├── Nargo.toml
├── spec.json        ← единый источник истины
└── src/
    ├── main.nr        ← circuit (главный файл)
    ├── merkle_tree.nr ← библиотека (compute_merkle_root)
    └── test_witness.nr ← генератор witness для Prover.toml
```

### `merkle_tree.nr`

**Функция:** `compute_merkle_root<let DEPTH: u32>(leaf, path, is_even) -> Field`.

**Что делает:** по листу и Merkle proof **вычисляет** root. Если root **совпал** с публичным входом — значит лист **действительно** в дереве.

**Разбор:**

```rust
pub fn compute_merkle_root<let DEPTH: u32>(
    leaf: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH],
) -> Field {
    let mut current = leaf;
    for i in 0..DEPTH {
        let sibling = path[i];
        let (left, right) = if is_even[i] {
            (current, sibling)
        } else {
            (sibling, current)
        };
        current = hash_2(left, right);
    }
    current
}
```

**Ключевые моменты:**
- **`<let DEPTH: u32>`** — const generic. Работает для **любой** глубины.
- **`is_even[i]`** — флаг стороны. `true` — наш узел **слева**.
- **Порядок важен:** Poseidon2 **не коммутативен**. `hash_2(a, b) != hash_2(b, a)`.

**Графически (DEPTH=3):**

```
         P2
        /  \
      P1    s2      ← is_even[2] = true: наш узел слева
      / \
    s1  P0          ← is_even[1] = false: наш узел справа
        / \
       C   s0       ← is_even[0] = true: наш узел слева
```

Путь для листа `C`:
- `P0 = hash_2(C, s0)` — `is_even[0]=true`.
- `P1 = hash_2(s1, P0)` — `is_even[1]=false`.
- `P2 = hash_2(P1, s2)` — `is_even[2]=true`.

**8 тестов:** одиночный уровень слева/справа, два уровня, детерминизм, разные листья, разные флаги, разные siblings.

### `main.nr`

**Публичные входы (порядок из spec):**

```rust
fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    recipient_binding: pub Field,
    amount: pub Field,
    // ... приватные
) { ... }
```

**Приватные входы:**

```rust
    nullifier: Field,
    secret: Field,
    note_secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH],
```

**`global TREE_DEPTH: u32 = 20;`**

**Constraints:**

```rust
// C1
let computed_nullifier_hash = hash_1(nullifier);
assert(computed_nullifier_hash == nullifier_hash);

// C2
let computed_binding = hash_2(note_secret, recipient);
assert(computed_binding == recipient_binding);

// C3
let commitment = hash_3(nullifier, secret, amount);
let computed_root = compute_merkle_root::<TREE_DEPTH>(commitment, merkle_proof, is_even);
assert(computed_root == root);
```

**8 тестов:** C1, C2, C3 по отдельности, wrong sibling, wrong flag, полный flow для 2-уровневого дерева, wrong recipient, TREE_DEPTH.

### `test_witness.nr`

**Зачем:** генерирует **корректный** witness и печатает в формате `Prover.toml`.

**Структура:**
- Создаёт три секрета + amount.
- Считает commitment, nullifier_hash, recipient_binding.
- Строит **синтетический** 20-уровневый Merkle tree.
- Печатает **все** входы.

**Подробности** — в `03-sunspot.md`, раздел 3.5.

### `Nargo.toml`

```toml
[package]
name = "withdrawal"
type = "bin"
authors = [""]
compiler_version = ">=1.0.0"

[dependencies]
poseidon = { path = "../poseidon" }
```

**`mod` в `main.nr`:**

```rust
mod merkle_tree;
mod test_witness;

use merkle_tree::compute_merkle_root;
use poseidon::{hash_1, hash_2, hash_3};
```

### ACIR

**42 808 байт**, SHA-256 `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050`.

**Для сравнения:** заглушка без `hash_1/2/3` и Merkle была **37 005** байт. Разница **5 803** — это реальный circuit.

**Важно:** после добавления `test_witness.nr` (этап 3.5) SHA-256 ACIR **изменился** на `29ac2e67...`. Это **ACIR-обёртка**, а **constraint system не изменился** (тест использует те же функции). Поэтому CCS/PK/VK/SO **не изменились**.

### ⚠️ Ошибка: `Integer literal is too large` (в test_witness)

См. выше — Field values **< 2^254**.

### ⚠️ Ошибка: `&str` и `f"...{arr[i]}"` в test_witness

**Симптом:** `error: str expects 1 generic` для `&str`; `error: Type annotation needed` для `f"...{arr[i]}"`.

**Причина:** в Noir 1.0.0-rc.2 строки требуют generic-размер (`str<N>`). Format-строки требуют явной типизации.

**Решение:** **не использовать** строковые параметры и format-строки с массивами. Печатать **только значения** через `println(value)`. Формат `Prover.toml` — собирать **на хосте** (см. `03-sunspot.md`, раздел 3.5.6).

### Коммит

`90afe4c` — feat(circuits): add withdrawal circuit with merkle_tree module (16 tests).

---

## 10. Под-этап 2.3: `sync-circuits` CLI

### Зачем

ACIR circuit'а нужен **трём** потребителям:
- **prover** — читает **напрямую** из `circuits/withdrawal/target/`.
- **merkle service** — копия в `services/merkle/circuits/`.
- **frontend** — копия в `web/public/circuits/`.

**Проблема:** если ACIR **изменился**, все копии **должны** обновиться. Если хоть одна **осталась** старой — рассинхрон → хеши не совпадут → **InvalidInstructionData**.

**Решение:** `sync-circuits` — CLI, который **синхронизирует** ACIR с потребителями и **проверяет** рассинхрон через SHA-256.

### Два режима

**`--check` (для CI):**
1. Компилирует все три circuit'а.
2. Считает SHA-256 источника и потребителей.
3. Сравнивает:
   - **совпадают** — no-op,
   - **потребитель отсутствует** — копирует,
   - **расходятся** — **ошибка**, exit 1, **не копирует**.

**`--apply` (для локальной разработки):**
- То же, но при расхождении — **копирует** с предупреждением, exit 0.

### Почему два режима

**Silent overwriting опасен.** Если кто-то **откатил** `circuits/`, но потребители **остались** новыми — молчаливое копирование **затрёт** новые данные. `--check` **обнаруживает** проблему, `--apply` **чинит** осознанно.

### Структура

```
scripts/sync-circuits/
├── Cargo.toml       ← deps: anyhow, sha2, hex
└── src/
    └── main.rs      ← 198 строк
```

**Hard-coded список:**
```rust
const CIRCUITS: &[(&str, &str, &str)] = &[
    ("hash2",      "circuits/hash2",      "hash2.json"),
    ("hashes",     "circuits/hashes",     "hashes.json"),
    ("withdrawal", "circuits/withdrawal", "withdrawal.json"),
];

const DESTINATIONS: &[&str] = &[
    "services/merkle/circuits",
    "web/public/circuits",
];
```

**Почему hard-coded:** список меняется раз в год. Расширять `spec.json` для `hash2` и `hashes` (без публичных входов) — **искусственно**.

### Запуск

```bash
# CI-режим
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- check'

# Локально — синхронизировать
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- apply'
```

### Проверено

| Сценарий | Результат |
|---|---|
| Первый запуск — создать 6 файлов | `copied: 6` |
| Второй запуск — no-op | `copied: 0, mismatches: 0` |
| Порча одного потребителя, `--check` | `mismatches: 1, EXIT_CODE=1` |
| Порча одного потребителя, `--apply` | `copied: 1, EXIT_CODE=0` |
| После `--apply` — снова в синхроне | `copied: 0, mismatches: 0` |

### `.gitignore`

Потребители (`services/merkle/circuits/`, `web/public/circuits/`) — **производные** файлы, **не коммитятся**. Восстанавливаются командой `sync-circuits --apply`.

### Коммит

`aa5d8d9` — feat(scripts): add sync-circuits with SHA-256 check/apply modes.

---

## 11. Под-этап 2.4: финальный чекпоинт

### Что сохранено

`.checkpoints/02.4-stage-2-final/`:
- `sources/hash2.json`, `sources/hashes.json`, `sources/withdrawal.json` — копии ACIR.
- `manifest.txt` — SHA-256 источников, потребителей, и финальный коммит.

### Хеши

| Файл | SHA-256 |
|---|---|
| `hash2.json` | `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6` |
| `hashes.json` | `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9` |
| `withdrawal.json` | `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050` (изменён на `29ac2e67...` в этапе 3.5) |

**Все 6 потребительских копий совпадают с источниками.** Это гарантия: merkle и frontend увидят **те же** байты, что и on-chain.

---

## 12. Итоги этапа 2

| Под-этап | Тесты | ACIR |
|---|---|---|
| 2.2.1 `poseidon` library | 11 | (library) |
| 2.2.2 `hash2` | 5 | 30 208 B |
| 2.2.2 `hashes` | 9 | 31 403 B |
| 2.2.3 `withdrawal` | 16 | 42 808 B |
| **Всего** | **41** | — |

---

## 13. Частые ошибки

См. разделы 5–9 для деталей. Краткий список:

1. `jsonls` не загружает `$schema: "internal://..."` → убрать.
2. `unused_import: Path` → убрать из `use`.
3. `cannot find function find_project_root` → сначала модули, потом `main.rs`.
4. `expected a type, found a trait` → убрать мёртвый код с `Context`.
5. `rule_public_inputs_bytes_sum` — собственный баг валидатора.
6. `Function expects 1 parameter but 2 were given` → `poseidon2_permutation(state)` без `4`.
7. 2 теста `*_differ` падали → zero-padding коллизии, документировать.
8. `Integer literal is too large` → Field < 2^254.
9. `missing pub keyword on return type` → `-> pub (Field, Field)`.
10. `Object type is unknown in field access` → нет float, только целые.
11. `&str` не работает в Noir → нет строковых параметров.
12. `f"...{arr[i]}"` требует типизации → `println(value)` без format.

---

## 14. Воспроизведение с нуля

Минимальный набор команд:

```bash
# 1. Структура
mkdir -p circuits/poseidon/src circuits/hash2/src circuits/hashes/src circuits/withdrawal/src

# 2. Poseidon library (11 tests)
# Создать Nargo.toml и lib.nr — см. PROJECT_CONTEXT.md, раздел 6.

# 3. hash2 и hashes (14 tests)
# Создать Nargo.toml и main.nr для каждого.

# 4. withdrawal (16 tests)
# Создать Nargo.toml, main.nr, merkle_tree.nr, test_witness.nr.

# 5. Проверка
for circuit in poseidon hash2 hashes withdrawal; do
  docker compose -f infra/docker-compose.yml exec solana bash -ic \
    "cd /home/ubuntu/circuits/$circuit && nargo test"
done

# 6. Компиляция
for circuit in hash2 hashes withdrawal; do
  docker compose -f infra/docker-compose.yml exec solana bash -ic \
    "cd /home/ubuntu/circuits/$circuit && nargo compile"
done

# 7. spec.json и validate-spec
# Создать spec.json — см. PROJECT_CONTEXT.md, раздел 6.

# 8. sync-circuits
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- apply'
```

---

## 15. Что дальше

**Следующий этап:** `03-sunspot.md` — превращение ACIR в on-chain verifier program.

**Что будет:**
- `sunspot compile` → `.ccs`.
- `sunspot setup` → `.pk` + `.vk`.
- `sunspot deploy` → `.so` + keypair.
- `solana program deploy` → verifier program на devnet.
- Локальная проверка: witness → proof → verify.
- Все ошибки и ручные операции — детально.

**Перед прочтением:**
- `00-zk-primer.md` — что такое Groth16, trusted setup.
- `00-glossary.md` — термины по мере необходимости.

---

## 16. Ссылки

- [Noir documentation](https://noir-lang.org/docs/)
- [Poseidon2 paper](https://eprint.iacr.org/2023/323)
- [Sunspot repository](https://github.com/reilabs/sunspot)
- [BN254 curve](https://neuromancer.sk/std/bn/bn254)
- `docs/notes/00-zk-primer.md` — введение в ZK.
- `docs/notes/00-glossary.md` — все термины.
- `docs/notes/03-sunspot.md` — следующий этап.
