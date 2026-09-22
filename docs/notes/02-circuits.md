# Этап 2. ZK-схемы на Noir

## Разбиение этапа

Этап 2 разбит на **микро-этапы** — в v3 мы не торопимся, каждый шаг чекпоинтится:

- **2.0** — `spec.json` (единый источник истины). ✅
- **2.1** — генераторы/валидаторы из spec (`.nr`, Rust, TS).
- **2.2** — circuit'ы (poseidon, hash2, hashes, withdrawal).
- **2.3** — `sync-circuits` с проверкой SHA-256.
- **2.4** — чекпоинт 2 (JSON-артефакты + `Prover.toml`).

---

## Этап 2.0. `spec.json` — единый источник истины

### Зачем

В v2 порядок публичных входов задавался **в трёх местах**:
1. `circuits/withdrawal/src/main.nr` — `pub` параметры `main()`.
2. `onchain/.../instructions.rs::encode_public_inputs` — порядок в witness.
3. `web/.../useWithdraw.ts` — порядок в instruction data.

Каждое изменение требовало **трёх правок**. Рассинхрон между ними — прямой путь к `InvalidInstructionData`.

**v3 fix:** один файл `circuits/withdrawal/spec.json` описывает всё. Остальные слои **генерируются из него** или **валидируются против него**.

### Что описано в `spec.json`

| Раздел | Что описывает |
|---|---|
| `circuit` | Имя, entrypoint, `TREE_DEPTH=20`, `NR_PUBLIC_INPUTS=5`. |
| `public_inputs` | 5 входов с типами, размерами, порядком, кодировкой. |
| `private_inputs` | 5 входов (nullifier, secret, note_secret, merkle_proof[20], is_even[20]). |
| `constraints` | C1, C2, C3 — формулы, которым должен удовлетворять proof. |
| `hash_functions` | `hash_1`, `hash_2`, `hash_3` — сигнатуры и реализации. `hash_4` — **forbidden**. |
| `witness_layout` | Байтовый формат witness: 12-байтный заголовок + 160 байт публичных входов = 172 байта. |
| `artifacts` | Какие JSON-артефакты производятся и кто их потребляет. |
| `consumers_of_public_layout` | Список файлов, которые обязаны соответствовать порядку публичных входов. |
| `checkpoints` | Список артефактов для чекпоинтов 2 и 3. |

### Публичные входы — порядок и формат

| # | Имя | Тип | Байт | Кодировка |
|---|---|---|---|---|
| 0 | `root` | field | 32 | BN254 big-endian |
| 1 | `nullifier_hash` | field | 32 | BN254 big-endian |
| 2 | `recipient` | Pubkey | 32 | Solana Pubkey, редуцированный к BN254 |
| 3 | `recipient_binding` | field | 32 | BN254 big-endian |
| 4 | `amount` | u64 | 8 | u64 big-endian, выровнен **вправо** в 32-байтное слово |

**Итого:** 5 × 32 = 160 байт.

### Приватные входы

| Имя | Тип | Описание |
|---|---|---|
| `nullifier` | field | Секрет; хешируется в `nullifier_hash`. |
| `secret` | field | Секрет; часть commitment. |
| `note_secret` | field | Секрет; часть `recipient_binding`. |
| `merkle_proof` | `[Field; 20]` | Сиблинги на пути от листа к корню. |
| `is_even` | `[bool; 20]` | Для каждого уровня: `true` — наш узел слева. |

### Constraints

| # | Формула |
|---|---|
| C1 | `hash_1(nullifier) == nullifier_hash` |
| C2 | `hash_2(note_secret, recipient) == recipient_binding` |
| C3 | `compute_merkle_root(hash_3(nullifier, secret, amount), merkle_proof, is_even) == root` |

### `hash_4` — forbidden

Poseidon2 с состоянием `t=4`, rate `r=3`, capacity `c=1`. Если использовать все 4 слота как вход — capacity становится 0, криптостойкость **теряется**. Поэтому `hash_4` **никогда** не используется.

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

Итого: **12 + 5×32 = 172 байта**.

### Consumers — кто обязан соответствовать spec

| Слой | Файл | Обязанность |
|---|---|---|
| Noir | `circuits/withdrawal/src/main.nr` | `pub` параметры `main()` в порядке из `public_inputs[]` |
| Anchor | `onchain/.../instructions.rs::encode_public_inputs` | Байты совпадают с `witness_layout` |
| Frontend | `web/src/services/poseidon.ts::generateWithdrawalWitness` | Witness-блоб совпадает с `witness_layout` |
| Frontend | `web/src/composables/useWithdraw.ts` | Порядок публичных входов в instruction data совпадает с `public_inputs[]` |

### Artifacts — JSON circuit'ов и их потребители

| Circuit | JSON | Prover | Merkle | Web |
|---|---|---|---|---|
| `hash2` | `hash2.json` | — | copy | copy |
| `hashes` | `hashes.json` | — | copy | copy |
| `withdrawal` | `withdrawal.json` | direct | copy | copy |

**`prover` — direct:** prover работает внутри контейнера `solana-zkpool-solana`, читает JSON напрямую из `circuits/withdrawal/target/`.
**`merkle` / `web` — copy:** копируются через `sync-circuits`.

---

## Ошибки, которые встретил

**`jsonls`: `Unable to load schema from 'internal://zkpool-solana/withdrawal-spec-v1'`**

LazyVim / `jsonls` пытается загрузить JSON Schema по URI `internal://...`, не может, и выдаёт предупреждение. Решение: убрать поле `$schema` из `spec.json` — оно не нужно, это просто маркер.

## Коммиты

- `6df5fa5` — feat(circuits): add withdrawal circuit spec.json as single source of truth

## Артефакты чекпоинта

Пока нет — `spec.json` сам по себе **не артефакт**, это **источник**. Артефакты появятся после этапа 2.2 (compile) и будут зачекпоинчены в 2.4.
