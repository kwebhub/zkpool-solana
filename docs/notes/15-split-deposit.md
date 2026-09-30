# 15. Split deposit

> **Этап 15** проекта zkpool-solana.
> Разбиение одного депозита на N неравных commitments. Разрывает связь по сумме между депозитом и выводом.

---

## TL;DR

**Что делаем:** один депозит → N = 3 commitments с неравными суммами. Схема вывода получает новые входы — агрегированную сумму и вектор разбиения. On-chain наблюдатель видит одну транзакцию, одну агрегированную сумму и N commitments, но не может определить, как именно сумма разбита.

**Зачем:** без разбиения депозит одной суммой → вывод одной суммой → связь восстанавливается по совпадению сумм. Разбиение разрывает эту связь.

**Ключевые решения:**
- **Путь 2** — схема вывода **меняется** (не только сторона депозита). Новые входы: `total_amount` (публичный), `splits[3]` (приватный). Новый constraint: `Σ splits_i == total_amount`.
- **N = 3, фиксированное.** Const-generic, как `TREE_DEPTH`.
- **Upgrade на месте.** Program ID verifier'а `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` **не меняется**. Меняется только `.so`.
- **Breaking change.** Ноты от v0.1.0 несовместимы с новой схемой. Версия проекта → **v0.2.0**.

**Сколько под-этапов:** 10 (15.1 – 15.10).

**Что понадобится:**
- `02-circuits.md` — что такое circuit, ACIR, constraint.
- `03-sunspot.md` — что такое `.ccs`, `.pk`, `.vk`, redeploy.
- `04-anchor.md` — что такое `encode_public_inputs`, public input layout.
- `10-e2e.md` — что такое полный E2E, какие баги уже находили.

**Что получится:**
- Новая схема `withdrawal` с 6 публичными входами (было 5).
- Новый verifier `.so` — тот же Program ID, новая логика.
- Новая инструкция `deposit_split` в Anchor-программе.
- Split-UI во фронтенде.
- E2E на devnet: один депозит 1 SOL → 3 ноты (0.5 + 0.3 + 0.2) → 3 независимых вывода.

**Следующий этап:** Stage 16 (marketing, статьи, портфолио) — вне этого публичного репозитория.

---

## 1. Зачем split deposit

> **Зачем этот раздел.** Понять, какую проблему решает фича — и почему без неё приватность неполная.

### 1.1. Проблема: связь по сумме

Пусть пользователь депонировал 1 SOL с адреса `A` и через час вывел 1 SOL на адрес `B`. Что видит наблюдатель:

```
Tx1: A → pool_vault   amount: 1.000 SOL   (deposit)
Tx2: pool_vault → B   amount: 1.000 SOL   (withdraw)
```

Суммы **совпадают**. Время — **близко**. Аналитик делает вывод: **A и B — один и тот же пользователь**. Приватность нарушена, несмотря на ZK.

ZK-proof скрывает **связь commitment → адрес вывода**. Но **сумма** депозита записана on-chain в публичном witness (вход `amount`), и сумма вывода — тоже.

**Классическая атака:** "amount correlation attack". Не требует взлома криптографии, только наблюдения за блокчейном.

### 1.2. Что даёт разбиение

Split deposit превращает **один** депозит в **N неравных** commitments **в одной транзакции**.

```
Tx1: A → pool_vault   total: 1.000 SOL   (deposit_split, N=3)
     создаёт commitments:
       c₁ = hash_3(n₁, s₁, 0.5 SOL)
       c₂ = hash_3(n₂, s₂, 0.3 SOL)
       c₃ = hash_3(n₃, s₃, 0.2 SOL)

Tx2: pool_vault → B₁  amount: 0.5 SOL   (withdraw c₁)
Tx3: pool_vault → B₂  amount: 0.3 SOL   (withdraw c₂)
Tx4: pool_vault → B₃  amount: 0.2 SOL   (withdraw c₃)
```

Наблюдатель видит: одна транзакция депозита 1 SOL, три commitment'а, три вывода по 0.5 / 0.3 / 0.2. **Он не может определить**, что эти три вывода — от одного депозита, если они идут на разные адреса с разными интервалами.

**Ключевое:** on-chain **не видно** самого разбиения. В transaction data депозита — только `total_amount = 1 SOL` и три непрозрачных 32-байтовых commitments. `Σ splits_i == total_amount` доказывается в circuit'е, не публикуется.

### 1.3. Почему Путь 2, а не Путь 1

Возможны два дизайна:

**Путь 1 — только сторона депозита.**
Инструкция `deposit_split` принимает N commitments + N new_roots + total_amount. Схема вывода **не меняется**. При выводе каждая нота доказывает "я знаю секреты некоторого commitment'а" — как сейчас.

- Плюс: минимум изменений, verifier не пересобирается.
- Минус: **on-chain виден N и виден факт разбиения**. Наблюдатель знает, что это split-депозит, и может коррелировать выводы по времени и по количеству.
- Минус: **никто не проверяет, что `Σ amount_i == total_amount`**. Злонамеренный депозитор может заявить `total_amount = 0.1 SOL`, а commitments построить с суммами `0.5 + 0.3 + 0.2 = 1.0`. Vault получит 0.1 SOL, а вывести можно 1.0. **Прямая кража из пула.**

**Путь 2 — схема вывода меняется.**
То же, плюс:
- Новый публичный вход `total_amount`.
- Новый приватный вход `splits[3]`.
- Новый constraint: `Σ splits_i == total_amount` **и** `splits[this_note_index] == amount`.

- Плюс: **сумма разбиения доказывается в ZK**. Атакующий из Пути 1 не может вывести больше, чем задепонировал.
- Плюс: **on-chain не видно разбиения**. Только `total_amount`.
- Минус: verifier пересобирается, ACIR меняется, старые ноты несовместимы.

**Выбор — Путь 2.** Путь 1 дырявый (кража), Путь 2 — единственный корректный. См. `PROJECT_CONTEXT.md` §7.6 — там же сказано *"Changes public input layout (N commitments, aggregated amount)"*, что описывает именно Путь 2.

---

## 2. Дизайн

> **Зачем этот раздел.** Зафиксировать все изменения в одном месте — чтобы не разошлись между слоями.

### 2.1. N = 3, фиксированное

**Почему фиксированное:** массив переменной длины в Noir — это `[Field]` с динамическим размером, что требует дополнительных constraints (проверка длины, padding). Фиксированное `[Field; 3]` — константа на этапе компиляции, как `TREE_DEPTH`.

**Почему именно 3:** два разбиения — едва ли разбиение (0.5 + 0.5 тривиально коррелируется). Четыре — убывающая отдача: больше commitments = больше `add_root` вызовов = дороже транзакция, а выигрыш в приватности растёт медленно. Три — достаточный минимум для "неравных сумм".

**Константа:** `SPLIT_COUNT: u32 = 3` в `spec.json`, в circuit'е (`global SPLIT_COUNT`), в Anchor (`constants::SPLIT_COUNT`).

### 2.2. Новые входы схемы

**Публичные входы — было 5, стало 6.**

| # | Имя | Тип | Размер | Что |
|---|---|---|---|---|
| 0 | `root` | field | 32 | без изменений |
| 1 | `nullifier_hash` | field | 32 | без изменений |
| 2 | `recipient` | pubkey | 32 | без изменений |
| 3 | `recipient_binding` | field | 32 | без изменений |
| 4 | `amount` | u64→field | 32 | без изменений |
| 5 | **`total_amount`** | **u64→field** | **32** | **НОВЫЙ** — сумма всего депозита |

**Порядок выбран так, чтобы существующие 5 позиций не сдвинулись.** `encode_public_inputs` дополняется одним блоком в конце, а не переписывается.

**Приватные входы — было 5, стало 6.**

| # | Имя | Тип | Что |
|---|---|---|---|
| 0 | `nullifier` | field | без изменений |
| 1 | `secret` | field | без изменений |
| 2 | `note_secret` | field | без изменений |
| 3 | `merkle_proof` | `[Field; 20]` | без изменений |
| 4 | `is_even` | `[bool; 20]` | без изменений |
| 5 | **`splits`** | **`[Field; 3]`** | **НОВЫЙ** — вектор разбиения |
| 6 | **`note_index`** | **`u32`** | **НОВЫЙ** — какой элемент `splits` соответствует этой ноте |

**Почему `note_index`:** в общем случае нота может быть **любой** из трёх. Без `note_index` схема не знает, какой `splits[i]` сравнивать с `amount`.

### 2.3. Новый constraint

Было три (C1, C2, C3). Стало **пять**:

| # | Формула | Что доказывает |
|---|---|---|
| C1 | `hash_1(nullifier) == nullifier_hash` | без изменений |
| C2 | `hash_2(note_secret, recipient) == recipient_binding` | без изменений |
| C3 | `compute_merkle_root(hash_3(nullifier, secret, amount), ...) == root` | без изменений |
| **C4** | **`Σ splits[i] == total_amount`** | **НОВЫЙ** — сумма разбиения совпадает с агрегатом |
| **C5** | **`splits[note_index] == amount`** | **НОВЫЙ** — amount этой ноты — один из элементов разбиения |

**C4** защищает от кражи (Путь 1), **C5** — от подмены индекса.

**Вопрос на будущее:** нужен ли C4 **и** C5 вместе, или достаточно C4 + проверки `amount ∈ splits`? Пока пишем оба — они дешёвые (сумма и lookup), а без C5 нельзя гарантировать, что именно `splits[note_index]` совпадает с `amount` (сумма могла сойтись с другими значениями).

### 2.4. Layout публичных входов

**Было (172 байта):**
```
[12-byte header]
  NR_PUBLIC_INPUTS (u32 BE) = 5
  0                 (u32 BE) = 0
  NR_PUBLIC_INPUTS (u32 BE) = 5
[5 × 32 bytes]
  root
  nullifier_hash
  recipient
  recipient_binding
  amount (right-aligned u64)
```

**Стало (204 байта):**
```
[12-byte header]
  NR_PUBLIC_INPUTS (u32 BE) = 6
  0                 (u32 BE) = 0
  NR_PUBLIC_INPUTS (u32 BE) = 6
[6 × 32 bytes]
  root
  nullifier_hash
  recipient
  recipient_binding
  amount (right-aligned u64)
  total_amount (right-aligned u64)
```

**Что меняется в коде:**
- `spec.json` → `circuit.nr_public_inputs = 6`, `witness_layout.total_bytes = 204`.
- `onchain/programs/zk_pool/src/constants.rs` → `NR_PUBLIC_INPUTS = 6`, `PUBLIC_INPUTS_BYTES = 204`.
- `onchain/programs/zk_pool/src/encoding.rs::encode_public_inputs` → +32 байта в конце.
- `services/prover/src/witness.rs` → новое поле в `Prover.toml`.
- `web/src/withdraw/buildWitness.ts` → новое поле в witness'е.

**Один и тот же порядок** во всех четырёх местах. `validate-spec` ловит рассинхрон.

### 2.5. Изменения на каждом слое

| Слой | Файлы | Что меняется |
|---|---|---|
| spec | `circuits/withdrawal/spec.json` | +1 публичный вход, +2 приватных, +2 constraint |
| validator | `scripts/validate-spec/src/rules.rs` | правила под новое число входов и constraints |
| circuit | `circuits/withdrawal/src/main.nr` | новые параметры `main()`, +2 constraint |
| circuit | `circuits/withdrawal/src/test_witness.nr` | генерация splits + note_index |
| ACIR | (генерируется) | `withdrawal.json` — новый хеш |
| Sunspot | (генерируется) | `.ccs`, `.pk`, `.vk` — новые; `.so` — новый |
| Anchor | `onchain/programs/zk_pool/src/constants.rs` | `NR_PUBLIC_INPUTS = 6`, `PUBLIC_INPUTS_BYTES = 204`, `SPLIT_COUNT = 3` |
| Anchor | `onchain/programs/zk_pool/src/encoding.rs` | +32 байта в конце |
| Anchor | `onchain/programs/zk_pool/src/instructions/deposit.rs` | **новый** `deposit_split` |
| Anchor | `onchain/programs/zk_pool/src/error.rs` | новые коды ошибок |
| Anchor | `onchain/programs/zk_pool/src/events.rs` | событие для split (или переиспользуем `DepositEvent` × N) |
| backend | `services/backend/src/indexer.rs` | парсинг N событий в одной транзакции |
| backend | `services/backend/src/api_types.rs` | `WithdrawRequest` +1 поле |
| backend | `services/backend/src/main.rs` | `WithdrawRequest::validate` +1 проверка |
| merkle | `services/merkle/src/app.js` | (не меняется) |
| prover | `services/prover/src/witness.rs` | +1 поле в `WitnessInputs`, +1 в `to_toml()` |
| web | `web/src/noir/*.ts` | (не меняется — те же ACIR `hash2.json`, `hashes.json`) |
| web | `web/src/deposit/useDeposit.ts` | ветка для split-депозита |
| web | `web/src/withdraw/buildWitness.ts` | +1 поле |
| web | `web/src/components/DepositForm.vue` | UI для N сумм |

---

## 3. Что НЕ меняется

> **Зачем этот раздел.** Явно перечислить, что остаётся — чтобы случайно не тронуть.

### 3.1. Verifier Program ID

**`5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`** — **не меняется**.

Используем `solana program deploy --program-id <existing>` с новым `.so`. Upgrade authority — `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (наш кошелёк).

**Следствие:** `VERIFIER_PROGRAM_ID` в четырёх местах **не трогаем**:
- `onchain/programs/zk_pool/src/constants.rs`.
- `web/src/constants.ts`.
- `tests/src/helpers.rs`.
- `docs/PROJECT_CONTEXT.md` §3.

Это убирает целый класс ошибок "какой адрес ты имел в виду".

### 3.2. `TREE_DEPTH = 20`

Не меняется. Merkle tree — тот же. `MAX_LEAVES = 2^20`.

### 3.3. Seeds PDA

Не меняются:
- `b"pool3"` — pool.
- `b"vault3"` — vault.
- `b"nullifier_record"` — nullifier record.

`pool3` / `vault3` — versioned. Если бы нам понадобилось **изменить layout PoolState** (не понадобится) — пришлось бы бампить до `pool4` / `vault4`. Мы layout не меняем.

### 3.4. Формат note

**JSON-структура note не меняется:**
```json
{
  "nullifier": "...",
  "secret": "...",
  "note_secret": "...",
  "amount": "...",
  "commitment": "...",
  "nullifier_hash": "...",
  "tx_signature": "...",
  "pool_pda": "..."
}
```

**Добавляются два новых поля для split-нот:**
```json
{
  ...,
  "splits": ["0x...", "0x...", "0x..."],
  "note_index": 0
}
```

**Почему добавляются, а не заменяются:** обратная совместимость формата (не семантики). Старые парсеры (`parseNote.ts`) не сломаются, если просто проигнорируют новые поля. Но **семантика** ломается: нота v0.1.0 без `splits` невыводима в новой схеме.

---

## 4. Риски

> **Зачем этот раздел.** Обозначить, где вероятнее всего сломаться — чтобы уделить этим под-этапам больше внимания.

### 4.1. 15.4 — передеплой verifier'а

**Самый рискованный под-этап.** Меняется `.so`, а значит — весь путь:

```
новый ACIR → новый CCS → новый PK/VK → новый .so → upgrade на devnet
```

**Что может пойти не так:**
- `sunspot compile` даёт другой хеш `.ccs` — но это ожидаемо.
- `sunspot setup` (новый PK/VK) требует **того же** toxic waste? **Нет** — setup генерирует новый toxic waste. Это второй trusted setup. Toxic waste **не сохраняется** — та же оговорка, что в Stage 3.2. Для devnet — приемлемо.
- Upgrade на месте — если `solana program deploy` пройдёт, но `.so` битый, verifier перестанет принимать **все** proof'ы, включая старые. Откат — только вручную (у нас есть старый `.so` в чекпоинте `03.3-sunspot-deploy`).

**Митигация:**
- Перед 15.4 — прогнать `sunspot verify` локально со **старым** `.vk` и **старым** `.pw` — убедиться, что старый `.so` работает.
- После 15.4 — прогнать `sunspot verify` с **новым** `.vk` и **новым** `.pw` — убедиться, что новый `.so` работает.
- Только потом — 15.5.

### 4.2. 15.3 — изменение ACIR

**Второй по риску.** Новые входы и constraints → новый ACIR → новые хеши.

**Что может пойти не так:**
- Constraint C4 (сумма) в Noir требует `assert(Σ == total)`. Если сумма считается в `Field` — нет проблем. Если в `u64` — потенциальное переполнение при сложении трёх больших сумм. **Решение:** считать в `Field` (это естественно для Noir).
- Constraint C5 (`splits[note_index] == amount`) требует, чтобы `note_index` был **валидным** (0, 1, 2). `splits[note_index]` при `note_index = 5` — ошибка компиляции (index out of bounds для массива размера 3). Нужна проверка `note_index < SPLIT_COUNT` **перед** индексацией.

**Митигация:**
- Написать тесты **до** изменения основной схемы: `test_split_sum_mismatch`, `test_split_index_out_of_bounds`, `test_split_single_element`.
- Прогнать `nargo test` — все 16 старых + новые.

### 4.3. Обратная совместимость с v0.1.0

**Breaking change.** Согласовано и зафиксировано.

**Что ломается:**
- Ноты от v0.1.0 (2 реальных депозита на devnet из Stage 10) **невыводимы** новой схемой.
- Старые commitments остаются в Merkle tree (листья 0, 1 не удаляются).
- `PoolState.roots` содержит старые roots — они **валидны** для on-chain проверки `is_known_root`, но новый circuit не примет proof для старых commitments (в них нет `splits`, а `total_amount` неизвестно).

**Что делаем:**
- Помечаем в `CHANGELOG.md`: **v0.2.0 — breaking change**.
- Помечаем в `docs/DEMO-NOTICE.md`: старые ноты несовместимы.
- Обновляем `PROJECT_CONTEXT.md` §7.6: "реализовано в Stage 15, breaking change".
- **Не** пытаемся поддерживать обе схемы. Это удвоило бы код, verifier'ы, тесты, E2E — ради двух тестовых нот на devnet.

**Что остаётся валидным:**
- Инфраструктура (Docker, Postgres, Redis, Makefile).
- Program ID verifier'а.
- PDA pool / vault.
- Seed'ы.
- Все тесты — переписываются под новую схему, но структура та же.

---

## 5. План под-этапов

| # | Тема | Слой | Риск |
|---|---|---|---|
| 15.1 | Этот документ | docs | нет |
| 15.2 | `spec.json` — новые входы, constraints | spec | низкий |
| 15.3 | Схема `withdrawal` — новые входы, C4, C5 | circuit | **высокий** |
| 15.4 | Sunspot re-run + upgrade verifier'а на devnet | zk | **наивысший** |
| 15.5 | Anchor — `deposit_split`, `encode_public_inputs` +32 | on-chain | высокий |
| 15.6 | LiteSVM — adversarial для split | tests | средний |
| 15.7 | Backend — N commitments per tx, +1 поле в witness | backend | средний |
| 15.8 | Frontend — split UI | web | средний |
| 15.9 | E2E — 1 SOL → 3 ноты → 3 вывода | e2e | **наивысший** |
| 15.10 | Финальный чекпоинт + CHANGELOG → v0.2.0 | docs | низкий |

**Каждый под-этап** = один commit + push + checkpoint (§0.15).

---

## 6. Common errors

> **Заполним по ходу Stage 15.**

Ожидаемые категории:
- Constraint C4/C5 — ошибки индексации и переполнения.
- Redeploy verifier'а — те же грабли, что в Stage 3.4 (GNARK_VERIFIER_BIN, upgrade authority).
- `encode_public_inputs` — рассинхрон с `.pw` (204 байта вместо 172).
- Borsh для `Vec<[u8; 32]>` — **уже проходили** в Stage 10.2 для `Vec<u8>`.
- Indexer — парсинг нескольких `DepositEvent` в одной транзакции.

---

## 7. Reproduction

> **Placeholder.** Заполним в 15.10 полным воспроизведением Stage 15.

---

## 8. What's next

**После Stage 15:**
- **Stage 16** — marketing, статьи, портфолио (вне этого репозитория).
- **Upgrade cycle** — dependency bumps, отложенные в Stage 14.12.
- **Production-hardening** — пункты из `docs/threat-model.md` §7: verify `new_root` on-chain, client-side proving, MPC trusted setup, увеличение `ROOT_HISTORY_SIZE`.

**Что Stage 15 **не** решает:**
- A1 (corrupt tree via malicious `new_root`) — остаётся известным ограничением.
- A6 (malicious prover видит witness) — остаётся.
- A10 (ROOT_HISTORY_SIZE = 10) — остаётся.

Split deposit добавляет **новый** вектор для будущего анализа: если `splits` утекает из witness, приватность разбиения теряется. Добавим в threat-model после 15.4.

---

## Ссылки

- `docs/notes/02-circuits.md` — circuit, ACIR, constraint.
- `docs/notes/03-sunspot.md` — Groth16, trusted setup, redeploy.
- `docs/notes/04-anchor.md` — `encode_public_inputs`, layout публичных входов.
- `docs/notes/10-e2e.md` — полный E2E, найденные баги.
- `docs/threat-model.md` — 12 атак, 7 инвариантов.

---

## 9. Прогресс

> **Заполняется по ходу Stage 15.**

### 15.1 — Дизайн-док

**Дата:** 2026-09-30. **Commit:** `2285ed4`. **Checkpoint:** `.checkpoints/15.1-design/`.

Создан этот документ. Зафиксированы решения:
- **Путь 2** — схема вывода меняется.
- **N = 3, фиксированное.**
- **Upgrade на месте** — Program ID verifier'а не меняется.
- **Breaking change** — нотам v0.1.0 несовместимы, версия → v0.2.0.

Артефакт чекпоинта: `15-split-deposit.md` (24771 B, SHA-256 `8e194799731d17f5658058eac8698e1d8657d1fc62133c1c3374dfef3b41369d`).

### 15.2 — `spec.json` + `validate-spec`

**Дата:** 2026-09-30. **Commit:** `cec750e`. **Checkpoint:** `.checkpoints/15.2-spec/`.

Обновлены три файла:

| Файл | Что |
|---|---|
| `circuits/withdrawal/spec.json` | +1 публичный вход `total_amount`, +2 приватных `splits[3]` / `note_index`, +2 constraints C4/C5, `circuit.split_count = 3`, `nr_public_inputs = 6`, `witness_layout` 172 → 204 |
| `scripts/validate-spec/src/rules.rs` | константы обновлены (6 / 204 / 192 / 144), +2 новых правила `rule_split_count`, `rule_private_inputs_lengths`, `rule_constraints` ожидает 5 id |
| `scripts/validate-spec/src/spec.rs` | +1 поле `split_count: u64` в `Circuit`, +1 строка в `print_summary` |

Попутно **исправлены устаревшие пути потребителей** в `consumers_of_public_layout[]`:
- `anchor`: `instructions.rs` → `encoding.rs` (функция `encode_public_inputs` там).
- `frontend`: `services/poseidon.ts` → `noir/poseidon.ts` (функция `poseidon2Hash`).
- `frontend`: obligations обновлены под 6 публичных входов и новые приватные.

**Результат:** `validate-spec` — 17 правил, все зелёные.

Артефакты чекпоинта: `spec.json` (8210 B, `ab61481a…8ab49`), `rules.rs` (12969 B, `4199ad04…7abfe`), `spec.rs` (5665 B, `1287d890…81c1b`).

### 15.3 — Изменение схемы

**Дата:** 2026-09-30. **Commit:** `1fc9a5d`. **Checkpoint:** `.checkpoints/15.3-circuit/`.

Обновлены `circuits/withdrawal/src/main.nr` и `test_witness.nr`:
- `global SPLIT_COUNT: u32 = 3;`
- Публичные входы: 6 (добавлен `total_amount` в конец).
- Приватные входы: 7 (добавлены `splits: [Field; 3]`, `note_index: u32`).
- C4: `splits[0] + splits[1] + splits[2] == total_amount`.
- C5: bounds-check `note_index < SPLIT_COUNT`, затем `splits[note_index]` через if/else, затем `== amount`.
- `test_witness.nr` печатает новые поля в `Prover.toml` — порядок: 6 public, 7 private (3 scalar + 20 merkle_proof + 20 is_even + 3 splits + 1 note_index = **52 строки**, было 48).

**Тесты:** 24 в `withdrawal` (было 16). Все проходят.
**Новый ACIR:** `a49bc877135ae75713d2ab7cbe39a69151a606c328f48fe8163c0629787d3262` (было `29ac2e67…91db`). Размер 46113 B (было 41414 B).

**Ошибка при компиляции:** `error: Fields cannot be compared, try casting to an integer first`.
- Причина: `assert((note_index as Field) < (SPLIT_COUNT as Field))`.
- Фикс: сравнивать как `u32` — `assert(note_index < SPLIT_COUNT)`. Оба операнда уже `u32`.
- Урок: в Noir `<`, `>=` работают только на целочисленных типах, не на `Field`.

Артефакты чекпоинта: `main.nr` (8542 B, `a8663293…a14a3`), `test_witness.nr` (3709 B, `ec993029…20bdf`), `withdrawal.json` (46113 B, `a49bc877…d3262`).

### 15.4 — Sunspot re-run + verifier upgrade

**Дата:** 2026-09-30. **Commit:** — (зафиксируем в следующем коммите). **Checkpoint:** `.checkpoints/15.4-verifier/`.

Пайплайн выполнен **полностью локально**, затем — upgrade на devnet **на месте**.

**Что сделано:**

1. **Witness** — `nargo execute` → `withdrawal.gz` (3888 B, было 3825). Новый witness содержит 53 значения (было 48): +1 scalar (`total_amount`) + 3 (`splits[3]`) + 1 (`note_index`).
2. **`Prover.toml`** — пересобран новым скриптом `/tmp/stage-15.4/build-prover-toml.sh`. Формат: 13 строк TOML. Значения этого witness'а представляют ноту `splits[1] = 300 000` lamports из депозита `1 000 000` lamports.
3. **`sunspot compile`** — `.ccs` 645 129 B, **nbConstraints 6308 → 6366** (+58 — стоимость C4 и C5). SHA `218c0524…`.
4. **`sunspot setup`** — `.pk` 2 166 095 B (было 2 145 109), `.vk` **1 360 B** (было 972, +388). SHA `b05df8a3…` / `9a25f71d…`.
5. **`sunspot deploy`** — `.so` **197 056 B** (было 87 312 — в 2.25 раза больше). SHA `f2c3bea0…`. Program ID **не изменился**.
6. **`sunspot prove`** — proof **388 B** (было 324, +64). SHA `8c3ca4e9…`.
7. **`sunspot verify`** — `✅ Verification successful!` (2.40 s, 6366 constraints).
8. **`solana program deploy`** — upgrade на месте, tx `5z68jatu…`, Program ID `5t51iu6a…` сохранён, Authority `5iM6nzaC…` сохранена.

**Артефакты чекпоинта `.checkpoints/15.4-verifier/`:**

| Файл | Размер | SHA-256 (первые 8) |
|---|---|---|
| `withdrawal.json` | 46 113 B | `a49bc877…` |
| `withdrawal.ccs` | 645 129 B | `218c0524…` |
| `withdrawal.pk` | 2 166 095 B | `b05df8a3…` |
| `withdrawal.vk` | 1 360 B | `9a25f71d…` |
| `withdrawal.so` | 197 056 B | `f2c3bea0…` |
| `withdrawal.gz` | 3 888 B | `b14d2314…` |
| `withdrawal.proof` | 388 B | `8c3ca4e9…` |
| `withdrawal.pw` | 204 B | `a6d3f877…` |

---

### Ключевое открытие: `sunspot deploy` перезаписывает keypair

**Симптом (обнаружен чтением исходников до запуска):** `sunspot deploy` **всегда** перезаписывает `<vkName>-keypair.json` в директории VK.

**Источник:** `~/sunspot/go/cmd/deploy.go`, шаги 3 и 4:

```go
// Step 4: Rename outputs
originalKeypair := filepath.Join(vkDir, "verifier_bin-keypair.json")
if err := os.Rename(originalKeypair, keypairPath); err != nil {
        log.Fatalf("Failed to rename keypair.json: %v", err)
}
```

`cargo build-sbf` создаёт **новый** keypair `verifier_bin-keypair.json` при каждой сборке, и `deploy.go` **безусловно** переименовывает его поверх `withdrawal-keypair.json`. Флага «reuse existing keypair» в CLI **нет** (`sunspot deploy --help` показывает только `-h`).

**Последствие:** прямолинейный запуск `sunspot deploy` **уничтожил бы** оригинальный keypair, чей pubkey = `5t51iu6a…`. Program ID изменился бы. Стратегия upgrade-in-place стала бы невозможной.

**Митигация (применена):**

1. **Перед** `sunspot deploy` — забэкапить оригинальный keypair:
   ```bash
   cp circuits/withdrawal/target/withdrawal-keypair.json /tmp/stage-15.4/withdrawal-keypair.ORIGINAL.json
   ```
2. Запустить `sunspot deploy` (он перезапишет `withdrawal-keypair.json`).
3. **Восстановить** оригинальный keypair поверх нового:
   ```bash
   cp /tmp/stage-15.4/withdrawal-keypair.ORIGINAL.json circuits/withdrawal/target/withdrawal-keypair.json
   ```
4. Проверить, что pubkey всё ещё `5t51iu6a…`:
   ```bash
   solana-keygen pubkey circuits/withdrawal/target/withdrawal-keypair.json
   ```
5. Только тогда — `solana program deploy --program-id <existing>`.

**Урок:** `sunspot deploy` — не upgrade-инструмент, а build+keypair-замена. Для upgrade-in-place исходный keypair нужно спасать **до** вызова, а восстанавливать **после**.

**Дополнительная страховка:** оригинальный `.so` (v0.1.0) лежит в `.checkpoints/03.3-sunspot-deploy/withdrawal.so` (87 312 B, `117fae71…`). Если новый `.so` окажется битым на chain — откат возможен одной командой `solana program deploy`.

---

### Второе открытие: proof вырос с 324 до 388 байт

**Симптом:** после `sunspot prove` файл `withdrawal.proof` имеет размер **388 байт**, а не 324.

**Причина — не установлена точно.** Возможные факторы:

- `sunspot deploy` в этот раз собрал `.so` на **новом toolchain** (`solana-program 3.0.0`, `solana-bn254 3.1.2`, `solana-syscalls`), тогда как оригинальный `.so` (Stage 3.3) собирался с более старым окружением. Это могло изменить layout proof'а.
- VK вырос 972 → 1 360 B (+388). Groth16 proof для BN254 обычно фиксирован (2×G1 + 1×G2 = 3 точки → 324 B). Но если Sunspot 1.0.0 для нового toolchain использует другой формат сериализации G2 — размер меняется на +64.
- Это **не ошибка**: `sunspot verify` подтвердил `✅ Verification successful!`. Proof согласован с VK и `.so`.

**Последствие:** on-chain константа `PROOF_LEN = 324` в `onchain/programs/zk_pool/src/constants.rs` **устарела**. Пока zk_pool не обновлён до `PROOF_LEN = 388`, инструкция `withdraw` будет отвергать новые proof'ы на шаге `require!(proof.len() == PROOF_LEN)`.

**Это ожидаемое окно** между 15.4 и 15.5 (принято решение «A» — сначала верификатор, потом Anchor). Дизайн-док §4.1 предвидел этот класс риска.

**TODO (15.5):** обновить в `constants.rs`:
- `NR_PUBLIC_INPUTS: 5 → 6`
- `PUBLIC_INPUTS_BYTES: 172 → 204`
- `PROOF_LEN: 324 → 388`

**TODO (15.5, тесты):** обновить assert'ы в `constants.rs`, `encoding.rs`, `instructions/withdraw.rs` под новые значения.

---

### Третье наблюдение: `.so` вырос в 2.25 раза

**Факт:** `.so` 87 312 → **197 056 B**. Constraint count вырос только на **0.9 %** (6308 → 6366). Прирост `.so` в 2.25 раза **не объясняется** одними constraints.

**Гипотеза:** новая сборка использует `solana-program 3.0.0` и `solana-bn254 3.1.2`, которые могли добавить новые зависимости (ark-ff 0.5.0 рядом с ark-ff 0.4.2 в графе сборки). Bundle вырос.

**Практическое следствие:** rent вырос с 0.444 до **1.0019 SOL** (`solana program show` после апгрейда). Владелец программы (наш кошелёк) заплатил разницу.

**Не блокирует.** Но для будущих апгрейдов — учитывать, что размер `.so` — не константа, а функция от toolchain и dependencies.

---

### Итог 15.4

- ✅ Локальный пайплайн (compile → setup → deploy → prove → verify) полностью пройден.
- ✅ Upgrade на месте: Program ID `5t51iu6a…` сохранён.
- ✅ Роллбэк-путь существует: оригинальный `.so` в `.checkpoints/03.3-sunspot-deploy/`.
- ⚠️ On-chain withdraw **временно сломан** — zk_pool ожидает 324 B proof, verifier возвращает 388 B. Окно закроется в 15.5.
- ⚠️ Три on-chain константы (`NR_PUBLIC_INPUTS`, `PUBLIC_INPUTS_BYTES`, `PROOF_LEN`) должны быть обновлены в 15.5.

### 15.5 — Anchor change

**Дата:** 2026-09-30. **Commit:** `a66bee2`. **Checkpoint:** `.checkpoints/15.5-anchor/`.

Обновлены 7 файлов Anchor-программы:

| Файл | Что |
|---|---|
| `constants.rs` | `NR_PUBLIC_INPUTS: 5 → 6`, `PUBLIC_INPUTS_BYTES: 172 → 204`, `PROOF_LEN: 324 → 388`, + новый `SPLIT_COUNT: 3`; обновлены тесты |
| `error.rs` | + два новых варианта: `SplitSumMismatch`, `NotEnoughRoom` |
| `encoding.rs` | `encode_public_inputs` принимает `total_amount: u64`, выход 204 байта; + тесты |
| `instructions.rs` | + `pub mod deposit_split;` + re-export |
| `instructions/deposit_split.rs` | **НОВЫЙ файл** — 182 строки |
| `instructions/withdraw.rs` | + параметр `total_amount: u64`, обновлён doc-комментарий |
| `lib.rs` | + инструкция `deposit_split` с `DepositSplitArgs` |

**Решения по `deposit_split` (зафиксированы):**
- Аргументы — одна структура `DepositSplitArgs` (не плоский список): Codama генерирует типизированный клиент, IDL чище.
- `add_root` — **один** вызов, с финальным корнем. Промежуточные корни не хранятся (транзакция атомарна, любой из трёх note'ов выводится против финального корня).
- `total_deposits` — **+1** за вызов (одна операция депозита, не три).
- `next_leaf_index` — три последовательных `checked_add(1)`, по одному на commitment.
- `Σ amounts[i] == total_amount` — проверяется **on-chain** + в схеме (C4).
- SOL-перевод — **один** `transfer(total_amount)`.
- События — **три** `DepositEvent`, каждый со своим промежуточным корнем.

**Тесты:**
- `cargo check -p zk_pool` — чисто.
- `cargo test -p zk_pool --lib` — **39 / 39** passed (было 37).
- Новые тесты: `test_split_count`, `test_encode_length_is_204`, `test_amount_and_total_amount_are_distinct`.

**Сборка:**
- `zk_pool.so` — **222 144 B** (было 210 000), SHA-256 `d4eee2d5ffc6d89337f917ea05c9b75b081203705fe79f17ed60e4370b86c735`.
- `zk_pool.json` (IDL) — **21 739 B** (было 16 800), SHA-256 `5ecbaed8a7497e70e407b0b9db32153967a5d8549b93cf89df3b9134cf1be29d`.

**Upgrade на devnet (in place):**
- Program ID `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` — **не изменился**.
- Upgrade tx: `5cZwCsBhxyB4qJACcLHUoGhQcrrxW6cH84R2jZU7D8hEdVb1FPvMMibSP2sNJdqT2ZVkU6rybr4dUHXpkLC5X47h`
- Data extended: 210 000 → 222 144 B (rent доплачен до 1.129 SOL).
- IDL metadata: `C931NVVbVKu4mjh1wjgh7bmFx6j6TfML89ut1GsRQHXk`.

**Окно поломки 15.4 → 15.5 закрыто.** После апгрейда zk_pool on-chain `withdraw` принимает 388-байтовые proof'ы и 204-байтовые public inputs, совпадающие с новым verifier'ом.

**Урок про `anchor program deploy --program-id`:** этот флаг ожидает **base58 Program ID**, а не путь к keypair-файлу (в отличие от `solana program deploy`). Симптом: `error: invalid value ... Invalid Base58 string`. Фикс: либо передать base58-строку, либо — что проще — не передавать флаг вообще, положившись на `Anchor.toml` + дефолтный путь `target/deploy/<program>.json`. Второй вариант применён.

**← next: 15.6 — LiteSVM adversarial tests for split.**
