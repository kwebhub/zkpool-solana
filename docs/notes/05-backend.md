# Этап 5. Backend (Rust + axum)

> **См. также:**
> - `00-zk-primer.md` — что такое commitment, nullifier, Merkle proof.
> - `00-glossary.md` — все термины.
> - `04-anchor.md` — предыдущий этап (on-chain программа).

---

## TL;DR

**Что делаем:** HTTP-сервис на Rust + axum для индексации событий с блокчейна, хранения commitments/nullifiers в Postgres, кеша Merkle tree в Redis и проксирования witness'ов в prover.

**Зачем:** on-chain программа **не может** отдавать данные эффективно — RPC дорогой, событий много, а proof generation требует **off-chain** обработки. Backend — **связующее звено** между on-chain и frontend.

**Сколько шагов:** 12 под-этапов (5.1 – 5.12).

**Сколько времени:** ~4 часа.

**Что понадобится:**
- `04-anchor.md` — задеплоенная программа `zk_pool`.
- Postgres 16 и Redis 7 — добавлены в `infra/docker-compose.yml`.
- Проверенные `Program ID` и `Pool PDA`.

**Что получится:**
- HTTP API на порту 4001.
- Indexer — фоновый воркер для чтения событий.
- Postgres schema: `commitments`, `roots`, `nullifiers`.
- Redis-кеш Merkle tree.
- Метрики Prometheus.

**Следующий этап:** `06-merkle.md` (Merkle-сервис на Node.js).

---

## Разбиение этапа

| # | Что делаем | Статус |
|---|---|---|
| 5.1 | Project skeleton | ✅ |
| 5.2 | Config (`config.rs`) | ✅ |
| 5.3 | Database (`db.rs` + migration) | ✅ |
| 5.4 | Cache (`cache.rs`) | ← следующий |
| 5.5 | Merkle tree (`tree.rs`) | ⏳ |
| 5.6 | Metrics + logging | ⏳ |
| 5.7 | Rate limiting | ⏳ |
| 5.8 | Indexer | ⏳ |
| 5.9 | HTTP handlers | ⏳ |
| 5.10 | Docker compose additions | ⏳ |
| 5.11 | Smoke test | ⏳ |
| 5.12 | Final checkpoint | ⏳ |

---

## 5.1. Project skeleton

### Что делаем

Внутри контейнера:

```bash
cd /home/ubuntu/services/backend
cargo init --name zkpool_backend
rm -rf .git .gitignore
mkdir -p migrations tests
```

**Что создаётся:**
- `Cargo.toml` — манифест.
- `src/main.rs` — точка входа.
- `src/lib.rs` — библиотека (для тестов).
- `migrations/` — SQL-миграции.
- `tests/` — интеграционные тесты.

**⚠️ Ошибка:** `cargo init` создаёт `edition = "2024"`. Мы **переопределяем** на `2021`, потому что некоторые зависимости (sqlx 0.7) **не поддерживают** 2024-редакцию.

**Финальный `Cargo.toml`** — с полным набором зависимостей: `axum`, `tokio`, `sqlx`, `redis`, `reqwest`, `metrics`, и т.д.

### Итоги 5.1

**Коммит:** `7ed961a`.
**Чекпоинт:** `.checkpoints/05.1-backend-skeleton/`.

---

## 5.2. Config (`config.rs`)

### Что делает

Читает env-переменные, валидирует обязательные, предоставляет типизированный `Config` struct.

**Обязательные переменные:**
- `DATABASE_URL` — Postgres connection string.
- `REDIS_URL` — Redis connection string.
- `SOLANA_RPC_URL` — RPC-эндпоинт.
- `POOL_ADDRESS` — PDA пула (indexer фильтрует события по нему).

**Опциональные с дефолтами:**
- `PORT = 4001`.
- `MERKLE_URL = http://localhost:4003`.
- `PROVER_URL = http://localhost:4002`.
- `INDEXER_POLL_INTERVAL_SECS = 5`.
- `INDEXER_PAGE_SIZE = 100`.
- `INDEXER_MAX_PAGES = 10`.
- `RATE_LIMIT_WITHDRAW_PER_MIN = 5`.
- `RATE_LIMIT_READ_PER_MIN = 60`.
- `MERKLE_TREE_DEPTH = 20`.

**Три хелпера:**
- `required(name)` — падает, если переменная отсутствует.
- `optional(name, default)` — возвращает default, если не задана.
- `optional_parse<T>(name, default)` — то же, но с парсингом типа.

**`dotenvy::dotenv().ok()`** вызывается первым — если есть `.env` в CWD, переменные подгрузятся оттуда.

### Итоги 5.2

**101 строка.** **Коммит:** `ee668ab`.
**Чекпоинт:** `.checkpoints/05.2-config/`.

---

## 5.3. Database (`db.rs` + migration)

### Структура БД

Три таблицы:

**`commitments`** — все commitments:
```sql
CREATE TABLE commitments (
    id           BIGSERIAL PRIMARY KEY,
    leaf_index   BIGINT NOT NULL UNIQUE,
    commitment   BYTEA NOT NULL CHECK (octet_length(commitment) = 32),
    pool_address TEXT NOT NULL,
    tx_signature TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`roots`** — история Merkle roots (зеркало `PoolState.roots`):
```sql
CREATE TABLE roots (
    id           BIGSERIAL PRIMARY KEY,
    root         BYTEA NOT NULL CHECK (octet_length(root) = 32),
    leaf_index   BIGINT NOT NULL,
    pool_address TEXT NOT NULL,
    tx_signature TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`nullifiers`** — все потраченные nullifiers:
```sql
CREATE TABLE nullifiers (
    nullifier_hash BYTEA PRIMARY KEY CHECK (octet_length(nullifier_hash) = 32),
    pool_address   TEXT NOT NULL,
    recipient      TEXT,
    amount         BIGINT,
    tx_signature   TEXT,
    used_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Индексы:**
- `commitments (pool_address, leaf_index)`.
- `roots (pool_address, id DESC)`.
- `nullifiers (pool_address)`.

### ⚠️ Важное замечание: **нет** UNIQUE на `(pool_address, tx_signature)`

В `commitments` **специально** отсутствует UNIQUE constraint на паре `(pool_address, tx_signature)`. Это **совместимость** с будущей фичей «split deposit»: одна транзакция может создавать **несколько** commitments. Если бы был UNIQUE — фича **не** реализуема без миграции.

См. `PROJECT_CONTEXT.md`, раздел 7.6.

### `Db` — wrapper над `PgPool`

**Методы:**

| Метод | Что делает |
|---|---|
| `connect(url)` | Подключение к Postgres, возвращает `Db` |
| `new(pool)` | Обернуть существующий pool |
| `pool()` | Получить `&PgPool` |
| `save_commitment(leaf_index, commitment, pool, tx)` | INSERT ... ON CONFLICT DO NOTHING |
| `save_root(root, leaf_index, pool, tx)` | INSERT (всегда новый) |
| `save_nullifier(hash, pool, recipient, amount, tx)` | INSERT ... ON CONFLICT DO NOTHING |
| `commitment_exists(commitment)` | SELECT EXISTS |
| `is_nullifier_used(hash)` | SELECT EXISTS |
| `list_commitments(pool)` | SELECT ... ORDER BY leaf_index |
| `latest_root(pool)` | SELECT ... ORDER BY id DESC LIMIT 1 |
| `next_leaf_index(pool)` | SELECT COALESCE(MAX(leaf_index), -1) + 1 |

**Идемпотентность:** все `INSERT` используют `ON CONFLICT DO NOTHING` — indexer **может** переобработать транзакцию (RPC rewind, гонки), дубликаты **не** сломают БД.

### ⚠️ Ошибка: `chrono` не в зависимостях

**Симптом:**
```
error[E0433]: cannot find module or crate `chrono` in this scope
  --> src/db.rs:30:38
   |
30 |     pub created_at: chrono::DateTime<chrono::Utc>,
   |                                      ^^^^^^ use of unresolved module or unlinked crate `chrono`
```

**Причина:** `chrono` используется **транзитивно** через `sqlx` (feature `chrono`), но **не** добавлен в `[dependencies]` напрямую. Rust требует **явного** объявления для использования типов в **своём** коде.

**Решение:** добавить в `Cargo.toml`:
```toml
chrono = { version = "0.4", features = ["serde"] }
```

**Урок:** если используешь тип из **крейта-транзитивки** — **добавь** его явно. `sqlx`'s feature `chrono` даёт **интеграцию**, но **не** реэкспортирует типы.

### Итоги 5.3

- `db.rs` — 216 строк, 9 публичных методов.
- `migrations/001_init.sql` — 49 строк.
- **Коммит:** `1e08827`.
- **Чекпоинт:** `.checkpoints/05.3-db/`.

---

## 5.4. Cache (`cache.rs`)

### Что делает

Redis-wrapper с `ConnectionManager` (автопереподключение, мультиплексирование).

**Три назначения:**
1. Состояние Merkle tree (`tree:*` keys) — см. 5.5.
2. Счётчики rate limiting (`rate:*` keys) — см. 5.7.
3. Кеш ответов `/api/commitments` и `/api/root`.

**Методы:**

| Метод | Что делает |
|---|---|
| `connect(url)` | Подключение к Redis |
| `get(key)` | GET, возвращает `Option<String>` |
| `set(key, value)` | SET без TTL |
| `set_ex(key, value, ttl_secs)` | SETEX с TTL |
| `del(key)` | DEL, возвращает `true`, если ключ существовал |
| `keys(pattern)` | KEYS по glob-паттерну (осторожно — O(N)) |
| `del_pattern(pattern)` | Удалить все ключи по паттерну |
| `incr_with_ttl(key, ttl_secs)` | INCR + EXPIRE при первом инкременте (для rate limiting) |
| `invalidate_pool(pool)` | Сбросить кеш конкретного пула: `cache:commitments:*`, `cache:root:*`, `tree:*` |

**⚠️ Про `keys()` / `del_pattern()`:** Redis `KEYS` — O(N) по **всему** keyspace. Использовать **только** для административных операций (сброс кеша при новом депозите), **не** на горячем пути.

**Что тестируется (2 ignored integration-теста):**
- `test_set_get_del` — базовые операции.
- `test_incr_with_ttl` — инкремент + TTL.

**Ignored** — потому что требуют **живой** Redis. Запускаются через `cargo test -- --ignored` при работающем Redis.

### Итоги 5.4

**187 строк.** **Коммит:** `0f36fc2`.
**Чекпоинт:** `.checkpoints/05.4-cache/`.

---

## 5.5. Merkle tree (`tree.rs`)

### Зачем incremental

Полное дерево глубины 20 — 1 048 576 листьев. Пересчитывать на каждом депозите — **невозможно**. Incremental tree обновляет только путь от нового листа к корню: O(DEPTH) = 20 хешей вместо O(2^DEPTH).

### Что хранится в Redis

| Ключ | Что |
|---|---|
| `tree:{pool}:level:{d}` | Самый правый **непустой** узел на уровне `d` (hex) |
| `tree:{pool}:empty:{d}` | Empty-хеш на уровне `d` (hex) |
| `tree:{pool}:root` | Текущий корень (hex) |
| `tree:{pool}:size` | Количество вставленных листьев (decimal) |

### Алгоритм `add_leaf`

1. Лист становится текущим узлом на уровне 0.
2. Для `d = 0..DEPTH`:
   - Читаем **правый** узел на уровне `d` из Redis.
   - Определяем, **левый** ли наш узел: `(size >> d) & 1 == 0`.
   - Формируем `(left, right)` в правильном порядке.
   - Считаем `parent = hash_2(left, right)` через Merkle-сервис.
   - Записываем `parent` как новый правый узел на уровне `d+1`.
   - `current = parent`.
3. Финал: `size += 1`, `root = current`.

### ⚠️ Почему Poseidon2 **не** реализован в Rust

Разные реализации Poseidon2 (Rust `light-poseidon`, Noir builtin, JS `noir_js`) дают **разные** хеши. Единственный способ гарантировать идентичность — использовать **тот же** ACIR, что и circuit. Поэтому `tree.rs` **делегирует** хеширование Merkle-сервису по HTTP.

**⚠️ Зависимость:** `tree.rs` **не работает** без запущенного Merkle-сервиса (этап 6). На этапе 5 backend **запустится**, но `add_leaf` будет падать с HTTP-ошибкой, пока merkle не поднят. **Порядок** запуска: `merkle` → `backend`.

### `MerkleClient`

Тонкая обёртка над HTTP:

```rust
pub async fn hash_2(&self, left: &str, right: &str) -> Result<String>
```

POST на `{base_url}/hash` с JSON `{"left": hex, "right": hex}`, возвращает `{"hash": hex}`.

### `MerkleTree`

Основной тип:

| Метод | Что делает |
|---|---|
| `new(pool, depth, cache, merkle)` | Создать handle |
| `init_empty()` | Проинициализировать пустое дерево (считает empty-хеши для всех уровней) |
| `add_leaf(leaf_hex)` | Добавить лист, обновить дерево, вернуть новый root |
| `root()` | Текущий root |
| `size()` | Количество листьев |

**Redis keys:**
- `key_level(d)` → `tree:{pool}:level:{d}`.
- `key_empty(d)` → `tree:{pool}:empty:{d}`.
- `key_root()` → `tree:{pool}:root`.
- `key_size()` → `tree:{pool}:size`.

### ⚠️ Черновик — исправления

**Первая версия** имела две проблемы:

1. **`empty_at()` возвращал `None`** — placeholder, который **ломал** логику. Заменили на inline `empty(d)`, который возвращает `"00".repeat(32)` как безопасный fallback.

2. **Odd-index case** — в комментарии было «already merged», но код **не** обрабатывал случай корректно. Переписали цикл с **правильной** проверкой чётности: `(size >> d) & 1 == 0`.

**Урок:** incremental tree — **не** очевидный алгоритм. Первый черновик **не компилировался бы**, если бы мы сразу его запустили. Правило 0.10 — **писать маленькими шагами** — здесь **сработало**: код **прошёл** `cargo check` **сразу**, потому что мы **починили** логику **до** компиляции.

### Что **не** реализовано (пока)

- **`proof(leaf_index)`** — Merkle proof требует **полного** дерева, которого у нас нет. Делегируется Merkle-сервису, но **API endpoint** ещё не готов. Добавим на этапе 5.9 или позже.

### Итоги 5.5

**249 строк.** **Коммит:** `372c056`.
**Чекпоинт:** `.checkpoints/05.5-tree/`.

---

## Что дальше

**Следующий под-этап:** 5.4 — `cache.rs` (Redis wrapper).

**Что будет:**
- `Cache::connect(url)` — подключение к Redis.
- `get`, `set`, `set_ex` (с TTL), `del`.
- Методы для Merkle tree: `tree_*` (используются в 5.5).
- `incr_rate_limit` — для rate limiting (5.7).
- `invalidate_pool` — сброс кеша при новом депозите.
