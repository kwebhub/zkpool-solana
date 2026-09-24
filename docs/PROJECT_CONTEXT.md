# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-24 (after stage 3 — Sunspot verifier deployed; documentation-as-portfolio policy added)

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

**CRITICAL:** `docs/notes/*.md` is not just internal memory. It is the raw material for guides, tutorials, and articles on **Medium** and **Mirror.xyz**. It is part of the GitHub portfolio. Readers range from beginners to professional engineers.

Every note must be:
- **Self-contained** — a reader with no context should be able to follow.
- **Motivated** — every step explains **why**.
- **Reproducible** — every command includes expected output.
- **Navigable** — has TL;DR, glossary, cross-references, "what's next".
- **Visual** — has ASCII diagrams (GitHub) and screenshots (Medium, in `docs/notes/assets/`).

**See section 14 for the full policy.**

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

**3.0 — Wallet:**
- Address: `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`.
- Balance: 5 SOL (via web faucet).
- Commit: `e9ddbe5`.

**3.1 — `sunspot compile`**: `.ccs` (642 177 B). SHA-256 `a2baffa4...`.

**3.2 — `sunspot setup`**: `.pk` (2 145 109 B) + `.vk` (972 B). SHA-256 `1e7a6642...` and `6279a9e6...`.

**3.3 — `sunspot deploy`**: `.so` (87 312 B) + keypair. Verifier Program ID: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`. SHA-256 `117fae71...`.

**3.4 — `solana program deploy`:**
- Program Id: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.
- ProgramData Address: `4RyRCMSUsA6VJkRLFABuTxT4e2baovEy9fWH67MTUUFt`.
- Authority: `5iM6nzaCqegVG3j4CSf19zmU3tmcs9KP51djaBXnAKGc`.
- Program balance: 0.444 SOL.
- Signature: `wE9xVi9Er6NX2GF57dzedxKtKP6KQPrdZS2Nb2oqsfrRkpSjRZm8PQA8WK8pjPa8G728cb1WRLfGFEm2uJ5kZVU`.

**3.5 — Local verification:**
- Added `circuits/withdrawal/src/test_witness.nr` (witness generator test).
- `mod test_witness;` added to `main.nr`.
- `withdrawal.json` SHA-256 changed: `f154aca0...` → `29ac2e67...` (only ACIR wrapper; CCS/PK/VK/SO unchanged because constraint system is identical).
- `Prover.toml` generated from test output.
- `nargo execute` → `withdrawal.gz` (3 825 B).
- `sunspot prove` → `withdrawal.proof` (324 B) + `withdrawal.pw` (172 B). Time: 43 s.
- `sunspot verify` → **`✅ Verification successful!`** (1.25 s).

**3.6 — Final checkpoint**: `.checkpoints/03.6-stage-3-final/` with 10 artifacts + `manifest.txt`.
Commit: `441b9d2`, docs commit `e32058e`.

### ✅ Docs

- `docs/notes/00-checkpoints.md`, `01-setup.md`, `02-circuits.md`, `03-sunspot.md` (792 lines).
- `docs/DEMO-NOTICE.md` (214 lines).
- `docs/PROJECT_CONTEXT.md` — this file.
- `docs/notes/` is committed.

**Checkpoints:** `01-setup`, `02.1-validate-spec`, `02.2.1-poseidon`, `02.2.2-hash2-hashes`, `02.2.3-withdrawal`, `02.3-sync-circuits`, `02.4-stage-2-final`, `03.1-sunspot-compile`, `03.2-sunspot-setup`, `03.3-sunspot-deploy`, `03.4-deploy-verifier`, `03.5-witness`, `03.6-stage-3-final`.

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
- **Stage 3.7 — Documentation enrichment (glossary, ZK-primer, notes enrichment)** ← next
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
│   │   ├── 00-glossary.md      ← next: create
│   │   ├── 00-zk-primer.md     ← next: create
│   │   ├── 01-setup.md
│   │   ├── 02-circuits.md
│   │   ├── 03-sunspot.md
│   │   └── assets/             ← screenshots for Medium (next)
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
- Committed: `docs/notes/`, including `assets/`.
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

**Last completed stage:** Stage 3.6 (Sunspot verifier deployed and verified locally).
**Next stage:** Stage 3.7 — Documentation enrichment (glossary, ZK-primer, notes enrichment).

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

---

## 13. Instructions for a new assistant

1. Read **section 0** first.
2. Read **section 14** for documentation policy.
3. Read this file completely.
4. Read `docs/notes/00-checkpoints.md`, `01-setup.md`, `02-circuits.md`, `03-sunspot.md`.
5. Last completed stage: **Stage 3.6**.
6. Next task: **Stage 3.7 — Documentation enrichment**.
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

Every note must therefore be **self-contained**, **motivated**, **reproducible**, **navigable**, and **visual**.

### 14.2. Structure requirements for every note

Every note in `docs/notes/` must contain:

**1. TL;DR at the top.**
- What this stage is about.
- How many steps.
- Estimated time.
- Prerequisites (links to other notes).

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
- What can go wrong (link to "Common errors" section).

**5. ASCII diagrams.**
- For architecture, data flow, algorithm structure.
- Rendered directly in Markdown (works on GitHub).

**6. Screenshots (for Medium).**
- Saved in `docs/notes/assets/NN-<name>/`.
- Referenced from the note with relative paths.
- Filename pattern: `NN-step-MM-description.png`.

**7. Cross-references.**
- Explicit links to related notes: "see `02-circuits.md`, section on Poseidon".
- Not vague references: "as before" is not allowed.

**8. "Common errors" section.**
- Every error encountered during the actual run.
- Full error message.
- Root cause.
- Fix.
- Lesson for the future.

**9. "Reproduction" section at the end.**
- Minimal set of commands to reproduce the stage from scratch.
- What must be in place **before** starting.

**10. "What's next" section.**
- Link to the next note.
- What will be covered.

### 14.3. Glossary — `docs/notes/00-glossary.md`

A **single** file listing all terms used across the project:
- ZK terms: ZK proof, circuit, witness, commitment, nullifier, constraint, Groth16, trusted setup.
- Noir terms: ACIR, Nargo, `poseidon2_permutation`, `main()`, `pub`, `Field`.
- Sunspot terms: CCS, PK, VK, verifier program, toxic waste.
- Solana terms: PDA, CPI, Program, ProgramData, BPF, rent, Authority.
- Crypto terms: Poseidon2, BN254, Merkle tree, Merkle proof, hash function.
- Project terms: deposit note, recipient_binding, nullifier_hash.

Each term: **one paragraph**, no jargon-within-jargon, with **example from the project** where applicable.

### 14.4. ZK-primer — `docs/notes/00-zk-primer.md`

A **single** file introducing ZK concepts, for readers with **zero** ZK background:
- What a ZK proof is, in plain language.
- Why we need it (privacy on a public blockchain).
- The three roles: prover, verifier, circuit.
- Public vs private inputs.
- The witness.
- Commitment and nullifier, and why they are separate.
- Merkle tree in one paragraph.
- Where Groth16 fits and why it is chosen.
- What "trusted setup" means and why it matters.

Ends with **"Now read `02-circuits.md`"**.

### 14.5. Screenshots policy

Screenshots are saved to `docs/notes/assets/NN-<stage>/` and referenced from the note:

```markdown
![Verifier program on Solana Explorer](assets/03-sunspot/03-deploy-verifier-explorer.png)
```

**What to screenshot:**
- Terminal output for key commands (if not already in text).
- Block explorers (Solana Explorer, SolanaFM).
- Browser DevTools (Network tab, Console).
- Grafana dashboards (once they exist).
- Wallet interactions (Phantom prompts).

**Not to screenshot:**
- Long terminal logs — paste as text instead.
- Anything already in the text body.

**Screenshots are committed** (they are portfolio material). Keep them under 500 KB each.

### 14.6. Enrichment procedure

When a stage is finished:
1. Write the technical note (`NN-<stage>.md`) — first version, raw.
2. **Immediately after** — enrich it with:
   - TL;DR, "why", "expected result", "common errors", "what's next".
   - Cross-references to other notes and to the glossary.
   - ASCII diagrams.
3. **Within the same session** — capture any screenshots that make sense.
4. Update `docs/notes/00-glossary.md` with any new terms.
5. If a new concept requires it, update `docs/notes/00-zk-primer.md`.

**Do not defer enrichment to the end of the project.** Context is freshest right after the stage.

### 14.7. Enrichment of stages 01–03

Stages 01, 02, 03 are already written but need enrichment:
- `01-setup.md` — add TL;DR, glossary links, "why" blocks, "expected result" blocks, ASCII diagrams, "common errors" (already partly there), cross-references, "what's next".
- `02-circuits.md` — same, plus more "why Noir", "why Poseidon2", "why ACIR".
- `03-sunspot.md` — same, plus more explanation of Groth16, trusted setup, BPF, Program vs ProgramData.

**Enrichment of 01–03 is Stage 3.7, done now, before Stage 4.1.**

### 14.8. Order of work in Stage 3.7

1. `00-glossary.md` — new file. All terms.
2. `00-zk-primer.md` — new file. ZK introduction.
3. Enrich `01-setup.md`.
4. Enrich `02-circuits.md`.
5. Enrich `03-sunspot.md`.
6. Create `docs/notes/assets/` directory structure.
7. Capture first screenshots (from stages 01–03 that are still reproducible).

Each item — one task at a time, per section 0.1.
