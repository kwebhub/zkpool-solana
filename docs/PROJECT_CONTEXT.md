# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-23 (after stage 2.2.3 — withdrawal circuit)

---

## 0. Rules for the assistant (READ FIRST)

These rules were established by the user after past sessions where ambiguity and partial file snippets caused three days of debugging to be lost.

### 0.1. One task at a time

The assistant gives **exactly one task** per message. Wait for the user to run it, paste the output, and only then give the next task.

**Do NOT** give multiple commands in one message. **Do NOT** chain "then do X, then do Y".

**Exception — git commit and git push are always grouped** as one task:
```bash
git commit -m "..." && git push
```
`git add -A` is still a **separate** task (so the user can verify what is staged).

### 0.2. Files are given in full — ALWAYS

When the assistant asks the user to create or modify a file, it gives **the complete file contents**. Not a fragment, not "replace line N", not "add this block after that block".

**Do NOT** say "replace this line". **DO** say "open file X and replace its entire contents with:".

### 0.3. Language

- Reply to the user **in Russian**.
- Files on disk: **English**, except `docs/notes/*.md` and `docs/ru/README.md` (Russian).

### 0.4. Never guess

If something is ambiguous — **ask the user before proceeding**. Do not invent answers, do not fill gaps with assumptions.

### 0.5. Checkpoints are mandatory

After each stage, save artifacts to `.checkpoints/NN-name/` with SHA-256 in `manifest.txt` and the commit hash in `commit.txt`. On the next stage, compare hashes before proceeding. If hashes do not match — **stop and investigate**.

### 0.6. Update context after each stage

After each completed stage:
- Update `docs/PROJECT_CONTEXT.md` (this file).
- Add or update `docs/notes/NN-name.md` (Russian).
- Commit and push.

### 0.7. No multi-command chains (except git commit && git push)

Do not combine unrelated commands into one task. The only exception is `git commit ... && git push`.

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

1. **Single source of truth.** A `spec.json` defines public inputs, private inputs, and byte layouts. All three layers (`.nr`, Rust, TypeScript) are **validated against** it via `scripts/validate-spec`.
2. **Explicit contract checks at every boundary.** No "should work" — only byte-level comparison.
3. **No magic numbers.** All constants live in one place and are propagated to all languages.
4. **End-to-end localnet test before any devnet integration.** LiteSVM test: init pool → deposit → withdraw with real proof → verify SOL moved. Runs after every stage from Stage 4 onwards.
5. **Checkpoints with artifacts, not just code.** After each stage, save SHA-256 of every generated artifact to `.checkpoints/NN-name/`.

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
| Spec validator | Rust CLI | in-repo (`scripts/validate-spec`) |
| Circuit sync | Rust CLI | in-repo (`scripts/sync-circuits`) — stage 2.3 |
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
- Folder structure: `onchain/`, `circuits/`, `services/{backend,merkle,prover}`, `web/`, `infra/docker/`, `infra/grafana/`, `scripts/{sync-circuits,validate-spec}/`, `docs/notes/`, `.secrets/`, `.checkpoints/`.
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

### ✅ Stage 2.0. `spec.json` — single source of truth (2026-09-22)

- `circuits/withdrawal/spec.json` — full specification.
- **Public inputs (5):** `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`.
- **Private inputs (5):** `nullifier`, `secret`, `note_secret`, `merkle_proof[20]`, `is_even[20]`.
- **Constraints:** C1, C2, C3.
- **Witness layout:** 12-byte header + 5×32 bytes = **172 bytes**.
- Commit: `6df5fa5`.

### ✅ Stage 2.1. `validate-spec` — Rust CLI (2026-09-23)

- **2.1.1 — skeleton:** `scripts/validate-spec/` — `Cargo.toml`, `Cargo.lock`, `src/{main,project,spec,rules}.rs`. Loads `spec.json`, prints summary. Commit: `e479137`.
- **2.1.2 — 15 rules for spec:** `rules::validate_all` covering version, circuit name, tree depth, nr_public_inputs, unique names, bytes sums (raw 136 / witness slot 160), bytes-vs-type, witness total, header, public section, constraints, hash functions, artifacts, consumers, checkpoints. First run caught its own bug (`rule_public_inputs_bytes_sum` compared raw bytes to witness slot expectation). Fixed: rule now checks both sums. Commit: `dbb4365`.

### ✅ Stage 2.2.1. `poseidon` library (2026-09-23)

- `circuits/poseidon/` — Noir library (`type = "lib"`).
- `src/lib.nr` — 151 lines: `hash_1`, `hash_2`, `hash_3` + 11 tests.
- `poseidon2_permutation` signature in nargo 1.0.0-rc.2 is **single-argument** — `poseidon2_permutation(state)`, not `(state, 4)`.
- **Zero-padding** for t=4, r=3, c=1: 1, 2, or 3 zeros depending on arity.
- **Domain separation — NOT present.** Zero-padding causes `hash_1(x) == hash_2(x, 0)` and `hash_2(x, y) == hash_3(x, y, 0)`. Documented as safe for zkpool-solana (we never hash the same value with two different arities in the same context). Two tests **assert** the collision as documented property; one test asserts non-zero extra input differs.
- Commit: `baade07`.

### ✅ Stage 2.2.2. `hash2` and `hashes` circuits (2026-09-23)

- `circuits/hash2/` — 5 tests. ACIR: `hash2.json`, SHA-256 `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`.
- `circuits/hashes/` — 9 tests. ACIR: `hashes.json`, SHA-256 `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`.
- `hashes` `main()` must use `-> pub (Field, Field)` — Noir requires `pub` on entry-point return type.
- Commit: `8354921`.

### ✅ Stage 2.2.3. `withdrawal` circuit (2026-09-23)

- `circuits/withdrawal/Nargo.toml` — `type = "bin"`, depends on `poseidon`.
- `circuits/withdrawal/src/merkle_tree.nr` — `compute_merkle_root<let DEPTH: u32>(leaf, path, is_even) -> Field` + 8 tests.
- `circuits/withdrawal/src/main.nr` — full circuit with `global TREE_DEPTH: u32 = 20`, 5 public inputs, 5 private inputs, constraints C1, C2, C3 + 8 tests.
- **16 tests total** (8 in `merkle_tree`, 8 in `main`).
- ACIR: `withdrawal.json`, **42 808 bytes**, SHA-256 `f154aca08c9a5872aec5cdd230036652bf7a87af1847fe4ff299e2b4bd932050`.
- Commit: `90afe4c`.

### ✅ Docs (2026-09-22 — 2026-09-23)

- `docs/notes/00-checkpoints.md` — checkpoint methodology (RU).
- `docs/PROJECT_CONTEXT.md` — this file (EN).
- `docs/notes/01-setup.md` — stage 1 notes (RU).
- `docs/notes/02-circuits.md` — stage 2 notes (RU).
- `docs/notes/` removed from `.gitignore` — notes are committed.

**Commits in order:**
- `e1fec4c` — un-ignore `docs/notes/`, add checkpoint methodology.
- `951f79d` — add `docs/PROJECT_CONTEXT.md`.
- `e55dcc6` — record `PROJECT_CONTEXT` commit hash.
- `d0f0f34` — add `docs/notes/01-setup.md`.
- `a92c3e3` — record stage 1 completion in `PROJECT_CONTEXT`.
- `0bfc5a4` — update `PROJECT_CONTEXT` with v3 architecture decisions.
- `6df5fa5` — add `circuits/withdrawal/spec.json`.
- `b61a20a` — add `docs/notes/02-circuits.md`.
- `a43fea4` — mark stage 2.0 as completed in `PROJECT_CONTEXT`.
- `e479137` — add `validate-spec` skeleton.
- `dbb4365` — add spec validation rules (15 rules).
- `3222231` — update stage 2 notes with validate-spec.
- `baade07` — add poseidon library with 11 tests.
- `ce45bb5` — update stage 2 notes with poseidon library and domain separation.
- `a508a46` — update `PROJECT_CONTEXT` with rules and domain separation notes.
- `8354921` — add hash2 and hashes circuits (14 tests).
- `90afe4c` — add withdrawal circuit with merkle_tree module (16 tests).

**Checkpoint artifacts:**
- `.checkpoints/01-setup/` — versions, sunspot-clone, commit, manifest.
- `.checkpoints/02.1-validate-spec/` — commit, validate-output.
- `.checkpoints/02.2.1-poseidon/` — commit, nargo-test-output.
- `.checkpoints/02.2.2-hash2-hashes/` — commit, hash2/hash2.json (+sha256, tests), hashes/hashes.json (+sha256, tests).
- `.checkpoints/02.2.3-withdrawal/` — commit-before, commit, nargo-test-output, withdrawal.json (+sha256).

---

## 7. Architecture decisions for v3

### 7.1. Spec validation — YES (not generation)

`spec.json` is the source of truth. Instead of generating `.nr`, Rust, TS from it, we **validate** existing files against it via `scripts/validate-spec`.

**Rationale:** generation of `.nr` and Rust is fragile; validation is simpler and catches the same class of bugs. Runs manually, from Makefile (Stage 9), from CI (Stage 10).

### 7.2. LiteSVM E2E test — YES

Add `onchain/programs/zk_pool/tests/e2e_deposit_withdraw.rs` that: init pool → deposit → generate proof with real Sunspot → withdraw → verify SOL moved. Runs after Stage 4 and every subsequent stage that touches circuit, program, or encoding.

### 7.3. One circuit with `recipient_binding` — YES

Public inputs (5): `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`. No mid-project format change.

### 7.4. Zero-padding (no domain separation) — CONFIRMED

Follows v2. Documented in `circuits/poseidon/src/lib.nr` and in `spec.json` (`hash_functions.hash_4.status == "forbidden"`).

**If domain separation is ever needed:**
1. Add tags to `poseidon/src/lib.nr`: `hash_1(in) → [in, 1, 0, 0]`, `hash_2(in1,in2) → [in1, in2, 2, 0]`, `hash_3(in1,in2,in3) → [in1, in2, in3, 3]`.
2. Update `spec.json` `hash_functions`.
3. Recompile **all** dependent circuits.
4. Rebuild Sunspot artifacts (ACIR, CCS, PK, VK, verifier .so — Program ID **will change**).
5. Re-sync `web/.env` via `sync-program-id`.
6. Recompute all checkpoints.

This is large-scale work — do only when explicitly needed.

### 7.5. `sync-circuits` — with SHA-256 verification and check/apply modes

Unlike v2 (which silently copied JSON files), v3 `scripts/sync-circuits` has **two modes**:

**`--check` (default, used in CI):**
1. Run `nargo compile` for each circuit.
2. Compute SHA-256 of source JSON (`circuits/*/target/*.json`).
3. Compute SHA-256 of each consumer copy (`services/merkle/circuits/`, `web/public/circuits/`) if present.
4. Compare per file:
   - **Match** → no-op.
   - **Missing consumer** → copy (first run — safe).
   - **Mismatch** → **error**, exit non-zero, do NOT copy.
5. Write all hashes to `.checkpoints/02.3-sync-circuits/manifest.txt`.

**`--apply` (used locally after editing circuits):**
Same, but on mismatch: **copy** from source to consumer, print warning, exit 0.

**Rationale:**
- In CI, `--check` catches drift between source and consumers — the exact class of bug that caused v2's three-day hunt.
- Silent overwriting on mismatch is dangerous: it can hide a real problem (e.g. someone reverted `circuits/` but consumers remained newer).
- On first run (consumer missing), copying is always safe.

**Hard-coded circuits list (no reading from spec.json):**

```rust
const CIRCUITS: &[(&str, &str, &str)] = &[
    ("hash2",      "circuits/hash2",      "hash2.json"),
    ("hashes",     "circuits/hashes",     "hashes.json"),
    ("withdrawal", "circuits/withdrawal", "withdrawal.json"),
];

const DESTINATIONS: &[&str] = &[
    "services/merkle/circuits",
    "web/public/circuits",
];
```

**Why hard-coded:** the list changes once a year; extending `spec.json` for `hash2` and `hashes` (which have no public inputs or constraints) would be artificial.

### 7.6. Split deposit — DEFERRED to Stage 12

**Architectural compatibility (to be preserved now):**
- `TREE_DEPTH` and other constants live in one place (spec).
- `PoolState` on-chain does not assume "1 deposit = 1 commitment" beyond what's needed.
- `commitments` table in Postgres has no `UNIQUE` on `(pool, tx_signature)`.
- `WithdrawEvent` includes explicit `amount` field.

### 7.7. Makefile — clean design (Stage 9)

Known issues in v2 Makefile to avoid:

| # | Issue | Fix in v3 |
|---|---|---|
| 1 | `sync-circuits` runs before `anchor build` | explicit order; both checkpointed |
| 2 | `sunspot compile+setup+deploy` in one line | three separate targets, checkpoint after each |
| 3 | `restore-keypair` after `sync-circuits` | `restore-keypair` **first** |
| 4 | `deploy` without `anchor build` | `deploy` = `anchor build && anchor deploy` |
| 5 | `pnpm codama` without IDL freshness check | hash IDL in checkpoint |
| 6 | `sync-circuits` and `pnpm codama` are separate flows, no cross-check | explicit cross-check |
| 7 | `_start_backend` before `_wait_merkle` | `_start_merkle` → `_wait_merkle` → `_start_backend` |
| 8 | `_wait_postgres` uses `pg_isready` only | add `SELECT 1` check |

### 7.8. Stages list (v3)

- Stage 0 — Repository skeleton ✅
- Stage 1 — Docker environment ✅
- Stage 2.0 — `spec.json` ✅
- Stage 2.1 — `validate-spec` ✅
  - 2.1.1 — skeleton ✅
  - 2.1.2 — spec-only rules ✅
  - 2.1.3 — `.nr` rules (after 2.2.3)
  - 2.1.4 — Rust rules (after 4.1)
  - 2.1.5 — TS rules (after 8)
- Stage 2.2 — circuits ✅
  - 2.2.1 — `poseidon` library ✅
  - 2.2.2 — `hash2` and `hashes` circuits ✅
  - 2.2.3 — `withdrawal` circuit ✅
- **Stage 2.3 — `sync-circuits` with SHA-256 (next)**
- Stage 2.4 — checkpoint 2
- Stage 3 — Sunspot verifier
- Stage 4.1 — Anchor program
- Stage 4.5 — LiteSVM E2E test
- Stage 5 — Backend
- Stage 6 — Merkle service
- Stage 7 — Prover
- Stage 8 — Frontend
- Stage 9 — Infrastructure (Makefile, Prometheus, Grafana)
- Stage 10 — Engineering processes
- Stage 11 — Security
- Stage 12 — Finalization

---

## 8. Known pitfalls (from v2)

- `anchor init --name <name>` does not create a subdirectory — use `mkdir X && cd X && nargo init --name X`.
- `sunspot --version` is not supported — use `sunspot --help`.
- `sunspot deploy` requires `GNARK_VERIFIER_BIN` — Sunspot repo must be cloned. In v3, cloned in Dockerfile.
- `anchor build` only uses an existing keypair if present in `target/deploy/`. Restore from `.secrets/` before build.
- `@solana/kit`, `@solana/program-client-core`, `@codama/*` — must be pinned to **exact** versions (no `^`).
- Codama path in `codama.json` must be `../onchain/target/idl/zk_pool.json` (from `web/`).
- `Prover.toml` is gitignored; regenerate via `nargo test test_generate_valid_inputs --show-output`.
- `bash -ic` is required for commands inside the container.
- `pnpm install` may bump `@solana/kit` to 8.x if versions use `^` — pin exact.
- `jsonls` warns about `$schema: "internal://..."` — do not add `$schema` to `spec.json`.
- **`poseidon2_permutation` in nargo 1.0.0-rc.2 takes ONE argument** (the state array), not two.
- **Zero-padding causes arity collisions** (`hash_1(x) == hash_2(x, 0)`) — documented, safe for our use.
- **`main()` in `type = "bin"` circuits must have `pub` on the return type.** Noir requires this for verifier access.
- **Field values must be < 2^254** (BN254 prime). A 256-bit literal (e.g. arbitrary Solana Pubkey) will fail to compile. Reduce modulo BN254 before use.

---

## 9. Repository structure

```
zkpool-solana/
├── .checkpoints/            ← gitignored
├── .github/
├── .secrets/                ← gitignored
├── docs/
│   ├── notes/               ← Russian, committed
│   ├── ru/README.md         ← Russian
│   ├── PROJECT_CONTEXT.md   ← English (this file)
│   └── threat-model.md      ← English
├── infra/
├── circuits/
│   ├── poseidon/            ← library ✅
│   ├── hash2/               ← circuit ✅
│   ├── hashes/              ← circuit ✅
│   └── withdrawal/
│       ├── spec.json        ← single source of truth ✅
│       └── src/
│           ├── main.nr      ← circuit ✅
│           └── merkle_tree.nr ← library ✅
├── onchain/
├── services/{backend,merkle,prover}/
├── web/
├── scripts/
│   ├── validate-spec/       ✅
│   └── sync-circuits/       ← (stage 2.3, next)
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
- `docs/notes/` — committed.
- After each stage: update `PROJECT_CONTEXT.md`, add `docs/notes/NN-name.md`.
- `git commit ... && git push` — one task. `git add -A` — separate task.

---

## 11. Commands

```bash
# Enter container
cd ~/Projects/Solana/zkpool-solana
docker compose -f infra/docker-compose.yml exec solana bash

# Run validate-spec
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'

# Run poseidon tests
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/poseidon && nargo test'

# Run hash2 tests
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/hash2 && nargo test'

# Run hashes tests
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/hashes && nargo test'

# Run withdrawal tests
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu/circuits/withdrawal && nargo test'
```

Full Makefile will arrive at Stage 9.

---

## 12. Current state

**Last completed stage:** Stage 2.2.3 (withdrawal circuit).
**Next stage:** Stage 2.3 — `sync-circuits` with SHA-256 (check/apply modes).

**Checkpoints saved:**
- `.checkpoints/01-setup/`
- `.checkpoints/02.1-validate-spec/`
- `.checkpoints/02.2.1-poseidon/`
- `.checkpoints/02.2.2-hash2-hashes/`
- `.checkpoints/02.2.3-withdrawal/`

---

## 13. Instructions for a new assistant

**If you are starting a new chat:**

1. Read **section 0** first — those are the hard rules.
2. Read this file completely.
3. Read `docs/notes/00-checkpoints.md`, `docs/notes/01-setup.md`, `docs/notes/02-circuits.md`.
4. Last completed stage: **Stage 2.2.3**.
5. Next task: **Stage 2.3 — `sync-circuits`**.
6. **One task at a time. Do not chain commands.** Only exception: `git commit ... && git push`.
7. **Give files in full.** Never fragments, never "replace line N".
8. **Never guess.** If ambiguous — ask the user before proceeding.
9. Reply in Russian. Files: English (except `docs/notes/*.md`, `docs/ru/README.md`).
