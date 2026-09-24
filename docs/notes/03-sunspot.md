# Этап 3. Sunspot: verifier program для Groth16

> **См. также:**
> - `00-zk-primer.md` — что такое Groth16, trusted setup, proof, witness.
> - `00-glossary.md` — все термины.
> - `02-circuits.md` — что такое circuit, ACIR (предыдущий этап).

---

## TL;DR

**Что делаем:** превращаем ACIR circuit'а `withdrawal.json` (41 КБ, 6308 constraints) в **Solana-программу**, которая проверяет Groth16-proof'ы. Плюс — генерируем локально proof и убеждаемся, что он **валиден**.

**Зачем:** on-chain `withdraw` **не может** сам проверять ZK-proof — это **дорого**. Вместо этого он вызывает **отдельную** программу-verifier через **CPI** (Cross-Program Invocation). Verifier получает proof и публичные входы, возвращает `true`/`false`.

**Сколько шагов:** 7 под-этапов (3.0 – 3.6).

**Сколько времени:** ~1.5 часа (из них 5 минут — первая сборка verifier'а).

**Что понадобится:**
- `01-setup.md` — Docker-окружение.
- `02-circuits.md` — ACIR `withdrawal.json`.
- Devnet-кошелёк с ≥ 3 SOL.

**Что получится:**
- Verifier Program ID на devnet: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.
- Локально проверенный proof.
- 9 артефактов (`.ccs`, `.pk`, `.vk`, `.so`, `.gz`, `.proof`, `.pw`, keypair, `Prover.toml`).

**Следующий этап:** `04-anchor.md` (Anchor-программа).

---

## 1. Что такое Groth16

**Groth16** — протокол ZK-доказательств, опубликованный Jens Groth в 2016 году. **Стандарт** в ZK-мире для блокчейнов.

### Свойства

| Свойство | Значение |
|---|---|
| **Размер proof** | **324 байта** — очень компактно |
| **Время верификации** | **1.25 секунды** — быстро |
| **Время генерации** | **43 секунды** — медленно (но offline) |
| **Trusted setup** | **Нужен** |

### Как это работает (упрощённо)

1. **Trusted setup** — генерируются `proving key` (PK) и `verifying key` (VK).
2. **Prover** использует PK + witness → генерирует **proof**.
3. **Verifier** использует VK + публичные входы + proof → возвращает `true`/`false`.

**Ключевое свойство:** verifier **не знает** witness, но **убеждается**, что prover его знает.

### Почему Groth16, а не альтернативы

| Протокол | Размер proof | Trusted setup | On-chain friendly |
|---|---|---|---|
| **Groth16** | 324 B | **Да** | ✅ |
| **PLONK** | ~1 КБ | Универсальный | ⚠️ Транзакция не влезает в Solana |
| **STARK** | ~50 КБ | Нет | ❌ Транзакция не влезает |

**Solana ограничивает** размер транзакции **1232 байта**. Proof 324 байта **влезает**, 1 КБ и 50 КБ — **нет**.

**Groth16 — единственный практичный выбор для Solana.**

### Почему верификация в 35 раз быстрее генерации

**Prover** делает **тяжёлую** работу:
- Вычисляет constraints для **всех** приватных и публичных входов.
- Строит полиномы.
- Применяет proving key.

**Verifier** делает **лёгкую** работу:
- Проверяет **три** парных спаривания (pairings) на эллиптической кривой.
- Парные спаривания — операция **дорогая** в общем случае, но их **всего три**.

**Итог:** asymmetry by design. Prover — медленный и **offline**, verifier — быстрый и **on-chain**.

### Что такое trusted setup и почему это **риск**

**Trusted setup** — разовая процедура генерации PK и VK из constraint system.

**Проблема:** во время setup генерируется **toxic waste** — промежуточные значения. Если их **сохранить**, можно **подделывать** proof, которые verifier **примет** (без знания witness).

**Простая аналогия:** представь, что при генерации ключа от сейфа ты видишь **комбинацию**. Если ты её **запомнил** — можешь открыть сейф **без** ключа.

**MPC ceremony** — способ **устранить** эту проблему: несколько независимых сторон комбинируют вклады так, что **никто** не знает полный toxic waste. Нужно, чтобы **все** стороны сговорились.

**В нашем проекте:** setup делается **одной** стороной (нами). Toxic waste **не сохраняется** (процесс завершается). Для **devnet/demo** это **приемлемо**. Для **production** — нужна **MPC ceremony** (см. `docs/DEMO-NOTICE.md`).

### Что такое CCS

**CCS** (Constraint System) — представление circuit'а для Groth16. Более **низкоуровневое**, чем ACIR.

**Что содержит:**
- **Переменные** (witness + публичные входы + промежуточные).
- **Constraints** в форме полиномиальных уравнений.
- **Метаданные** для генерации PK/VK.

**Файл:** `withdrawal.ccs` — **642 177 байт** (627 КБ).

**Для сравнения:** ACIR `withdrawal.json` — 41 КБ. CCS **в 15 раз больше** — потому что он разворачивает высокоуровневые операции (Poseidon2) в **тысячи** низкоуровневых constraints.

---

## 2. Общая картина

### Пайплайн

```
┌─────────────────────┐
│ withdrawal.json     │  ACIR (41 КБ, 6308 constraints)
│ (from stage 2)      │
└──────────┬──────────┘
           │ sunspot compile
           ▼
┌─────────────────────┐
│ withdrawal.ccs      │  Constraint System (642 КБ)
└──────────┬──────────┘
           │ sunspot setup
           ├─────────────────────┐
           ▼                     ▼
┌─────────────────────┐  ┌─────────────────────┐
│ withdrawal.pk       │  │ withdrawal.vk       │  Verifying key (972 B)
│ Proving key (2 МБ)  │  │                     │
└──────────┬──────────┘  └──────────┬──────────┘
           │                        │ sunspot deploy
           │                        ▼
           │              ┌─────────────────────┐
           │              │ withdrawal.so       │  Solana BPF program (87 КБ)
           │              └──────────┬──────────┘
           │                        │ solana program deploy
           │                        ▼
           │              ┌─────────────────────┐
           │              │ Verifier on devnet  │  Program ID 5t51iu6a...
           │              └─────────────────────┘
           │
           │  ┌─────────────────────┐
           │  │ withdrawal.gz       │  Witness (3825 B)
           │  │ (from nargo execute)│
           │  └──────────┬──────────┘
           │             │
           │  sunspot prove
           ▼             ▼
┌─────────────────────┐  ┌─────────────────────┐
│ withdrawal.proof    │  │ withdrawal.pw       │  Public witness (172 B)
│ Groth16 proof (324) │  │                     │
└──────────┬──────────┘  └──────────┬──────────┘
           │                        │
           └────────────┬───────────┘
                        │ sunspot verify
                        ▼
                    ✅ valid
```

### Два независимых артефакта

**Обрати внимание:** у нас **два независимых** результата:

1. **Verifier program** (`.so` → Solana program) — используется **on-chain** в этапе 4.
2. **Proof** (`.proof` + `.pw`) — генерируется **offline** и передаётся on-chain.

Они **связаны** через VK: proof **принимается** только если он **валиден** для **этого** VK.

---

## 3. Под-этап 3.0: кошелёк Solana

### Зачем

Для деплоя verifier program на devnet нужен **кошелёк** с SOL. Также кошелёк станет **upgrade authority** (в будущем — multisig).

### Создание

**Важно:** создаём **новый** кошелёк (не переиспользуем из v2 — принцип «всё с нуля»).

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana-keygen new --no-bip39-passphrase -o /home/ubuntu/.config/solana/id.json
'
```

### Ожидаемый результат

```
Generating a new keypair
Wrote new keypair to /home/ubuntu/.config/solana/id.json
=====================================================================
pubkey: 5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc
=====================================================================
Save this seed phrase to recover your new keypair:
mixed occur kite boring game enact shadow dream tree hollow cube pole
=====================================================================
```

**⚠️ Сохрани seed phrase!** Запиши **вне** проекта. Не коммить.

### Настройка RPC на devnet

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana config set --url devnet
'
```

### Проверка

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana config get
  solana address
  solana balance
'
```

### Ожидаемый результат

```
RPC URL: https://api.devnet.solana.com
Keypair Path: /home/ubuntu/.config/solana/id.json

5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc

0 SOL
```

### Пополнение через faucet

**Команда `solana airdrop` часто падает** с `429 Too Many Requests`. Надёжнее — **веб-фаусет:**

1. Открыть https://faucet.solana.com
2. Вставить адрес `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`.
3. Выбрать **Devnet**.
4. Запросить **5 SOL**.

### Проверка баланса

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic 'solana balance'
```

**Ожидаемый результат:** `5 SOL`.

### ⚠️ Ошибка №1: `Permission denied` при создании кошелька

**Симптом:**

```
Error: Unable to write /home/ubuntu/.config/solana/id.json: Permission denied (os error 13)
```

**Причина:** Volume `solana/` на хосте создан **от root** (UID 0) при первом запуске контейнера. Пользователь `ubuntu` (UID 1000) **не может** писать.

**Диагностика:**

```bash
ls -la solana/                                                    # на хосте
docker compose -f infra/docker-compose.yml exec solana bash -ic 'id'
```

Показывает: `solana/` — `root:root`, `ubuntu` внутри — `uid=1000(ubuntu)`.

**Решение:**

```bash
sudo chown -R 1000:1000 solana/
```

**Урок:** Volume, впервые созданный контейнером **от root**, остаётся root-owned. Все volumes проекта должны быть созданы **от пользователя** (UID 1000) или приведены к нему.

### ⚠️ Ошибка №2: `solana/cli/config.yml` попал в git

**Симптом:** `git status` показывает `new file: solana/cli/config.yml`.

**Причина:** в `.gitignore` было `solana/*.json`, но **не** `solana/**`. Файлы yml не игнорировались.

**Решение:** заменить в `.gitignore`:

```
# Solana keypairs
solana/*.json
```

на:

```
# Solana (wallet, CLI config)
solana/
**/solana/
```

Плюс убрать из staging:

```bash
git rm --cached solana/cli/config.yml
```

**Урок:** если папка содержит **и** приватное, **и** конфиги — игнорируй **всю** папку.

---

## 4. Под-этап 3.1: `sunspot compile`

### Зачем

Преобразовать **ACIR** (`withdrawal.json`) в **CCS** — представление для Groth16.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot compile target/withdrawal.json
'
```

### Ожидаемый результат

```
Loading ACIR file: target/withdrawal.json
06:10:00 INF compiling circuit
06:10:00 INF parsed circuit inputs nbPublic=0 nbSecret=0
06:10:00 INF building constraint builder nbConstraints=6308
Compilation successful.
💾 CCS written to target/withdrawal.ccs
```

### Что значат эти строки

- **`nbPublic=0 nbSecret=0`** — Sunspot **не видит** публичные и приватные входы **на этом уровне**. Они появятся **позже** — при генерации witness.
- **`nbConstraints=6308`** — количество ограничений в circuit'е. Влияет на размер `.ccs` и время proof'а.
- **`Compilation successful`** — `.ccs` записан.

### Артефакт

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.ccs` | 642 177 байт | `a2baffa46b1b3e04631c68d58069d0909bb535297972ed466a29b14b1ee2fa0f` |

**Чекпоинт:** `.checkpoints/03.1-sunspot-compile/`.

---

## 5. Под-этап 3.2: `sunspot setup`

### Зачем

Из CCS **сгенерировать** proving key (PK) и verifying key (VK).

### ⚠️ Критично: что такое trusted setup

См. раздел 1 «Что такое trusted setup и почему это **риск**».

**Коротко:** во время setup генерируется **toxic waste**. Если его сохранить — можно **подделывать** proof. Для **demo** это **приемлемо**. Для **production** нужна **MPC ceremony**.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot setup target/withdrawal.ccs
'
```

### Ожидаемый результат

```
🔧 Loading CCS file: target/withdrawal.ccs
💾 Proving key written to target/withdrawal.pk
💾 Verifying key written to target/withdrawal.vk
✅ Setup complete!
```

### Артефакты

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.pk` | 2 145 109 байт (2 МБ) | `1e7a66426f613ff51e356023ae3499711b2c7f4259d5eb74c40e036f994f87f7` |
| `withdrawal.vk` | 972 байта | `6279a9e6e434c108869bb9b64c8ff20e66cecd3461a5d8cdd4941b817c9d7aed` |

### Что значат `.pk` и `.vk`

- **`.pk` (proving key)** — используется prover'ом. **Секретный**. Если утечёт — атакующий **сможет** генерировать proof'ы, но **не сможет** их **подделать** (для подделки нужен toxic waste).
- **`.vk` (verifying key)** — используется verifier'ом. **Публичный**. **Встраивается** в verifier program на этапе 3.3.

**Чекпоинт:** `.checkpoints/03.2-sunspot-setup/`.

---

## 6. Под-этап 3.3: `sunspot deploy`

### Зачем

Из VK собрать **Solana-программу** (verifier) в формате **BPF**.

### Что такое BPF

**BPF** (Berkeley Packet Filter) — формат байткода, который исполняет Solana.

**Почему BPF:**
- **Безопасность:** программа не имеет прямого доступа к памяти хоста.
- **Детерминизм:** одинаковая логика на всех валидаторах.
- **Портативность:** один формат для всех языков (Rust, C, C++).

**Компиляция:** Rust → BPF через `cargo build-sbf` (**S**olana **B**inary **F**ormat — старая аббревиатура, теперь BPF).

**Ключевое:** `cargo build-sbf` **не то же самое**, что `cargo build`. Первое компилирует под **Solana VM**, второе — под **обычный** Linux.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot deploy target/withdrawal.vk
'
```

**⏱ Первая сборка — 3–5 минут.** Rust-крейт `gnark-solana` содержит **сотни** зависимостей (BN254, ark-*, solana-program). SBF-компиляция **медленная**.

### Что происходит внутри

1. Sunspot проверяет наличие `~/sunspot/gnark-solana/crates/verifier-bin`.
2. Запускает `cargo build-sbf` для крейта `verifier-bin`.
3. Собирает BPF-программу, **встраивая** VK как константу.
4. Создаёт **keypair** для Program ID.

### Ожидаемый результат (сокращённо)

```
Using VK file: /home/ubuntu/circuits/withdrawal/target/withdrawal.vk
Using verifier-bin crate directory: /home/ubuntu/sunspot/gnark-solana/crates/verifier-bin
Running cargo build-sbf...
   Compiling proc-macro2 v1.0.103
   ... (сотни крейтов) ...
warning: use of deprecated constant `solana_bn254::prelude::ALT_BN128_ADD`
... (6 warnings) ...
    Finished `release` profile [optimized] target(s) in 51.60s
Build completed successfully:
  Program: /home/ubuntu/circuits/withdrawal/target/withdrawal.so
  Keypair: /home/ubuntu/circuits/withdrawal/target/withdrawal-keypair.json
```

### ⚠️ 6 предупреждений `deprecated` — это нормально

```
warning: use of deprecated constant `solana_bn254::prelude::ALT_BN128_ADD`:
         Please use `ALT_BN128_G1_ADD_BE` instead
```

**Что это:** Sunspot использует **устаревшие** константы `solana-bn254` (старый API). Новый API — `*_BE` (big-endian).

**Это не ошибка** — компиляция **проходит**. **Известная** проблема upstream Sunspot. Не влияет на корректность verifier'а.

**Что НЕ делать:** «починить» warnings. Это **внешний** крейт, мы его **не контролируем**.

### Артефакты

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.so` | 87 312 байт | `117fae71a4e6f421a0b31200f98e80d4955474d60080006c22881f9354e908b7` |
| `withdrawal-keypair.json` | 224 байта | `460c1eb4637a80d6cf22508eb292492533e736c74f2cc6ff0808ab20c6d597a1` |

**Verifier Program ID:** `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.

**Чекпоинт:** `.checkpoints/03.3-sunspot-deploy/`.

### ⚠️ Ошибка: `GNARK_VERIFIER_BIN directory does not exist`

**Симптом:** `sunspot deploy` падает с сообщением, что `GNARK_VERIFIER_BIN` **не существует**.

**Причина:** Sunspot установлен через `.deb` — содержит **только** CLI-бинарь. Но `sunspot deploy` **компилирует** verifier program из **Rust-крейта** `gnark-solana`, которого в `.deb` **нет**. Нужен **полный git-клон** репозитория Sunspot.

**Решение:**

```bash
git clone https://github.com/reilabs/sunspot.git /home/ubuntu/sunspot
```

**Куда это делось в v3:** в `infra/docker/Dockerfile.solana` **сразу** добавлен `git clone sunspot`. Клон **персистентен** в контейнере.

**В v2:** клон делался **вручную** после первого падения `sunspot deploy`. При пересоздании контейнера **терялся** — и деплой **снова** падал.

**Переменная окружения** (прописана в Dockerfile):

```bash
export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"
```

**Урок:** `.deb` содержит **только** CLI. Для компиляции verifier'а нужен **полный** клон.

---

## 7. Под-этап 3.4: `solana program deploy`

### Зачем

Загрузить verifier program (`.so`) на **devnet**, чтобы она стала **доступна** для вызовов.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  solana program deploy target/withdrawal.so \
    --program-id target/withdrawal-keypair.json \
    --url devnet
'
```

**Что значат флаги:**
- `--program-id` — использовать **существующий** keypair для получения Program ID.
- `--url devnet` — сеть devnet (не mainnet).

### Ожидаемый результат

```
Program Id: 5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ

Signature: wE9xVi9Er6NX2GF57dzedxKtKP6KQPrdZS2Nb2oqsfrRkpSjRZm8PQA8WK8pjPa8G728cb1WRLfGFEm2uJ5kZVU
```

### Проверка

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana program show 5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ --url devnet
  solana balance
'
```

### Ожидаемый результат

```
Program Id: 5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ
Owner: BPFLoaderUpgradeab1e11111111111111111111111
ProgramData Address: 4RyRCMSUsA6VJkRLFABuTxT4e2baovEy9fWH67MTUUFt
Authority: 5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc
Last Deployed In Slot: 503334917
Data Length: 87312 (0x15510) bytes
Balance: 0.4444238 SOL

4.55428808 SOL
```

### Что значат эти строки

- **Program Id** — публичный адрес verifier'а.
- **Owner: `BPFLoaderUpgradeab1e...`** — программа **upgradeable**. Код хранится **отдельно**, а Program ID — ссылка.
- **ProgramData Address** — где **реально** лежит код.
- **Authority** — кто **может** обновлять программу. У нас — наш кошелёк.
- **Data Length: 87312** — размер кода в байтах.
- **Balance: 0.444 SOL** — **rent** за хранение программы на блокчейне.

### Что такое Solana program vs ProgramData

В Solana **upgradeable** программы устроены так:
- **Program ID** — «метка» (адрес), **постоянная**.
- **ProgramData** — аккаунт, где **хранится код**. Может **меняться** при обновлении.

**Зачем разделение:** чтобы **обновлять** код, **не меняя** адрес. Клиенты **всегда** знают, куда обращаться.

**Authority** — кошелёк, имеющий право **обновлять** ProgramData. У нас — наш кошелёк. В production — **multisig + timelock**.

### Что такое rent

В Solana **хранение данных** платное. Аккаунт должен иметь баланс ≥ **rent-exempt** минимум (чтобы его не удалили). Для verifier'а 87 КБ — **0.444 SOL**.

**Rent — не списывается** со временем (rent-exempt). Просто **замораживается** в аккаунте. Если программу **удалить** — SOL вернётся.

**Чекпоинт:** `.checkpoints/03.4-deploy-verifier/`.

---

## 8. Под-этап 3.5: локальная проверка

### Зачем

**Убедиться**, что весь пайплайн работает **end-to-end**:
1. Circuit **корректен** — все constraints удовлетворяются witness'ом.
2. Proof **валиден** — verifier с **этим** VK его **принимает**.

Если proof **валиден** локально, то **тот же** VK, **встроенный** в on-chain verifier, даст **тот же** результат. Это **гарантия** согласованности.

### 8.1. Генерация witness-теста

**Проблема:** `nargo execute` требует `Prover.toml` с **конкретными** значениями входов.

**Решение:** **специальный тест** `test_generate_valid_inputs` в `circuits/withdrawal/src/test_witness.nr`, который:
1. Строит **синтетический** 20-уровневый Merkle tree.
2. Вычисляет **root**.
3. Печатает **все** входы в stdout.

**Полный код** — в `02-circuits.md`, раздел 9.

**Почему это меняет ACIR:** тест **добавляется** в circuit, поэтому `withdrawal.json` **изменяется**. Но **constraint system не меняется** — тест использует **те же** функции. Поэтому `.ccs`, `.pk`, `.vk`, `.so` **остаются теми же**.

**Следствие:** **передеплой verifier'а НЕ нужен**. Program ID **остаётся**.

### 8.2. Сборка `Prover.toml`

**Команда:**

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  nargo test test_generate_valid_inputs --show-output 2>&1
'
```

**Вывод:**

```
[withdrawal] Running 1 test function
[withdrawal] Testing test_witness::test_generate_valid_inputs ... ok
--- test_witness::test_generate_valid_inputs stdout ---
=== Prover.toml ===
0x1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2
0x1412cc9d862599e6869a1881c8562062b98537456d9035819288219b5cd3e6e4
0x062afbde1181c71c
0x200cfdb247b0436ba0483327abcf8f83ed65f75a0db8d9ebbb611561d7d3b2e1
0x0f4240
0x018abef7846071c7
0x03157def08c0e38e
0x04a03ce68d215555
0x07b5bad595e238e3
... (19 нулей) ...
0x01
... (19 единиц) ...
=== End ===
-------------------------------------------------------
[withdrawal] 1 test passed
```

**48 строк данных:** 8 одиночных полей + 20 `merkle_proof` + 20 `is_even`.

### 8.3. Извлечение данных

**Сохранение сырого вывода:**

```bash
mkdir -p .checkpoints/03.5-witness
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/withdrawal && nargo test test_generate_valid_inputs --show-output 2>&1' \
  > .checkpoints/03.5-witness/witness-raw.txt
```

**Извлечение только блока между маркерами:**

```bash
awk '
  /^=== Prover.toml ===$/ { capture=1; next }
  /^=== End ===$/        { capture=0 }
  capture
' .checkpoints/03.5-witness/witness-raw.txt > .checkpoints/03.5-witness/data-only.txt
```

**Результат:** 48 строк — по одной hex-величине в каждой.

### 8.4. ⚠️ Ошибка: TOML не принимает hex-числа > 2^63

**Симптом:**

```
Failed to deserialize inputs: input file is badly formed, could not parse,
TOML parse error at line 1, column 8
  |
1 | root = 0x1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2
  |        ^
number too large to fit in target type

note: large Field numbers can be written by wrapping them in double quotes
```

**Причина:** TOML парсер `nargo` видит `0x1e85...` как **число** и пытается распарсить в `i64`. Оно **больше** 2^63. Провал.

**Решение:** **обернуть** значения в **двойные кавычки**. nargo специально поддерживает эту форму для **больших** Field-значений.

**Сборка `Prover.toml` (на хосте):**

```bash
{
  echo "root = \"$(sed -n '1p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "nullifier_hash = \"$(sed -n '2p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "recipient = \"$(sed -n '3p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "recipient_binding = \"$(sed -n '4p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "amount = \"$(sed -n '5p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "nullifier = \"$(sed -n '6p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "secret = \"$(sed -n '7p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo "note_secret = \"$(sed -n '8p'  .checkpoints/03.5-witness/data-only.txt)\""
  echo -n "merkle_proof = [\""
  for i in $(seq 9 28); do
    if [ "$i" -gt 9 ]; then echo -n "\", \""; fi
    echo -n "$(sed -n "${i}p" .checkpoints/03.5-witness/data-only.txt)"
  done
  echo "\"]"
  echo -n "is_even = ["
  for i in $(seq 29 48); do
    if [ "$i" -gt 29 ]; then echo -n ", "; fi
    v=$(sed -n "${i}p" .checkpoints/03.5-witness/data-only.txt)
    if [ "$v" = "0x01" ]; then echo -n "true"; else echo -n "false"; fi
  done
  echo "]"
} > circuits/withdrawal/Prover.toml
```

**Ожидаемый результат:** `Prover.toml` с 10 строками:

```toml
root = "0x1e8508c3c11def8bfecd33c4bf97ce7fd065fea15eafe55c47e35bd056f1c6b2"
nullifier_hash = "0x1412cc9d862599e6869a1881c8562062b98537456d9035819288219b5cd3e6e4"
recipient = "0x062afbde1181c71c"
recipient_binding = "0x200cfdb247b0436ba0483327abcf8f83ed65f75a0db8d9ebbb611561d7d3b2e1"
amount = "0x0f4240"
nullifier = "0x018abef7846071c7"
secret = "0x03157def08c0e38e"
note_secret = "0x04a03ce68d215555"
merkle_proof = ["0x07b5bad595e238e3", "0x00", ..., "0x00"]
is_even = [true, true, ..., true]
```

**Важно:** `Prover.toml` **в `.gitignore`** — не коммитится. Восстанавливается **скриптом** из `test_witness.nr`.

### 8.5. `nargo execute` — генерация witness

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  nargo execute
'
```

**Ожидаемый результат:**

```
[withdrawal] Circuit witness successfully solved
[withdrawal] Witness saved to target/withdrawal.gz
```

**Артефакт:** `withdrawal.gz` — **3 825 байт**.

### 8.6. `sunspot prove` — генерация proof

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot prove \
    target/withdrawal.json \
    target/withdrawal.gz \
    target/withdrawal.ccs \
    target/withdrawal.pk
'
```

**Порядок аргументов:** `circuit.json`, `witness.gz`, `ccs`, `pk`.

**Ожидаемый результат:**

```
Loading ACIR file: target/withdrawal.json
🔧 Loading CCS: target/withdrawal.ccs
🔑 Loading Proving Key: target/withdrawal.pk
📄 Loading Witness: target/withdrawal.gz
⚙️  Generating Groth16 proof...
06:38:26 DBG constraint system solver done nbConstraints=6308 took=6.795249
06:38:26 DBG prover done acceleration=none backend=groth16 curve=bn254
         nbConstraints=6308 took=42.999219
💾 Writing proof to target/withdrawal.proof
💾 Writing public witness to target/withdrawal.pw
✅ Proof generation complete!
```

**⏱ Время: ~43 секунды.** Первая генерация самая медленная.

**Артефакты:**

| Файл | Размер | Что |
|---|---|---|
| `withdrawal.proof` | 324 байта | Groth16 proof |
| `withdrawal.pw` | 172 байта | Public witness |

### Почему proof 324 байта

Groth16 **фиксирует** размер proof:
- **2** элемента G1 (по 32 байта каждый) = 64 байта.
- **1** элемент G2 (64 байта).
- **Плюс** метаданные.

**Итого ~324 байта** независимо от размера circuit'а. Это **фундаментальное** свойство Groth16.

### Почему public witness 172 байта

**Публичный witness** — это **те самые** 5 публичных входов **плюс** заголовок:

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
  amount (right-aligned u64 in 32-byte word)
```

**Итого:** 12 + 160 = **172 байта**.

**КРИТИЧНО:** on-chain `encode_public_inputs` (этап 4) **должен** произвести **точно те же** 172 байта. Мы **сверим** это на этапе 4.

### 8.7. ⚠️ Ошибка: неправильный порядок аргументов `sunspot verify`

**Симптом:**

```bash
sunspot verify target/withdrawal.proof target/withdrawal.pw target/withdrawal.vk
```

↓

```
Error: invalid verification key file: target/withdrawal.proof (must end with .vk)
```

**Причина:** первым аргументом `sunspot verify` ожидает **`.vk`**, а не `.proof`.

**Решение:** порядок — **`.vk`, `.proof`, `.pw`**.

### 8.8. `sunspot verify` — финальная проверка

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot verify \
    target/withdrawal.vk \
    target/withdrawal.proof \
    target/withdrawal.pw
'
```

**Ожидаемый результат:**

```
🔑 Loading Verification Key: target/withdrawal.vk
Loading Proof: target/withdrawal.proof
Loading public witness: target/withdrawal.pw
06:39:24 DBG verifier done backend=groth16 curve=bn254 took=1.249545
✅ Verification successful!
```

**`took=1.25`** — верификация **в 35 раз** быстрее генерации. Это **нормально** для Groth16.

---

## 9. Под-этап 3.6: финальный чекпоинт

### Что сохранено

`.checkpoints/03.6-stage-3-final/`:
- `sources/` — 10 артефактов (`.json`, `.ccs`, `.pk`, `.vk`, `.so`, `keypair.json`, `.gz`, `.proof`, `.pw`, `Prover.toml`).
- `manifest.txt` — SHA-256 всех артефактов.
- `commit.txt` — финальный коммит.

### Все артефакты этапа 3

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.json` | 41 414 байт | `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db` |
| `withdrawal.ccs` | 642 177 байт | `a2baffa46b1b3e04631c68d58069d0909bb535297972ed466a29b14b1ee2fa0f` |
| `withdrawal.pk` | 2 145 109 байт | `1e7a66426f613ff51e356023ae3499711b2c7f4259d5eb74c40e036f994f87f7` |
| `withdrawal.vk` | 972 байта | `6279a9e6e434c108869bb9b64c8ff20e66cecd3461a5d8cdd4941b817c9d7aed` |
| `withdrawal.so` | 87 312 байт | `117fae71a4e6f421a0b31200f98e80d4955474d60080006c22881f9354e908b7` |
| `withdrawal-keypair.json` | 224 байта | `460c1eb4637a80d6cf22508eb292492533e736c74f2cc6ff0808ab20c6d597a1` |
| `withdrawal.gz` | 3 825 байт | `da6779ae9457891ca85378718e2a9637186306790347f1cfe9f29df964321201` |
| `withdrawal.proof` | 324 байта | `d454b20105b339f893b80a64081a76f6f4f9b3fb6927b5eaf9fb1e96e2c05c79` |
| `withdrawal.pw` | 172 байта | `c4ffea4a4f03c123977f3931b548b01f55820840bc144495063798f2acadb07f` |
| `Prover.toml` | 723 байта | `fb144adb85ea5061c13e89b5d217b4e21a485829cf52149a1bc53571cabc92b2` |

---

## 10. Полный список ручных операций

Собрано в одном месте — для будущего туториала:

1. Создать кошелёк (`solana-keygen new`).
2. Настроить RPC (`solana config set --url devnet`).
3. Пополнить через faucet (веб, не CLI).
4. `sunspot compile target/withdrawal.json`.
5. `sunspot setup target/withdrawal.ccs`.
6. `sunspot deploy target/withdrawal.vk`.
7. `solana program deploy target/withdrawal.so --program-id target/withdrawal-keypair.json --url devnet`.
8. `nargo test test_generate_valid_inputs --show-output` (witness-данные).
9. Извлечь блок между маркерами через `awk`.
10. Собрать `Prover.toml` скриптом на хосте (с кавычками).
11. `nargo execute` (генерация `.gz`).
12. `sunspot prove target/withdrawal.json target/withdrawal.gz target/withdrawal.ccs target/withdrawal.pk`.
13. `sunspot verify target/withdrawal.vk target/withdrawal.proof target/withdrawal.pw`.

**13 команд.** Каждая — со своими подводными камнями.

---

## 11. Полный список ошибок

| # | Ошибка | Решение |
|---|---|---|
| 1 | `Permission denied` при создании кошелька | `sudo chown -R 1000:1000 solana/` |
| 2 | `solana/cli/config.yml` попал в git | `.gitignore`: `solana/` полностью |
| 3 | `GNARK_VERIFIER_BIN directory does not exist` | `git clone sunspot` в Dockerfile |
| 4 | `error: str expects 1 generic` | `&str` в Noir не работает |
| 5 | `error: Type annotation needed` для `f"...{arr[i]}"` | Печатать без format-строки |
| 6 | `Integer literal is too large` | Field values < 2^254 |
| 7 | `TOML parse error: number too large` | Обернуть hex в двойные кавычки |
| 8 | `invalid verification key file` | Порядок аргументов `sunspot verify`: `.vk` первым |

---

## 12. Воспроизведение с нуля

Минимальный набор команд для **полного** повторения этапа:

```bash
# 1. Кошелёк (если ещё нет)
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana-keygen new --no-bip39-passphrase -o /home/ubuntu/.config/solana/id.json
  solana config set --url devnet
'
# → записать адрес, пополнить через https://faucet.solana.com

# 2. Компиляция circuit'а в ACIR (если ещё нет)
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal && nargo compile
'

# 3. ACIR → CCS
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal && sunspot compile target/withdrawal.json
'

# 4. CCS → PK + VK
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal && sunspot setup target/withdrawal.ccs
'

# 5. VK → verifier program (.so + keypair)
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal && sunspot deploy target/withdrawal.vk
'

# 6. Деплой на devnet
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  solana program deploy target/withdrawal.so \
    --program-id target/withdrawal-keypair.json \
    --url devnet
'

# 7. Генерация witness-данных и Prover.toml (см. раздел 8.2–8.4)
# 8. nargo execute → withdrawal.gz
# 9. sunspot prove → withdrawal.proof + withdrawal.pw
# 10. sunspot verify → "✅ Verification successful!"
```

---

## 13. Что дальше

**Следующий этап:** `04-anchor.md` — Anchor-программа `zk_pool`.

**Что будет:**
- Три инструкции: `pool` (инициализация), `deposit`, `withdraw`.
- `withdraw` вызывает verifier через **CPI**, передавая 172 байта публичных входов.
- `encode_public_inputs` — Rust-функция, генерирующая **точно те же** 172 байта, что и `withdrawal.pw`.
- Anchor-программа задеплоена на devnet.
- **LiteSVM E2E test** — полный цикл deposit → withdraw с реальным proof.

**Перед прочтением:**
- `00-zk-primer.md` — что такое CPI, PDA, Solana-программы.
- `00-glossary.md` — термины по мере необходимости.

**Ключевой момент:** на этапе 4 мы **сверим** байты, генерируемые `encode_public_inputs`, с **`withdrawal.pw`** — 172 байта. Если **не совпадут** — увидим **сразу**, а не на этапе 8 (frontend).

---

## 14. Ссылки

- [Groth16 paper (Jens Groth, 2016)](https://eprint.iacr.org/2016/260)
- [Sunspot repository](https://github.com/reilabs/sunspot)
- [Solana programs](https://solana.com/docs/core/programs)
- [Solana rent](https://solana.com/docs/core/accounts#rent)
- [BPF loader](https://docs.solanalabs.com/runtime/programs#bpf-loader)
- `docs/notes/00-zk-primer.md` — введение в ZK.
- `docs/notes/00-glossary.md` — все термины.
- `docs/notes/02-circuits.md` — предыдущий этап.
- `docs/notes/04-anchor.md` — следующий этап.
