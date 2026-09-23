# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-23 (after stage 3.0 — Solana devnet wallet created)

---

## 0. Rules for the assistant (READ FIRST)

### 0.1. One task at a time

The assistant gives **exactly one task** per message. Wait for the user to run it, paste the output, and only then give the next task.

**Do NOT** give multiple commands in one message. **Do NOT** chain "then do X, then do Y".

**Exception — git commit and git push are always grouped** as one task:
```bash
git commit -m "..." && git push
```
`git add -A` is still a **separate** task.

### 0.2. Files are given in full — ALWAYS

When the assistant asks the user to create or modify a file, it gives **the complete file contents**. Not a fragment, not "replace line N".

**Do NOT** say "replace this line". **DO** say "open file X and replace its entire contents with:".

### 0.3. Language

- Reply to the user **in Russian**.
- Files on disk: **English**, except `docs/notes/*.md` and `docs/ru/README.md` (Russian).

### 0.4. Never guess

If something is ambiguous — **ask the user before proceeding**.

### 0.5. Checkpoints are mandatory

After each stage, save artifacts to `.checkpoints/NN-name/` with SHA-256 in `manifest.txt` and the commit hash in `commit.txt`.

### 0.6. Update context after each stage

After each completed stage:
- Update `docs/PROJECT_CONTEXT.md` (this file).
- Add or update `docs/notes/NN-name.md` (Russian).
- Commit and push.

### 0.7. No multi-command chains (except git commit && git push)

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

## 2. Why v3 exists

In v2, `withdraw` failed with `InvalidInstructionData`. On-chain logs showed `Proof verification failed!`. Root cause not found in three days. The class of bug: **mismatch between public inputs at one of the three layers** (circuit, Anchor, frontend).

**v3 goal:** build the project such that this class of bug cannot exist by construction.

### Five principles

1. **Single source of truth** — `spec.json` defines layout; layers **validated against** it via `scripts/validate-spec`.
2. **Contract checks at every boundary** — byte-level, not "should work".
3. **No magic numbers** — all constants in one place.
4. **LiteSVM E2E test before devnet** — from Stage 4.5 onwards.
5. **Checkpoints with artifacts** — SHA-256, `.checkpoints/NN-name/`.

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
| Spec validator | Rust CLI | `scripts/validate-spec` |
| Circuit sync | Rust CLI | `scripts/sync-circuits` |
| DB | PostgreSQL | 16 |
| Cache | Redis | 7 |
| Container | Docker | latest |
| Node.js | Node.js | 24.21.0 |
| pnpm | pnpm | 12.5.1 |

---

## 5. Checkpoint methodology

See `docs/notes/00-checkpoints.md`.

**Rule:** after each stage, copy generated artifacts to `.checkpoints/NN-name/`, record SHA-256 in `manifest.txt`, record commit hash in `commit.txt`.

**Key comparisons planned:**

| Artifact | Created in stage | Compared in stage |
|---|---|---|
| `hash2.json`, `hashes.json`, `withdrawal.json` | 2 | 3, 6, 7, 8 |
| `Prover.toml` (reference witness) | 3 | 4, 7, 8 |
| `withdrawal.ccs`, `.pk`, `.vk`, `.so` | 3 | 4, 7 |
| `proof`, `public_witness` (local) | 3 | 7 |
| `encode_public_inputs` (172 bytes) | 4 | 7, 8 |
| `zk_pool.json` (IDL) | 4 | 8 (Codama client) |
| witness from `useWithdraw.ts` | 8 | must equal `Prover.toml` |
| Instruction data from `useWithdraw.ts` | 8 | must equal bytes built by `encode_public_inputs` |

---

## 6. What has been done

### ✅ Stage 0. Repository skeleton (2026-09-22)

- Repo at `~/Projects/Solana/zkpool-solana`.
- Structure: `onchain/`, `circuits/`, `services/`, `web/`, `infra/`, `scripts/`, `docs/`, `.secrets/`, `.checkpoints/`.
- `.gitignore` covers Rust, Node, Vue, Anchor, Env, keys, IDE, tests, monitoring, Noir, secrets, checkpoints, **circuit consumers**.
- Commit: `8a41984f046a7c1deca7ed75493903de97df659a`.

### ✅ Stage 1. Docker environment (2026-09-22)

- `infra/docker-compose.yml`, `infra/docker/Dockerfile.solana`.
- Sunspot cloned in Dockerfile (persistent).
- Commit: `94c18fe522e39822692fff9b9b22b2fe0f8e00d0`.

**Versions inside container:** rustc 1.98.1, cargo 1.98.1, solana-cli 3.1.10, anchor-cli 1.1.2, nargo 1.0.0-rc.2, sunspot 1.0.0, node v24.21.0, pnpm 12.5.1.

### ✅ Stage 2. Circuits on Noir (2026-09-22 — 2026-09-23)

- **2.0** — `circuits/withdrawal/spec.json`. Commit: `6df5fa5`.
- **2.1.1** — `scripts/validate-spec/` skeleton. Commit: `e479137`.
- **2.1.2** — 15 validation rules. Commit: `dbb4365`.
- **2.2.1** — `circuits/poseidon/` (11 tests). Commit: `baade07`.
- **2.2.2** — `circuits/hash2/` (5 tests), `circuits/hashes/` (9 tests). Commit: `8354921`.
- **2.2.3** — `circuits/withdrawal/` (`main.nr` + `merkle_tree.nr`, 16 tests). Commit: `90afe4c`.
- **2.3** — `scripts/sync-circuits/` with `check`/`apply` modes. Commit: `aa5d8d9`.
- **2.4** — final checkpoint. Commit: `e0a3725`.

**Circuit ACIRs (all in sync with consumers):**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050`

**Tests:** 41 total (11 + 5 + 9 + 16).

### ✅ Stage 3.0. Solana devnet wallet (2026-09-23)

- **Address:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`
- **Path inside container:** `/home/ubuntu/.config/solana/id.json`
- **Host volume:** `~/Projects/Solana/zkpool-solana/solana/`
- **RPC:** devnet
- **Balance:** 5 SOL
- **Created with:** `solana-keygen new --no-bip39-passphrase`
- **Seed phrase:** stored offline by user (12 words). Not committed.

**Note:** `solana/` volume was initially root-owned inside the container. Fixed with `sudo chown -R 1000:1000 solana/` on host.

### ✅ Docs

- `docs/notes/00-checkpoints.md`, `01-setup.md`, `02-circuits.md`.
- `docs/PROJECT_CONTEXT.md` — this file.
- `docs/notes/` is committed (removed from `.gitignore`).

**All commits in order:**
`8a41984`, `94c18fe`, `e1fec4c`, `951f79d`, `e55dcc6`, `d0f0f34`, `a92c3e3`, `0bfc5a4`, `6df5fa5`, `b61a20a`, `a43fea4`, `e479137`, `dbb4365`, `3222231`, `baade07`, `ce45bb5`, `a508a46`, `8354921`, `90afe4c`, `4b938f2`, `aa5d8d9`, `404b5f4`, `e0a3725`.

**Checkpoints:** `01-setup`, `02.1-validate-spec`, `02.2.1-poseidon`, `02.2.2-hash2-hashes`, `02.2.3-withdrawal`, `02.3-sync-circuits`, `02.4-stage-2-final`.

---

## 7. Architecture decisions

### 7.1. Spec validation — YES (not generation)

`spec.json` is the source of truth. Files **validated against** it. Generation of `.nr` and Rust is fragile; validation is simpler and catches the same class of bugs.

### 7.2. LiteSVM E2E test — YES

`onchain/programs/zk_pool/tests/e2e_deposit_withdraw.rs`: init pool → deposit → generate proof with real Sunspot → withdraw → verify SOL moved. Runs after Stage 4.5.

### 7.3. One circuit with `recipient_binding` — YES

Public inputs (5): `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`.

### 7.4. Zero-padding (no domain separation) — CONFIRMED

`hash_1(x) == hash_2(x, 0)`, `hash_2(x, y) == hash_3(x, y, 0)`. Documented. Safe for zkpool-solana.

**If domain separation is ever needed:** see v2 notes; large-scale work.

### 7.5. `sync-circuits` — check/apply modes

`--check`: mismatch → error, exit 1, no copy. `--apply`: mismatch → copy + warning, exit 0. First run (consumer missing) always copies.

### 7.6. Split deposit — DEFERRED to Stage 12

### 7.7. Makefile — clean design (Stage 9)

### 7.8. Stages list (v3)

- Stage 0 — Repository skeleton ✅
- Stage 1 — Docker environment ✅
- Stage 2 — Circuits on Noir ✅
- **Stage 3 — Sunspot verifier** 🚧
  - 3.0 — Wallet ✅
  - 3.1 — `sunspot compile` (next)
  - 3.2 — `sunspot setup`
  - 3.3 — `sunspot deploy`
  - 3.4 — `solana program deploy`
  - 3.5 — Local verify (proof + verify)
  - 3.6 — Checkpoint 3
- Stage 4.1 — Anchor program
- Stage 4.5 — LiteSVM E2E
- Stage 5 — Backend
- Stage 6 — Merkle service
- Stage 7 — Prover
- Stage 8 — Frontend
- Stage 9 — Infrastructure
- Stage 10 — Engineering processes
- Stage 11 — Security
- Stage 12 — Finalization

---

## 8. Known pitfalls

- `anchor init --name <name>` does not create a subdirectory.
- `sunspot --version` is not supported.
- `sunspot deploy` requires `GNARK_VERIFIER_BIN` — Sunspot cloned in Dockerfile.
- `anchor build` uses existing keypair only if in `target/deploy/`.
- `@solana/kit`, `@codama/*` — pinned exact versions.
- Codama path: `../onchain/target/idl/zk_pool.json` (from `web/`).
- `Prover.toml` is gitignored.
- `bash -ic` required inside the container.
- `poseidon2_permutation` in nargo 1.0.0-rc.2 takes **one** argument.
- Zero-padding causes arity collisions — documented.
- `main()` in `type = "bin"` circuits must have `pub` on return type.
- Field values must be **< 2^254**.
- Noir does not support **float literals** (`0.001 as Field` fails). Use lamports.
- **`solana/` volume may be root-owned** — fix with `sudo chown -R 1000:1000 solana/`.

---

## 9. Repository structure

```
zkpool-solana/
├── .checkpoints/            ← gitignored
├── .secrets/                ← gitignored
├── docs/{notes,ru,PROJECT_CONTEXT.md,threat-model.md}
├── infra/{docker-compose.yml,docker/Dockerfile.solana}
├── circuits/{poseidon,hash2,hashes,withdrawal}/
├── onchain/
├── services/{backend,merkle,prover}/
├── web/
├── scripts/{validate-spec,sync-circuits}/
├── solana/                  ← gitignored, wallet
├── .env.example
└── .gitignore
```

---

## 10. Git workflow

- Conventional commits.
- English files except `docs/notes/*.md`, `docs/ru/README.md`.
- Gitignored: `.secrets/`, `.checkpoints/`, `solana/`, `services/merkle/circuits/`, `web/public/circuits/`.
- Committed: `docs/notes/`.
- `git commit ... && git push` — one task. `git add -A` — separate.

---

## 11. Commands

```bash
# Enter container
docker compose -f infra/docker-compose.yml exec solana bash

# validate-spec
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'

# sync-circuits --check
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml --release -- check'

# nargo test (any circuit)
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/<name> && nargo test'
```

Full Makefile arrives at Stage 9.

---

## 12. Current state

**Last completed stage:** Stage 3.0 (wallet).
**Next stage:** Stage 3.1 — `sunspot compile`.

**Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (5 SOL, devnet).

**Circuit ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050`

---

## 13. Instructions for a new assistant

1. Read **section 0**.
2. Read this file completely.
3. Read `docs/notes/00-checkpoints.md`, `01-setup.md`, `02-circuits.md`.
4. Last completed stage: **Stage 3.0**.
5. Next task: **Stage 3.1 — `sunspot compile`**.
6. **One task at a time.** Only exception: `git commit ... && git push`.
7. **Give files in full.**
8. **Never guess.**
9. Reply in Russian. Files: English (except notes and `docs/ru/README.md`).
