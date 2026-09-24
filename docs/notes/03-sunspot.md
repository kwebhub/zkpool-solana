# Этап 3. Sunspot: верifier program для Groth16

## Обзор

Этот этап превращает Noir circuit (`withdrawal.json`, 41 КБ, 6308 constraints) в **on-chain verifier program** на Solana. Это самая **неочевидная** и **ручная** часть проекта — много CLI-команд, каждая со своими подводными камнями.

**Ключевой факт:** Sunspot генерирует **отдельную** Solana-программу (verifier), **независимую** от основной программы `zk_pool`. Основная программа будет вызывать verifier через **CPI** (Cross-Program Invocation).

**Пайплайн:**

```
withdrawal.json  (ACIR, 41 КБ)
    │ sunspot compile
    ▼
withdrawal.ccs   (CCS — Constraint System, 642 КБ)
    │ sunspot setup
    ▼
withdrawal.pk  +  withdrawal.vk  (proving key 2.1 МБ + verifying key 972 Б)
    │ sunspot deploy
    ▼
withdrawal.so  +  withdrawal-keypair.json  (verifier program 87 КБ)
    │ solana program deploy
    ▼
verifier on devnet  (Program ID: 5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ)
```

---

## Разбиение этапа

- **3.0** — Кошелёк Solana (devnet). ✅
- **3.1** — `sunspot compile`. ✅
- **3.2** — `sunspot setup`. ✅
- **3.3** — `sunspot deploy`. ✅
- **3.4** — `solana program deploy`. ✅
- **3.5** — Локальная проверка: witness → proof → verify. ✅
- **3.6** — Финальный чекпоинт этапа 3. ⏳

---

## Этап 3.0. Кошелёк Solana (devnet)

### Что сделано

Создан **новый** devnet-кошелёк (не переиспользован из v2 — принцип «всё с нуля»).

**Адрес:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`

**Путь в контейнере:** `/home/ubuntu/.config/solana/id.json`

**Volume на хосте:** `~/Projects/Solana/zkpool-solana/solana/`

**Баланс:** 5 SOL (пополнено через faucet).

### Команды

**Создание кошелька:**

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana-keygen new --no-bip39-passphrase -o /home/ubuntu/.config/solana/id.json
'
```

**Вывод:**

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

**Настройка RPC:**

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana config set --url devnet
'
```

**Проверка:**

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  solana config get
  solana address
  solana balance
'
```

### Пополнение через faucet

**CLI airdrop часто падает** с `429 Too Many Requests`. **Надёжнее — веб-фаусет:** https://faucet.solana.com

Вставляешь адрес `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`, выбираешь Devnet, запрашиваешь 5 SOL.

**Проверка баланса:**

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic 'solana balance'
```

**Ожидаемый вывод:** `5 SOL`.

### ⚠️ Ошибка: `Permission denied` при создании кошелька

**Симптом:**

```
Error: Unable to write /home/ubuntu/.config/solana/id.json: Permission denied (os error 13)
```

**Причина:** Volume `solana/` на хосте был создан **от root** (UID 0) при первом запуске. Пользователь `ubuntu` в контейнере имеет **UID 1000**, не может писать.

**Диагностика:**

```bash
ls -la solana/                                         # на хосте
docker compose -f infra/docker-compose.yml exec solana bash -ic 'id'
docker compose -f infra/docker-compose.yml exec solana bash -ic 'ls -la /home/ubuntu/.config/solana/'
```

Показывает: `solana/` → `root:root`, а `ubuntu` внутри → `uid=1000(ubuntu)`.

**Решение (на хосте):**

```bash
sudo chown -R 1000:1000 solana/
```

**После этого кошелёк создаётся.**

**Урок:** Volume, который впервые создаётся контейнером **от root**, навсегда остаётся root-owned. Все volumes проекта должны быть созданы **от пользователя**, либо — после первого запуска — приведены к UID 1000 через `chown`.

### ⚠️ Ошибка: `solana/cli/config.yml` попал в git

**Симптом:** `git status` показывает `new file: solana/cli/config.yml`.

**Причина:** `.gitignore` содержал `solana/*.json`, но **не** `solana/**`. Подпапки и yml-файлы не игнорировались.

**Решение:** Заменить в `.gitignore` блок:

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

И убрать из staging:

```bash
git rm --cached solana/cli/config.yml
```

**Урок:** Игнорировать **всю** папку `solana/`, а не отдельные расширения — там лежит и keypair (`id.json`), и конфиг (`cli/config.yml`), и потенциальные lock-файлы.

---

## Этап 3.1. `sunspot compile`

### Что делает

Преобразует ACIR (`.json`) в **CCS** (Constraint System) — представление для Groth16.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot compile target/withdrawal.json
'
```

### Вывод

```
Loading ACIR file: target/withdrawal.json
06:10:00 INF compiling circuit
06:10:00 INF parsed circuit inputs nbPublic=0 nbSecret=0
06:10:00 INF building constraint builder nbConstraints=6308
Compilation successful.
💾 CCS written to target/withdrawal.ccs
```

### Что важно

- **`nbConstraints=6308`** — это количество ограничений в circuit. Влияет на **время** генерации proof (43 секунды) и **размер** `.ccs` (642 КБ).
- **`nbPublic=0 nbSecret=0`** — Sunspot **не видит** публичные входы на этом уровне. Они передаются **отдельно** через witness (`.pw` — public witness) при `prove`.

### Артефакты

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.ccs` | 642 177 байт | `a2baffa46b1b3e04631c68d58069d0909bb535297972ed466a29b14b1ee2fa0f` |

**Чекпоинт:** `.checkpoints/03.1-sunspot-compile/`.

---

## Этап 3.2. `sunspot setup`

### Что делает

Генерирует **proving key** (`.pk`) и **verifying key** (`.vk`) из CCS. Это **trusted setup** — разовая операция.

**⚠️ КРИТИЧНО:** Без **MPC ceremony** тот, кто запускал setup, **видел toxic waste**. Если он его сохранил — может **подделывать** proof. Для **devnet/demo** это **не критично**. Для **production** — **обязательно** MPC.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot setup target/withdrawal.ccs
'
```

### Вывод

```
🔧 Loading CCS file: target/withdrawal.ccs
💾 Proving key written to target/withdrawal.pk
💾 Verifying key written to target/withdrawal.vk
✅ Setup complete!
```

### Артефакты

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.pk` | 2 145 109 байт | `1e7a66426f613ff51e356023ae3499711b2c7f4259d5eb74c40e036f994f87f7` |
| `withdrawal.vk` | 972 байта | `6279a9e6e434c108869bb9b64c8ff20e66cecd3461a5d8cdd4941b817c9d7aed` |

**Чекпоинт:** `.checkpoints/03.2-sunspot-setup/`.

**Что такое toxic waste:** промежуточные значения, использованные при setup. Если их сохранить, можно генерировать **фейковые** proof, которые verifier примет. В **MPC-церемонии** несколько независимых участников комбинируют свои вклады так, что **никто** не знает полный toxic waste.

---

## Этап 3.3. `sunspot deploy`

### Что делает

Собирает **Solana BPF-программу** (verifier) из verifying key.

**Как именно:** `sunspot deploy`:
1. Клонирует (если ещё не клонирован) `gnark-solana` из репозитория Sunspot.
2. Запускает `cargo build-sbf` для крейта `verifier-bin` — Rust-код, который реализует Groth16-верификацию на Solana.
3. Встраивает `.vk` в программу как константу.
4. Создаёт keypair для Program ID.

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot deploy target/withdrawal.vk
'
```

**⏱ Первая сборка — 3–5 минут.** Rust-крейт `gnark-solana` содержит много зависимостей (BN254, ark-*, solana-program), SBF-компиляция медленная.

### Вывод (сокращённо)

```
Using VK file: /home/ubuntu/circuits/withdrawal/target/withdrawal.vk
Using verifier-bin crate directory: /home/ubuntu/sunspot/gnark-solana/crates/verifier-bin
Running cargo build-sbf...
   Compiling proc-macro2 v1.0.103
   ... (много крейтов) ...
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

**Что это:** Sunspot использует **устаревшие** константы `solana-bn254` (старый API). Новый API — `*_BE` (big-endian). **Это не ошибка**, компиляция проходит. **Известная** проблема upstream Sunspot. Не влияет на корректность verifier'а.

**Что НЕ надо делать:** пытаться «починить» эти warnings. Это внешний крейт, мы его не контролируем.

### Артефакты

| Файл | Размер | SHA-256 |
|---|---|---|
| `withdrawal.so` | 87 312 байт | `117fae71a4e6f421a0b31200f98e80d4955474d60080006c22881f9354e908b7` |
| `withdrawal-keypair.json` | 224 байта | `460c1eb4637a80d6cf22508eb292492533e736c74f2cc6ff0808ab20c6d597a1` |

**Verifier Program ID:** `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`

**Чекпоинт:** `.checkpoints/03.3-sunspot-deploy/`.

### ⚠️ Ошибка: `GNARK_VERIFIER_BIN directory does not exist`

**Симптом:** `sunspot deploy` падает с сообщением о том, что `GNARK_VERIFIER_BIN` не существует.

**Причина:** Sunspot установлен через `.deb` пакет — он содержит **только** CLI-бинарь. Но `sunspot deploy` **компилирует** verifier program из **Rust-крейта** `gnark-solana`, который **не входит** в `.deb`. Нужен **полный git-клон** репозитория Sunspot.

**Решение:**

```bash
git clone https://github.com/reilabs/sunspot.git /home/ubuntu/sunspot
```

**Куда это делось в v3:** в `infra/docker/Dockerfile.solana` **сразу добавлен** `git clone sunspot`. Клон **персистентен** в контейнере. В v2 этого не было — и `sunspot deploy` падал после каждого пересоздания контейнера.

**Путь по умолчанию:** `~/sunspot/gnark-solana/crates/verifier-bin`. Sunspot ожидает его наличия.

**Переменная окружения** (уже прописана в `Dockerfile.solana`):

```bash
export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"
```

---

## Этап 3.4. `solana program deploy`

### Команда

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  solana program deploy target/withdrawal.so \
    --program-id target/withdrawal-keypair.json \
    --url devnet
'
```

### Вывод

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

### Вывод

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

### Что важно

- **`Owner: BPFLoaderUpgradeab1e...`** — программа **upgradeable**. Authority — наш кошелёк. В production — **multisig + timelock** (см. `docs/DEMO-NOTICE.md`).
- **`ProgramData Address`** — адрес аккаунта, где хранятся данные программы (не путать с Program ID).
- **Rent:** 0.444 SOL за 87 КБ. Списано из нашего кошелька.
- **Остаток:** 4.554 SOL — **достаточно** для дальнейшей работы (Anchor-программа ~1.5 SOL + запас).

**Чекпоинт:** `.checkpoints/03.4-deploy-verifier/`.

---

## Этап 3.5. Локальная проверка: witness → proof → verify

Это **самая неочевидная** часть. Здесь мы убеждаемся, что:
1. Witness, сгенерированный из `Prover.toml`, **удовлетворяет** constraints.
2. Proof, сгенерированный из witness, **валиден** по `.vk`.

Если **локальный** proof валиден — **on-chain verifier** (тот же `.vk` внутри `.so`) даст **тот же** результат. Это **гарантия**, что вся связка circuit↔verifier↔proof согласована.

### 3.5.1. Генерация witness-теста

**Проблема:** `nargo execute` требует `Prover.toml` с **конкретными** значениями. В v2 `Prover.toml` генерировался из теста `test_generate_valid_inputs --show-output`.

**Решение:** добавить **специальный тест** `test_generate_valid_inputs` в `circuits/withdrawal/src/test_witness.nr`, который:
1. Строит **синтетический** 20-уровневый Merkle tree вокруг commitment.
2. Вычисляет **root**.
3. Печатает **все** входы (публичные и приватные) в stdout.

### 3.5.2. ⚠️ Проблема: изменение `main.nr` меняет ACIR

**Ключевая мысль:** любые изменения в circuit'е (даже добавление `mod test_witness;`) **меняют ACIR**. Это значит:
- `.json`, `.ccs`, `.pk`, `.vk`, `.so` **могут измениться**.
- Verifier Program ID **может измениться** (если пересоздан keypair).
- **Нужно передеплоить** verifier.

**Мы пошли на это сознательно.** Изменения circuit'а **неизбежны** при разработке, лучше сделать их **сейчас**, до on-chain интеграции.

**НО:** в нашем случае **`.json` изменился** (`f154aca0...` → `29ac2e67...`), а **`.ccs`, `.pk`, `.vk`, `.so` НЕ изменились**. Почему? Потому что `mod test_witness;` **не добавляет constraints** — тест использует **те же** `hash_1/2/3` и `compute_merkle_root`, что `main()`. Constraint system **идентичен**. Только **ACIR-обёртка** (сериализация) немного другая.

**Следствие:** **передеплой verifier НЕ нужен.** Program ID остаётся `5t51iu6apRxgLbt91eVZ6YYzHsnmHCVnLGqJtqdfMFWJ`.

### 3.5.3. ⚠️ Проблема: `&str` в Noir 1.0.0-rc.2

**Симптом:**

```rust
fn print_field_array<let N: u32>(name: &str, arr: [Field; N]) { ... }
```

↓

```
error: str expects 1 generic but 0 were given
error: Expected type &str<error>, found type str<12>
```

**Причина:** в Noir 1.0.0-rc.2 строки **не передаются** как `&str`. Тип `str` **требует** generic-параметр (размер): `str<N>`.

**Решение:** **избегать** строковых параметров. Просто **два разных** теста или функция без строкового параметра, где имя **зашито**.

В нашем случае — упростили: **печатаем только значения**, а формат `Prover.toml` собираем **на хосте** (см. 3.5.5).

### 3.5.4. ⚠️ Проблема: `f"...{arr[i]}"` требует явного типа

**Симптом:**

```rust
println(f"  {arr[i]},");
```

↓

```
error: Type annotation needed
error: Could not determine the type of the generic argument `T` declared on the function `println`
```

**Причина:** format-строка `f"..."` в Noir требует **явной** типизации generic-аргумента.

**Решение:** печатать **без** format-строки. `println(value)` работает, если `value` — конкретного типа (`Field`, `u64`, `bool`). Для отступов — не использовать.

### 3.5.5. ⚠️ Проблема: Field values должны быть < 2^254

**Симптом:**

```rust
let note_secret: Field = 0x3333333333333333333333333333333333333333333333333333333333333333;
```

↓

```
error: Integer literal is too large
value exceeds limit of 21888242871839275222246405745257275088548364400416034343698204186575808495616
```

**Причина:** BN254 prime ≈ **2^254** (точнее `21888...616`). Числа **≥ 2^254** не помещаются в `Field`. `0x3333...3333` — это **256 бит** → слишком много.

**Решение:** использовать **маленькие** литералы, **гарантированно** меньше 2^254. Например, `111111111111111111` (18 девяток) — с запасом.

**Урок на будущее:** любой `Field` из внешнего мира (Solana Pubkey — 32 байта, 256 бит) **обязательно** должен быть **редуцирован** к BN254 через модульную арифметику. Это причина, по которой в проекте нужна функция `reduce_to_field`.

### 3.5.6. Полный witness: сборка `Prover.toml`

**Команда (внутри контейнера):**

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
0x00
... (19 нулей) ...
0x01
... (19 единиц) ...
=== End ===
-------------------------------------------------------
[withdrawal] 1 test passed
```

**48 строк данных:** 8 одиночных полей + 20 `merkle_proof` + 20 `is_even`.

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

Получаем **48 строк**, по одной hex-величине в каждой.

### 3.5.7. ⚠️ Проблема: TOML не принимает hex-числа > 2^63

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

**Причина:** TOML парсер nargo видит `0x1e85...` как **число** и пытается его распарсить в `i64`. Оно **больше** 2^63. Провал.

**Решение:** **обернуть** значения в **двойные кавычки**. nargo специально поддерживает эту форму для больших Field-значений.

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

**Результат `Prover.toml`:**

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

**Важно:** `Prover.toml` **в `.gitignore`**. Он **не коммитится**. Восстанавливается **скриптом** из `test_witness.nr`.

### 3.5.8. `nargo execute` — генерация witness

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  nargo execute
'
```

**Вывод:**

```
[withdrawal] Circuit witness successfully solved
[withdrawal] Witness saved to target/withdrawal.gz
```

**`target/withdrawal.gz`** — 3 825 байт. Это **сериализованный witness** для Sunspot.

### 3.5.9. `sunspot prove` — генерация proof

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

**Вывод:**

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

**⏱ Время: ~43 секунды.** Первая генерация самая медленная; последующие могут быть быстрее за счёт кеша.

**Артефакты:**

| Файл | Размер | Что это |
|---|---|---|
| `withdrawal.proof` | 324 байта | Groth16 proof |
| `withdrawal.pw` | 172 байта | Public witness (5 × 32 + 12 header) |

**`withdrawal.pw` — 172 байта.** Это **тот самый** формат `encode_public_inputs`, который мы зафиксировали в `spec.json`. **Важно:** в on-chain программе мы будем **строить** эти 172 байта **сами**, из публичных входов. **Хеш должен совпасть.**

### 3.5.10. ⚠️ Проблема: неправильный порядок аргументов `sunspot verify`

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

### 3.5.11. `sunspot verify` — финальная проверка

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic '
  cd /home/ubuntu/circuits/withdrawal
  sunspot verify \
    target/withdrawal.vk \
    target/withdrawal.proof \
    target/withdrawal.pw
'
```

**Вывод:**

```
🔑 Loading Verification Key: target/withdrawal.vk
Loading Proof: target/withdrawal.proof
Loading public witness: target/withdrawal.pw
06:39:24 DBG verifier done backend=groth16 curve=bn254 took=1.249545
✅ Verification successful!
```

**`took=1.25`** — верификация **быстрее** генерации в **35 раз**. Это **нормально** для Groth16.

---

## Итоги этапа 3.5

**Полный криптографический пайплайн работает:**

1. **Circuit** — 6308 constraints.
2. **Witness** — 3 825 байт, все constraints удовлетворены.
3. **Proof** — 324 байта, Groth16 на BN254.
4. **Verification** — 1.25 секунды, `✅ successful`.

**Что это доказывает:**
- Poseidon2 zero-padding **работает**.
- Merkle root **вычисляется корректно**.
- `recipient_binding` **корректен**.
- Witness **согласован** с circuit.
- Verifying key **валиден**.

**Что осталось:** убедиться, что **on-chain verifier** даёт **тот же** результат. Это будет проверено:
- **Либо** на этапе 3.6 — через отправку proof в verifier program напрямую.
- **Либо** на этапе 4.5 — через LiteSVM E2E-тест.

---

## Полный список ручных операций

Собрано в одном месте — для будущего туториала:

1. Создать кошелёк (`solana-keygen new`).
2. Настроить RPC (`solana config set --url devnet`).
3. Пополнить через faucet (веб, не CLI).
4. `sunspot compile target/withdrawal.json`.
5. `sunspot setup target/withdrawal.ccs`.
6. `sunspot deploy target/withdrawal.vk`.
7. `solana program deploy target/withdrawal.so --program-id target/withdrawal-keypair.json --url devnet`.
8. `nargo test test_generate_valid_inputs --show-output` (генерация witness-данных).
9. Извлечь блок между маркерами через `awk`.
10. Собрать `Prover.toml` скриптом на хосте (с кавычками).
11. `nargo execute` (генерация `.gz`).
12. `sunspot prove target/withdrawal.json target/withdrawal.gz target/withdrawal.ccs target/withdrawal.pk`.
13. `sunspot verify target/withdrawal.vk target/withdrawal.proof target/withdrawal.pw`.

**13 команд.** Каждая со своими **подводными камнями**.

---

## Полный список ошибок

1. **`Permission denied`** при создании кошелька → `sudo chown -R 1000:1000 solana/`.
2. **`solana/cli/config.yml` попал в git** → `.gitignore`: `solana/` полностью.
3. **`GNARK_VERIFIER_BIN directory does not exist`** → `git clone sunspot` в Dockerfile.
4. **`error: str expects 1 generic`** → `&str` в Noir не работает.
5. **`error: Type annotation needed`** для `f"...{arr[i]}"` → печатать без format-строки.
6. **`Integer literal is too large`** → Field values < 2^254.
7. **`TOML parse error: number too large`** → обернуть hex в двойные кавычки.
8. **`invalid verification key file`** → порядок аргументов `sunspot verify`: `.vk` первым.

**8 ошибок.** Каждая зафиксирована выше.

---

## Артефакты этапа 3

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

**Программы на devnet:**

- **Verifier:** `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`

**Чекпоинты:**

- `.checkpoints/03.1-sunspot-compile/`
- `.checkpoints/03.2-sunspot-setup/`
- `.checkpoints/03.3-sunspot-deploy/`
- `.checkpoints/03.4-deploy-verifier/`
- `.checkpoints/03.5-witness/`
