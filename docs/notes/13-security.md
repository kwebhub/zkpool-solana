# 13. Security

> **Этап 13** проекта zkpool-solana.
> Threat model, adversarial tests, hardening.

---

## TL;DR

Stage 13 — систематический аудит безопасности. Threat model формализован, негативные тесты написаны, зависимости проверены, ограничения демо зафиксированы.

**Связь с DEMO-NOTICE.md:** этот этап — техническое обоснование того, что в `DEMO-NOTICE.md` сказано простыми словами.

---

## 13.1. Threat model

**Дата:** 2026-09-29
**Commit:** `96303b1`

### Что задокументировано

`docs/threat-model.md` — 269 строк, 8 секций:

1. **Assets** — что защищаем (SOL в vault, notes, PoolState, nullifier records, БД, Merkle state).
2. **Actors** — кто угрожает (пользователь, оператор, malicious depositor, malicious withdrawer, frontend host, backend operator, prover, merkle service, devnet RPC).
3. **Invariants (I1–I7)** — что должно быть верно всегда.
4. **Attack scenarios (A1–A12)** — 12 сценариев.
5. **Summary table** — статус каждого.
6. **Trust assumptions** — что мы предполагаем работающим.
7. **Recommendations for production** — что менять для боевого использования.
8. **References.**

### 7 инвариантов

| ID | Инвариант | Обеспечивается |
|---|---|---|
| I1 | Vault balance ≥ sum of unspent commitments | `deposit` переводит SOL, `withdraw` проверяет баланс |
| I2 | Nullifier используется не более одного раза | Anchor `init` на `NullifierRecord` PDA |
| I3 | Root, используемый для вывода, в истории | `pool.is_known_root(&root)` |
| I4 | Recipient в proof совпадает с `to` | `require_keys_eq!` |
| I5 | Proof валиден | CPI к verifier'у |
| I6 | Public inputs совпадают | `encode_public_inputs` + verifier |
| I7 | `commitment = hash_3(nullifier, secret, amount)` | Circuit constraint C3 |

### 12 атак

**Защищены (5):**
- A2 — double-spend (nullifier record PDA).
- A3 — front-running (`recipient_binding`).
- A4 — replay against old root (roots ring buffer).
- A11 — verifier substitution (`#[account(address = ...)]`).
- A12 — vault drainage (проверка баланса + proof binding).

**Частично (3):**
- A7 — XSS в показе note (Vue auto-escape покрывает UI; дальнейшая судьба note — вне нашего контроля).
- A8 — rate-limit bypass (IP-based, обходится через proxy).
- A10 — root history exhaustion (10 записей; attacker может вытеснить legitimate root частыми депозитами).

**Не защищены (4):**
- **A1** — corrupt tree via malicious `new_root`. **Архитектурное** — не проверяется on-chain. Фундаментальное ограничение демо. См. `deposit.rs`, Trust model.
- **A5** — phishing frontend. **Пользовательская** — не наша зона.
- **A6** — malicious prover видит witness → может украсть. **Архитектурное** — prover как сервис. Для production — proving client-side.
- **A9** — commitment forgery (self-harm). **Низкий** — attacker портит только свой лист.

### Ключевые ограничения (важно для портфолио)

1. **Trusted setup для Groth16.** Без MPC-церемонии. Ключ сгенерирован локально в Stage 3.2.
2. **`new_root` не проверяется on-chain.** Классический класс — trusted depositor. Для production требуется Merkle path verification в `deposit`.
3. **Backend видит witness.** Весь `/api/withdraw` payload (включая nullifier, secret, note_secret) проходит через backend → prover.
4. **`ROOT_HISTORY_SIZE = 10`.** Уязвимость к spam-депозитам.
5. **Notes в `localStorage`.** XSS-риск (в текущей реализации notes не сохраняются автоматически — пользователь копирует их вручную).

### Что дальше (для production)

Восемь рекомендаций в §7 threat-model:
1. Verify `new_root` on-chain.
2. Move proving client-side (browser Groth16).
3. Increase `ROOT_HISTORY_SIZE` to ≥ 100.
4. MPC trusted setup.
5. Multisig upgrade authority.
6. Frontend SRI + CSP.
7. Rate limiting by wallet, not IP.
8. Client-side note storage (file, HW wallet, encrypted keystore).

### Уроки

1. **Threat model — не паранойя, а инвентаризация.** Формальный список атак делает явным то, что раньше было в голове.
2. **Пять защищено из двенадцати — честная оценка.** Демо-проект, не production.
3. **A1 (corrupt tree) — самый важный.** Это архитектурный выбор: не тащить Merkle verification в on-chain (дорого по compute). Задокументировано.
4. **A6 (malicious prover) — вторая критичная проблема.** Пока prover — сервис, доверие к нему выше, чем к contract'у. Для production — вынести proving в браузер.
5. **Честная документация ≠ слабость.** "Мы знаем про эту атаку, она вне scope" — сильнее, чем молчание.

---

## 13.2. Расширенные LiteSVM тесты

**Дата:** 2026-09-29
**Commit:** `69bf648`

### Что сделано

`tests/src/test_adversarial.rs` — 7 тестов, каждый соответствует пункту из `docs/threat-model.md`.

### Тесты

| Тест | Сценарий | Ожидание | Результат |
|---|---|---|---|
| `deposit_below_minimum_rejected` | Депозит < MIN_DEPOSIT_AMOUNT | Ошибка | ✅ |
| `deposit_at_minimum_accepted` | Депозит = MIN_DEPOSIT_AMOUNT | OK | ✅ |
| `deposit_with_unchanged_root_rejected` | `new_root == current_root` | Ошибка | ✅ |
| `a1_garbage_new_root_is_accepted` | Произвольный `new_root` | **OK** (документируем ограничение) | ✅ |
| `a9_arbitrary_commitment_is_accepted` | Произвольный commitment | **OK** (self-harm only) | ✅ |
| `a10_root_history_is_bounded` | 11 депозитов → первый root вытеснен | `is_known_root(first) == false` | ✅ |
| `a11_wrong_verifier_program_rejected` | Wrong verifier program ID | Ошибка | ✅ |

### Два типа тестов

**1. "Атака провалится" — проверка защиты:**
- `deposit_below_minimum_rejected`
- `deposit_with_unchanged_root_rejected`
- `a11_wrong_verifier_program_rejected`

**2. "Атака проходит" — документирование ограничения:**
- `a1_garbage_new_root_is_accepted` — не проверяется on-chain.
- `a9_arbitrary_commitment_is_accepted` — self-harm only.
- `a10_root_history_is_bounded` — ROOT_HISTORY_SIZE = 10.

**Второй тип — не баг в тесте.** Это способ зафиксировать в коде, что атака возможна. Если кто-то случайно "починит" (например, добавит проверку root'а в deposit), тест упадёт — и мы об этом узнаем.

### Полный прогон

```
running 15 tests
test result: ok. 15 passed
```

**15 = 8 оригинальных + 7 adversarial.**

### Грабли

1. **`Fake111...` не парсится как base58 Address** (`WrongSize`). Заменили на `payer.pubkey()` — валидный 32-байтовый адрес, но не verifier.
2. **`VERIFIER_ID` не использовался** — импорт удалён.
3. **Clippy: `payer.pubkey().into()` — useless conversion.** `pubkey()` уже возвращает `Address`. Оставили, чтобы соответствовать стилю остальных тестов (`test_deposit.rs` использует тот же паттерн).
4. **Pattern "self-contained test" — не TestSetup.** В `helpers.rs` только `setup_svm()`. Каждый тест-файл сам определяет `init_pool`, `do_deposit` и т.д. Дублирование, но проще в навигации.

### Уроки

1. **Adversarial tests документируют, не только защищают.** `a1_..._is_accepted` — это **валидный** тест. Он фиксирует known limitation.
2. **`#[test]` без `should_panic`** для "attack succeeds" случаев. Просто `assert!(result.is_ok())` с комментарием.
3. **7 тестов покрывают 3 из 12 атак** из threat-model (A1, A9, A10, A11). Остальные (A2–A8, A12) — либо уже в `test_double_spend.rs`/`test_withdraw.rs`, либо вне scope LiteSVM (например, A6 malicious prover).
4. **`cargo test` в `tests/`** — 15 тестов за 0.5 сек. Быстро, без реальной сети.

---

## Что дальше

- **13.2** — расширенные LiteSVM тесты (негативные сценарии).
- **13.3** — backend input validation audit.
- **13.4** — frontend security review.
- **13.5** — `deny.toml` + strict audit.
- **13.6** — финальный чекпоинт.
