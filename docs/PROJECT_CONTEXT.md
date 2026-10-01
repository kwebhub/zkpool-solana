# PROJECT_CONTEXT.md

## Setup

* Laptop Model: Dell Latitude 5330
* OS: Debian GNU/Linux 13 (trixie) x86_64
* Kernel: 6.12.107+deb13-amd64 (Preempt Dynamic)
* CPU: 12th Gen Intel Core i5-1245U (10 Cores / 12 Threads)
* RAM: 16 GiB (15 GiB usable) — 2.5 GiB used / 11 GiB free
* Swap: 12 GiB (0 B used)
* NVMe SSD: 256 GB (nvme0n1, 238.5 GiB) — Fully encrypted (LUKS + LVM)
* Text Editor: LazyVim (Neovim distribution)

---

## 0. Rules for the assistant (READ FIRST)
This is the **single normative section** of this file. Everything else describes the project. If in doubt — this section wins.

**If you are starting a new chat:**

Last completed stage: **Stage 15.8** (frontend: split deposit UI).
Next task: **Stage 15.9 — E2E: 1 SOL → 3 notes → 3 withdrawals**.

1. **Read section 0 completely**.
2. **Chat communication:** English, except if user ask Russian.
3. **Files on disk:**
  - **English** by default — code, configs, docs at root level.
  - **Russian** only for `docs/ru/*.md` and `docs/notes/*.md`.
4. **Do not narrate reasoning in chat:** in the chat, assistant only write tasks, request files, and confirm task completion. After `git commit && git push`, assistant describe the actions performed, their causes and explanations, as well as errors and solutions in notes. Don't write your thoughts in the chat.
5. The assistant gives **exactly one task** per message. Wait for the user to run it, paste the output, and only then give the next task.
  - **Do NOT** give multiple commands in one message.
  - **Do NOT** chain "then do X, then do Y".
  - **Exception — `git add -A` with `git status` are always grouped** as one task and `git commit` with `git push` are always grouped** as one task:
  ```bash
  git add -A && git status
  git commit -m "..." && git push
  ```
6. **After each `git commit && git push` of a sub-stage, not after documents:**
  - create the checkpoint, save artifacts to `.checkpoints/NN-name/` with SHA-256 in `manifest.txt` and the commit hash in `commit.txt`
  - Update `docs/PROJECT_CONTEXT.md` (this file).
  - Add or update `docs/notes/NN-name.md` (Russian).
  - Commit and push.
7. All commits go to the **feature branch first**, then the pull request.
  - Merge to main only if all workflows are successful.
  - Write text to populate the query pool template.
  - **Exception** — docs-only commits, no code, and no workflow trigger for a docs change goes via --ff-only without a PR.
8. **Never guess.** If ambiguous — use best practices. Any non-obvious action required to complete a stage must be:
  - Recorded in `PROJECT_CONTEXT.md`, section 8 (Known pitfalls).
  - Included in `docs/notes/NN-name.md` as a **lesson** with symptom, cause, fix.
  - Examples:
    - Manually reading crate sources to check exact API.
    - Adding packages to `workspace.exclude`.
    - Running `cargo fetch` in a sub-crate.
    - Any unusual CLI flag or env var.
9. **CRITICAL:** `docs/notes/*.md` is raw material for guides, tutorials, and articles on **Medium** and **Mirror.xyz**, and part of the GitHub portfolio.
10. **Section 8 (Known pitfalls)** — it's the fastest way to avoid re-learning our mistakes.
11. **Give files in full for new files - one file per message; insertion point + block for existing.**
12. **Test in small steps.** 20 lines, not 200. When working with a **new** library (LiteSVM, Anchor macros, sqlx, axum), **do not** write a large file in one shot. Write **20 lines**, compile, verify the API matches, then expand. **How to check exact API:** read crate sources at `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`. Look at `pub use` / `pub fn` / `pub struct` lines.

---

## 1. What this project is

**zkpool-solana** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (four secrets), and later withdraws SOL to any address without revealing the link between deposit and withdrawal.

**Inspired by:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous versions:**
- [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1.
- `kwebhub/solana-zk-pool` — v2 (abandoned; bug: `withdraw` fails with `InvalidInstructionData`, root cause not found).

**This repository:** https://github.com/kwebhub/zkpool-solana (public) — v3.
**Release:** [v0.1.0](https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0) (2026-09-29).
**License:** MIT
**Network:** Solana Devnet

**Demo notice:** this is a demo / educational project. See `docs/DEMO-NOTICE.md`.

---

## 2. Why v3 exists

In v2, `withdraw` failed with `InvalidInstructionData`. Root cause not found in three days. The class of bug: **mismatch between public inputs** across circuit, Anchor program, and frontend.

**v3 goal:** build the project such that this class of bug cannot exist by construction.

### Five principles

1. **Single source of truth** — `spec.json`; layers validated via `scripts/validate-spec`.
2. **Contract checks at every boundary** — byte-level.
3. **No magic numbers** — all constants in one place.
4. **LiteSVM E2E test before devnet.**
5. **Checkpoints with artifacts** — SHA-256.

---

## 3. Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) — port 5173                      │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — port 4001                                │
│   /api/health  /api/commitments  /api/root  /api/proof           │
│   /api/withdraw  /metrics                                        │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003  (Stage 6)    │
└──────────────┘   └──────────────┘   └──────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — port 4002                                 │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ MONITORING                                                       │
│ • Prometheus :9090  (scrapes backend /metrics every 15s)         │
│ • Grafana    :3000  (admin/admin) — dashboard "zkpool-backend"   │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA                                                           │
│ • Wallet:            5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc │
│ • Verifier program:  5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ │
│ • zk_pool program:   8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm │
│ • Pool PDA:          B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf │
│ • Vault PDA:         HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq │
│   (Both created 2026-09-27, tx 2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1...)│
└──────────────────────────────────────────────────────────────────┘
```

### Services and ports
| Service | Port | Where | Status |
|---------|------|-------|--------|
| frontend | 5173 | `solana` container → host | ✅ Stage 9 |
| backend | 4001 | `solana` container → host | ✅ Stage 5 |
| prover | 4002 | `solana` container → host | ✅ Stage 7 |
| merkle | 4003 | `solana` container → host | ✅ Stage 6 |
| postgres | 5432 | Docker `zkpool-postgres` | ✅ Stage 5.10 |
| redis | 6379 | Docker `zkpool-redis` | ✅ Stage 5.10 |
| prometheus | 9090 | Docker `zkpool-prometheus` | ✅ Stage 11.3a |
| grafana | 3000 | Docker `zkpool-grafana` | ✅ Stage 11.3b |

**Port mapping:** `solana` container exposes 4001 (backend), 4002 (prover), 4003 (merkle), 5173 (Vite). All reachable from host as `http://localhost:<port>`.

---

## 4. Technologies

| Layer | Technology | Version |
|-------|-----------|---------|
| Smart contract | Anchor | 1.1.2 (resolved to 1.2.0) |
| ZK circuit | Noir / nargo | 1.0.0-rc.2 |
| ZK backend | Sunspot | 1.0.0 |
| ZK verification | gnark-solana | v2.0.0 |
| Backend | Rust + axum | 1.98.1 + 0.7 |
| Merkle service | Node.js + Fastify | 24.21.0 + 5.12.5 |
| Noir JS | @noir-lang/noir_js | 1.0.0-rc.2 |
| Frontend | Vue 3 + Vite + Pinia + TypeScript + Pug + SCSS | 3.5+ / 5.4+ / 2.2+ / 5.6+ / 3.0+ / 1.79+ |
| Solana CLI | Agave | 3.1.10 |
| LiteSVM | litesvm | 0.16 |
| Tests Rust toolchain | Rust | 1.98.1 |
| On-chain Rust toolchain | Rust | 1.89.0 |
| Spec validator | Rust CLI | `scripts/validate-spec` |
| Circuit sync | Rust CLI | `scripts/sync-circuits` |
| DB | PostgreSQL | 16 |
| Cache | Redis | 7 |
| Node.js | Node.js | 24.21.0 |
| pnpm | pnpm | 12.5.1 |

---

## 5. Checkpoint methodology

See `docs/notes/00-checkpoints.md` for the full policy.

**Short version:** after each stage, copy generated artifacts to `.checkpoints/NN-name/` and record:
- `manifest.txt` — SHA-256 of each artifact.
- `commit.txt` — final commit hash.

On the next stage, compare hashes before proceeding.

---

## 6. What has been done

### ✅ Stage 0. Repository skeleton (2026-09-22)

Commit: `8a41984f046a7c1deca7ed75493903de97df659a`.

### ✅ Stage 1. Docker environment (2026-09-22)

- `infra/docker-compose.yml`, `infra/docker/Dockerfile.solana`.
- Sunspot cloned in Dockerfile (persistent).
- Commit: `94c18fe522e39822692fff9b9b22b2fe0f8e00d0`.

**Verified versions inside container:** rustc 1.98.1, cargo 1.98.1, solana-cli 3.1.10, anchor-cli 1.1.2, nargo 1.0.0-rc.2, sunspot 1.0.0, node v24.21.0, pnpm 12.5.1.

### ✅ Stage 2. Circuits on Noir (2026-09-22 — 2026-09-23)

- **2.0** — `spec.json`. Commit: `6df5fa5`.
- **2.1** — `validate-spec` (15 rules). Commits: `e479137`, `dbb4365`.
- **2.2.1** — `poseidon` library (11 tests). Commit: `baade07`.
- **2.2.2** — `hash2` (5 tests), `hashes` (9 tests). Commit: `8354921`.
- **2.2.3** — `withdrawal` + `merkle_tree` (16 tests). Commit: `90afe4c`.
- **2.3** — `sync-circuits` CLI. Commit: `aa5d8d9`.
- **2.4** — final checkpoint. Commit: `e0a3725`.

### ✅ Stage 3. Sunspot verifier (2026-09-23 — 2026-09-24)

- **3.0** — Wallet. Commit: `e9ddbe5`.
- **3.1** — `withdrawal.ccs` (642 177 B).
- **3.2** — `withdrawal.pk` (2 145 109 B) + `withdrawal.vk` (972 B).
- **3.3** — `withdrawal.so` (87 312 B). Verifier Program ID: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.
- **3.4** — deployed to devnet, balance 0.444 SOL.
- **3.5** — local verification: `✅ Verification successful!`.
- **3.6** — final checkpoint. Commit: `441b9d2`.

### ✅ Stage 3.7. Documentation enrichment (2026-09-24)

- `docs/notes/00-glossary.md` — 545 lines, 76 terms.
- `docs/notes/00-zk-primer.md` — 317 lines, 14 sections.
- `docs/notes/01-setup.md` — 762 lines.
- `docs/notes/02-circuits.md` — 957 lines.
- `docs/notes/03-sunspot.md` — 996 lines.

### ✅ Stage 4.1. Anchor program (2026-09-24)

| # | Sub-stage | Commit |
|---|---|---|
| 4.1.1 | Anchor workspace | `b8f6fdf` |
| 4.1.2 | `constants.rs` | `23c4dd9` |
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | `c45cb75` |
| 4.1.4 | `encoding.rs` | `dc0fb5e` |
| 4.1.5 | `pool` instruction | `553634e` |
| 4.1.6 | `deposit` instruction | `5455d04` |
| 4.1.7 | `withdraw` instruction | `ce71a47` |
| 4.1.8 | 37 unit tests | `8261530` |
| 4.1.9 | Deploy to devnet | `3f6dcd6` |

**Deployed program:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` (210 000 B, rent 1.068 SOL). ProgramData: `FaLqLdL1FVLcwZPpJTw2ugG67KKnEbZRuUJmqvyNtCeA`.

**Instruction discriminators:**
- `pool`: `[134, 215, 119, 168, 28, 199, 193, 127]`
- `deposit`: `[242, 35, 198, 137, 82, 225, 242, 182]`
- `withdraw`: `[183, 18, 70, 156, 148, 109, 161, 34]`

### ✅ Stage 4.5. LiteSVM E2E test (2026-09-24)

| # | Sub-stage | Commit |
|---|---|---|
| 4.5.1 | Tests reorg (moved out of `onchain/`) | `050e58f` |
| 4.5.2 | `helpers.rs` (loads both programs) | `cd01255` |
| 4.5.3 | `test_pool.rs` (airdrop + pool) | `59cf81e` |
| 4.5.4 | `test_deposit.rs` | `838cf85` |
| 4.5.5 | `test_withdraw.rs` (3 validation tests) | `c90442a` |
| 4.5.6 | `test_double_spend.rs` | `b7c4284` |
| 4.5.7 | Final checkpoint | `b7c4284` |

**8 tests passing (+ 7 adversarial added in Stage 13.2):**
- `helpers::tests::test_setup_svm_loads_both_programs`
- `test_pool::test_airdrop_works`
- `test_pool::test_pool_creates_state_and_vault`
- `test_deposit::test_deposit_updates_state_and_vault`
- `test_withdraw::test_withdraw_rejects_wrong_proof_length`
- `test_withdraw::test_withdraw_rejects_recipient_mismatch`
- `test_withdraw::test_withdraw_rejects_unknown_root`
- `test_double_spend::test_double_spend_rejected`

**Known limitation:** E2E withdraw with a real proof is not feasible in LiteSVM. Full E2E deferred to devnet + frontend (Stage 8).

### ✅ Stage 5. Backend (2026-09-24 — 2026-09-25)

| # | Sub-stage | Commit |
|---|---|---|
| 5.1 | Project skeleton | `7ed961a` |
| 5.2 | `config.rs` | `ee668ab` |
| 5.3 | `db.rs` + migration | `1e08827` |
| 5.4 | `cache.rs` (Redis) | `0f36fc2` |
| 5.5 | `tree.rs` (Merkle tree) | `372c056` |
| 5.6 | `logging.rs` + `metrics.rs` | `af219cd` |
| 5.7 | `rate_limit.rs` | `7d85c29` |
| 5.8 | `indexer.rs` | `1629bd1` |
| 5.9.1 | HTTP server + `/api/health` | `025fbd6` |
| 5.9.2 | Read endpoints + `/metrics` | `c0a9984` |
| 5.9.3 | `POST /api/withdraw` + rate limiting | `8085b24` |
| 5.10 | Docker compose: Postgres + Redis | `a260475` |
| 5.11 | Smoke test (+ `touch_startup_metrics` fix) | `1b815bc` |
| 5.12 | Final checkpoint | `78a7bfe` |

**Backend module layout:**
- `src/lib.rs` — module declarations.
- `src/main.rs` — axum server (6 routes) + `AppState`.
- `src/api_types.rs` — `WithdrawRequest` / `WithdrawResponse` (shared bin↔lib).
- `src/config.rs` — env → `Config`.
- `src/db.rs` — Postgres wrapper, 9 methods.
- `src/cache.rs` — Redis wrapper, 9 methods.
- `src/tree.rs` — incremental Merkle tree.
- `src/logging.rs` — tracing init.
- `src/metrics.rs` — Prometheus metrics (with `touch_startup_metrics`).
- `src/rate_limit.rs` — axum middleware.
- `src/indexer.rs` — RPC polling + event parsing.

**Endpoints:**
| Method | Path | Rate limit | Returns |
|---|---|---|---|
| GET | `/api/health` | — | `{status, db, version}` |
| GET | `/api/commitments` | 60/min | `{pool_address, count, commitments[]}` |
| GET | `/api/root` | 60/min | `{pool_address, root}` |
| GET | `/api/proof` | 60/min | `{pool_address, leaf_index, proof[], is_even[]}` |
| POST | `/api/root-preview` | 60/min | `{root}` — proxies Merkle `/root` |
| POST | `/api/withdraw` | 5/min | `{proof, public_witness}` (base64) |
| GET | `/metrics` | — | Prometheus text |

**Database schema:** 3 tables (`commitments`, `roots`, `nullifiers`) + 3 indexes.

**Backend tests:** 5 unit tests (4 in `indexer.rs`, 1 in `metrics.rs`) + 2 ignored integration tests (in `cache.rs`, need live Redis).

### ✅ Stage 6. Merkle service (2026-09-25 — 2026-09-26)

| # | Sub-stage | Commit |
|---|---|---|
| 6.1 | `package.json` + dependencies | `4142789` |
| 6.2 | `src/poseidon.js` | `d021210` |
| 6.3 | `src/merkle.js` | `0e9d766` |
| 6.4 | `src/server.js` (Fastify) | `fa67096` |
| 6.5 | Tests (`node --test`) | `cf34064` |
| 6.6 | Smoke test + backend integration | `d090a3b` |
| 6.7 | Final checkpoint | `99a228a` |


**Service:** `services/merkle/`, port 4003, Node.js 24.21.0 + Fastify 5.12.5.
**Convention:** bare hex (no `0x`) at HTTP boundary — matches `tree.rs`.
**Tests:** 23 (`node --test`) + 8 smoke checks.
**Integration proven:** `MerkleClient` (backend) → Merkle service, correct hash.

**Goal:** Node.js + Fastify HTTP service on port 4003 that exposes Poseidon2 hashing and Merkle tree operations to the backend and frontend.

**Why Node.js:** the backend and frontend both need Poseidon2 hashes that match the Noir circuit byte-for-byte. Rust and JS Poseidon2 implementations produce **different** hashes. The only way to guarantee identity is to use the same ACIR (`hash2.json`) via `@noir-lang/noir_js`, which runs **only in JavaScript**. So the Merkle service is a thin JS wrapper around `noir_js`.

**Endpoints:**
- `POST /hash` — body `{left: hex, right: hex}` → `{hash: hex}`. Used by `tree.rs` in the backend.
- `POST /root` — body `{commitments: [hex]}` → `{root: hex}`. Debug helper.
- `POST /proof` — body `{commitments: [hex], leaf_index: N}` → `{proof: [hex], is_even: [bool]}`.
- `GET /health` → `{status: "ok"}`.

**Dependencies:** `fastify`, `@fastify/cors`, `@noir-lang/noir_js@1.0.0-rc.2`.

**Circuits needed in `services/merkle/circuits/`:** `hash2.json`, `hashes.json`, `withdrawal.json` — already copied by `sync-circuits` at Stage 2.3, and the folder is gitignored.

**6.1 findings (2026-09-25):**
- `@noir-lang/noir_js@1.0.0-rc.2` exists on npm; pulls `acvm_js@1.0.0-rc.2`, `types@1.0.0-rc.2`, `noirc_abi@1.0.0-rc.2`, `pako@^3.0.1`.
- `fastify@5.12.5`, `@fastify/cors@11.3.0` — latest stable.
- `Noir` API: `new Noir(circuit)` → `execute(inputs) → { witness, returnValue }`.
- `returnValue` is a `0x`-prefixed hex string (single output) or an **array** of hex strings (tuple output).
- **Cross-check passed:** `noir_js` recomputed `nullifier_hash` and `root` from the Stage 3.5 `Prover.toml` witness and matched byte-for-byte. This validates the entire premise of using JS for Poseidon2.
- **Pitfall caught:** stale ACIR copies in `services/merkle/circuits/` (see section 8). Run `sync-circuits --check` at the start of every ACIR-consuming stage.

### ✅ Stage 7. Prover (2026-09-26)

| # | Sub-stage | Commit |
|---|---|---|
| 7.1 | Project skeleton | `ff31292` |
| 7.2 | `config.rs` | `4678050` |
| 7.3 | `witness.rs` | `95052dd` |
| 7.4 | `prover.rs` (mutex + subprocess) | `8103dd1` |
| 7.5 | `server.rs` (axum) | `6aa5fb4` |
| 7.6 | HTTP tests | `7603672` |
| 7.7 | Smoke test | `89ad83d` |
| 7.8 | Final checkpoint | `f5a48bf` |


**Service:** `services/prover/`, port 4002, Rust 1.89.0 + axum 0.7.
**Convention:** bare hex at HTTP boundary; `0x` only inside `Prover.toml`.
**Tests:** 8 unit + 3 integration + 1 ignored (real proof) + 6 smoke checks.
**Real proof:** 324 B proof + 172 B public witness, `sunspot verify` → valid.

**Goal:** Rust + axum service on port 4002 that accepts a withdrawal witness over HTTP and returns `{proof, public_witness}` as hex.

**Why shell out to sunspot:** `sunspot` is a Go CLI binary — no Rust library. The pipeline must invoke `nargo execute` (to produce the witness) and `sunspot prove` (to produce the proof).

**Critical constraint — `sunspot prove` output naming:**
`sunspot prove` always writes `withdrawal.proof` / `withdrawal.pw` — named after the **ACIR**, not the witness. Concurrent requests would clobber each other. Solution: **serialize with an async mutex.**

**Endpoint:**
```
POST /prove
  body: {
    root, nullifier_hash, recipient, recipient_binding, amount,   // public
    nullifier, secret, note_secret,                                // private
    merkle_proof: [20 hex], is_even: [20 bool]
  }
  response: { proof: hex, public_witness: hex }
```

**Pipeline per request (under mutex):**
1. Write `circuits/withdrawal/Prover-<uuid>.toml` with the witness values.
2. `nargo execute -p Prover-<uuid> w-<uuid>` → `target/w-<uuid>.gz`.
3. `sunspot prove target/withdrawal.json target/w-<uuid>.gz target/withdrawal.ccs target/withdrawal.pk` → `target/withdrawal.proof` + `target/withdrawal.pw` (fixed names).
4. Read `.proof` and `.pw`, hex-encode.
5. Clean up: `Prover-<uuid>.toml`, `target/w-<uuid>.gz`.
6. Return `{proof, public_witness}`.

**Design decisions:**
- **Sync, not async job queue.** v1 — HTTP request blocks until the proof is ready (~500 ms). Acceptable for a demo.
- **Mutex, not per-request temp dir.** Simpler, correct; throughput ~2–5 proofs/sec is enough.
- **No caching.** v1 — deterministic inputs → deterministic proof, but no cache layer yet.
- **Reference `circuits/withdrawal/` directly.** No copy into the service workspace; `nargo execute -p` allows unique input filenames.

**Integration:** the backend's `POST /api/proof` (currently `501 STUB`) will call `POST http://localhost:4002/prove` — planned for a later sub-stage of 7 or 8.

### ✅ Stage 8. Deploy pool on-chain (2026-09-27)

| # | Sub-stage | Commit |
|---|---|---|
| 8.1 | `scripts/pool-init/` skeleton | `b342417` |
| 8.2 | Pool initialized on devnet | `a4d274a` |
| 8.3 | Final checkpoint | `734664f` |

**Script:** `scripts/pool-init/`, Rust CLI, raw JSON-RPC (no `solana-client`).
**Pool PDA:** `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`
**Vault PDA:** `HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq`
**Init tx:** `2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1Cfh4pWrTFC2TcnEEiKfjq6e6AeBvNixxGQcBYnQZnB2Y3TPGFQ98zSeC`
**PoolState:** 384 bytes, owner = zk_pool program, authority = wallet, counters zero.

### ✅ Stage 9. Frontend (2026-09-27)

| # | Sub-stage | Commit |
|---|---|---|
| 9.1 | Vue 3 + TS + Pug + SCSS skeleton | `45b887e` |
| 9.2 | Minimal wallet connect | `d06a931` |
| 9.3 | Codama client for `zk_pool` | `70c1b6d` |
| 9.4 | `@noir-lang/noir_js` in browser | `c3537c4` |
| 9.5 | Typed API client | `d28f530` |
| 9.6a | Backend `POST /api/root-preview` | `dc0a59b` |
| 9.6b | Deposit logic (note, root preview, tx build) | `0a086a5` |
| 9.6c | Deposit UI screen | `fae22c4` |
| 9.7a | BN254 field reduction (recipient) + tests | `c8ef710` |
| 9.7b | Withdrawal witness assembly | `4238ed4` |
| 9.7c | `useWithdraw` composable | `28351ea` |
| 9.7d | Withdrawal UI + tabs | `867529e` |
| 9.7e | Refactor: composables to `src/composables/` | `9f1445e` |
| 9.8 | Final checkpoint | `cf65ec3` |
**Stack:** Vue 3.5 + Vite 5.4 + Pinia 2.2 + TypeScript 5.9 + Pug 3.0 + SCSS 1.105.
**No `@solana/wallet-adapter-vue`** — uses `window.phantom.solana` / `window.solflare` / `window.solana` directly. Avoids legacy `@solana/web3.js` dependency.
**Client SDK:** `@solana/kit@8.3.0`.

### ✅ Stage 10. Full E2E (2026-09-28)

| # | Sub-stage | Commit |
|---|---|---|
| 10.1a | Merkle `POST /hashes` | `e1a47a5` |
| 10.1b | `scripts/e2e-deposit/` | `2748d39` |
| 10.1c | First deposit + BN254 mask fix | `adcb441` |
| 10.2 | `scripts/e2e-withdraw/` + full E2E | `1f6da41` |
| 10.3 | Second E2E (arbitrary recipient) + double-spend | `3697d75` |
| 10.4 | Final checkpoint | `b750465` |

**Result:** full E2E (deposit → withdraw) verified twice on devnet. Groth16 proof verified on-chain. Double-spend protection works (Anchor `init` constraint).

**Deployed on devnet:**
- E2E #1: deposit `5LzXw…`, withdraw `5f3Lq…`, recipient = pool PDA
- E2E #2: deposit `36Z43…`, withdraw `5Tt2o…`, recipient = `3LChuQNFEYz8kTVrVPuAsbeyZxNpt8HKTsUGcRHnRgjP`

**DB state:** 2 commitments, 2 roots, 2 nullifiers.

**5 bugs found and fixed:**
1. BN254 mask `& 0x3f` → `& 0x1f`.
2. Borsh `Vec<u8>` missing 4-byte LE length prefix.
3. Seed `b"nullifier3"` → `b"nullifier_record"`.
4. Compute budget exceeded (182k CU needed, 200k default).
5. (Test observation, not a bug.)

### ✅ Stage 11. Infrastructure (2026-09-28)

| # | Sub-stage | Commit |
|---|---|---|
| 11.1 | Makefile | `1b1f80d` |
| 11.2 | Port mappings (4001–4003, 5173) | `6d2eb42` |
| 11.3a | Prometheus | `c90e09a` |
| 11.3b | Grafana provisioning | `a58731d` |
| 11.4 | Final checkpoint | `776ea2f` |

**Makefile targets:** `up`, `down`, `reset`, `status`, `logs`, `web`, `build`, `clean`, `help`.

**Stack:** 5 containers — `solana`, `postgres`, `redis`, `prometheus`, `grafana`.

**Monitoring:** Prometheus scrapes backend `/metrics` every 15s. Grafana dashboard `zkpool-backend` (uid stable).

**All ports from host:** 4001 (backend), 4002 (prover), 4003 (merkle), 5173 (web), 9090 (Prometheus), 3000 (Grafana, admin/admin).

### ✅ Stage 12. Engineering processes (2026-09-29)

| # | Sub-stage | Commit |
|---|---|---|
| 12.1 | CI workflow (7 jobs) | `f61b865` |
| 12.2 | Docker build workflow (GHCR) | `bb5830b` |
| 12.3 | Release workflow (tagged artifacts) | `b7b5903` |
| 12.4 | Security workflow (audit) | `6533d07` |
| 12.5 | Templates (PR, issues, dependabot) | `0ffc08d` |
| 12.6 | Final checkpoint | `d4a2c36` |

**4 workflows:** `ci.yml`, `docker.yml`, `release.yml`, `security.yml`.
**7 CI jobs:** validate-spec, onchain, litesvm, backend, prover, merkle, web.
**Templates:** PR, issue (bug/feature), dependabot (7 cargo + 2 npm + 1 gh-actions).

**Known gaps:** `security` audit jobs are report-only (`|| true`); `security.yml` `cargo-deny` uses root `deny.toml` run from `services/backend`.

**Stage 12 follow-up (2026-10-01):** the `merkle` and `litesvm` CI jobs were fixed:
- `merkle` — now installs `nargo` via `noirup`, compiles `poseidon`/`hash2`/`hashes`/`withdrawal`, and copies the ACIRs into `services/merkle/circuits/` before running tests.
- `litesvm` — now builds `zk_pool.so` via `anchor build` (Anchor CLI from the release binary, not `avm`), builds a **mock verifier** `.so` under `onchain/tests/mock-verifier/` (a 20-line program returning `Ok(())`), and renames it to `withdrawal.so` so the tests find it at the expected path.
- `tests/src/helpers.rs` — `.so` paths are now resolved relative to `CARGO_MANIFEST_DIR` with `ZK_POOL_SO` / `VERIFIER_SO` env overrides, instead of hardcoded `/home/ubuntu/...` paths.
- `security.yml` `cargo-deny` — now runs from `services/backend` with `--config ../../deny.toml` (cargo-deny requires a `Cargo.toml` in the working directory).

### ✅ Stage 13. Security (2026-09-29)

| # | Sub-stage | Commit |
|---|---|---|
| 13.1 | Threat model (12 attacks, 7 invariants) | `96303b1` |
| 13.2 | Adversarial tests (7, total 15) | `69bf648` |
| 13.3 | Backend input validation hardening | `d0f3f85` |
| 13.4 | Frontend security review | `60038a7` |
| 13.5 | sqlx 0.7→0.8, deny.toml, make exec-c | `1f35743`, `f31480f` |
| 13.6 | Final checkpoint | `a241083` |

**Threat model:** 12 attacks (5 protected, 3 partial, 4 not prevented). All documented in `docs/threat-model.md`.

**Fixes:**
- `RUSTSEC-2024-0363` — sqlx 0.7.4 → 0.8.6.
- Backend validation: hex64 checks, leaf_index < 2^20, commitments < 2^20.

**Open items:** root `deny.toml` allow list; `security.yml` `|| true`; CSP for production.

### ✅ Stage 14. Finalization (2026-09-29)

| # | Sub-stage | Commit |
|---|---|---|
| 14.1 | `.editorconfig` + LICENSE (MIT) | `843ac88` |
| 14.2 | `README.md` | `274bc4d` |
| 14.3 | `docs/ru/README.md` | `94d17b8` |
| 14.4 | `CONTRIBUTING.md` | `747a711` |
| 14.5 | `SECURITY.md` | `28553d4` |
| 14.6 | `CHANGELOG.md` (v0.1.0) | `537ea0d` |
| 14.7 | `docs/DEMO-NOTICE.md` update | `f8b3adc` |
| 14.8 | `cargo-deny` passes | `e09e8fd` |
| 14.9 | `cargo-deny` + `cargo-audit` in Dockerfile | `226401c` |
| 14.10 | Release `v0.1.0` (tag on `48302ed`) | `48302ed` (tag) |
| 14.11 | Final checkpoint | `56eeaf1` |

**Release:** `v0.1.0` — https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0
**Assets:** `zk_pool.so` (205 KB), `zk_pool.json` (16.8 KB), `zkpool-backend` (9.53 MB), `zkpool-prover` (3.46 MB), `zkpool-web.tar.gz` (1.16 MB), `SHA256SUMS` (401 B).

**Repo visibility:** public (made public 2026-09-29 to unblock GitHub Actions billing).

**Dependabot triage:** 29 PRs closed (2026-09-29) — all version bumps. Project is on pinned-version release; bumps deferred to a future upgrade cycle.

### 🚧 Stage 15. Split deposit (2026-09-30 — in progress)

| # | Sub-stage | Commit |
|---|---|---|
| 15.1 | Design doc | `2285ed4` |
| 15.2 | `spec.json` + `rules.rs` + `spec.rs` | `cec750e` |
| 15.3 | Circuit change (C4, C5, new ACIR) | `1fc9a5d` |
| 15.4 | Sunspot re-run + verifier upgrade | `0f02d34` |
| 15.5 | Anchor — `deposit_split`, `encode_public_inputs` +32 | `a66bee2` |
| 15.6 | LiteSVM — adversarial for split | `131b2ac` |
| 15.7 | Backend + prover — split-deposit witness fields | `7936d3a` |
| 15.8 | Frontend — split UI | `b7a8561` |
| 15.9 | E2E — 1 SOL → 3 notes → 3 withdrawals | ← next |
| 15.10 | Final checkpoint + CHANGELOG → v0.2.0 | |

### ✅ Docs (2026-09-22 — 2026-09-25)

- `docs/notes/00-checkpoints.md`, `00-glossary.md`, `00-zk-primer.md`.
- `docs/notes/01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md` (~1014 lines), `05-backend.md` (~1200 lines).
- `docs/DEMO-NOTICE.md`.
- `docs/PROJECT_CONTEXT.md` — this file.
- `docs/notes/assets/{01-setup,02-circuits,03-sunspot}/` — empty folders for screenshots.

---

## 7. Architecture decisions

- **Spec validation:** `spec.json` is the source of truth; layers validated against it via `scripts/validate-spec`. Generation of `.nr` and Rust is fragile; validation is simpler and catches the same class of bugs.
- **LiteSVM E2E test:** `tests/src/` contains LiteSVM integration tests. Full E2E with real proof deferred to devnet.
- **One circuit with `recipient_binding`** public inputs (5): `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`. No mid-project format change.
- **Zero-padding (no domain separation):** `hash_1(x) == hash_2(x, 0)` and `hash_2(x, y) == hash_3(x, y, 0)`. Documented. Safe for zkpool-solana because we never hash the same value with two arities in one context.
- **`sync-circuits` — check/apply modes:** `--check`: mismatch → error, exit 1, no copy. `--apply`: mismatch → copy + warning, exit 0.
- **Split deposit - feature:** split a single deposit into multiple unequal commitments (improves privacy by breaking the amount link) — IN PROGRESS (Stage 15).
- **`tests/` — separate crate at root:**
  - **Decision (2026-09-24):** `tests/` is **outside** `onchain/`, at the project root.
  - **Reason:** `litesvm 0.16` requires Agave 4.2, which requires **Rust ≥ 1.90**. `onchain/rust-toolchain.toml` pins **1.89.0** (needed for SBF). Keeping tests inside `onchain/` means they inherit the 1.89 toolchain and fail with `E0658`.
  - **Solution:** separate crate at root with its own `rust-toolchain.toml` (`channel = "1.98.1"`).
  - **Connection:** `zk_pool = { path = "../onchain/programs/zk_pool" }`.
- **Merkle service ordering dependency:** `tree.rs` (backend) uses HTTP to the Merkle service for Poseidon2 hashing. The backend **starts fine** without the Merkle service, but any `add_leaf` call (triggered by the indexer when it sees a deposit) will fail with an HTTP error until Merkle service (Node.js) is running.

---

## 8. Known pitfalls

> Organized by topic, not chronologically. Each bullet: symptom → cause → fix.
> This is the fastest way to avoid re-learning our mistakes.

### 8.1. Docker & container

- **`bash -c` does not read `.bashrc`.**
  - Symptom: `node: command not found`, `cargo: command not found`, `pnpm: command not found` inside the container.
  - Cause: `nvm`, `cargo`, `pnpm` add themselves to `PATH` via `.bashrc`, which non-interactive shells skip.
  - Fix: always use `bash -ic` (interactive) inside the container.

- **Volume created from root becomes root-owned on host.**
  - Symptom: `Error: Unable to write /home/ubuntu/.config/solana/id.json: Permission denied (os error 13)`.
  - Cause: container (before `user: "1000:1000"` was set) created the folder as UID 0.
  - Fix: `sudo chown -R 1000:1000 solana/`.

- **Port mappings require `--force-recreate`.**
  - Symptom: added `ports:` to `docker-compose.yml`, but the container still has no host port.
  - Cause: `docker compose up -d` sees "container already running" and skips recreation.
  - Fix: `docker compose up -d --force-recreate solana`.

- **All three services die on container restart.**
  - Symptom: after `docker compose up -d --force-recreate solana`, `curl localhost:4001/api/health` → connection refused.
  - Cause: `merkle`, `prover`, `backend` run as background processes inside the container — not as services.
  - Fix: `make up` restarts all three.

- **`cargo install` inside the container is not persistent.**
  - Symptom: after `docker compose up -d --force-recreate solana`, `cargo-deny: command not found`.
  - Cause: `cargo install` writes to `/home/ubuntu/.cargo/bin/` **inside** the container filesystem, not to a volume.
  - Fix: bake into `infra/docker/Dockerfile.solana` (done in Stage 14.9).

### 8.2. Noir / nargo

- **`poseidon2_permutation` takes one argument, not two.**
  - Symptom: `Function expects 1 parameter but 2 were given`.
  - Cause: nargo 1.0.0-rc.2 signature is `poseidon2_permutation(state: [Field; 4]) -> [Field; 4]`.
  - Fix: `poseidon2_permutation(state)` — size is inferred from the array.

- **Zero-padding causes arity collisions.**
  - Fact: `hash_1(x) == hash_2(x, 0)` and `hash_2(x, y) == hash_3(x, y, 0)`.
  - Cause: `poseidon2` with `t=4`, `rate=3`, `capacity=1` — zero-padding to fill the rate.
  - Safe here because we never hash the same value with two arities in one context.

- **`main()` in `type = "bin"` circuits must have `pub` on return type.**
  - Symptom: `missing pub keyword on return type of function main`.
  - Cause: verifier cannot get private outputs without `pub`.
  - Fix: `fn main(...) -> pub (Field, Field)`.

- **Field values must be < 2^254.**
  - Symptom: `Integer literal is too large`.
  - Cause: BN254 prime ≈ 2^254.
  - Fix: keep literals < 2^254. Solana Pubkey (32 bytes) must be reduced modulo BN254.

- **Noir does not support float literals.**
  - Symptom: `Object type is unknown in field access` on `0.001`.
  - Fix: use integer lamports (`1_000_000`, not `0.001`).

- **`&str` parameters don't work in Noir.**
  - Symptom: `error: str expects 1 generic`.
  - Fix: use `str<N>` with explicit size, or avoid strings entirely.

- **Format strings with arrays need explicit type annotation.**
  - Symptom: `error: Type annotation needed` on `f"...{arr[i]}"`.
  - Fix: print values directly with `println(value)`, no format string.

- **`Field` values cannot be compared with `<`, `<=`, `>`, `>=`.**
  - Symptom: `error: Fields cannot be compared, try casting to an integer first`.
  - Cause: Noir only allows ordering comparisons on integer types.
  - Fix: compare as `u32` / `u64` directly. Do not cast to `Field`.

### 8.3. Sunspot / Groth16

- **`sunspot deploy` requires `GNARK_VERIFIER_BIN`.**
  - Symptom: `directory does not exist`.
  - Cause: `.deb` package contains only the CLI binary — not the Rust crate that compiles the verifier.
  - Fix: `git clone https://github.com/reilabs/sunspot.git ~/sunspot` (baked into Dockerfile).
  - Env: `export GNARK_VERIFIER_BIN="$HOME/sunspot/gnark-solana/crates/verifier-bin"`.

- **`sunspot verify` argument order matters.**
  - Symptom: `Error: invalid verification key file: target/withdrawal.proof (must end with .vk)`.
  - Fix: order is `.vk`, `.proof`, `.pw`.

- **Sunspot emits 6 `deprecated` warnings during deploy.**
  - Cause: uses old `solana-bn254` constants (`ALT_BN128_ADD`, not `*_BE`).
  - Fix: none needed — it's an upstream issue, compilation succeeds.

- **TOML parser rejects hex literals > 2^63.**
  - Symptom: `number too large to fit in target type` when parsing `Prover.toml`.
  - Cause: nargo's TOML parser reads bare hex as `i64`.
  - Fix: wrap values in double quotes: `root = "0x1e85..."`.

### 8.4. Anchor / on-chain program

- **`anchor init .` fails.**
  - Symptom: `Anchor workspace name must be a valid Rust identifier`.
  - Cause: directory name (`onchain`) is not a valid Rust identifier.
  - Fix: init in a temp dir with the right name, then copy the needed files.

- **`invalid --check-cfg argument`.**
  - Cause: `[lints.rust]` cannot declare `cfg(anchor-debug)`.
  - Fix: declare `anchor-debug`, `custom-heap`, `custom-panic` as **features** in `Cargo.toml`.

- **`E0107: struct takes 0 lifetime arguments`.**
  - Cause: empty `#[derive(Accounts)]` struct doesn't work.
  - Fix: don't declare it; Anchor allows a program without instructions during scaffolding.

- **`E0308` in `CpiContext::new`: expected `Pubkey`, found `AccountInfo`.**
  - Cause: Anchor 1.2.0 changed the signature.
  - Fix: `CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts)`.

- **`E0432: unresolved import crate` after removing glob re-exports.**
  - Cause: Anchor macro needs the glob re-exports to access `__client_accounts_*`.
  - Fix: keep `pub use instructions::*;`.

- **`E0277: #[instruction] type mismatch`.**
  - Cause: `#[instruction(...)]` attributes list all handler args in order.
  - Fix: list every arg in the same order as the handler.

- **CPI to Sunspot verifier: data layout is `[proof || public_witness]` — proof FIRST.**
  - Cause: verifier program reads `proof_len = len - (12 + NR_INPUTS * 32)`.
  - Fix: `data = proof_bytes ++ public_inputs_bytes`.

- **`ambiguous glob re-exports`.**
  - Cause: multiple `handler` functions with the same name.
  - Fix: rename to `handler_pool`, `handler_deposit`, `handler_withdraw`.

### 8.5. Solana program semantics

- **Vault PDA has no private key.**
  - Fix: manipulate lamports directly with `try_borrow_mut_lamports()`.

- **PDA is not zero-balance after `init`.**
  - Symptom: `assert_eq!(vault_balance, 0)` fails with `Some(890880)`.
  - Cause: 0-byte account still requires rent-exempt minimum.
  - Fix: read `vault_initial` after init, then `assert_eq!(vault_after, vault_initial + amount)`.

- **Anchor Borsh: `Vec<u8>` needs a `u32 LE` length prefix.**
  - Symptom: `memory allocation failed, out of memory` on instruction entry.
  - Cause: wrote bytes without the length prefix; Anchor read a bogus large length.
  - Fix: `data.extend_from_slice(&(vec.len() as u32).to_le_bytes()); data.extend_from_slice(&vec);`.
  - Fixed-size arrays (`[u8; 32]`) are serialized raw — no length prefix.

- **Groth16 verification consumes ~182k CU.**
  - Symptom: `ComputationalBudgetExceeded` with `Proof verified successfully!` in logs.
  - Cause: default 200k is not enough for CPI to verifier + PDA creation + lamport transfer.
  - Fix: prepend `ComputeBudgetInstruction::SetComputeUnitLimit(400_000)`.
  - Instruction data: `[2, u32 LE limit]`. Program ID: `ComputeBudget111111111111111111111111111111`.

- **`anchor deploy` is deprecated.**
  - Fix: use `anchor program deploy`.

- **`anchor build` uses existing keypair only if present in `target/deploy/`.**

### 8.6. Seeds and addresses

- **Seeds may be versioned — do not guess.**
  - `POOL_SEED = b"pool3"`, `VAULT_SEED = b"vault3"` (versioned).
  - `NULLIFIER_RECORD_SEED = b"nullifier_record"` (**not** `b"nullifier3"`).
  - Symptom when guessed wrong: `ConstraintSeeds. Left: X Right: Y`.
  - Fix: always read `onchain/programs/zk_pool/src/constants.rs`.

- **Verifier Program ID is hard-coded in four places.**
  - `onchain/programs/zk_pool/src/constants.rs` (`VERIFIER_PROGRAM_ID`).
  - `web/src/constants.ts`.
  - `tests/src/helpers.rs` (`VERIFIER_ID`).
  - `docs/PROJECT_CONTEXT.md` §3.
  - If the verifier is redeployed with a new Program ID, all four must be updated simultaneously.

### 8.7. BN254 field arithmetic

- **Random field element mask must account for the modulus's top byte.**
  - BN254 prime = `0x30644e72…` — top byte is `0x30`.
  - Mask `& 0x3f` allows values `0x31–0x3f`, which **exceed the modulus**.
  - Symptom: `POST /hashes` → 500, `Value 0x31020e14… exceeds field modulus`.
  - Fix: mask `& 0x1f` (top byte ≤ `0x1f < 0x30`).
  - Applied in `scripts/e2e-deposit/src/main.rs` and `web/src/deposit/generateNote.ts`.

- **Solana Pubkey does not fit in a BN254 `Field`.**
  - Cause: pubkey is 32 bytes (256 bits); field is ~254 bits.
  - Fix: `reduce_to_field(pubkey) = pubkey mod BN254_prime`. Implemented in `onchain/programs/zk_pool/src/encoding.rs` and ported to TS in `web/src/withdraw/reduce.ts`.

- **Reduction loop needs 5 iterations, not 4.**
  - Symptom: `test_reduce_to_field_max` fails with `!is_ge(&reduced, &BN254_PRIME_BE)`.
  - Cause: max input `2^256 - 1` ≈ `4.006 × p`. 4 subtractions may leave the result ≥ p.
  - Fix: loop `0..5`.

### 8.8. Rust toolchain / workspace

- **`cargo fetch` inside `onchain/tests/` fails with "current package believes it's in a workspace when it's not".**
  - Cause: `onchain/Cargo.toml` is a workspace root, `members = ["programs/*"]` doesn't cover `tests/`.
  - Fix (final): move `tests/` to project root with its own `rust-toolchain.toml`.

- **`E0658: use of unstable library feature maybe_uninit_write_slice` in `solana-syscalls 4.2.2`.**
  - Cause: `litesvm 0.16` pulls Agave 4.2, which needs Rust ≥ 1.90.
  - `onchain/rust-toolchain.toml` pins 1.89.0 — `tests/` inherited it.
  - Fix: separate `tests/rust-toolchain.toml` with `channel = "1.98.1"`.

- **`failed to select a version for solana-hash`.**
  - Cause: `litesvm 0.16` requires `solana-hash ~4.5.0`; `solana-message 5` requires `>= 4.6.0`. No compatible versions.
  - Fix: use **exact** versions from litesvm's `Cargo.toml`:
    `solana-account = "4.3.0"`, `solana-address = "~2.6.1"`, `solana-hash = "4.5.0"`, `solana-instruction = "3.4.0"`, `solana-keypair = "3.1.2"`, `solana-message = "4.2.4"`, `solana-sdk-ids = "3.1.0"`, `solana-signer = "3.0.1"`, `solana-transaction = "4.1.5"`, `solana-transaction-error = "3.3.1"`.
  - Process: find the main crate, read its `Cargo.toml`, use exactly its `solana-*` versions. Do not use semver ranges.

- **`E0308: Transaction::new_signed_with_payer expects &[Instruction], found &Vec<CompiledInstruction>`.**
  - Cause: `Message::new_with_blockhash` compiles instructions.
  - Fix: use `Transaction::new_signed_with_payer(&[ix], ...)` with raw `Instruction`.
  - Lesson: two paths — via `Message` (compiled) and via raw `Instruction`. Don't mix them.

- **`solana-message::Message::serialize` is gated behind the `wincode` feature.**
  - Symptom: `E0599: no method named serialize`.
  - Cause: feature-gated, and enabling `wincode` pulls two incompatible `wincode` versions.
  - Fix: serialize the legacy message format by hand (~60 lines: header, compact-u16, account keys, blockhash, instructions).

- **`solana-transaction` methods `sign`, `partial_sign`, `new_signed_with_payer` are gated behind `wincode`.**
  - Fix: build wire format manually: `compact_u16(1) || sig(64) || message_bytes`.

### 8.9. Backend (Rust + axum)

- **`chrono` used transitively but not declared.**
  - Symptom: `E0433: cannot find module or crate chrono`.
  - Cause: `sqlx`'s `chrono` feature integrates but doesn't re-export types.
  - Fix: `chrono = { version = "0.4", features = ["serde"] }` in `[dependencies]`.
  - Lesson: if you use a type from a transitive dep, declare it explicitly.

- **`E0277: WithdrawResponse: DeserializeOwned not satisfied` with note "multiple different versions of crate `zkpool_backend`".**
  - Cause: `WithdrawResponse` was defined in `main.rs` (bin), while `reqwest::json::<T>()` needs `T` from the same crate instance as `serde`.
  - Fix: move shared types to lib — created `src/api_types.rs`.
  - Lesson: in crates with both `[[bin]]` and `[lib]`, all shared types must live in the lib.

- **`/metrics` returns 200 with empty body.**
  - Cause: `metrics-exporter-prometheus` doesn't render metrics that were never recorded. Plus default idle-timeout drops old metrics.
  - Fix: `touch_startup_metrics()` records each metric once with a zero value.
  - Note: `MetricKindMask` is in `metrics-util`, not re-exported by `metrics-exporter-prometheus`.

- **`sqlx 0.7.4` has RUSTSEC-2024-0363** (SQL injection via protocol smuggling).
  - Fix: upgrade to `sqlx 0.8`. No code changes required.

- **Commitments schema has a global `UNIQUE (leaf_index)`, not per-pool.**
  - Symptom: inserting `leaf_index = 0` for a second pool → `INSERT 0 0` (silently swallowed by `ON CONFLICT DO NOTHING`).
  - Impact: multi-pool support broken. Single-pool unaffected.
  - Fix (deferred): drop `commitments_leaf_index_key`, keep `commitments_pool_leaf_idx` but make it UNIQUE.

- **Backend ↔ prover wire format was mismatched at first.**
  - Cause: `WithdrawRequest` (Stage 5) used `{witness: base64}`; prover expected 10 hex fields (Stage 7).
  - Fix: rewrote `WithdrawRequest` to match `WitnessInputs`; backend converts prover's hex output → base64 for `WithdrawResponse`.
  - Lesson: when a second service appears, verify wire formats immediately.

### 8.10. Merkle service (Node.js)

- **`node --test <dir>` treats `<dir>` as a module.**
  - Symptom: `Error: Cannot find module '/path/test'`.
  - Cause: Node 24 changed the behavior — needs a glob or explicit file list.
  - Fix: `node --test test/*.test.js` (unquoted, shell expands).

- **Fastify app must be split for testability.**
  - Reason: `app.inject()` requires an app instance without a bound port.
  - Fix: `buildApp()` in `app.js`, `listen()` in `server.js`.

- **Stale ACIR copies in `services/merkle/circuits/` and `web/public/circuits/`.**
  - Symptom: hash mismatch between the backend and the circuit.
  - Cause: `sync-circuits` not run after a circuit change.
  - Fix: `cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- apply`.
  - Lesson: run `sync-circuits --check` at the start of any ACIR-consuming stage.

### 8.11. Prover (Rust + Sunspot)

- **`nargo execute -p <name>` requires `<name>` without dots.**
  - Symptom: `Cannot find input file '…/Prover-7.toml'` when passing `Prover-7.3-test`.
  - Cause: nargo truncates at the first dot.
  - Fix: use names without dots — `Prover-<uuid-simple>`.

- **`Prover.toml` hex values must be `0x`-prefixed.**
  - Symptom: `Expected witness values to be integers, but '00f4240' failed with 'invalid digit found in string'`.
  - Cause: bare hex fails for `amount` and `merkle_proof[i]`.
  - Fix: `witness.rs::with_0x()` adds the prefix idempotently.

- **`sunspot prove` always writes `withdrawal.proof` / `withdrawal.pw` — named after the ACIR, not the witness.**
  - Cause: fixed output filenames.
  - Impact: concurrent requests clobber each other.
  - Fix: serialize with an async mutex in `prover.rs`.

### 8.12. Frontend (Vue 3 + TS)

- **`@noir-lang/noir_js` does not implicitly reduce values modulo BN254.**
  - Symptom: `The value passed for parameter left is invalid: Expected witness values to be integers, but '31020e...' failed with 'invalid digit found in string'`.
  - Fix: reduce explicitly in TS before passing to `noir.execute`.

- **TypeScript 5.9 strict `Uint8Array` generic.**
  - Symptom: `Uint8Array<ArrayBufferLike>` is not assignable to `Uint8Array<ArrayBuffer>`.
  - Fix: annotate explicitly as `Uint8Array<ArrayBuffer>`.

- **`node --experimental-strip-types` requires `.ts` in import specifiers.**
  - Symptom: `Cannot find module`.
  - Cause: Node doesn't do auto-extension resolution like a bundler.
  - Fix: import `./reduce.ts` (with extension) and set `allowImportingTsExtensions: true` in `tsconfig.json`.

- **`resp.json()` throws on empty body.**
  - Cause: `Content-Length: 0` → `Unexpected end of JSON input`.
  - Fix: read `resp.text()` first, then `JSON.parse` only if non-empty.

- **`@solana/wallet-adapter-vue` pulls legacy `@solana/web3.js`.**
  - Cause: peer dependency on `@solana/web3.js@^1.99`.
  - Impact: bundle bloat, `PublicKey ↔ Address` conversions at every boundary.
  - Fix: use `window.phantom.solana` / `window.solflare` / `window.solana` directly — no wallet adapter.

- **Codama requires a `TransactionSigner` for signer accounts.**
  - Cause: `AccountSignerMeta` needs `signer` field, not just `{address, role}`.
  - Fix: `makeNoopSigner(address)` — a type-safe stub whose `signTransactions` throws. Actual signing goes through `provider.signAndSendTransaction(base64)`.

- **Codama-generated `*InstructionDataArgs` are flattened into the async input, not wrapped in `args`.**
  - Symptom: `error TS2353: Object literal may only specify known properties, and 'args' does not exist in type 'DepositSplitAsyncInput<...>'`.
  - Cause: Codama 1.11's `renderers-js` spreads the instruction-data fields directly onto the async input type; there is no `args` wrapper.
  - Fix: pass `commitments`, `newRoots`, `amounts`, `totalAmount` as top-level fields to `getDepositSplitInstructionAsync({ ... })`.
  - To confirm the exact shape, `grep -n "DepositSplitAsyncInput\|commitments\|totalAmount" web/src/generated/zk_pool/src/generated/instructions/depositSplit.ts`.

- **`@solana/kit` does not export `setTransactionMessageFeeLifetimeUsingBlockhash`.**
  - Symptom: `error TS2724: '"@solana/kit"' has no exported member named 'setTransactionMessageFeeLifetimeUsingBlockhash'. Did you mean 'setTransactionMessageLifetimeUsingBlockhash'?`
  - Cause: autocomplete typo — there is no fee-lifetime variant.
  - Fix: use `setTransactionMessageLifetimeUsingBlockhash` and `setTransactionMessageFeePayer` separately.

- **`pnpm build` bundles the two WASM binaries (acvm_js, noirc_abi_wasm) unconditionally.**
  - Fact: 3.84 MB of WASM in `dist/assets/`, plus ~184 KB of JS.
  - Cause: `@noir-lang/noir_js` is statically imported by `noir/poseidon.ts` and `noir/hashes.ts`, which are imported by `deposit/generateNote.ts` and `withdraw/buildWitness.ts`, which are imported by the composables, which are imported by `App.vue`.
  - Impact: initial page load carries ~3.84 MB of WASM even before the user opens any form.
  - Deferred fix: dynamic `import()` inside `poseidon2Hash`/`computeHashes`, so WASM loads only when a proof-relevant action starts. Not in scope for Stage 15.8.

- **`git push` after `git merge --ff-only` may fail with `remote: fatal error in commit_refs`.**
  - Symptom: `! [remote rejected] main -> main (failure)` — remote refused the push, local merge succeeded.
  - Cause: transient server-side error on GitHub's ref update. No commit is lost locally; the feature branch is already pushed.
  - Fix: run `git push` again immediately. Second attempt succeeded on Stage 15.8.

### 8.13. Git / tooling

- **Editor "replace fully" applied to the wrong file.**
  - Symptom: test appears under wrong module in `cargo test` output.
  - Cause: both "replace fully" instructions applied to the same file.
  - Fix: rewrote both files.
  - Lesson: after "replace fully", check `wc -l` and `grep` for characteristic function names.

- **`cargo-deny 0.20.2` schema is incompatible with `version = 2` in `deny.toml`.**
  - Symptom: `licenses FAILED` on own crates with `license = "MIT"` even when MIT is in the allow list.
  - Cause: schema changed.
  - Fix: regenerate `deny.toml` via `cargo deny init` inside a crate directory (requires `Cargo.toml`).

- **`cargo deny --config <path>` must precede the subcommand.**
  - Correct: `cargo deny --config /path/deny.toml check licenses`.
  - Wrong: `cargo deny check licenses --config /path/deny.toml`.

- **`deny.toml` must be mounted into the container.**
  - Symptom: `[WARN] config path doesn't exist, falling back to default config` — then all licenses fail.
  - Cause: only subdirectories are mounted; the repo root is not.
  - Fix: add `../deny.toml:/home/ubuntu/deny.toml:ro` to `docker-compose.yml` volumes.
  - Lesson: always `head -3` the output, not only `tail -5` — the warning is at the top.

- **Shell heredocs occasionally hang in the interactive terminal.**
  - Symptom: `<<'EOF'` shown as literal text, prompt doesn't return.
  - Fix: `Ctrl+C`, `rm -f` the partial file, retry. If it hangs again, write the file in the editor.
  - Lesson: for files with many special characters, prefer the editor over heredoc.

- **GitHub Actions fails in 4 seconds without starting the job.**
  - Symptom: workflow failed without logs; annotation says "recent account payments have failed or spending limit needs to be increased".
  - Cause: private repo, exhausted Actions minutes.
  - Fix: make the repository public — public repos get unlimited Actions minutes.

### 8.14. How to find exact signatures for the installed crate version

- Rust crate sources: `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`.
- Anchor: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anchor-lang-1.2.0/src/`.
- LiteSVM: `~/.cargo/registry/src/index.crates.io-*/litesvm-0.16.0/src/`.
- Always check **your own** version, not v2 examples.

### 8.15. How to resolve `solana-*` version conflicts

1. Find the "main" crate (e.g. `litesvm`) and its exact `solana-*` versions in its `Cargo.toml`.
2. Use **exactly** those versions.
3. Do not guess or use semver ranges (`"4"`, `"5"`) — they pull incompatible versions.
4. Run `cargo tree -p <main-crate>` to verify.
5. If a newer toolchain is required, isolate the crate (see §7.8).

## 9. Repository structure

```
zkpool-solana/
├── .github/                 ← CI/CD, templates
│   ├── workflows/{ci,docker,release,security}.yml
│   ├── ISSUE_TEMPLATE/{bug_report,feature_request,config}
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── dependabot.yml
├── .checkpoints/            ← gitignored
├── .secrets/                ← gitignored (wallet + program keypair)
├── Makefile                 ← `make up` / `down` / `reset` / ...
├── README.md                ← main (English)
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
├── LICENSE                  ← MIT
├── .editorconfig
├── docs/
│   ├── ru/README.md         ← Russian version
│   ├── notes/               ← Russian, committed, PORTFOLIO MATERIAL
│   │   ├── 00-checkpoints.md
│   │   ├── 00-glossary.md
│   │   ├── 00-zk-primer.md
│   │   ├── 01-setup.md
│   │   ├── 02-circuits.md
│   │   ├── 03-sunspot.md
│   │   ├── 04-anchor.md
│   │   ├── 05-backend.md
│   │   └── assets/{01-setup,02-circuits,03-sunspot}/
│   ├── ru/README.md
│   ├── DEMO-NOTICE.md
│   ├── PROJECT_CONTEXT.md   ← this file
│   └── threat-model.md      ← Stage 13.1
├── infra/
│   ├── docker-compose.yml   ← solana + postgres + redis
│   └── docker/Dockerfile.solana
├── circuits/
│   ├── poseidon/            ← library
│   ├── hash2/               ← circuit (ACIR)
│   ├── hashes/              ← circuit (ACIR)
│   └── withdrawal/
│       ├── spec.json
│       ├── Prover.toml      ← gitignored
│       └── src/{main,merkle_tree,test_witness}.nr
├── onchain/                 ← Rust 1.89.0
│   ├── Anchor.toml
│   ├── Cargo.toml           ← workspace root
│   ├── rust-toolchain.toml
│   └── programs/zk_pool/
│       ├── Cargo.toml
│       └── src/{lib,constants,error,events,state,instructions,encoding}.rs
│       └── src/instructions/{pool,deposit,withdraw}.rs
├── tests/                   ← standalone crate (Rust 1.98.1)
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── src/{lib,helpers,test_pool,test_deposit,test_withdraw,test_double_spend}.rs
├── services/
│   ├── backend/             ← Rust + axum (Stage 5)
│   │   ├── Cargo.toml
│   │   ├── migrations/001_init.sql
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── main.rs
│   │       ├── api_types.rs
│   │       ├── config.rs
│   │       ├── db.rs
│   │       ├── cache.rs
│   │       ├── tree.rs
│   │       ├── logging.rs
│   │       ├── metrics.rs
│   │       ├── rate_limit.rs
│   │       └── indexer.rs
│   ├── merkle/              ← Node.js + Fastify (Stage 6)
│   └── prover/              ← Rust + Sunspot (Stage 7)
├── web/                     ← Vue 3 (Stage 9)
│   └── src/
│       ├── api/             ← typed HTTP client
│       ├── components/      ← .vue files
│       ├── composables/     ← use*.ts files
│       ├── deposit/         ← deposit logic (generateNote, etc.)
│       ├── generated/       ← Codama output
│       ├── noir/            ← poseidon, hashes
│       ├── stores/          ← Pinia
│       ├── wallet/          ← wallet provider + kit signer
│       └── withdraw/        ← withdraw logic (parseNote, reduce, buildWitness, etc.)
├── scripts/
│   ├── validate-spec/       ← Rust CLI
│   ├── sync-circuits/       ← Rust CLI
│   ├── pool-init/           ← Rust CLI (Stage 8)
│   ├── e2e-deposit/         ← Rust CLI (Stage 10)
│   └── e2e-withdraw/        ← Rust CLI (Stage 10)
├── solana/                  ← gitignored (wallet volume)
├── .env.example
└── .gitignore
```

---

## 11. Commands

### Enter container

```bash
docker compose -f infra/docker-compose.yml exec solana bash

make exec-c
# means docker compose -f infra/docker-compose.yml exec solana bash -ic
```

### validate-spec / sync-circuits

```bash
make exec-c CMD='cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'

make exec-c CMD='cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- check'
```

### Anchor build / test / deploy

```bash
make exec-c CMD='cd /home/ubuntu/onchain && anchor build'

make exec-c CMD='cd /home/ubuntu/onchain && cargo test -p zk_pool --lib'

make exec-c CMD='cd /home/ubuntu/onchain && anchor program deploy --provider.cluster devnet'
```

### LiteSVM tests (from project root `tests/`)

```bash

make exec-c CMD='cd /home/ubuntu/tests && cargo test'
```

### Backend — build, run, smoke test

```bash
# Build
make exec-c CMD='cd /home/ubuntu/services/backend && cargo build --release'

# Run in background
make exec-c -d CMD='cd /home/ubuntu/services/backend && ./target/release/zkpool-backend > /tmp/backend.log 2>&1 &'

# Tail logs
make exec-c CMD='tail -30 /tmp/backend.log'

# Kill
make exec-c CMD='pkill -f zkpool-backend'
```

### Postgres / Redis

```bash
# Apply migration
docker exec -i zkpool-postgres psql -U zkpool -d zkpool < services/backend/migrations/001_init.sql

# Tables
docker exec zkpool-postgres psql -U zkpool -d zkpool -c "\dt"

# Redis ping
docker exec zkpool-redis redis-cli ping
```

### Full Sunspot pipeline (from `circuits/withdrawal/`)

```bash
nargo compile
nargo execute
sunspot compile target/withdrawal.json
sunspot setup target/withdrawal.ccs
sunspot deploy target/withdrawal.vk
solana program deploy target/withdrawal.so --program-id target/withdrawal-keypair.json --url devnet
sunspot prove target/withdrawal.json target/withdrawal.gz target/withdrawal.ccs target/withdrawal.pk
sunspot verify target/withdrawal.vk target/withdrawal.proof target/withdrawal.pw
```

---

## 12. Current state

**Last updated:** 2026-10-01 (Stage 15 in progress — 15.1 through 15.8 done)
**Last completed stage:** Stage 15.8 (frontend: split deposit UI).
**Next stage:** Stage 15.9 — E2E: 1 SOL → 3 notes → 3 withdrawals.

**Stages list:**
  - Stage 0 — Repository skeleton ✅
  - Stage 1 — Docker environment ✅
  - Stage 2 — Circuits on Noir ✅
  - Stage 3 — Sunspot verifier ✅
  - Stage 3.7 — Documentation enrichment ✅
  - Stage 4.1 — Anchor program ✅
  - Stage 4.5 — LiteSVM E2E test ✅
  - Stage 5 — Backend ✅
  - Stage 6 — Merkle service (Node.js) ✅
  - Stage 7 — Prover (Rust + Sunspot) ✅
  - Stage 8 — Deploy pool on-chain ✅
  - Stage 9 — Frontend (Vue 3 + TS + Pug + SCSS) ✅
  - Stage 10 — Full E2E (deposit → withdraw on devnet) ✅
  - Stage 11 — Infrastructure (Makefile, Prometheus, Grafana, port mappings) ✅
  - Stage 12 — Engineering processes (CI/CD, templates) ✅
  - Stage 13 — Security (threat model, expanded tests) ✅
  - Stage 14 — Finalization ✅ (release v0.1.0)
  - **Stage 15 — Split deposit ← in progress** (15.1 through 15.8 done)

**Release:** [v0.1.0](https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0) (2026-09-29) — tag on `48302ed`. 6 assets.

**Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (~3.4 SOL, devnet).
**Deployed programs on devnet:**
- Verifier: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` (**197 056 B**, upgraded in Stage 15.4 — was 87 312 B; Program ID unchanged).
- zk_pool: `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` (**222 144 B**, upgraded in Stage 15.5 — was 210 000 B; Program ID unchanged).

**Pool state (created 2026-09-27):**
- Pool PDA: `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`
- Vault PDA: `HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq`
- `PoolState`: 384 bytes, authority = wallet, all counters zero.
- Init tx: `2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1Cfh4pWrTFC2TcnEEiKfjq6e6AeBvNixxGQcBYnQZnB2Y3TPGFQ98zSeC`

**Tests:** 41 (circuits) + 37 (on-chain unit) + 15 (LiteSVM incl. adversarial) + 5 (backend unit) + 23 (merkle) + 17+1 (prover) = **139+**.

**Circuit ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` (v0.1.0) — `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db`
- `withdrawal.json` (Stage 15.3, **current**) — `a49bc877135ae75713d2ab7cbe39a69151a606c328f48fe8163c0629787d3262`

**Verifier artifacts (Stage 15.4, deployed in place on 2026-09-30):**
- `withdrawal.ccs` — `218c0524ae950e1b9c99df489ebace8b3cc15ac121b5312fcf92e49ecfb85260` (645 129 B)
- `withdrawal.pk` — `b05df8a3c906bc2fd3a1912896e6c8e7fbbab2e7625e809695f4cedf18ae284f` (2 166 095 B)
- `withdrawal.vk` — `9a25f71d916d85154fa8280361e27833a4c6ac535e2ea89c1db65d374e92b5a3` (1 360 B)
- `withdrawal.so` — `f2c3bea017b113100c31289b939d9f6d0c1d5a2602216d778000ac97260669ca` (197 056 B)
- Upgrade tx: `5z68jatuiaScN5ff5NXYpgRsgfhT7om9FGSGErkcJJj5JDJXwKD4Zs7Bjet9ST28d5Q9G2quEXCTcwQdiq9KMKSV`

**On-chain state after Stage 15.5.** Both programs on devnet are consistent with the Stage 15.3 ACIR:

- Verifier (`5t51iu6a…`) — upgraded in place (Stage 15.4), 197 056 B, runs the new `.vk`.
- zk_pool (`8cGzkFK9H…`) — upgraded in place (Stage 15.5), 222 144 B, `PROOF_LEN = 388`, `NR_PUBLIC_INPUTS = 6`.

**The 15.4 → 15.5 broken window is closed.** On-chain `withdraw` now expects 388-byte proofs and 204-byte public inputs.

**zk_pool upgrade tx:** `5cZwCsBhxyB4qJACcLHUoGhQcrrxW6cH84R2jZU7D8hEdVb1FPvMMibSP2sNJdqT2ZVkU6rybr4dUHXpkLC5X47h`
**New zk_pool `.so`:** `d4eee2d5ffc6d89337f917ea05c9b75b081203705fe79f17ed60e4370b86c735` (222 144 B)
**New zk_pool IDL:** `5ecbaed8a7497e70e407b0b9db32153967a5d8549b93cf89df3b9134cf1be29d` (21 739 B)

**Running containers:** `solana-zkpool-solana`, `zkpool-postgres` (healthy), `zkpool-redis` (healthy).
**Running processes (inside `solana` container):** merkle (4003), prover (4002), backend (4001) — must be restarted after container restart.

**Docs sizes:**
- `00-glossary.md` — 545 lines
- `00-zk-primer.md` — 317 lines
- `01-setup.md` — 762 lines
- `02-circuits.md` — 957 lines
- `03-sunspot.md` — 996 lines
- `04-anchor.md` — ~1014 lines
- `05-backend.md` — ~1200 lines

**Design (locked in Stage 15.1, see `docs/notes/15-split-deposit.md`):**
- **Path 2** — the withdrawal circuit **changes**, not just the deposit side. New public input `total_amount` (aggregated deposit amount). New private inputs `splits[3]` and `note_index`. New constraints C4 (`Σ splits[i] == total_amount`) and C5 (`splits[note_index] == amount`).
- **N = 3, fixed.** Const-generic, same pattern as `TREE_DEPTH`.
- **Verifier upgrade in place.** Program ID `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` does **not** change. Only the `.so` is replaced.
- **Breaking change.** Notes from v0.1.0 (2 real deposits on devnet) are incompatible with the new circuit. Version will bump to **v0.2.0**.

**Why Path 2 and not Path 1:** Path 1 (deposit-side only) leaves the amount link intact at the transaction level and — critically — allows a malicious depositor to claim a smaller `total_amount` than the actual sum of the notes, enabling direct theft from the vault. C4 closes that hole.

**Architectural compatibility anchors that held:**
- `TREE_DEPTH` and `SPLIT_COUNT` live in one place (`spec.json`), validated via `scripts/validate-spec`.
- `PoolState` on-chain does **not** assume "1 deposit = 1 commitment".
- `commitments` table in Postgres has **no** `UNIQUE` on `(pool, tx_signature)` — one tx can create N commitments.
- `WithdrawEvent` already has an explicit `amount` field.

**Progress:**
- 15.1 — design doc (`docs/notes/15-split-deposit.md`). ✅ Commit `2285ed4`.
- 15.2 — `spec.json` + `rules.rs` + `spec.rs` updated. ✅ Commit `cec750e`.
- 15.3 — circuit change (`main.nr`, `test_witness.nr`), new ACIR. ✅ Commit `1fc9a5d`.
- 15.4 — Sunspot re-run + verifier upgrade. ✅ Commit `0f02d34`.
- 15.5 — Anchor: `deposit_split`, constants updated, upgraded in place. ✅ Commit `a66bee2`.
- 15.6 — LiteSVM tests for `deposit_split` (5 tests, 20 total). ✅ Commit `131b2ac`.
- 15.7 — Backend + prover: split-deposit witness fields. ✅ Commit `7936d3a`.
- 15.8 — Frontend: split deposit UI + Codama regeneration. ✅ Commit `b7a8561`.
- 15.9 — E2E: 1 SOL → 3 notes → 3 withdrawals. **← next**

---

## 13. Documentation as portfolio material

**Set on 2026-09-24.**

### 13.1. Context

`docs/notes/*.md` is:
1. Raw material for guides, tutorials, and articles on **Medium** and **Mirror.xyz**.
2. Part of the GitHub portfolio.
3. Teaching material.

### 13.2. Structure requirements for every note

1. TL;DR at the top.
2. Glossary links.
3. "Why" block before each command.
4. "Expected result" block after each command.
5. ASCII diagrams.
6. Screenshots in `docs/notes/assets/NN-<name>/`.
7. Cross-references.
8. "Common errors" section.
9. "Reproduction" section.
10. "What's next" section.

### 13.3. Glossary

**Status:** ✅ done. 545 lines, 76 terms.

### 13.4. ZK-primer

**Status:** ✅ done. 317 lines, 14 sections.

### 13.5. Screenshots policy

- Saved to `docs/notes/assets/NN-<stage>/`.
- Naming: `NN-MM-<description>.png`.
- Committed.
- Max 500 KB per file.

### 13.6. Enrichment procedure

After each stage: write the technical note → enrich with why-blocks, expected results, common errors, what's next → capture screenshots → update glossary.

### 13.7–13.8. Enrichment of stages 01–03 and 3.7 — DONE

### 13.9. Ongoing

- Screenshots captured as we go.
- Glossary updated when new terms appear.

---

