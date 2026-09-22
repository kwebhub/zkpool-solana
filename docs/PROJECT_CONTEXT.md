# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-22 (after stage 1; architecture v3 approved)

---

## 1. What this project is

**zkpool-solana** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (four secrets), and can later withdraw SOL to any address without revealing the link between deposit and withdrawal.

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous versions:**
- [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1.
- `kwebhub/solana-zk-pool` — v2 (abandoned; bug: `withdraw` fails with `InvalidInstructionData`, root cause not found).

**This repository:** https://github.com/kwebhub/zkpool-solana (private) — v3.
**License:** MIT
**Network:** Solana Devnet

---

## 2. Why v3 exists — and what is different

In v2, `withdraw` failed with `InvalidInstructionData`. On-chain logs showed `Proof verification failed!`. Three days of debugging did not find the root cause. The class of bug: **mismatch between public inputs at one of the three layers**:

1. `circuits/withdrawal/src/main.nr` — order and types of `pub` inputs.
2. `onchain/.../instructions.rs::encode_public_inputs` — byte layout of the witness.
3. `web/.../useWithdraw.ts` — instruction data assembly.

**v3 goal is NOT to find the v2 bug. v3 goal is to build the project such that this class of bug cannot exist by construction.**

### Five principles of v3

1. **Single source of truth.** A `spec.json` defines public inputs, private inputs, and byte layouts. All three layers (`.nr`, Rust, TypeScript) are **generated from** or **validated against** it.
2. **Explicit contract checks at every boundary.** No "should work" — only byte-level comparison.
3. **No magic numbers.** All constants (`TREE_DEPTH`, `NR_PUBLIC_INPUTS`, `MIN_DEPOSIT_AMOUNT`, byte layouts) live in one place and are propagated to all languages.
4. **End-to-end localnet test before any devnet integration.** LiteSVM test: init pool → deposit → withdraw with real proof → verify SOL moved. Runs after every stage from Stage 4 onwards.
5. **Checkpoints with artifacts, not just code.** After each stage, save SHA-256 of every generated artifact to `.checkpoints/NN-name/`. On the next stage, compare hashes before proceeding.

See `docs/notes/00-checkpoints.md` for details.

---

## 3. Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) — port 5173                      │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — port 4001                                │
│ • /api/commitments  • /api/root  • /api/proof  • /api/withdraw   │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003               │
└──────────────┘   └──────────────┘   └──────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — port 4002                                 │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ MONITORING                                                       │
│ • Prometheus :9090  • Grafana :3000                              │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA                                                           │
│ • Program zk_pool:  (TBD after first deploy)                     │
│ • Verifier program: (TBD after first deploy)                     │
└──────────────────────────────────────────────────────────────────┘
```

### Services and ports

| Service | Port | Where |
|---------|------|-------|
| frontend | 5173 | host |
| backend | 4001 | `solana-zkpool-solana` container |
| prover | 4002 | `solana-zkpool-solana` container |
| merkle | 4003 | `solana-zkpool-solana` container |
| postgres | 5432 | Docker |
| redis | 6379 | Docker |
| prometheus | 9090 | Docker |
| grafana | 3000 | Docker |

---

## 4. Technologies

| Layer | Technology | Version |
|-------|-----------|---------|
| Smart contract | Anchor | 1.1.2 |
| ZK circuit | Noir / nargo | 1.0.0-rc.2 |
| ZK backend | Sunspot | 1.0.0 |
| ZK verification | gnark-solana | v2.0.0 |
| Backend | Rust + axum | 1.98.1 + 0.7 |
| Merkle service | Node.js + Fastify | 24.21.0 + 5.12.5 |
| Noir JS | @noir-lang/noir_js | 1.0.0-rc.2 |
| Prover | Rust + axum | 1.98.1 + 0.7 |
| Frontend | Vue 3 + Vite + Pinia | 3.5+ / 8.x / 4.x |
| Solana CLI | Agave | 3.1.10 |
| DB | PostgreSQL | 16 |
| Cache | Redis | 7 |
| Container | Docker | latest |
| Node.js | Node.js | 24.21.0 |
| pnpm | pnpm | 12.5.1 |

---

## 5. Checkpoint methodology

See `docs/notes/00-checkpoints.md` for full details.

**Rule:** after each stage, copy generated artifacts to `.checkpoints/NN-name/`, record SHA-256 in `manifest.txt`, record commit hash in `commit.txt`. On the next stage, compare hashes before proceeding.

**Key comparisons planned:**

| Artifact | Created in stage | Compared in stage |
|---|---|---|
| `hash2.json`, `hashes.json`, `withdrawal.json` | 2 | 3, 6, 7, 8 |
| `Prover.toml` (reference witness) | 2 | 3, 4, 7, 8 |
| `withdrawal.ccs`, `.pk`, `.vk`, `.so` | 3 | 4, 7 |
| `proof`, `public_witness` (local) | 3 | 7 |
| `encode_public_inputs` (172 bytes) | 4 | 7, 8 |
| `zk_pool.json` (IDL) | 4 | 8 (Codama client) |
| witness from `useWithdraw.ts` | 8 | must equal `Prover.toml` |
| Instruction data from `useWithdraw.ts` | 8 | must equal bytes built by `encode_public_inputs` |

---

## 6. What has been done

### ✅ Stage 0. Repository skeleton (2026-09-22)

- Repo created, cloned to `~/Projects/Solana/zkpool-solana`.
- Folder structure: `onchain/`, `circuits/`, `services/{backend,merkle,prover}`, `web/`, `infra/docker/`, `infra/grafana/`, `scripts/sync-circuits/`, `docs/notes/`, `.secrets/`, `.checkpoints/`.
- `.gitignore` — Rust, Node, Vue, Anchor, Env, keys, IDE, tests, monitoring, Noir, secrets, checkpoints.
- `.env.example` — placeholders.
- Commit: `8a41984f046a7c1deca7ed75493903de97df659a`.

### ✅ Stage 1. Docker environment (2026-09-22)

- `infra/docker-compose.yml` — service `solana-zkpool-solana`, explicit ports, 6 volumes.
- `infra/docker/Dockerfile.solana` — Ubuntu 24.04, Sunspot `.deb` + **git clone** of Sunspot (persistent), Solana CLI, noirup + nargo 1.0.0-rc.2, nvm + Node 24, pnpm 12.5.1.
- Commit: `94c18fe522e39822692fff9b9b22b2fe0f8e00d0`.

**Verified versions inside container:**
- rustc 1.98.1
- cargo 1.98.1
- solana-cli 3.1.10
- anchor-cli 1.1.2
- nargo 1.0.0-rc.2
- sunspot 1.0.0 (no `--version`, only `--help`)
- node v24.21.0
- pnpm 12.5.1

**Verified** Sunspot clone present: `~/sunspot/gnark-solana/crates/verifier-bin`.

### ✅ Docs (2026-09-22)

- `docs/notes/00-checkpoints.md` — checkpoint methodology (RU).
- `docs/PROJECT_CONTEXT.md` — this file (EN).
- `docs/notes/01-setup.md` — stage 1 notes (RU).
- `docs/notes/` removed from `.gitignore` — notes are committed now.

**Commits in order:**
- `e1fec4c` — un-ignore `docs/notes/`, add checkpoint methodology.
- `951f79d` — add `docs/PROJECT_CONTEXT.md`.
- `e55dcc6` — record `PROJECT_CONTEXT` commit hash.
- `d0f0f34` — add `docs/notes/01-setup.md`.
- `a92c3e3` — record stage 1 completion in `PROJECT_CONTEXT`.

**Checkpoint 1 artifacts:**
- `.checkpoints/01-setup/versions.txt` — tool versions.
- `.checkpoints/01-setup/sunspot-clone.txt` — Sunspot clone listing.
- `.checkpoints/01-setup/commit.txt` — `a92c3e34de520a7781fee0f592b72ac0ec9986b6`.
- `.checkpoints/01-setup/manifest.txt` — 7 tracked files + hashes.

---

## 7. Architecture decisions for v3 (approved 2026-09-22)

These decisions were made during discussion after Stage 1. They override the v2 approach where they differ.

### 7.1. Spec generation — YES

Create `circuits/withdrawal/spec.json` as **single source of truth** for:
- public inputs (name, type, size in bytes, order),
- private inputs (name, type),
- byte layout of the witness,
- artifact consumers (who consumes which JSON).

From this spec:
- **Validate** `withdrawal.nr` public inputs (order, count, types).
- **Generate** `encode_public_inputs` in Rust (or validate against existing).
- **Generate** TypeScript witness builder in `web/`.

**Rationale:** eliminates the class of bugs where the order of public inputs diverges between three layers.

### 7.2. LiteSVM E2E test — YES

Add `onchain/programs/zk_pool/tests/e2e_deposit_withdraw.rs` (or similar) that:
1. Initializes pool.
2. Deposits.
3. Generates proof **with real Sunspot** (or uses a precomputed proof from checkpoint).
4. Calls withdraw.
5. Verifies SOL moved.

Runs after Stage 4 and every subsequent stage that touches circuit, program, or encoding.

**Rationale:** v2 never ran a full local test; the bug only appeared in the browser on devnet.

### 7.3. One circuit with `recipient_binding` — YES

Unlike v2 (which had Stage 4.1a as a separate sub-stage), v3 includes `recipient_binding` **from the start**. Public inputs (5):
1. `root`
2. `nullifier_hash`
3. `recipient`
4. `recipient_binding`
5. `amount`

**Rationale:** uniform format across all layers from Stage 2 onwards; no mid-project format change.

### 7.4. Split deposit — DEFERRED to Stage 12

**Deferred feature:** split a single deposit into multiple unequal commitments (for privacy). Reason for deferral:
- Changes public input layout (N commitments, aggregated amount).
- Would obscure the root-cause search for `InvalidInstructionData` if introduced now.
- Requires changes at every layer.

**Architectural compatibility (to be preserved now):**
- `TREE_DEPTH` and other constants live in one place (spec).
- `PoolState` on-chain does not assume "1 deposit = 1 commitment" beyond what's needed.
- `commitments` table in Postgres does not have `UNIQUE` on `(pool, tx_signature)` or similar.
- `WithdrawEvent` includes explicit `amount` field.

### 7.5. `sync-circuits` — with SHA-256 verification

Unlike v2 (which just copied JSON files), v3 `scripts/sync-circuits` will:
1. Run `nargo compile` for each circuit.
2. Compute SHA-256 of source JSON (`circuits/*/target/*.json`).
3. Compute SHA-256 of each consumer copy (`services/merkle/circuits/`, `web/public/circuits/`) if present.
4. Compare:
   - Match → no-op.
   - Mismatch → copy + **print warning** about drift.
   - Missing → copy.
5. Write all hashes to `.checkpoints/02-circuits/manifest.txt`.

**Rationale:** v2's script silently overwrote files; drift could live in the repo for months.

### 7.6. Makefile — clean design (Stage 9)

Known issues in v2 Makefile to avoid:

| # | Issue | Fix in v3 |
|---|---|---|
| 1 | `sync-circuits` runs before `anchor build`, two independent artifact flows | explicit order; both artifacts checkpointed |
| 2 | `sunspot compile+setup+deploy` in one line | three separate targets, checkpoint after each |
| 3 | `restore-keypair` after `sync-circuits` | `restore-keypair` **first** |
| 4 | `deploy` without `anchor build` | `deploy` = `anchor build && anchor deploy` |
| 5 | `pnpm codama` without IDL freshness check | hash IDL in checkpoint |
| 6 | `sync-circuits` (JSON) and `pnpm codama` (IDL) are separate flows, no cross-check | explicit cross-check |
| 7 | `_start_backend` before `_wait_merkle` | `_start_merkle` → `_wait_merkle` → `_start_backend` |
| 8 | `_wait_postgres` uses `pg_isready` only | add `SELECT 1` check |

### 7.7. Stages list (v3, updated)

- Stage 0 — Repository skeleton ✅
- Stage 1 — Docker environment ✅
- **Stage 2.0 — `spec.json` (single source of truth)**
- **Stage 2.1 — generators/validators from spec (`.nr`, Rust, TS)**
- **Stage 2.2 — circuits (poseidon, hash2, hashes, withdrawal with binding)**
- **Stage 2.3 — `sync-circuits` with SHA-256 verification**
- **Stage 2.4 — checkpoint 2**
- Stage 3 — Sunspot verifier (compile, setup, deploy, local `verify`)
- Stage 4.1 — Anchor program (pool, deposit, withdraw with CPI)
- **Stage 4.5 — LiteSVM E2E test**
- Stage 5 — Backend
- Stage 6 — Merkle service
- Stage 7 — Prover
- Stage 8 — Frontend
- Stage 9 — Infrastructure (Makefile, Prometheus, Grafana, sync-circuits CLI)
- Stage 10 — Engineering processes
- Stage 11 — Security (threat model, tests)
- Stage 12 — Finalization (including split deposit if time permits)

---

## 8. Known pitfalls (from v2)

- `anchor init --name <name>` does not create a subdirectory — use `mkdir X && cd X && nargo init --name X`.
- `sunspot --version` is not supported — use `sunspot --help`.
- `sunspot deploy` requires `GNARK_VERIFIER_BIN` — Sunspot repo must be cloned. In v3, cloned in Dockerfile.
- `anchor build` only uses an existing keypair if present in `target/deploy/`. Restore from `.secrets/` before build to keep Program ID stable.
- `@solana/kit`, `@solana/program-client-core`, `@codama/*` — must be pinned to **exact** versions (no `^`).
- Codama path in `codama.json` must be `../onchain/target/idl/zk_pool.json` (from `web/`).
- `Prover.toml` is gitignored; regenerate via `nargo test test_generate_valid_inputs --show-output`.
- `bash -ic` is required for commands inside the container (`.bashrc` is not read by `bash -c`).
- `pnpm install` may bump `@solana/kit` to 8.x if versions use `^` — pin exact.

---

## 9. Repository structure

```
zkpool-solana/
├── .checkpoints/            ← gitignored
│   ├── README.md
│   ├── 00-init/
│   ├── 01-setup/
│   └── ...
├── .github/
│   ├── workflows/{ci,docker,release,security}.yml
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── dependabot.yml
├── .secrets/                ← gitignored
│   ├── zk_pool-keypair.json
│   ├── program_id
│   └── verifier_program_id
├── docs/
│   ├── notes/               ← Russian, committed
│   │   ├── 00-checkpoints.md
│   │   ├── 01-setup.md
│   │   ├── 02-circuits.md   (TBD)
│   │   └── ...
│   ├── ru/README.md         ← Russian
│   ├── PROJECT_CONTEXT.md   ← English (this file)
│   └── threat-model.md      ← English
├── infra/
│   ├── docker-compose.yml
│   ├── docker/Dockerfile.solana
│   ├── prometheus.yml
│   └── grafana/provisioning/
├── circuits/
│   ├── poseidon/            ← library: hash_1, hash_2, hash_3
│   ├── hash2/               ← circuit for external Poseidon2
│   ├── hashes/              ← commitment + nullifier_hash
│   └── withdrawal/          ← main circuit
│       ├── spec.json        ← single source of truth (Stage 2.0)
│       └── src/main.nr
├── onchain/
│   ├── Anchor.toml
│   └── programs/zk_pool/
│       ├── src/{lib,constants,error,events,state,instructions}.rs
│       └── tests/
├── services/
│   ├── backend/
│   ├── merkle/
│   └── prover/
├── web/
│   ├── public/circuits/
│   ├── src/generated/       ← Codama
│   └── src/services/{api,crypto,poseidon,storage}.ts
├── scripts/sync-circuits/   ← Rust CLI
├── solana/                  ← gitignored (keypairs)
├── .env.example
├── .gitignore
├── LICENSE
└── README.md
```

---

## 10. Git workflow

- Conventional commits.
- **All committed files in English**, except `docs/notes/*.md` and `docs/ru/README.md` (Russian).
- `.secrets/` and `.checkpoints/` — gitignored.
- `docs/notes/` — **not** gitignored, committed.
- After each stage: update `docs/PROJECT_CONTEXT.md` and add `docs/notes/NN-name.md`.

---

## 11. Commands (currently available)

```bash
# Enter container
cd ~/Projects/Solana/zkpool-solana
docker compose -f infra/docker-compose.yml exec solana bash

# One-shot command inside container
docker compose -f infra/docker-compose.yml exec solana bash -ic '<command>'
```

Full Makefile will arrive at Stage 9.

---

## 12. Current state

**Last completed stage:** Stage 1 (Docker environment verified).
**Next stage:** Stage 2.0 — create `circuits/withdrawal/spec.json`.

**Checkpoint 1:**
- Commit: `a92c3e34de520a7781fee0f592b72ac0ec9986b6`.
- Artifacts: `.checkpoints/01-setup/`.

---

## 13. Instructions for a new assistant

**If you are starting a new chat:**

1. Read this file completely.
2. Read `docs/notes/00-checkpoints.md` for checkpoint rules.
3. Read `docs/notes/01-setup.md` for Stage 1 details.
4. The last completed stage is **Stage 1**.
5. The next task is **Stage 2.0 — create `circuits/withdrawal/spec.json`**.
6. **One task at a time.** After each task:
   - Update `docs/PROJECT_CONTEXT.md` (this file).
   - Add or update `docs/notes/NN-name.md` (Russian).
   - Save artifacts to `.checkpoints/NN-name/` with SHA-256.
   - Commit and push.
7. **Do not jump ahead.** Do not implement Stage 3 before Stage 2 is complete and checkpointed.
8. **Do not skip checkpoint verification.** If hashes do not match — stop and investigate.
9. User's language: Russian. Reply in Russian. Files: English (except `docs/notes/*.md`, `docs/ru/README.md`).
10. **Never guess.** If something is ambiguous — ask the user before proceeding.
