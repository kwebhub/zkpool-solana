# PROJECT_CONTEXT.md

> **Purpose:** single entry point for an AI assistant in a new chat. Load this file first.
>
> **Last updated:** 2026-09-27 (Stage 8.2 — pool-init in progress)
>
> **Handoff note:** this version was condensed from a 720-line file to fit cleanly into a new chat's context. Every section is preserved; some have been tightened. Nothing was removed.

---

## Setup

### System Overview

* Laptop Model: Dell Latitude 5330
* OS: Debian GNU/Linux 13 (trixie) x86_64
* Kernel: 6.12.107+deb13-amd64 (Preempt Dynamic)

### Hardware Specs

* CPU: 12th Gen Intel Core i5-1245U (10 Cores / 12 Threads)
* RAM: 16 GiB (15 GiB usable) — 2.5 GiB used / 11 GiB free
* Swap: 12 GiB (0 B used)

### Storage & Drives

* NVMe SSD: 256 GB (nvme0n1, 238.5 GiB) — Fully encrypted (LUKS + LVM)

### Text Editor

* LazyVim (Neovim distribution)

---

## 0. Rules for the assistant (READ FIRST)
This is the **single normative section** of this file. Everything else describes the project. If in doubt — this section wins.

### 0.1. Quick-start for a new assistant

1. **Read section 0 completely** — especially 0.9 (document non-obvious), 0.10 (small steps), 0.11 (never delete info), 0.13 (record), 0.14 (record on push), 0.15 (checkpoint immediately), 0.16 (no mid-message retractions).
2. Read **section 12 (Current state)**.
3. Read **section 13 (Documentation as portfolio material)**.
4. Read these notes in order:
   - `docs/notes/00-checkpoints.md`
   - `docs/notes/00-glossary.md`
   - `docs/notes/00-zk-primer.md`
   - `docs/notes/01-setup.md`
   - `docs/notes/02-circuits.md`
   - `docs/notes/03-sunspot.md`
   - `docs/notes/04-anchor.md`
   - `docs/notes/05-backend.md`
   - `docs/notes/06-merkle.md`
   - `docs/notes/07-prover.md`
   - `docs/notes/08-pool-init.md`
   - `docs/notes/09-frontend.md`
   - `docs/notes/10-e2e.md`
   - `docs/notes/11-infra.md`
   - `docs/notes/12-engineering.md`
   - `docs/notes/13-security.md` (in progress)
5. Skim **section 8 (Known pitfalls)** — it's the fastest way to avoid re-learning our mistakes.
6. Then go to **section 12** and start with **Next task**.

### 0.2. One task at a time

The assistant gives **exactly one task** per message. Wait for the user to run it, paste the output, and only then give the next task.

**Do NOT** give multiple commands in one message. **Do NOT** chain "then do X, then do Y".

**Exception — `git commit` and `git push` are always grouped** as one task:
```bash
git commit -m "..." && git push
```
`git add -A` is still a **separate** task.

### 0.3. Language

- **Chat communication: English.**
- **Files on disk:**
  - **English** by default — code, configs, docs at root level.
  - **Russian** for `docs/ru/*.md` and `docs/notes/*.md`.

**Rationale:** Russian tokens cost 2–3× more in the tokenizer; English chat preserves context. Notes stay Russian — they are portfolio material for a Russian-speaking audience.

### 0.4. Never guess

If something is ambiguous — **ask the user before proceeding**.

### 0.5. Checkpoints are mandatory

After each stage, save artifacts to `.checkpoints/NN-name/` with SHA-256 in `manifest.txt` and the commit hash in `commit.txt`.

### 0.6. Update context after each stage

After each completed stage:
- Update `docs/PROJECT_CONTEXT.md` (this file).
- Add or update `docs/notes/NN-name.md` (Russian).
- Commit and push.

### 0.7. No multi-command chains (except `git commit && git push`)

### 0.8. Documentation is portfolio material — see section 13

**CRITICAL:** `docs/notes/*.md` is raw material for guides, tutorials, and articles on **Medium** and **Mirror.xyz**, and part of the GitHub portfolio. See section 13.

### 0.9. Non-obvious project actions must be documented

Any non-obvious action required to complete a stage must be:
1. Recorded in `PROJECT_CONTEXT.md`, section 8 (Known pitfalls).
2. Included in `docs/notes/NN-name.md` as a **lesson** with symptom, cause, fix.

Examples:
- Manually reading crate sources to check exact API.
- Adding packages to `workspace.exclude`.
- Running `cargo fetch` in a sub-crate.
- Any unusual CLI flag or env var.

### 0.10. Test in small steps — do not write 200 lines at once

When working with a **new** library (LiteSVM, Anchor macros, sqlx, axum), **do not** write a large file in one shot. Write **20 lines**, compile, verify the API matches, then expand.

**How to check exact API:** read crate sources at `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`. Look at `pub use` / `pub fn` / `pub struct` lines.

### 0.11. Updating existing files — never delete information

When updating an existing file in `docs/notes/` or `docs/PROJECT_CONTEXT.md`:

1. **Take the current content** (user pastes it, or assistant has it from chat history).
2. **Preserve it in full** — no information removed.
3. **Add** new sections in the appropriate place.
4. **Update** the glossary if new terms appeared.

**Output format:**

- **New files** — give the **full file**.
- **Existing files** — give **insertion point + block**. Do **not** give the full file unless the user explicitly asks.
- **Never** shorten, "simplify", or replace existing sections with a summary.

**Allowed:** rephrase, reorder sections, improve wording — as long as information is preserved.
**Forbidden:** remove or shorten existing sections.

### 0.12. Chat language — English only

All chat messages between user and assistant are in **English**.
Russian is used **only** for files in `docs/ru/*.md` and `docs/notes/*.md`.

### 0.13. "record" means record it in notes and context

When the user writes just **"record"** (or Russian "зафиксируй"):

1. Update `docs/notes/NN-name.md` — add a section for the current sub-stage.
2. Update `docs/PROJECT_CONTEXT.md`:
   - Stage NN table (mark current sub-stage done, next as `← next`).
   - Section 12 (Current state).
   - Section 0.1 (Quick-start) — Last completed / Next task.
3. Commit both files together.

**Do not** ask "should I record it?" — just do it.

### 0.14. Record with each `git push`

**After each `git push`** — immediately update `docs/notes/NN-name.md` and `docs/PROJECT_CONTEXT.md`. Do not wait for the user to say "record".

**Exception:** purely-docs commits (which are themselves the "record" step) don't need a second record pass.

### 0.15. Checkpoint immediately after each sub-stage commit

**Immediately after `git commit && git push` of a sub-stage, create the checkpoint** — do not defer.

The checkpoint directory name matches the sub-stage: `.checkpoints/NN.M-<name>/`.

Contents (per the `05.5-tree` convention):
- Artifact files **copied** into the checkpoint directory.
- `manifest.txt` — lines of `<sha256>  <filename>` (relative names).
- `commit.txt` — the sub-stage commit hash.

**Reason:** the Stage 6.1–6.3 checkpoints were skipped and had to be backfilled. Backfill is recovery, not the process.

---

### 0.16. No mid-message retractions

**Give one action per message.** No "Wait —", no self-contradicting corrections after a task in the same message, no "Actually — let me reconsider". If a task turns out wrong, state the new task in the **next** message, after the user has run the previous one.

**Reason:** mid-message retractions confuse the user and waste tokens. Figure it out **before** sending.

---

## 1. What this project is

**zkpool-solana** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (four secrets), and later withdraws SOL to any address without revealing the link between deposit and withdrawal.

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous versions:**
- [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1.
- `kwebhub/solana-zk-pool` — v2 (abandoned; bug: `withdraw` fails with `InvalidInstructionData`, root cause not found).

**This repository:** https://github.com/kwebhub/zkpool-solana (private) — v3.
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

**Tests:** 41 across all circuits.
**ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db`

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

**Known gaps:** `merkle` job is placeholder (no nargo); `security` is report-only (`|| true`); `cargo-deny` needs `deny.toml`.

### ✅ Docs (2026-09-22 — 2026-09-25)

- `docs/notes/00-checkpoints.md`, `00-glossary.md`, `00-zk-primer.md`.
- `docs/notes/01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md` (~1014 lines), `05-backend.md` (~1200 lines).
- `docs/DEMO-NOTICE.md`.
- `docs/PROJECT_CONTEXT.md` — this file.
- `docs/notes/assets/{01-setup,02-circuits,03-sunspot}/` — empty folders for screenshots.

---

## 7. Architecture decisions

### 7.1. Spec validation — YES (not generation)

`spec.json` is the source of truth; layers validated against it via `scripts/validate-spec`. Generation of `.nr` and Rust is fragile; validation is simpler and catches the same class of bugs.

### 7.2. LiteSVM E2E test — YES

`tests/src/` contains LiteSVM integration tests. Full E2E with real proof deferred to devnet.

### 7.3. One circuit with `recipient_binding` — YES

Public inputs (5): `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`. No mid-project format change.

### 7.4. Zero-padding (no domain separation) — CONFIRMED

`hash_1(x) == hash_2(x, 0)` and `hash_2(x, y) == hash_3(x, y, 0)`. Documented. Safe for zkpool-solana because we never hash the same value with two arities in one context.

### 7.5. `sync-circuits` — check/apply modes

`--check`: mismatch → error, exit 1, no copy. `--apply`: mismatch → copy + warning, exit 0.

### 7.6. Split deposit — DEFERRED to Stage 12/13

**Feature:** split a single deposit into multiple unequal commitments (improves privacy by breaking the amount link).

**Status:** deferred. Not implemented in v3 either.

**Rationale:**
- Changes public input layout (N commitments, aggregated amount).
- Would obscure root-cause analysis of the original `InvalidInstructionData` bug.
- Requires changes at every layer.

**Architectural compatibility preserved:**
- `TREE_DEPTH` and other constants live in one place (spec).
- `PoolState` on-chain doesn't assume "1 deposit = 1 commitment" beyond what's needed.
- `commitments` table in Postgres has **no** `UNIQUE` on `(pool, tx_signature)`.
- `WithdrawEvent` includes an explicit `amount` field.

**When implemented:** after v0.1.0, as a separate stage (13) or follow-up project.

### 7.7. Makefile — clean design (Stage 9)

### 7.8. `tests/` — separate crate at root

**Decision (2026-09-24):** `tests/` is **outside** `onchain/`, at the project root.

**Reason:** `litesvm 0.16` requires Agave 4.2, which requires **Rust ≥ 1.90**. `onchain/rust-toolchain.toml` pins **1.89.0** (needed for SBF). Keeping tests inside `onchain/` means they inherit the 1.89 toolchain and fail with `E0658`.

**Solution:** separate crate at root with its own `rust-toolchain.toml` (`channel = "1.98.1"`).

**Connection:** `zk_pool = { path = "../onchain/programs/zk_pool" }`.

### 7.9. Stages list

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
- **Stage 13 — Security (threat model, expanded tests) ← in progress (13.1–13.5 done)**
- Stage 14 — Finalization
- Stage 15 — Deferred: split deposit (after v0.1.0)

### 7.10. Merkle service ordering dependency

`tree.rs` (backend) uses HTTP to the Merkle service for Poseidon2 hashing. The backend **starts fine** without the Merkle service, but any `add_leaf` call (triggered by the indexer when it sees a deposit) will fail with an HTTP error until Stage 6 is running.

**Startup order in Stage 9 Makefile must be:** `merkle` → `backend`.

---

## 8. Known pitfalls

### From v2

- `anchor init --name <name>` does not create a subdirectory.
- `sunspot --version` is not supported.
- `sunspot deploy` requires `GNARK_VERIFIER_BIN` — Sunspot cloned in Dockerfile.
- `anchor build` uses existing keypair only if present in `target/deploy/`.
- `@solana/kit`, `@codama/*` — pin **exact** versions (no `^`).
- Codama path: `../onchain/target/idl/zk_pool.json` (from `web/`).
- `Prover.toml` is gitignored.
- `bash -ic` required inside the container (`.bashrc` not read by `bash -c`).

### From Stage 2 (circuits)

- `poseidon2_permutation` in nargo 1.0.0-rc.2 takes **one** argument.
- Zero-padding causes arity collisions — documented.
- `main()` in `type = "bin"` circuits must have `pub` on return type.
- Field values must be < 2^254.
- Noir does not support float literals.
- `&str` parameters in Noir don't work — use `str<N>`.
- `f"...{value}"` format strings need explicit type annotation.

### From Stage 3 (sunspot)

- `solana/` volume may be root-owned — fix with `sudo chown -R 1000:1000 solana/`.
- Ignore entire `solana/` directory in `.gitignore`.
- `sunspot verify` argument order: `.vk`, `.proof`, `.pw`.
- TOML parser rejects hex literals > 2^63 — wrap in quotes.
- Sunspot generates 6 `deprecated` warnings during deploy — normal.

### From Stage 4.1 (Anchor)

- `anchor init .` fails — use a temp dir.
- `invalid --check-cfg argument` — declare `anchor-debug`, `custom-heap`, `custom-panic` as **features**, not via `[lints.rust]`.
- `E0107: struct takes 0 lifetime arguments` — empty `#[derive(Accounts)]` doesn't work.
- `unused import: super::*`.
- `test_reduce_to_field_max` FAILED — loop `0..4` → `0..5`.
- `ambiguous glob re-exports` — rename `handler` → `handler_pool` etc.
- `E0308` in `CpiContext::new` — Anchor 1.2.0 takes `Pubkey`, not `AccountInfo`. Use `.key()`.
- `E0432: unresolved import crate` — keep glob re-exports.
- `E0277: #[instruction] type mismatch` — list **all** handler args in order.
- CPI to Sunspot verifier: data layout is `[proof || public_witness]` — proof FIRST.
- Vault PDA has no private key — direct lamport manipulation.
- `anchor deploy` is deprecated — use `anchor program deploy`.
- `zk_pool.so` grows to 210 KB with 3 instructions.

### From Stage 4.5 (LiteSVM)

- `cargo fetch` inside `onchain/tests/` fails with "current package believes it's in a workspace when it's not".
  - Cause: `onchain/Cargo.toml` is a workspace root, `members = ["programs/*"]` doesn't cover `tests/`.
  - First fix: `workspace.exclude = ["tests"]`. Helped fetch, but didn't fix the Rust version problem.
  - Final fix: move `tests/` to project root (see 7.8).
- `E0658: use of unstable library feature maybe_uninit_write_slice` in `solana-syscalls 4.2.2`.
  - Cause: `litesvm 0.16` pulls Agave 4.2, which needs Rust ≥ 1.90.
  - `onchain/rust-toolchain.toml` pins 1.89.0 — `tests/` inherited it.
  - Fix: `tests/rust-toolchain.toml` with `channel = "1.98.1"`.
- `failed to select a version for solana-hash` — version conflict between `litesvm 0.16` (`solana-hash ~4.5.0`) and `solana-message 5` (`>= 4.6.0`).
  - Fix: use **exact** versions from litesvm's `Cargo.toml`:
    `solana-account = "4.3.0"`, `solana-address = "~2.6.1"`, `solana-hash = "4.5.0"`, `solana-instruction = "3.4.0"`, `solana-keypair = "3.1.2"`, `solana-message = "4.2.4"`, `solana-sdk-ids = "3.1.0"`, `solana-signer = "3.0.1"`, `solana-transaction = "4.1.5"`, `solana-transaction-error = "3.3.1"`.
- `/home/ubuntu/tests: No such file or directory` — volume not mounted.
  - Fix: add `- ../tests:/home/ubuntu/tests` in `docker-compose.yml`, then `docker compose up -d --force-recreate solana`.
- `E0583: file not found for module` — expected, files not written yet.
  - Fix: create placeholder files (`//! Placeholder`).
- Random `.rs` file (bash heredoc in a for-loop).
  - Cause: bash mishandled the first iteration (`$f` not expanded).
  - Fix: `rm tests/src/.rs`.
  - Lesson: when using loops with heredoc — check `ls` immediately after.
- `E0308: Transaction::new_signed_with_payer expects &[Instruction], found &Vec<CompiledInstruction>`.
  - Cause: `Message::new_with_blockhash` compiles instructions.
  - Fix: use `Transaction::new_signed_with_payer(&[ix], ...)` directly with raw instructions.
  - Lesson: in `solana-transaction` there are two paths — via `Message` (compiled) and via raw `Instruction`. Don't mix them.
- Deposit test: vault balance is NOT zero after init (890 880 lamports rent-exempt).
  - Fix: read `vault_initial` after init, then `assert_eq!(vault_after, vault_initial + amount)`.
  - Lesson: never assume a PDA has zero lamports after `init`.
- Withdraw test: Borsh serialization for `Vec<u8>`.
  - Fact: in Anchor, `Vec<u8>` is serialized as `u32 LE length` + bytes. Fixed-size arrays are raw.
- Withdraw E2E with real proof is not feasible in LiteSVM.
  - Reason: proof from stage 3.5 was generated for a synthetic state.
  - Decision: test validations + CPI reach only. Full E2E on devnet (Stage 8).

### From Stage 5 (backend)

- `chrono` used transitively via `sqlx` but not in direct `[dependencies]`.
  - Cause: `sqlx`'s `chrono` feature integrates but doesn't re-export types.
  - Fix: add `chrono = { version = "0.4", features = ["serde"] }`.
  - Lesson: if you use a type from a transitive dep, declare it explicitly.
- Content of `logging.rs` and `metrics.rs` swapped (test appeared in wrong file).
  - Cause: editor applied both "replace fully" instructions to the same file.
  - Fix: rewrote both files.
  - Lesson: after "replace fully", always check `wc -l` and `grep` for characteristic function names. If a test appears under the wrong module — files got swapped.
- `E0277: WithdrawResponse: DeserializeOwned not satisfied` with note "multiple different versions of crate `zkpool_backend`".
  - Cause: `WithdrawResponse` was defined in `main.rs` (bin), while `reqwest::json::<T>()` needs `T` from the same crate instance as `serde`.
  - Fix: move shared types to lib — created `src/api_types.rs`.
  - Lesson: in crates with both `[[bin]]` and `[lib]`, all shared types must live in the lib.
- `/metrics` returned 200 with empty body.
  - Cause: `metrics-exporter-prometheus` doesn't render metrics that were never recorded. Plus default idle-timeout drops old metrics.
  - Fix: `touch_startup_metrics()` records each metric once with a zero value.
  - Note: `MetricKindMask` is in `metrics-util`, not re-exported by `metrics-exporter-prometheus`. Adding `metrics-util` as a direct dep would let us call `idle_timeout(MetricKindMask::ALL, None)` instead.
- Ports 4001–4003, 5173 not reachable from the host.
  - Cause: no `ports:` mapping in `docker-compose.yml` for `solana` service.
  - Workaround: `docker compose exec solana curl ...`.
  - Fix (Stage 9): add port mappings.
- `jq` and `python3` not installed in `solana` container.
  - Workaround: use raw `curl` output.

### From Stage 6 (merkle service)

- `node --test <dir>` in Node 24 treats `<dir>` as a module, not a directory to scan.
  - Symptom: `Error: Cannot find module '/path/test'`.
  - Fix: use a glob — `node --test test/*.test.js` (unquoted, so the shell expands it).
  - `package.json` script: `"test": "node --test test/*.test.js"`.
- Fastify app must be split for testability: `buildApp()` in `app.js`, `listen()` in `server.js`.
  - Reason: tests use `app.inject()` which requires an app instance without a bound port.
- `sqlx-postgres v0.7.4` emits a future-incompatibility warning under Rust 1.98.
  - Not blocking; note for Stage 11 / 12.
- Stale ACIR copies in `services/merkle/circuits/` and `web/public/circuits/`.
  - Fix: `sync-circuits --apply` (see earlier bullet).
  - Lesson: run `sync-circuits --check` at the start of any ACIR-consuming stage.

### From Stage 7 (prover)

- `nargo execute -p <name>` requires `<name>` to have **no dots**. `-p Prover-7.3-test` is parsed as `Prover-7`.
  - Fix: use uuid-based names without dots (`Prover-<uuid-simple>`).
- `Prover.toml` hex values must be **`0x`-prefixed**. Bare hex fails for `amount` and `merkle_proof[i]`:
  - Error: `Expected witness values to be integers, but '00f4240' failed with 'invalid digit found in string'`.
  - Fix: `witness.rs::with_0x()` adds the prefix idempotently.

### From Stage 10 (E2E deposit)

- **BN254 field mask must account for the modulus's top byte.**
  - BN254 prime = `0x30644e72…` — top byte is `0x30`.
  - Mask `& 0x3f` allows top byte up to `0x3f` — **exceeds the modulus** for values `0x31–0x3f`.
  - Symptom: `POST /hashes` → 500, `Value 0x31020e14… exceeds field modulus`.
  - Fix: mask `& 0x1f` (top byte ≤ `0x1f < 0x30`).
  - Applied in both `scripts/e2e-deposit/src/main.rs` and `web/src/deposit/generateNote.ts`.

### From Stage 10 (E2E withdraw)

- **Anchor Borsh: `Vec<u8>` needs a `u32 LE` length prefix.**
  - Symptom: `memory allocation failed, out of memory` on instruction entry.
  - Cause: wrote `[disc || proof_bytes || ...]` instead of `[disc || u32_le_len || proof_bytes || ...]`.
  - Fix: prepend `(proof.len() as u32).to_le_bytes()`.
- **`NULLIFIER_RECORD_SEED = b"nullifier_record"`** — no `"3"` suffix (unlike `pool3`/`vault3`).
  - Symptom: `ConstraintSeeds. Left: HaviR… Right: DNfyZ…`.
  - Always read `constants.rs` — do not guess seeds.
- **Groth16 verification consumes ~182k CU.** Default 200k is not enough.
  - Symptom: `ComputationalBudgetExceeded` with `Proof verified successfully!` in logs.
  - Fix: prepend `ComputeBudgetInstruction::SetComputeUnitLimit(400_000)`.
  - Instruction data: `[2, u32 LE limit]`. Program ID: `ComputeBudget111111111111111111111111111111`.

### From Stage 13 (security)

- **`sqlx 0.7.4` has RUSTSEC-2024-0363** (SQL injection via protocol smuggling).
  - Fix: upgrade to `sqlx 0.8`. No code changes required.
- **`cargo-deny 0.20.2` schema is incompatible with `version = 2` in `deny.toml`.**
  - Symptom: `licenses FAILED` on our own crates with `license = "MIT"` even when MIT is in the allow list.
  - Fix: regenerate `deny.toml` via `cargo deny init` inside a crate directory (requires `Cargo.toml` present).

### From the backend↔prover wiring (2026-09-27)

- **Schema bug:** `commitments` has `commitments_leaf_index_key UNIQUE (leaf_index)` — global uniqueness, not per-pool.
  - Symptom: inserting `leaf_index = 0` for a second pool → `INSERT 0 0` (silently swallowed by `ON CONFLICT DO NOTHING`).
  - Impact: multi-pool support broken. Single-pool (this project) unaffected.
  - Fix (deferred to Stage 11/12): drop `commitments_leaf_index_key`, keep `commitments_pool_leaf_idx` but make it UNIQUE.
- **Backend↔prover wire format was mismatched** at first: `WithdrawRequest` (Stage 5) used `{witness: base64}`, prover expected 10 hex fields.
  - Fix: rewrote `WithdrawRequest` to match `WitnessInputs`; backend converts prover's hex output → base64 for `WithdrawResponse`.
  - Lesson: when a second service appears, verify wire formats immediately — both were written by us, but at different times with different assumptions.
- **All three services die on container restart.** After `docker compose up -d --force-recreate solana`, restart merkle (4003), prover (4002), backend (4001) manually. Makefile planned for Stage 9.

### How to find exact signatures for the installed crate version
- Rust crate sources: `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`.
- Anchor: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anchor-lang-1.2.0/src/`.
- LiteSVM: `~/.cargo/registry/src/index.crates.io-*/litesvm-0.16.0/src/`.
- Always check your **own** version, not v2 examples.

### How to resolve solana-* version conflicts

1. Find the "main" crate (e.g. `litesvm`) and its exact `solana-*` versions in its `Cargo.toml`.
2. Use **exactly** those versions.
3. Don't guess or use semver ranges (`"4"`, `"5"`) — they pull incompatible versions.
4. Run `cargo tree -p <main-crate>` to verify.
5. If a newer toolchain is required, isolate the crate (see 7.8).

---

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
├── docs/
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

## 10. Git workflow

- Conventional commits.
- English files except `docs/notes/*.md`, `docs/ru/README.md`.
- Gitignored: `.secrets/`, `.checkpoints/`, `solana/`, `services/merkle/circuits/`, `web/public/circuits/`, `Prover.toml`, `**/target/`, `node_modules/`.
- Committed: `docs/notes/` including `assets/`.
- `git commit ... && git push` — one task. `git add -A` — separate.

---

## 11. Commands

### Enter container

```bash
docker compose -f infra/docker-compose.yml exec solana bash
```

### validate-spec / sync-circuits

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'

docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- check'
```

### Anchor build / test / deploy

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/onchain && anchor build'

docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/onchain && cargo test -p zk_pool --lib'

docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/onchain && anchor program deploy --provider.cluster devnet'
```

### LiteSVM tests (from project root `tests/`)

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/tests && cargo test'
```

### Backend — build, run, smoke test

```bash
# Build
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/services/backend && cargo build --release'

# Run in background
docker compose -f infra/docker-compose.yml exec -d solana bash -ic \
  'cd /home/ubuntu/services/backend && ./target/release/zkpool-backend > /tmp/backend.log 2>&1 &'

# Tail logs
docker compose -f infra/docker-compose.yml exec solana bash -ic 'tail -30 /tmp/backend.log'

# Kill
docker compose -f infra/docker-compose.yml exec solana bash -ic 'pkill -f zkpool-backend'
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

**Last completed stage:** Stage 13.5 (sqlx 0.7→0.8 upgrade for RUSTSEC-2024-0363; deny.toml; make exec-c).
**Next stage:** Stage 13.6 — final checkpoint for Stage 13.

**Recent bridge commits (between Stages 7 and 8):**
- `e1eae1d` — wire `/api/withdraw` → prover (hex in, base64 out).
- `3780473` — wire `/api/proof` → Merkle service.

**Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (~3.4 SOL, devnet).

**Deployed programs on devnet:**
- Verifier: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` (87 312 B).
- zk_pool: `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` (210 000 B, rent 1.068 SOL).

**Pool state (created 2026-09-27):**
- Pool PDA: `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`
- Vault PDA: `HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq`
- `PoolState`: 384 bytes, authority = wallet, all counters zero.
- Init tx: `2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1Cfh4pWrTFC2TcnEEiKfjq6e6AeBvNixxGQcBYnQZnB2Y3TPGFQ98zSeC`

**Tests:** 41 (circuits) + 37 (on-chain unit) + 15 (LiteSVM incl. adversarial) + 5 (backend unit) + 23 (merkle) + 17+1 (prover) = **139+**.

**Circuit ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db`

**Running containers:** `solana-zkpool-solana`, `zkpool-postgres` (healthy), `zkpool-redis` (healthy).
**Running processes (inside `solana` container):** merkle (4003), prover (4002), backend (4001) — must be restarted after container restart.

**Pool state (created 2026-09-27):**
- Pool PDA: `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`
- Vault PDA: `HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq`
- `PoolState`: 384 bytes, authority = wallet, all counters zero.
- Init tx: `2EHRSsJeSUcFrhkn4rJWznihMjQqjqd1Cfh4pWrTFC2TcnEEiKfjq6e6AeBvNixxGQcBYnQZnB2Y3TPGFQ98zSeC`

**Docs sizes:**
- `00-glossary.md` — 545 lines
- `00-zk-primer.md` — 317 lines
- `01-setup.md` — 762 lines
- `02-circuits.md` — 957 lines
- `03-sunspot.md` — 996 lines
- `04-anchor.md` — ~1014 lines
- `05-backend.md` — ~1200 lines

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

## 14. Stage 6 — Merkle service (done)

**Goal:** Node.js + Fastify HTTP service on port 4003 that exposes Poseidon2 hashing and Merkle tree operations to the backend and frontend.

**Why Node.js:** the backend and frontend both need Poseidon2 hashes that match the Noir circuit byte-for-byte. Rust and JS Poseidon2 implementations produce **different** hashes. The only way to guarantee identity is to use the same ACIR (`hash2.json`) via `@noir-lang/noir_js`, which runs **only in JavaScript**. So the Merkle service is a thin JS wrapper around `noir_js`.

**Endpoints:**
- `POST /hash` — body `{left: hex, right: hex}` → `{hash: hex}`. Used by `tree.rs` in the backend.
- `POST /root` — body `{commitments: [hex]}` → `{root: hex}`. Debug helper.
- `POST /proof` — body `{commitments: [hex], leaf_index: N}` → `{proof: [hex], is_even: [bool]}`.
- `GET /health` → `{status: "ok"}`.

**Dependencies:** `fastify`, `@fastify/cors`, `@noir-lang/noir_js@1.0.0-rc.2`.

**Circuits needed in `services/merkle/circuits/`:** `hash2.json`, `hashes.json`, `withdrawal.json` — already copied by `sync-circuits` at Stage 2.3, and the folder is gitignored.

**Sub-stages planned:**
- 6.1 — `package.json` + dependencies + folder structure. ✅ (`4142789`)
- 6.2 — `src/poseidon.js` — Noir instance caching, `poseidon2Hash(left, right)`. ✅ (`d021210`)
- 6.3 — `src/merkle.js` — build tree, root, proof. ✅ (`0e9d766`)
- 6.4 — `src/server.js` — Fastify routes. ✅ (`fa67096`)
- 6.5 — tests (`node --test`). ✅ (`cf34064`) — 23 tests, split `app.js`/`server.js`
- 6.6 — run + smoke test with `curl`. ✅ (`d090a3b`) — 8/8 smoke + backend integration
- 6.7 — final checkpoint. ✅ (`99a228a`)

**6.1 findings (2026-09-25):**
- `@noir-lang/noir_js@1.0.0-rc.2` exists on npm; pulls `acvm_js@1.0.0-rc.2`, `types@1.0.0-rc.2`, `noirc_abi@1.0.0-rc.2`, `pako@^3.0.1`.
- `fastify@5.12.5`, `@fastify/cors@11.3.0` — latest stable.
- `Noir` API: `new Noir(circuit)` → `execute(inputs) → { witness, returnValue }`.
- `returnValue` is a `0x`-prefixed hex string (single output) or an **array** of hex strings (tuple output).
- **Cross-check passed:** `noir_js` recomputed `nullifier_hash` and `root` from the Stage 3.5 `Prover.toml` witness and matched byte-for-byte. This validates the entire premise of using JS for Poseidon2.
- **Pitfall caught:** stale ACIR copies in `services/merkle/circuits/` (see section 8). Run `sync-circuits --check` at the start of every ACIR-consuming stage.

## 15. Stage 7 — Prover (done)

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

**Sub-stages:**
- 7.1 — project skeleton (`services/prover/`, `Cargo.toml`, `main.rs`). ✅ (`ff31292`)
- 7.2 — `config.rs` (paths to circuit artifacts + nargo/sunspot binaries). ✅ (`4678050`)
- 7.3 — `witness.rs` (serialize `Prover.toml`). ✅ (`95052dd`)
- 7.4 — prover.rs (nargo execute + sunspot prove, mutex, cleanup). ✅ (8103dd1)
- 7.5 — `server.rs` (axum `POST /prove`, `GET /health`). ✅ (`6aa5fb4`)
- 7.6 — HTTP-level tests (`tests/server_test.rs`). ✅ (`7603672`)
- 7.7 — run + smoke test (real witness → real proof → verify with `sunspot verify`). ✅ (`89ad83d`) — 6/6 checks
- 7.8 — final checkpoint. ✅ (`f5a48bf`)

**Design decisions:**
- **Sync, not async job queue.** v1 — HTTP request blocks until the proof is ready (~500 ms). Acceptable for a demo.
- **Mutex, not per-request temp dir.** Simpler, correct; throughput ~2–5 proofs/sec is enough.
- **No caching.** v1 — deterministic inputs → deterministic proof, but no cache layer yet.
- **Reference `circuits/withdrawal/` directly.** No copy into the service workspace; `nargo execute -p` allows unique input filenames.

**Integration:** the backend's `POST /api/proof` (currently `501 STUB`) will call `POST http://localhost:4002/prove` — planned for a later sub-stage of 7 or 8.

---

## 16. Instructions for a new assistant

**If you are starting a new chat:**

1. Read **section 0** first — especially 0.9 (document non-obvious), 0.10 (small steps), 0.11 (never delete info), 0.13 (record), 0.14 (record on push), 0.15 (checkpoint immediately after each sub-stage).
2. Read this file completely.
3. Read `docs/notes/00-glossary.md`, `00-zk-primer.md`, `01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md`, `05-backend.md`, `06-merkle.md`, `07-prover.md`, `08-pool-init.md`, `09-frontend.md`.
4. Last completed stage: **Stage 13.5** (sqlx upgrade, deny.toml, make exec-c).
5. Next task: **Stage 13.6** — final checkpoint for Stage 13.
6. **One task at a time.** Only exception: `git commit ... && git push`.
7. **Give files in full for new files; insertion point + block for existing.**
8. **Never guess.** If ambiguous — ask.
9. **Test in small steps.** 20 lines, not 200.
10. **Never delete information from existing files.**
11. **Do not narrate reasoning in chat.** State the task; explain rationale in the notes *after* the push.
12. Reply in **English** in chat. Files: English, except `docs/notes/*.md` and `docs/ru/*.md` (Russian).
13. **After each `git commit && git push` of a sub-stage:** create the checkpoint, then record in notes + context, then commit and push the record. Do not defer.
