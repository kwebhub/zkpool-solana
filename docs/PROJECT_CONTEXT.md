# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-24 (after stage 3.7 — documentation enrichment complete)

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
│ • zk_pool program:   (TBD after Stage 4)                         │
└──────────────────────────────────────────────────────────────────┘
```

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

Commit: `8a41984f046a7c1deca7ed75493903de97df659a`.

### ✅ Stage 1. Docker environment (2026-09-22)

- `infra/docker-compose.yml`, `infra/docker/Dockerfile.solana`.
- Sunspot cloned in Dockerfile (persistent).
- Commit: `94c18fe522e39822692fff9b9b22b2fe0f8e00d0`.

**Versions:** rustc 1.98.1, cargo 1.98.1, solana-cli 3.1.10, anchor-cli 1.1.2, nargo 1.0.0-rc.2, sunspot 1.0.0, node v24.21.0, pnpm 12.5.1.

### ✅ Stage 2. Circuits on Noir (2026-09-22 — 2026-09-23)

- **2.0** — `spec.json`. Commit: `6df5fa5`.
- **2.1** — `validate-spec` (15 rules). Commits: `e479137`, `dbb4365`.
- **2.2.1** — `poseidon` (11 tests). Commit: `baade07`.
- **2.2.2** — `hash2` (5 tests), `hashes` (9 tests). Commit: `8354921`.
- **2.2.3** — `withdrawal` + `merkle_tree` (16 tests). Commit: `90afe4c`.
- **2.3** — `sync-circuits`. Commit: `aa5d8d9`.
- **2.4** — final checkpoint. Commit: `e0a3725`.

### ✅ Stage 3. Sunspot verifier (2026-09-23 — 2026-09-24)

- **3.0 — Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`, 5 SOL via faucet. Commit: `e9ddbe5`.
- **3.1 — `sunspot compile`:** `.ccs` (642 177 B), SHA-256 `a2baffa4...`.
- **3.2 — `sunspot setup`:** `.pk` (2 145 109 B) + `.vk` (972 B), SHA-256 `1e7a6642...` and `6279a9e6...`.
- **3.3 — `sunspot deploy`:** `.so` (87 312 B) + keypair, verifier Program ID `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`, SHA-256 `117fae71...`.
- **3.4 — `solana program deploy`:** ProgramData `4RyRCMSUsA6VJkRLFABuTxT4e2baovEy9fWH67MTUUFt`, balance 0.444 SOL, signature `wE9xVi9Er6NX2GF57dzedxKtKP6KQPrdZS2Nb2oqsfrRkpSjRZm8PQA8WK8pjPa8G728cb1WRLfGFEm2uJ5kZVU`.
- **3.5 — Local verification:** witness → proof → verify. `sunspot verify` → **`✅ Verification successful!`**.
- **3.6 — Final checkpoint**: `.checkpoints/03.6-stage-3-final/`. Commit: `441b9d2`, docs commit `e32058e`.

### ✅ Stage 3.7. Documentation enrichment (2026-09-24)

**Goal:** turn `docs/notes/*.md` into portfolio-quality tutorial material for Medium / Mirror.xyz.

Created:
- `docs/notes/00-glossary.md` — 545 lines, 76 terms across 7 sections.
- `docs/notes/00-zk-primer.md` — 317 lines, 14 sections. Introduction to ZK for readers without prior experience.

Enriched:
- `docs/notes/01-setup.md` — 762 lines (was 165), 16 sections. TL;DR, why-blocks, expected results, diagrams, cross-references.
- `docs/notes/02-circuits.md` — 957 lines (was 301), 17 sections. Same additions plus a full error catalog.
- `docs/notes/03-sunspot.md` — 996 lines (was 792), 15 sections. Same additions plus a pipeline diagram.

Structure:
- `docs/notes/assets/` — folders for screenshots (01-setup, 02-circuits, 03-sunspot). No files yet; screenshots are captured as we go.
- `docs/notes/assets/README.md` — created, then removed (rules live only in section 14.5).

Commits:
- `02fc3ea` — glossary.
- `69005a9` — ZK primer.
- `f996e2b` — enrich 01-setup.
- `49968fd` — enrich 02-circuits.
- `73f00e0` — enrich 03-sunspot.

### ✅ Docs (2026-09-22 — 2026-09-24)

- `docs/notes/00-checkpoints.md` — checkpoint methodology (RU).
- `docs/notes/00-glossary.md` — glossary (RU).
- `docs/notes/00-zk-primer.md` — ZK introduction (RU).
- `docs/notes/01-setup.md` — stage 1 (RU, 762 lines).
- `docs/notes/02-circuits.md` — stage 2 (RU, 957 lines).
- `docs/notes/03-sunspot.md` — stage 3 (RU, 996 lines).
- `docs/DEMO-NOTICE.md` — missing production features (EN, 214 lines).
- `docs/PROJECT_CONTEXT.md` — this file (EN).
- `docs/notes/assets/` — folders for screenshots (empty).
- `docs/notes/` is committed.

**Checkpoints:** 01-setup, 02.1-validate-spec, 02.2.1-poseidon, 02.2.2-hash2-hashes, 02.2.3-withdrawal, 02.3-sync-circuits, 02.4-stage-2-final, 03.1-sunspot-compile, 03.2-sunspot-setup, 03.3-sunspot-deploy, 03.4-deploy-verifier, 03.5-witness, 03.6-stage-3-final.

---

## 7. Architecture decisions

### 7.1. Spec validation — YES (not generation)

`spec.json` is the source of truth. Files **validated against** it.

### 7.2. LiteSVM E2E test — YES

`onchain/programs/zk_pool/tests/e2e_deposit_withdraw.rs`: init pool → deposit → generate proof with real Sunspot → withdraw → verify SOL moved. Runs after Stage 4.5.

### 7.3. One circuit with `recipient_binding` — YES

Public inputs (5): `root`, `nullifier_hash`, `recipient`, `recipient_binding`, `amount`.

### 7.4. Zero-padding (no domain separation) — CONFIRMED

`hash_1(x) == hash_2(x, 0)`, `hash_2(x, y) == hash_3(x, y, 0)`. Documented. Safe for zkpool-solana.

### 7.5. `sync-circuits` — check/apply modes

`--check`: mismatch → error, exit 1, no copy. `--apply`: mismatch → copy + warning, exit 0.

### 7.6. Split deposit — DEFERRED to Stage 12

### 7.7. Makefile — clean design (Stage 9)

### 7.8. Stages list (v3)

- Stage 0 — Repository skeleton ✅
- Stage 1 — Docker environment ✅
- Stage 2 — Circuits on Noir ✅
- Stage 3 — Sunspot verifier ✅
- Stage 3.7 — Documentation enrichment ✅
- **Stage 4.1 — Anchor program (next)**
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
│   │   └── assets/          ← screenshots for Medium
│   │       ├── 01-setup/    ← empty
│   │       ├── 02-circuits/ ← empty
│   │       └── 03-sunspot/  ← empty
│   ├── ru/README.md
│   ├── DEMO-NOTICE.md
│   ├── PROJECT_CONTEXT.md   ← this file
│   └── threat-model.md
├── infra/
├── circuits/
├── onchain/
├── services/
├── web/
├── scripts/
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

**Last completed stage:** Stage 3.7 (documentation enrichment).
**Next stage:** Stage 4.1 — Anchor program.

**Wallet:** `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc` (4.554 SOL, devnet).

**Deployed programs on devnet:**
- Verifier: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ` (87 312 B, balance 0.444 SOL).

**Circuit ACIRs:**
- `hash2.json` — `27c1937b46ea693a627400a8040fbce816e2bfd7c8ef07ae40df00e1f13b37c6`
- `hashes.json` — `ca81b13700eac8caddde2fcce235d8b138ee249abf70d5c6ab64317e0c2bfbe9`
- `withdrawal.json` — `29ac2e677229f4518944c76161933416e2a083dee83da638a9dfd01226cc91db`

**Sunspot artifacts:**
- `withdrawal.ccs` — `a2baffa46b1b3e04631c68d58069d0909bb535297972ed466a29b14b1ee2fa0f`
- `withdrawal.pk` — `1e7a66426f613ff51e356023ae3499711b2c7f4259d5eb74c40e036f994f87f7`
- `withdrawal.vk` — `6279a9e6e434c108869bb9b64c8ff20e66cecd3461a5d8cdd4941b817c9d7aed`
- `withdrawal.so` — `117fae71a4e6f421a0b31200f98e80d4955474d60080006c22881f9354e908b7`
- `withdrawal-keypair.json` — `460c1eb4637a80d6cf22508eb292492533e736c74f2cc6ff0808ab20c6d597a1`
- `withdrawal.gz` — `da6779ae9457891ca85378718e2a9637186306790347f1cfe9f29df964321201`
- `withdrawal.proof` — `d454b20105b339f893b80a64081a76f6f4f9b3fb6927b5eaf9fb1e96e2c05c79`
- `withdrawal.pw` — `c4ffea4a4f03c123977f3931b548b01f55820840bc144495063798f2acadb07f`
- `Prover.toml` — `fb144adb85ea5061c13e89b5d217b4e21a485829cf52149a1bc53571cabc92b2`

**Tests:** 41 (11 poseidon + 5 hash2 + 9 hashes + 16 withdrawal).

**Docs sizes:**
- `00-checkpoints.md` — 130 lines.
- `00-glossary.md` — 545 lines.
- `00-zk-primer.md` — 317 lines.
- `01-setup.md` — 762 lines.
- `02-circuits.md` — 957 lines.
- `03-sunspot.md` — 996 lines.
- `DEMO-NOTICE.md` — 214 lines.

---

## 13. Instructions for a new assistant

1. Read **section 0** first.
2. Read **section 14** for documentation policy.
3. Read this file completely.
4. Read `docs/notes/00-checkpoints.md`, `00-glossary.md`, `00-zk-primer.md`, `01-setup.md`, `02-circuits.md`, `03-sunspot.md`.
5. Last completed stage: **Stage 3.7**.
6. Next task: **Stage 4.1 — Anchor program**.
7. **One task at a time.** Only exception: `git commit ... && git push`.
8. **Give files in full.**
9. **Never guess.**
10. Reply in Russian. Files: English (except notes and `docs/ru/README.md`).

---

## 14. Documentation as portfolio material

**Set on 2026-09-24.**

### 14.1. Context

`docs/notes/*.md` is **not only** internal project memory. It is:

1. **Raw material** for guides, tutorials, and articles on **Medium** and **Mirror.xyz**.
2. **Part of the GitHub portfolio** — the repository is shown to potential employers and collaborators.
3. **Teaching material** for readers ranging from **beginners** to **professional engineers**.

### 14.2. Structure requirements for every note

Every note in `docs/notes/` must contain:

**1. TL;DR at the top.**
- What this stage is about.
- How many steps.
- Estimated time.
- Prerequisites (links to other notes).
- What will be produced.
- Link to the next stage.

**2. Glossary links.**
- At the top: link to `docs/notes/00-glossary.md`.
- Inline: terms are linked on first use.

**3. "Why" block before each step.**
- Before every command or code block, explain **why** we do it.
- What it produces.
- How it fits into the bigger picture.

**4. "Expected result" block after each step.**
- Real output (pasted from the actual run).
- What to look for.
- Link to "Common errors" section if relevant.

**5. ASCII diagrams.**
- For architecture, data flow, algorithm structure.
- Rendered directly in Markdown (works on GitHub).

**6. Screenshots (for Medium).**
- Saved in `docs/notes/assets/NN-<name>/`.
- Referenced from the note with relative paths.
- Filename pattern: `NN-MM-description.png`.

**7. Cross-references.**
- Explicit links to related notes: "see `02-circuits.md`, section on Poseidon".

**8. "Common errors" section.**
- Every error encountered during the actual run.
- Full error message.
- Root cause.
- Fix.
- Lesson for the future.

**9. "Reproduction" section at the end.**
- Minimal set of commands to reproduce the stage from scratch.

**10. "What's next" section.**
- Link to the next note.
- What will be covered.

### 14.3. Glossary — `docs/notes/00-glossary.md`

**Status: ✅ done.** 545 lines, 76 terms, 7 sections.

### 14.4. ZK-primer — `docs/notes/00-zk-primer.md`

**Status: ✅ done.** 317 lines, 14 sections.

### 14.5. Screenshots policy

Screenshots are saved to `docs/notes/assets/NN-<stage>/` and referenced from the note:

```markdown
![Verifier program on Solana Explorer](assets/03-sunspot/03-04-deploy-verifier-explorer.png)
```

**Naming convention:** `NN-MM-<description>.png`
- `NN` — stage number.
- `MM` — step number within the note.
- `<description>` — short kebab-case description.

**What to screenshot:**
- Terminal output for key commands (short, focused).
- Block explorers (Solana Explorer, SolanaFM).
- Browser DevTools (Network tab, Console).
- Grafana dashboards (once they exist).
- Wallet interactions (Phantom prompts).

**Not to screenshot:**
- Long terminal logs — paste as text instead.
- Anything already in the text body as a code block.

**Format:**
- PNG, max 500 KB per file.
- 2× retina preferred.
- Redact seed phrases, private keys, other users' addresses.

**Screenshots are committed** (portfolio material).

**Folders:** `docs/notes/assets/{01-setup,02-circuits,03-sunspot}` exist. No files yet — captured as we go.

### 14.6. Enrichment procedure

When a stage is finished:
1. Write the technical note (`NN-<stage>.md`) — first version, raw.
2. **Immediately after** — enrich with TL;DR, why-blocks, expected results, common errors, what's next.
3. **Within the same session** — capture screenshots.
4. Update `docs/notes/00-glossary.md` with new terms.
5. Update `docs/notes/00-zk-primer.md` if a new foundational concept appeared.

**Do not defer enrichment to the end of the project.**

### 14.7. Enrichment of stages 01–03 — DONE

- `01-setup.md` — 762 lines. ✅
- `02-circuits.md` — 957 lines. ✅
- `03-sunspot.md` — 996 lines. ✅

### 14.8. Stage 3.7 — DONE

1. `00-glossary.md` — ✅.
2. `00-zk-primer.md` — ✅.
3. Enrich `01-setup.md` — ✅.
4. Enrich `02-circuits.md` — ✅.
5. Enrich `03-sunspot.md` — ✅.
6. `docs/notes/assets/` structure — ✅ (folders only).
7. Screenshots — captured as we go (not a blocking task).
