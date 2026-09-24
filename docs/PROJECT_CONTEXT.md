# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-24 (after stage 4.5.3 — test_pool.rs)

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

When the assistant asks the user to create or modify a file, it gives **the complete file contents**. Not a fragment, not "replace line N", not "find section X".

**Do NOT** say "replace this section". **DO** say "open file X and replace its entire contents with:".

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

### 0.8. Documentation is portfolio material — see section 14

**CRITICAL:** `docs/notes/*.md` is not just internal memory. It is the raw material for guides, tutorials, and articles on **Medium** and **Mirror.xyz**, and part of the GitHub portfolio. See section 14 for the full policy.

### 0.9. Non-obvious project actions must be documented

Any **non-obvious** action required to complete a stage must be:
1. Recorded in `PROJECT_CONTEXT.md`, section 8 "Known pitfalls".
2. Included in `docs/notes/NN-name.md` as a **lesson** with symptom, cause, and fix.

The user explicitly requested this (2026-09-24): "все эти неочевидные для проекта действия нужно фиксировать".

Examples:
- Manual reading of crate sources to check exact API.
- Adding packages to `workspace.exclude`.
- Running `cargo fetch` in a sub-crate.
- Any unusual CLI flag or environment variable.

### 0.10. Test in small steps — do not write 200 lines at once

When working with a **new** library (like LiteSVM), **do not** write a large file in one shot. Write **20 lines**, compile, verify the API matches. Only then expand.

**Rationale:** LiteSVM (and other Solana crates) change API between minor versions. A 200-line file may produce 50 compile errors, all mixed. A 20-line file produces 1–2 errors, easy to isolate.

**How to check exact API:** read the crate sources in `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`. Look at `pub use` / `pub fn` / `pub struct` lines.

### 0.11. Updating existing files — never delete information

When the assistant updates an existing file in `docs/notes/` or `docs/PROJECT_CONTEXT.md`:

1. **Take the current file content** (the user pastes it, or the assistant has it from the chat history).
2. **Preserve it in full** — no information is removed.
3. **Add** the new sections in the appropriate place.
4. **Update** the glossary if new terms appeared.
5. **Give back the full text** — old content + additions.

**Allowed:** rephrase, reorder sections, improve wording — as long as the **information is preserved**.

**Forbidden:** remove, shorten, "simplify", or replace existing sections with a summary.

The user explicitly requested this (2026-09-24): "к существующему тексту добавляешь описание своих действий, ошибок и их решений, объяснений почему, если нужно дополняешь глоссарий и после этого даёшь мне полный текст файла с учетом того что в нём было и добавлений".

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

**Demo notice:** this is a demo / educational project. See `docs/DEMO-NOTICE.md`.

---

## 2. Why v3 exists

In v2, `withdraw` failed with `InvalidInstructionData`. Root cause not found in three days. The class of bug: **mismatch between public inputs** (circuit, Anchor, frontend).

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
│ MONITORING — Prometheus :9090, Grafana :3000                     │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA                                                           │
│ • Wallet:            5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc │
│ • Verifier program:  5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ │
│ • zk_pool program:   8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm │
└──────────────────────────────────────────────────────────────────┘
```

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
| Frontend | Vue 3 + Vite + Pinia | 3.5+ / 8.x / 4.x |
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

See `docs/notes/00-checkpoints.md`.

**Rule:** after each stage, copy generated artifacts to `.checkpoints/NN-name/`, record SHA-256 in `manifest.txt`, record commit hash in `commit.txt`.

---

## 6. What has been done

### ✅ Stage 0. Repository skeleton (2026-09-22)

Commit: `8a41984f046a7c1deca7ed75493903de97df659a`.

### ✅ Stage 1. Docker environment (2026-09-22)

- `infra/docker-compose.yml`, `infra/docker/Dockerfile.solana`.
- Sunspot cloned in Dockerfile (persistent).
- Commit: `94c18fe522e39822692fff9b9b22b2fe0f8e00d0`.

### ✅ Stage 2. Circuits on Noir (2026-09-22 — 2026-09-23)

- **2.0** — `spec.json`. Commit: `6df5fa5`.
- **2.1** — `validate-spec` (15 rules). Commits: `e479137`, `dbb4365`.
- **2.2.1** — `poseidon` (11 tests). Commit: `baade07`.
- **2.2.2** — `hash2` (5 tests), `hashes` (9 tests). Commit: `8354921`.
- **2.2.3** — `withdrawal` + `merkle_tree` (16 tests). Commit: `90afe4c`.
- **2.3** — `sync-circuits`. Commit: `aa5d8d9`.
- **2.4** — final checkpoint. Commit: `e0a3725`.

### ✅ Stage 3. Sunspot verifier (2026-09-23 — 2026-09-24)

- **3.0** — Wallet. Commit: `e9ddbe5`.
- **3.1** — `.ccs` (642 177 B).
- **3.2** — `.pk` (2 145 109 B) + `.vk` (972 B).
- **3.3** — `.so` (87 312 B), verifier `5t51iu6a...`.
- **3.4** — deployed to devnet, balance 0.444 SOL.
- **3.5** — local verification: `✅ Verification successful!`.
- **3.6** — final checkpoint. Commit: `441b9d2`.

### ✅ Stage 3.7. Documentation enrichment (2026-09-24)

- `00-glossary.md` — 545 lines.
- `00-zk-primer.md` — 317 lines.
- `01-setup.md` — 762 lines.
- `02-circuits.md` — 957 lines.
- `03-sunspot.md` — 996 lines.

### ✅ Stage 4.1. Anchor program (2026-09-24)

All 9 sub-stages complete:

| # | Sub-stage | Commit |
|---|---|---|
| 4.1.1 | Anchor workspace | `b8f6fdf` |
| 4.1.2 | `constants.rs` | `23c4dd9` |
| 4.1.3 | `error.rs`, `events.rs`, `state.rs` | `c45cb75` |
| 4.1.4 | `encoding.rs` | `dc0fb5e` |
| 4.1.5 | `pool` instruction | `553634e` |
| 4.1.6 | `deposit` instruction | `5455d04` |
| 4.1.7 | `withdraw` instruction | `ce71a47` |
| 4.1.8 | Tests (37 unit) | `8261530` |
| 4.1.9 | Deploy to devnet | `3f6dcd6` |

**Deployed program:** `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` (210 000 B, rent 1.068 SOL).

### 🚧 Stage 4.5. LiteSVM E2E test (in progress)

| # | Sub-stage | Status |
|---|---|---|
| 4.5.1 | Tests reorganization (move out of `onchain/`) | ✅ commit `050e58f` |
| 4.5.2 | `helpers.rs` (load both programs) | ✅ commit `cd01255` |
| 4.5.3 | `test_pool.rs` (airdrop + pool instruction) | ✅ commit `59cf81e` |
| 4.5.4 | `test_deposit.rs` | ← next |
| 4.5.5 | `test_withdraw.rs` | ⏳ |
| 4.5.6 | `test_double_spend.rs` | ⏳ |
| 4.5.7 | Final checkpoint | ⏳ |

### ✅ Docs (2026-09-22 — 2026-09-24)

- `docs/notes/00-checkpoints.md`, `00-glossary.md`, `00-zk-primer.md`.
- `docs/notes/01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md` (1014 lines).
- `docs/DEMO-NOTICE.md`.
- `docs/PROJECT_CONTEXT.md` — this file.
- `docs/notes/assets/` — folders for screenshots.

---

## 7. Architecture decisions

### 7.1. Spec validation — YES (not generation)

### 7.2. LiteSVM E2E test — YES

### 7.3. One circuit with `recipient_binding` — YES

### 7.4. Zero-padding (no domain separation) — CONFIRMED

### 7.5. `sync-circuits` — check/apply modes

### 7.6. Split deposit — DEFERRED to Stage 12

### 7.7. Makefile — clean design (Stage 9)

### 7.8. Tests/ directory — separate crate at root

**Decision (2026-09-24):** `tests/` is **outside** `onchain/`, at the project root.

**Reason:** `litesvm 0.16` requires **Agave 4.2**, which requires **Rust ≥ 1.90**. `onchain/rust-toolchain.toml` pins **1.89.0** (needed for SBF). Keeping tests inside `onchain/` means they **inherit** the 1.89 toolchain and fail with `E0658`.

**Solution:** separate crate at root with its own `rust-toolchain.toml` (`channel = "1.98.1"`).

**Connection to program:** via `zk_pool = { path = "../onchain/programs/zk_pool" }`.

**Not in workspace:** `onchain/Cargo.toml` does **not** include `tests`. It is a standalone crate.

### 7.9. Stages list

- Stage 0 — Repository skeleton ✅
- Stage 1 — Docker environment ✅
- Stage 2 — Circuits on Noir ✅
- Stage 3 — Sunspot verifier ✅
- Stage 3.7 — Documentation enrichment ✅
- Stage 4.1 — Anchor program ✅
- **Stage 4.5 — LiteSVM E2E test 🚧**
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

### From v2

- `anchor init --name <name>` does not create a subdirectory.
- `sunspot --version` is not supported.
- `sunspot deploy` requires `GNARK_VERIFIER_BIN` — Sunspot cloned in Dockerfile.
- `anchor build` uses existing keypair only if in `target/deploy/`.
- `@solana/kit`, `@codama/*` — pinned exact versions.
- Codama path: `../onchain/target/idl/zk_pool.json` (from `web/`).
- `Prover.toml` is gitignored.
- `bash -ic` required inside the container.

### From Stage 2 (circuits)

- **`poseidon2_permutation` in nargo 1.0.0-rc.2 takes ONE argument.**
- **Zero-padding causes arity collisions** — documented.
- **`main()` in `type = "bin"` circuits must have `pub` on return type.**
- **Field values must be < 2^254.**
- **Noir does not support float literals.**
- **`&str` parameters in Noir don't work.**
- **`f"...{value}"` format strings need explicit type annotation.**

### From Stage 3 (sunspot)

- **`solana/` volume may be root-owned.**
- **Ignore entire `solana/` directory.**
- **`sunspot verify` argument order** is `.vk`, `.proof`, `.pw`.
- **TOML parser rejects hex literals > 2^63** — wrap in quotes.
- **Sunspot generates 6 `deprecated` warnings** — normal.

### From Stage 4.1 (Anchor)

- **`anchor init .` fails** — use a temp dir.
- **`invalid --check-cfg argument`** — declare `anchor-debug`, `custom-heap`, `custom-panic` as features.
- **`E0107: struct takes 0 lifetime arguments`** — empty `#[derive(Accounts)]` doesn't work.
- **`unused import: super::*`.**
- **`test_reduce_to_field_max` FAILED** — loop `0..4` → `0..5`.
- **`ambiguous glob re-exports`** — rename handlers.
- **`E0308: mismatched types` in `CpiContext::new`** — `Pubkey` in Anchor 1.2.0, not `AccountInfo`.
- **`E0432: unresolved import crate`** — keep glob re-exports.
- **`E0277: #[instruction] type mismatch`** — list **all** handler args in order.
- **CPI to Sunspot verifier: `[proof || public_witness]`** — proof FIRST.
- **Vault PDA has no private key** — direct lamport manipulation.
- **`anchor deploy` is deprecated** — use `anchor program deploy`.
- **`zk_pool.so` grows to 210 KB** with 3 instructions.

### From Stage 4.5 (LiteSVM) — IN PROGRESS

- **`cargo fetch` fails in `onchain/tests/`**: "current package believes it's in a workspace when it's not".
  - **Cause:** `onchain/Cargo.toml` is a workspace root, `members = ["programs/*"]` doesn't cover `tests/`.
  - **First fix:** add `tests` to `workspace.exclude`. This **helped** `cargo fetch`, but did not fix the Rust version problem (see next).
  - **Final fix:** move `tests/` to project root (see 4.5.1).
- **`E0658: use of unstable library feature maybe_uninit_write_slice`** in `solana-syscalls 4.2.2`.
  - **Cause:** `litesvm 0.16` pulls Agave 4.2 which uses an unstable Rust feature. Needs Rust ≥ 1.90.
  - **Why it failed:** `onchain/rust-toolchain.toml` pins 1.89.0, and `tests/` inherited it.
  - **Fix:** move `tests/` out of `onchain/`; use `tests/rust-toolchain.toml` with `channel = "1.98.1"`.
- **`failed to select a version for solana-hash`** — version conflict between `litesvm 0.16` (needs `solana-hash ~4.5.0`) and `solana-message 5` (needs `solana-hash >= 4.6.0`).
  - **Fix:** use exact versions from litesvm's `Cargo.toml`:
    - `solana-account = "4.3.0"`
    - `solana-address = "~2.6.1"`
    - `solana-hash = "4.5.0"`
    - `solana-instruction = "3.4.0"`
    - `solana-keypair = "3.1.2"`
    - `solana-message = "4.2.4"`
    - `solana-sdk-ids = "3.1.0"`
    - `solana-signer = "3.0.1"`
    - `solana-transaction = "4.1.5"`
    - `solana-transaction-error = "3.3.1"`
- **`/home/ubuntu/tests: No such file or directory`** — volume not mounted.
  - **Fix:** add `- ../tests:/home/ubuntu/tests` to `infra/docker-compose.yml`; recreate container with `docker compose up -d --force-recreate solana`.
- **`E0583: file not found for module`** for 5 modules — expected, files not written yet.
  - **Fix:** create placeholder files (`//! Placeholder`) for each missing module.
- **Random `.rs` file (bash heredoc in a for-loop).**
  - **Symptom:** after running `for f in a b c; do cat > tests/src/$f.rs <<EOF ... EOF; done`, an unexpected file `tests/src/.rs` appeared.
  - **Cause:** bash mishandled the first iteration (variable expansion / heredoc interaction).
  - **Fix:** `rm tests/src/.rs`.
  - **Lesson:** when using loops with heredoc — check `ls` immediately after.
- **`E0308: Transaction::new_signed_with_payer` expects `&[Instruction]`, found `&Vec<CompiledInstruction>`.**
  - **Cause:** `Message::new_with_blockhash` compiles instructions. We were passing `&msg.instructions` (compiled) to a function that expects raw instructions.
  - **Fix:** use `Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash)` directly with raw instructions, without `Message::new_with_blockhash`.
  - **Lesson:** in `solana-transaction` there are two paths — through `Message` (compiled) and through raw `Instruction`. Do not mix them.

### How to find exact signatures for the installed crate version

- Rust crate sources: `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`.
- Anchor: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anchor-lang-1.2.0/src/`.
- LiteSVM: `~/.cargo/registry/src/index.crates.io-*/litesvm-0.16.0/src/`.
- Always check your **own** version, not v2 examples.

### How to resolve solana-* version conflicts

1. Find the "main" crate (e.g. `litesvm`) and its exact `solana-*` versions in its `Cargo.toml`.
2. Use **exactly** those versions in your `Cargo.toml`.
3. Do NOT guess or use semver ranges (`"4"`, `"5"`) — they pull incompatible versions.
4. Run `cargo tree -p <main-crate>` to verify resolution.
5. If a newer toolchain is required, isolate the crate (see 4.5.1).

---

## 9. Repository structure

```
zkpool-solana/
├── .checkpoints/            ← gitignored
├── .secrets/                ← gitignored
├── docs/
│   ├── notes/               ← Russian, committed, PORTFOLIO MATERIAL
│   │   ├── 00-checkpoints.md
│   │   ├── 00-glossary.md
│   │   ├── 00-zk-primer.md
│   │   ├── 01-setup.md
│   │   ├── 02-circuits.md
│   │   ├── 03-sunspot.md
│   │   ├── 04-anchor.md (1014 lines)
│   │   └── assets/{01-setup,02-circuits,03-sunspot}/
│   ├── ru/README.md
│   ├── DEMO-NOTICE.md
│   ├── PROJECT_CONTEXT.md   ← this file
│   └── threat-model.md
├── infra/
│   ├── docker-compose.yml
│   └── docker/Dockerfile.solana
├── circuits/{poseidon,hash2,hashes,withdrawal}/
├── onchain/
│   ├── Anchor.toml
│   ├── Cargo.toml           ← workspace root (Rust 1.89.0)
│   ├── rust-toolchain.toml
│   └── programs/zk_pool/
├── tests/                   ← standalone crate (Rust 1.98.1)
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── src/
│       ├── lib.rs
│       ├── helpers.rs       ← load both programs
│       ├── test_pool.rs     ← airdrop + pool instruction
│       ├── test_deposit.rs  ← placeholder
│       ├── test_withdraw.rs ← placeholder
│       └── test_double_spend.rs ← placeholder
├── services/{backend,merkle,prover}/
├── web/
├── scripts/{validate-spec,sync-circuits}/
├── solana/                  ← gitignored
├── .env.example
└── .gitignore
```

---

## 10. Git workflow

- Conventional commits.
- English files except `docs/notes/*.md`, `docs/ru/README.md`.
- Gitignored: `.secrets/`, `.checkpoints/`, `solana/`, `services/merkle/circuits/`, `web/public/circuits/`, `Prover.toml`.
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

**Last completed stage:** Stage 4.5.3 (`test_pool.rs`).
**Next stage:** Stage 4.5.4 — `test_deposit.rs`.

**Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (3.468 SOL, devnet).

**Deployed programs on devnet:**
- Verifier: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` (87 312 B).
- zk_pool: `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` (210 000 B, rent 1.068 SOL).

**Tests:** 41 (circuits) + 37 (on-chain unit) + 3 (LiteSVM) = **81**.

**LiteSVM tests passing:** `helpers::test_setup_svm_loads_both_programs`, `test_pool::test_airdrop_works`, `test_pool::test_pool_creates_state_and_vault`.

**Circuit ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db`

**Docs sizes:**
- `00-glossary.md` — 545 lines.
- `00-zk-primer.md` — 317 lines.
- `01-setup.md` — 762 lines.
- `02-circuits.md` — 957 lines.
- `03-sunspot.md` — 996 lines.
- `04-anchor.md` — 1014 lines.

---

## 13. Instructions for a new assistant

1. Read **section 0** first — especially **0.9** (document non-obvious), **0.10** (test in small steps), **0.11** (never delete information).
2. Read **section 14** for documentation policy.
3. Read this file completely.
4. Read `docs/notes/00-glossary.md`, `00-zk-primer.md`, `01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md`.
5. Last completed stage: **Stage 4.5.3**.
6. Next task: **Stage 4.5.4 — `test_deposit.rs`**.
7. **One task at a time.** Only exception: `git commit ... && git push`.
8. **Give files in full — always.**
9. **Never guess.** If ambiguous — ask.
10. **Test in small steps.** 20 lines, not 200.
11. **Never delete information from existing files.**
12. Reply in Russian. Files: English (except notes and `docs/ru/README.md`).

---

## 14. Documentation as portfolio material

**Set on 2026-09-24.**

### 14.1. Context

`docs/notes/*.md` is:
1. **Raw material** for guides, tutorials, and articles on **Medium** and **Mirror.xyz**.
2. **Part of the GitHub portfolio.**
3. **Teaching material.**

### 14.2. Structure requirements for every note

1. **TL;DR** at the top.
2. **Glossary links.**
3. **"Why" block** before each command.
4. **"Expected result" block** after each command.
5. **ASCII diagrams.**
6. **Screenshots** in `docs/notes/assets/NN-<name>/`.
7. **Cross-references.**
8. **"Common errors" section.**
9. **"Reproduction" section.**
10. **"What's next" section.**

### 14.3. Glossary

**Status:** ✅ done. 545 lines, 76 terms.

### 14.4. ZK-primer

**Status:** ✅ done. 317 lines, 14 sections.

### 14.5. Screenshots policy

- Saved to `docs/notes/assets/NN-<stage>/`.
- Naming: `NN-MM-<description>.png`.
- Committed (portfolio material).
- Max 500 KB per file.

### 14.6. Enrichment procedure

After each stage: write technical note → enrich → capture screenshots → update glossary.

### 14.7. Enrichment of stages 01–03 — DONE

### 14.8. Stage 3.7 — DONE

### 14.9. Ongoing

- Screenshots captured as we go.
- `04-anchor.md` — 1014 lines.
- Glossary updated when new terms appear.
