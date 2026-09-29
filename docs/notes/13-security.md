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

## 13.3. Backend input validation audit

**Дата:** 2026-09-29
**Commit:** `d0f3f85`

### Что нашли

До аудита валидация на backend'е была **минимальной**:

**`WithdrawRequest::validate`:**
- Проверялись **только** длины массивов `merkle_proof` и `is_even` (= 20).
- Все hex-строки (`root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`, `nullifier`, `secret`, `note_secret`) — без проверок.

**`ProofQuery`:**
- `leaf_index: u64` — без верхней границы. Атакующий мог послать `leaf_index=18446744073709551615`.
- Backend форвардил в Merkle-сервис → там `.findIndex` → ошибка.

**`RootPreviewRequest`:**
- `commitments: Vec<String>` — без ограничения на длину и содержимое.
- Атакующий мог послать 10M строк → forward в Merkle-сервис → OOM Node.js.

### Что добавили

**1. `validate_hex64(name, v)` — строгая проверка 64-символьного hex.**

Применяется к: `root`, `nullifier_hash`, `recipient_binding`, `nullifier`, `secret`, `note_secret`.

**2. `validate_hex_max(name, v, 64)` — hex до 64 символов.**

Применяется к: `recipient` (может быть короче — leading zeros elided), `amount`, `merkle_proof[i]`.

**3. `MAX_LEAVES = 2^20 = 1_048_576` для `/api/proof`.**

`leaf_index >= MAX_LEAVES` → `400 BAD_REQUEST` **до** обращения к БД и Merkle-сервису.

**4. `MAX_PREVIEW_COMMITMENTS = 2^20` для `/api/root-preview`.**

Плюс каждый commitment проверяется `is_hex64`.

### Тесты (curl)

```
POST /api/withdraw {"root":"not-hex", ...}
→ {"error":"root must be 64 hex chars, got 7"}

POST /api/withdraw {"nullifier_hash":"nothex", ...}
→ {"error":"nullifier_hash must be 64 hex chars, got 6"}

POST /api/withdraw <valid witness>
→ 200 {"proof":"K7lHW...","public_witness":"..."}
  (forwarded to prover, Groth16 generated)

GET /api/proof?leaf_index=99999999999999
→ {"error":"leaf_index must be < 1048576","leaf_index":99999999999999}

POST /api/root-preview {"commitments":["nothex"]}
→ {"error":"commitments[0] must be 64-char hex"}
```

### Грабли

1. **`recipient` короче 64 символов — это OK.** BN254 field element, leading zeros могут быть отброшены. Отсюда `validate_hex_max`, не `validate_hex64`.
2. **`amount` тоже может быть короче.** `0f4240` = 1M lamports.
3. **Верхняя граница по `leaf_index`** — на 2^20 (MAX_LEAVES). Совпадает с on-chain `MAX_LEAVES = 1 << TREE_DEPTH`.
4. **`is_hex64` дублируется** в `main.rs` (для root-preview) и `api_types.rs` (`validate_hex64`). Разные модули — небольшая копипаста. Если появится третий потребитель — вынести в общий модуль.

### Уроки

1. **Валидация входа — не бюрократия.** `leaf_index=99999999999999` без границы — реальная уязвимость DoS. Форвард вниз по стеку — самая частая атака на микросервисную архитектуру.
2. **Проверка на длину ≠ проверка на содержимое.** `len == 64` не говорит, что строка hex. Нужны обе.
3. **Разные типы полей — разные правила.** `root` (32-байтовый хэш) — всегда 64 hex. `recipient` (field element) — до 64 hex. `merkle_proof[i]` — то же. Одна проверка не подходит всем.
4. **Ошибки 400 vs 404.** `leaf_index >= MAX_LEAVES` — `400` (невалидный ввод). `leaf_index >= count` — `404` (валидный, но не существует). Разные семантики.
5. **Валидация **перед** forward'ом.** Если проверять после вызова Merkle-сервиса — смысла нет, атака уже дошла.

---

## 13.4. Frontend security review

**Дата:** 2026-09-29
**Commit:** (docs-only, no source changes)

### Что проверяли

1. XSS через `v-html`, `innerHTML`, `eval`, `Function(`.
2. Хранение notes (`localStorage`, `sessionStorage`, cookies).
3. Взаимодействие с кошельком (`signAndSendTransaction`, `signMessage`).
4. Сетевые запросы (`fetch`, `http://`, `https://`).

### Результаты

**1. XSS — чисто.**

```
$ grep -rn "v-html\|innerHTML\|eval\|Function(" web/src/
(no matches)
```

Vue-шаблоны используют `{{ ... }}` — авто-эскейпинг. Никаких `v-html`.

**2. Хранение notes — чисто.**

```
$ grep -rn "localStorage\|sessionStorage\|document.cookie" web/src/
(no matches)
```

Notes не сохраняются в браузере. Пользователь копирует JSON вручную. **Плюс для безопасности:** XSS не может украсть notes из storage.

**Минус для UX:** при обновлении страницы всё теряется. Но это осознанное решение — notes слишком чувствительны для автоматического хранения.

**3. Кошелёк — чисто.**

Единственный используемый метод — `signAndSendTransaction`. Это:
- **Атомарно** — пользователь видит транзакцию и подтверждает её.
- **Прозрачно** — кошелёк показывает, что именно подписывается.
- **Не `signMessage`** — нет риска, что пользователь подпишет произвольное сообщение (которое в некоторых протоколах может быть опасным).

`makeNoopSigner` (`web/src/wallet/kitSigner.ts`) — **заглушка**, её `signTransactions` кидает ошибку. Реальное подписание идёт через `provider.signAndSendTransaction(base64)`. Noop нужен только чтобы Codama приняла вход.

**4. Сетевые запросы — чисто.**

- `https://api.devnet.solana.com` — hardcoded RPC.
- `/circuits/*.json` — same-origin.
- Никаких сторонних аналитик, CDN, `fetch` на внешние домены.
- Остальные `https://` — doc-комментарии Codama.

### Что осталось вне scope

1. **`Content-Security-Policy`** — не установлен. Для production желателен:
   ```
   default-src 'self';
   connect-src 'self' https://api.devnet.solana.com;
   script-src 'self' 'wasm-unsafe-eval';  // для noir_js WASM
   ```
   Для dev — не критично.

2. **SRI для WASM** — не нужен. WASM грузится из нашего origin (`noir_js` бандлится Vite).

3. **XSS через сохранённый note.** Если пользователь вставит note в другой сервис (чат, email) — мы уже не контролируем. Vue auto-escape покрывает отображение **внутри** нашего приложения.

4. **Phishing frontend** (A5 из threat model). Не защищено. Пользовательская ответственность.

### Грабли

1. **`grep` без `-E`** не работает с alternation `|`. Нужно `grep -rn "a\|b"` (bash) или `grep -rEn "a|b"`.
2. **Comments в Codama** содержат `https://github.com/codama-idl/codama` — grep их подхватывает. Не настоящие запросы.

### Уроки

1. **Vue auto-escape — сильная защита.** Пока не используем `v-html` — XSS через пользовательский ввод практически невозможен.
2. **Отсутствие storage — фича, не баг.** Notes не персистятся → их нельзя украсть через XSS из `localStorage`.
3. **`signAndSendTransaction` предпочтительнее `signMessage`.** Первое — атомарная подпись транзакции, второе — произвольные байты. Для наших сценариев первого достаточно.
4. **Same-origin circuits + hardcoded RPC = минимум внешних запросов.** Не даём XSS-скриптам возможности отправить данные на чужой домен (CSP усилит этот эффект в production).

---

## 13.5. `deny.toml` + строгий аудит

**Дата:** 2026-09-29
**Commits:** `1f35743`, `f31480f`

### Что нашли

`cargo deny check` на backend'е:

```
advisories FAILED, bans ok, licenses FAILED, sources ok
```

**Advisories FAILED:**
```
RUSTSEC-2024-0363 — sqlx 0.7.4 SQL injection via protocol smuggling
  (encoding values > 4 GiB overflows the length prefix)
  Fix: upgrade to >= 0.8.1
```

**Licenses FAILED:**
```
error[rejected]: failed to satisfy license requirements
  ┌─ /home/ubuntu/services/backend/Cargo.toml
  │ license = "MIT"
  │            ━━━
  │            rejected: license is not explicitly allowed
```

**Странно:** MIT был в нашем allow-списке. Причина — schema mismatch: `cargo-deny 0.20.2` не читает `version = 2` в `deny.toml` так, как мы ожидали.

### Что сделано

**1. `sqlx 0.7` → `0.8`:**

```toml
sqlx = { version = "0.8", default-features = false, features = [
  "runtime-tokio", "tls-native-tls", "postgres", "uuid", "chrono", "json",
] }
```

**Собралось без изменений в коде.** Backend перезапущен, `/api/health` → `{"db":true,"status":"ok"}`. Advisories: `ok`.

**2. `cargo-deny init`:**

`cargo-deny init` создал свежий `deny.toml` в `services/backend/`. Схема под 0.20.2 — корректная.

**3. `make exec-c CMD='...'`** — новая цель в Makefile:

```bash
make exec-c CMD='ls -la /home/ubuntu'
```

Обёртка над `docker compose exec solana bash -ic '...'`. Полезно для ad-hoc команд без повторения длинного префикса.

### Что осталось

- **Root `deny.toml`** — пустой `allow` (после копирования свежего init). Нужно заполнить: MIT, Apache-2.0, BSD-3, ISC, Unicode-3.0, Zlib, OpenSSL, CC0-1.0, MPL-2.0.
- **`licenses FAILED`** для наших собственных крейтов (`zkpool_backend v0.1.0` с `license = "MIT"`) — schema mismatch. После заполнения root `deny.toml` — проверить снова.
- **`security.yml`** — `|| true` на audit steps. Убрать после того, как `cargo deny check` полностью зелёный.

### Грабли

1. **`cargo deny init` требует `Cargo.toml` в текущей директории.** `cd /home/ubuntu && cargo deny init` падает — нет `Cargo.toml` на уровне репозитория. `cd services/backend && cargo deny init` — работает.
2. **`sqlx 0.8` не сломал код.** `SqlitePool` / `PgPool`, `Row::get`, `.bind()` — сигнатуры не изменились. Только версия.
3. **`sqlx 0.8.6` всё ещё тянет `time`, `aws-lc-sys`** — компиляция дольше (49 сек vs 12 сек на 0.7). Но advisories OK.

### Уроки

1. **`cargo audit` vs `cargo deny check`.** Первое — только advisories. Второе — advisories + licenses + bans + sources. `cargo deny` строже, но требует правильной конфигурации.
2. **RUSTSEC-2024-0363 — не теоретическая уязвимость.** Демонстрация эксплойта опубликована. Upgrade обязателен, не «recommended».
3. **Version schema в `deny.toml` — меняется между версиями `cargo-deny`.** Наш старый `version = 2` не работает с 0.20.2. Всегда генерировать свежий через `cargo deny init`.
4. **`advisories FAILED` — реально важно. `licenses FAILED` — часто про конфиг.** Разделять эти сигналы.

## Что дальше

- **13.6** — финальный чекпоинт Stage 13.
