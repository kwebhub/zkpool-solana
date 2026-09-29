# zkpool-solana

> **Приватные переводы SOL в сети Solana с использованием доказательств с нулевым разглашением.**
> Депозит в общий vault. Вывод на любой адрес. Никакой связи между депозитом и выводом на блокчейне.

[![CI](https://github.com/kwebhub/zkpool-solana/actions/workflows/ci.yml/badge.svg)](https://github.com/kwebhub/zkpool-solana/actions/workflows/ci.yml)
[![Security](https://github.com/kwebhub/zkpool-solana/actions/workflows/security.yml/badge.svg)](https://github.com/kwebhub/zkpool-solana/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)

> ⚠️ **Демонстрационный / образовательный проект.** Не аудирован, не для production. См. [`docs/DEMO-NOTICE.md`](../DEMO-NOTICE.md) и [`docs/threat-model.md`](../threat-model.md).

---

## Что делает

Пользователь вносит SOL в общий on-chain vault. Взамен получает **deposit note** — четыре случайных секрета:

- `nullifier` — позже доказывает владение, используется для защиты от двойной траты.
- `secret` — часть commitment'а.
- `note_secret` — привязывает вывод к конкретному получателю (защита от front-running).
- `amount` — сумма депозита (lamports).

Позже пользователь доказывает в zero knowledge, что знает секреты за некоторым commitment'ом в Merkle-дереве пула, **не раскрывая какого именно**. Средства выводятся на любой Solana-адрес.

**Результат:** внешний наблюдатель видит транзакцию депозита и транзакцию вывода, но не может их связать.

**Технология:** Groth16 (BN254), пруф генерируется через [Sunspot](https://github.com/reilabs/sunspot), проверяется программой [gnark-solana](https://github.com/Lightprotocol/gnark-solana), задеплоенной на Solana.

---

## Архитектура

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + TS) — :5173                                    │
│ • Подключение кошелька (Phantom/Solflare)                        │
│ • Генерация note (noir_js в браузере)                            │
│ • Депозит / Вывод                                                 │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — :4001                                    │
│ • /api/health   /api/commitments   /api/root   /api/proof        │
│ • /api/root-preview   /api/withdraw   /metrics                   │
│ • Indexer (RPC polling → events → DB + Merkle tree)              │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003               │
└──────────────┘   └──────────────┘   └──────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — :4002                                     │
│ • nargo execute + sunspot prove                                  │
│ • Groth16 proof generation (serialized by mutex)                 │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ МОНИТОРИНГ                                                       │
│ • Prometheus :9090  • Grafana :3000  (admin/admin)               │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA (devnet)                                                  │
│ • zk_pool program:   8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm│
│ • verifier program:  5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ│
│ • pool PDA:          B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf│
│ • vault PDA:         HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq│
└──────────────────────────────────────────────────────────────────┘
```

**Пять принципов** (зачем существует v3 — v2 погибла из-за расхождения между схемой, Anchor-программой и фронтендом):

1. **Единый источник правды** — `circuits/withdrawal/spec.json`, валидируется через `scripts/validate-spec`.
2. **Проверки контрактов на каждой границе** — по байтам. `scripts/sync-circuits` сверяет хэши ACIR.
3. **Никаких магических чисел** — все константы из spec, распространяются через `validate-spec`.
4. **LiteSVM E2E-тест до devnet.**
5. **Чекпоинты с SHA-256.**

---

## Быстрый старт

### Требования

- Docker + Docker Compose
- ~5 ГБ свободного места (образ + volumes)
- Solana CLI (только если хотите взаимодействовать с программами)

### Запуск

```bash
git clone https://github.com/kwebhub/zkpool-solana
cd zkpool-solana
make up
```

Через ~1 минуту:

| Сервис | URL |
|---|---|
| Frontend | http://localhost:5173 |
| Backend | http://localhost:4001/api/health |
| Prover | http://localhost:4002/health |
| Merkle | http://localhost:4003/health |
| Prometheus | http://localhost:9090 |
| Grafana | http://localhost:3000 (`admin` / `admin`) |

### Makefile

```
make up          — docker compose + merkle + prover + backend
make down        — остановить всё
make reset       — стоп, очистка БД + Redis, рестарт
make status      — статус сервисов
make logs        — tail логов всех сервисов
make web         — Vite dev server (5173)
make build       — собрать backend, prover, merkle, web
make clean       — удалить target/, node_modules/, dist/
make exec-c CMD='...'  — выполнить команду внутри контейнера
```

---

## Полный E2E-сценарий

Реальный цикл депозит → вывод → защита от повторного использования:

```bash
# Депозит 0.001 SOL. Печатает JSON с note.
make exec-c CMD='cd /home/ubuntu/scripts/e2e-deposit && ./target/release/e2e-deposit 1_000_000'

# Подождать ~10 сек, пока indexer подхватит депозит.
# Затем вывести на любой адрес:
make exec-c CMD='cd /home/ubuntu/scripts/e2e-withdraw && ./target/release/e2e-withdraw /tmp/note.json <RECIPIENT>'
```

Подробный разбор с реальными подписями транзакций — в [`docs/notes/10-e2e.md`](../notes/10-e2e.md).

---

## Структура репозитория

```
zkpool-solana/
├── circuits/            ← Noir-схемы (poseidon library, hash2, hashes, withdrawal)
├── onchain/             ← Anchor-программа `zk_pool` (Rust 1.89.0)
├── tests/               ← LiteSVM интеграционные тесты (Rust 1.98.1)
├── services/
│   ├── backend/         ← Rust + axum API + indexer
│   ├── merkle/          ← Node.js + Fastify Poseidon2 + Merkle tree
│   └── prover/          ← Rust + axum Groth16 proof generation
├── web/                 ← Vue 3 + TypeScript + Pug + SCSS
├── scripts/             ← Rust CLI (validate-spec, sync-circuits, pool-init, e2e-*)
├── infra/               ← docker-compose, Dockerfile, prometheus, grafana
├── docs/
│   ├── PROJECT_CONTEXT.md   ← каноническое состояние проекта
│   ├── threat-model.md      ← 12 сценариев атак
│   ├── DEMO-NOTICE.md
│   └── notes/               ← русские технические заметки
├── .github/             ← CI/CD
├── Makefile
└── LICENSE              ← MIT
```

---

## Тесты

| Слой | Тестов | Команда |
|---|---|---|
| Схемы | 41 | `nargo test` в каждом circuit'е |
| On-chain unit | 37 | `make exec-c CMD='cd /home/ubuntu/onchain && cargo test -p zk_pool --lib'` |
| LiteSVM | 15 | `make exec-c CMD='cd /home/ubuntu/tests && cargo test'` |
| Backend unit | 5 | `make exec-c CMD='cd /home/ubuntu/services/backend && cargo test --lib'` |
| Merkle service | 23 | `make exec-c CMD='cd /home/ubuntu/services/merkle && pnpm test'` |
| Prover | 17+1 | `make exec-c CMD='cd /home/ubuntu/services/prover && cargo test --lib'` |
| Web | typecheck | `make exec-c CMD='cd /home/ubuntu/web && pnpm typecheck'` |

**Итого: 139+ тестов.**

---

## Безопасность

Threat model — [`docs/threat-model.md`](../threat-model.md).

**12 сценариев атак:**
- **Защищено (5):** double-spend, front-running, replay старого root'а, подмена verifier'а, слив vault'а.
- **Частично (3):** XSS, обход rate-limit, истощение root history.
- **Не защищено (4, документировано):** порча дерева через `new_root`, фишинг, malicious prover видит witness, forged commitment (self-harm).

**Известные ограничения:**
- Trusted setup для Groth16 (без MPC).
- `new_root` не проверяется on-chain.
- Backend видит все commitments и nullifiers.
- Notes сохраняются пользователем (нет восстановления).

Нашли уязвимость? См. [`SECURITY.md`](../../SECURITY.md).

---

## Документация

- **[PROJECT_CONTEXT.md](../PROJECT_CONTEXT.md)** — каноническое состояние, правила, грабли (единая точка входа для AI-ассистента в новом чате).
- **[threat-model.md](../threat-model.md)** — анализ безопасности.
- **[DEMO-NOTICE.md](../DEMO-NOTICE.md)** — что это за проект и чем не является.
- **[docs/notes/](../notes/)** — технические заметки по каждому этапу.

---

## Происхождение

Основано на [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

Предыдущие версии:
- [`kwebhub/private-transfer`](https://github.com/kwebhub/private-transfer) — v1.
- `kwebhub/solana-zk-pool` — v2 (заброшена; баг в layout'е публичных входов).

v3 — переписан с нуля со строгими контрактами между слоями и побайтовой валидацией.

---

## Лицензия

MIT — см. [LICENSE](../../LICENSE).
