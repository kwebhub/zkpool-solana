# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-24 (after stage 4.1.9 — zk_pool deployed to devnet)

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

**Demo notice:** this is a demo / educational project. See `docs/DEMO-NOTICE.md` for the list of missing production features.

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

- **3.0 — Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`, 5 SOL. Commit: `e9ddbe5`.
- **3.1 — `sunspot compile`:** `.ccs` (642 177 B).
- **3.2 — `sunspot setup`:** `.pk` (2 145 109 B) + `.vk` (972 B).
- **3.3 — `sunspot deploy`:** `.so` (87 312 B), verifier Program ID `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.
- **3.4 — `solana program deploy`:** on devnet, balance 0.444 SOL.
- **3.5 — Local verification:** `sunspot verify` → `✅ Verification successful!`.
- **3.6 — Final checkpoint.** Commit: `441b9d2`.

### ✅ Stage 3.7. Documentation enrichment (2026-09-24)

- `docs/notes/00-glossary.md` — 545 lines, 76 terms.
- `docs/notes/00-zk-primer.md` — 317 lines, 14 sections.
- `docs/notes/01-setup.md` — 762 lines.
- `docs/notes/02-circuits.md` — 957 lines.
- `docs/notes/03-sunspot.md` — 996 lines.

### ✅ Stage 4.1. Anchor program — all 9 sub-stages complete

**4.1.1 — Anchor workspace** ✅ (commit `b8f6fdf`).
**4.1.2 — `constants.rs`** ✅ (commit `23c4dd9`).
**4.1.3 — `error.rs`, `events.rs`, `state.rs`** ✅ (commit `c45cb75`).
**4.1.4 — `encoding.rs`** ✅ (commit `dc0fb5e`).
**4.1.5 — `pool` instruction** ✅ (commit `553634e`).
**4.1.6 — `deposit` instruction** ✅ (commit `5455d04`).
**4.1.7 — `withdraw` instruction** ✅ (commit `ce71a47`).
**4.1.8 — Tests** ✅ (commit `8261530`, 37 unit tests).
**4.1.9 — Deploy to devnet** ✅ (program deployed).

**Deployed program details:**
- Program Id: `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
- ProgramData Address: `FaLqLdL1FVLcwZPpJTw2ugG67KKnEbZRuUJmqvyNtCeA`.
- Authority: `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`.
- Data Length: 210 000 bytes.
- Program balance: 1.068 SOL.
- Wallet balance: 3.468 SOL.
- Metadata account (IDL): `C931NVVbVKu4mjh1wjgh7bmFx6j6TfML89ut1GsRQHXk`.
- Keypair backed up to `.secrets/zk_pool-keypair.json` (stable Program ID).

### ✅ Docs (2026-09-22 — 2026-09-24)

- `docs/notes/00-checkpoints.md`, `00-glossary.md`, `00-zk-primer.md`.
- `docs/notes/01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md` (813 lines).
- `docs/DEMO-NOTICE.md` — missing production features.
- `docs/PROJECT_CONTEXT.md` — this file.
- `docs/notes/assets/` — folders for screenshots.

**Checkpoints:** 01-setup, 02.1-validate-spec, 02.2.1-poseidon, 02.2.2-hash2-hashes, 02.2.3-withdrawal, 02.3-sync-circuits, 02.4-stage-2-final, 03.1-sunspot-compile, 03.2-sunspot-setup, 03.3-sunspot-deploy, 03.4-deploy-verifier, 03.5-witness, 03.6-stage-3-final, 04.1.1-anchor-init, 04.1.2-constants, 04.1.3-types, 04.1.4-encoding, 04.1.5-pool, 04.1.6-deposit, 04.1.7-withdraw, 04.1.8-tests, 04.1.9-deploy.

---

## 7. Architecture decisions

### 7.1. Spec validation — YES (not generation)

`spec.json` is the source of truth. Files **validated against** it.

### 7.2. LiteSVM E2E test — YES

Runs after Stage 4.5.

### 7.3. One circuit with `recipient_binding` — YES

Public inputs (5): `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`.

### 7.4. Zero-padding (no domain separation) — CONFIRMED

`hash_1(x) == hash_2(x, 0)`, `hash_2(x, y) == hash_3(x, y, 0)`. Documented.

### 7.5. `sync-circuits` — check/apply modes

`--check`: mismatch → error, exit 1. `--apply`: mismatch → copy + warning.

### 7.6. Split deposit — DEFERRED to Stage 12

### 7.7. Makefile — clean design (Stage 9)

### 7.8. Stages list

- Stage 0 — Repository skeleton ✅
- Stage 1 — Docker environment ✅
- Stage 2 — Circuits on Noir ✅
- Stage 3 — Sunspot verifier ✅
- Stage 3.7 — Documentation enrichment ✅
- Stage 4.1 — Anchor program ✅
- **Stage 4.5 — LiteSVM E2E test ← next**
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

- **`poseidon2_permutation` in nargo 1.0.0-rc.2 takes ONE argument** (the state array).
- **Zero-padding causes arity collisions** (`hash_1(x) == hash_2(x, 0)`) — documented.
- **`main()` in `type = "bin"` circuits must have `pub` on return type.**
- **Field values must be < 2^254** (BN254 prime).
- **Noir does not support float literals.** Use lamports.
- **`&str` parameters in Noir don't work** — `str` requires a generic size (`str<N>`).
- **`f"...{value}"` format strings need explicit type annotation.**

### From Stage 3 (sunspot)

- **`solana/` volume may be root-owned** — fix with `sudo chown -R 1000:1000 solana/` on host.
- **Ignore entire `solana/` directory** in `.gitignore`.
- **`sunspot verify` argument order** is: `.vk`, `.proof`, `.pw`.
- **TOML parser rejects hex literals > 2^63** — wrap large Field values in double quotes.
- **Sunspot generates 6 `deprecated` warnings** during `sunspot deploy` — this is normal.

### From Stage 4.1 (Anchor)

- **`anchor init .` fails** — workspace name must be a valid Rust identifier. Use a temp dir: `cd /tmp && anchor init zk_pool --no-git --test-template rust`, then copy.
- **`invalid --check-cfg argument`** — declare `anchor-debug`, `custom-heap`, `custom-panic` as **features**, not through `[lints.rust]`.
- **`E0107: struct takes 0 lifetime arguments`** — `#[derive(Accounts)]` does **not** work with empty structs. Use an empty `#[program]` instead.
- **`unused import: super::*`** — remove the line from empty `#[program]`.
- **`test_reduce_to_field_max` FAILED** — loop `0..4` → `0..5`. `2^256 / p ≈ 4.006`, edge case needs the full 5 subtractions.
- **`ambiguous glob re-exports`** — rename `handler` → `handler_pool`, `handler_deposit`, `handler_withdraw`. In Anchor you still need glob re-exports (`pub use module::*;`).
- **`E0308: mismatched types` in `CpiContext::new`** — in Anchor 1.2.0 the first argument is `Pubkey`, not `AccountInfo`. Use `.key()` not `.to_account_info()`.
- **`E0432: unresolved import crate`** — Anchor's `#[program]` macro needs the generated `__client_accounts_*` structures. Do not replace glob re-exports with explicit imports.
- **`E0277: #[instruction] type mismatch`** — `#[instruction(...)]` must list **all** handler arguments **in order**. `#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32])]`.
- **CPI to Sunspot verifier: data layout is `[proof || public_witness]`** — proof FIRST (324 bytes), then public witness (172 bytes). Verify by reading `~/sunspot/gnark-solana/crates/verifier-bin/src/lib.rs`. Wrong order → `InvalidInstructionData`.
- **Vault PDA has no private key** — use direct lamport manipulation (`try_borrow_mut_lamports`), not `system_program::transfer`.
- **`anchor deploy` is deprecated** — use `anchor program deploy`. Both work.
- **`zk_pool.so` grows from 57 KB (empty) to 210 KB (3 instructions)** — expected.

### How to find exact signatures for the installed crate version

- Rust crate sources: `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`.
- Anchor: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anchor-lang-1.2.0/src/`.
- Always check your **own** version, not v2 examples.

---

## 9. Repository structure

```
zkpool-solana/
├── .checkpoints/            ← gitignored
├── .secrets/                ← gitignored (keypair backup)
├── docs/
│   ├── notes/               ← Russian, committed, PORTFOLIO MATERIAL
│   │   ├── 00-checkpoints.md
│   │   ├── 00-glossary.md
│   │   ├── 00-zk-primer.md
│   │   ├── 01-setup.md
│   │   ├── 02-circuits.md
│   │   ├── 03-sunspot.md
│   │   ├── 04-anchor.md
│   │   └── assets/{01-setup,02-circuits,03-sunspot}/
│   ├── ru/README.md
│   ├── DEMO-NOTICE.md
│   ├── PROJECT_CONTEXT.md   ← this file
│   └── threat-model.md
├── infra/{docker-compose.yml,docker/Dockerfile.solana}
├── circuits/{poseidon,hash2,hashes,withdrawal}/
├── onchain/
│   ├── Anchor.toml
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── programs/zk_pool/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── constants.rs
│   │       ├── encoding.rs
│   │       ├── error.rs
│   │       ├── events.rs
│   │       ├── instructions.rs
│   │       ├── instructions/{deposit,pool,withdraw}.rs
│   │       ├── lib.rs
│   │       └── state.rs
│   └── tests/               ← not in workspace members yet
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

### validate-spec

```bash
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  'cd /home/ubuntu && cargo run --manifest-path scripts/validate-spec/Cargo.toml --release'
```

### sync-circuits

```bash
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

**Last completed stage:** Stage 4.1.9 (zk_pool deployed to devnet).
**Next stage:** Stage 4.5 — LiteSVM E2E test.

**Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (3.468 SOL, devnet).

**Deployed programs on devnet:**
- Verifier: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` (87 312 B, balance 0.444 SOL).
- zk_pool: `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm` (210 000 B, balance 1.068 SOL).

**Instruction discriminators:**
- `pool`: `[134, 215, 119, 168, 28, 199, 193, 127]`
- `withdraw`: `[183, 18, 70, 156, 148, 109, 161, 34]`
- `deposit`: see IDL

**Tests:** 41 (circuits) + 37 (on-chain unit) = **78**.

**Circuit ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db`

**Sunspot artifacts:**
- `withdrawal.ccs` — `a2baffa46b1b3e04631c68d58069d0909bb535297972ed466a29b14b1ee2fa0f`
- `withdrawal.pk` — `1e7a66426f613ff51e356023ae3499711b2c7f4259d5eb74c40e036f994f87f7`
- `withdrawal.vk` — `6279a9e6e434c108869bb9b64c8ff20e66cecd3461a5d8cdd4941b817c9d7aed`
- `withdrawal.so` — `117fae71a4e6f421a0b31200f98e80d4955474d60080006c22881f9354e908b7`
- `withdrawal.pw` — `c4ffea4a4f03c123977f3931b548b01f55820840bc144495063798f2acadb07f`
- `withdrawal.proof` — `d454b20105b339f893b80a64081a76f6f4f9b3fb6927b5eaf9fb1e96e2c05c79`
- `Prover.toml` — `fb144adb85ea5061c13e89b5d217b4e21a485829cf52149a1bc53571cabc92b2`

**Docs sizes:**
- `00-glossary.md` — 545 lines.
- `00-zk-primer.md` — 317 lines.
- `01-setup.md` — 762 lines.
- `02-circuits.md` — 957 lines.
- `03-sunspot.md` — 996 lines.
- `04-anchor.md` — 813 lines.

---

## 13. Instructions for a new assistant

1. Read **section 0** first.
2. Read **section 14** for documentation policy.
3. Read this file completely.
4. Read `docs/notes/00-glossary.md`, `00-zk-primer.md`, `01-setup.md`, `02-circuits.md`, `03-sunspot.md`, `04-anchor.md`.
5. Last completed stage: **Stage 4.1.9**.
6. Next task: **Stage 4.5 — LiteSVM E2E test**.
7. **One task at a time.** Only exception: `git commit ... && git push`.
8. **Give files in full — always.** Never "find section X and replace".
9. **Never guess.** If ambiguous — ask.
10. Reply in Russian. Files: English (except notes and `docs/ru/README.md`).

---

## 14. Documentation as portfolio material

**Set on 2026-09-24.**

### 14.1. Context

`docs/notes/*.md` is:

1. **Raw material** for guides, tutorials, and articles on **Medium** and **Mirror.xyz**.
2. **Part of the GitHub portfolio** — shown to potential employers and collaborators.
3. **Teaching material** for readers from beginners to professional engineers.

### 14.2. Structure requirements for every note

1. **TL;DR** at the top: what, why, how many steps, prerequisites, next stage.
2. **Glossary links** at the top and inline.
3. **"Why" block** before each command.
4. **"Expected result" block** after each command.
5. **ASCII diagrams** for architecture, data flow.
6. **Screenshots** for Medium, in `docs/notes/assets/NN-<name>/`.
7. **Cross-references** to related notes.
8. **"Common errors" section** with full message, root cause, fix, lesson.
9. **"Reproduction" section** at the end.
10. **"What's next" section.**

### 14.3. Glossary

**Status:** ✅ done. 545 lines, 76 terms, 7 sections.

### 14.4. ZK-primer

**Status:** ✅ done. 317 lines, 14 sections.

### 14.5. Screenshots policy

- Saved to `docs/notes/assets/NN-<stage>/`.
- Naming: `NN-MM-<description>.png`.
- Committed (portfolio material).
- Max 500 KB per file.

### 14.6. Enrichment procedure

After each stage: write technical note → enrich with TL;DR, why-blocks, expected results, common errors, what's next → capture screenshots → update glossary.

### 14.7. Enrichment of stages 01–03 — DONE

### 14.8. Stage 3.7 — DONE

### 14.9. Ongoing

- Screenshots captured as we go.
- `04-anchor.md` — 813 lines, complete for 4.1.1–4.1.8.
- Glossary updated when new terms appear.
