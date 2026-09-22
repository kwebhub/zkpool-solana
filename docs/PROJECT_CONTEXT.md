# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant in a new chat. Load it first — the assistant will understand the project state without reading every note.
>
> **Last updated:** 2026-09-22 (after stage 1 completion + docs)

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

**This is a rewrite.** The goal of v3 is to reproduce every stage of v2 **with byte-level checkpoints after each stage** to catch the `withdraw` bug at the moment it is introduced.

---

## 2. Why v3 exists

In v2, `withdraw` failed with `InvalidInstructionData`. On-chain logs showed `Proof verification failed!` from the verifier program. Three days of debugging did not find the root cause. The likely cause is a **mismatch between public inputs** at one of the three layers:

1. `circuits/withdrawal/src/main.nr` — order and types of `pub` inputs.
2. `onchain/.../instructions.rs::encode_public_inputs` — byte layout of the witness.
3. `web/.../useWithdraw.ts` — instruction data assembly (discriminator, proof length, 32-byte words, amount).

**v3 fix strategy:** after each stage, save artifacts to `.checkpoints/NN-name/` and compare SHA-256 at the next stage where they are consumed.

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
| backend | 4001 | `solana` container |
| prover | 4002 | `solana` container |
| merkle | 4003 | `solana` container |
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
| `hash2.json`, `hashes.json`, `withdrawal.json` | 2 | 6, 8 |
| `Prover.toml` (reference witness) | 2 | 3, 4, 7, 8 |
| `withdrawal.ccs`, `.pk`, `.vk`, `.so` | 3 | 4, 7 |
| `proof`, `public_witness` (local) | 3 | 7 |
| `encode_public_inputs` (172 bytes) | 4 | 7, 8 |
| witness from `useWithdraw.ts` | 8 | must equal `Prover.toml` |

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

**Checkpoint 1 artifacts:**
- `.checkpoints/01-setup/versions.txt` — tool versions.
- `.checkpoints/01-setup/sunspot-clone.txt` — Sunspot clone listing.
- `.checkpoints/01-setup/commit.txt` — `d0f0f34b593a8a0e32bdc5dfded7f30b243ec660`.
- `.checkpoints/01-setup/manifest.txt` — 7 tracked files + hashes.

---

## 7. What's next

### Immediate next stage: **Stage 2 — ZK circuits on Noir**

Plan:
1. Create `circuits/poseidon/` — library with `hash_1`, `hash_2`, `hash_3`.
2. Create `circuits/hash2/` — circuit for external Poseidon2 hashing (JS via `noir_js`).
3. Create `circuits/hashes/` — commitment + nullifier_hash.
4. Create `circuits/withdrawal/` — main circuit with Merkle proof verification + `recipient_binding`.
5. Generate `Prover.toml` from test `test_generate_valid_inputs`.
6. Save artifacts to `.checkpoints/02-circuits/`: `hash2.json.sha256`, `hashes.json.sha256`, `withdrawal.json.sha256`, `Prover.toml`, `manifest.txt`.

**Stop condition:** `nargo test` passes for all three circuits; `nargo execute` produces a valid witness; hashes of `.json` saved.

---

## 8. Roadmap (v3)

- [x] Stage 0. Repository skeleton
- [x] Stage 1. Docker environment
- [ ] Stage 2. ZK circuits (Noir)
- [ ] Stage 3. Sunspot verifier
- [ ] Stage 4.1a. Front-running protection (`recipient_binding`)
- [ ] Stage 4.2. Anchor program
- [ ] Stage 5. Backend
- [ ] Stage 6. Merkle service
- [ ] Stage 7. Prover
- [ ] Stage 8. Frontend
- [ ] Stage 9. Infrastructure
- [ ] Stage 10. Engineering processes
- [ ] Stage 11. Security
- [ ] Stage 12. Finalization

---

## 9. Git workflow

- Conventional commits.
- **All committed files in English**, except `docs/notes/*.md` and `docs/ru/README.md` (Russian).
- `.secrets/` and `.checkpoints/` — gitignored.
- `docs/notes/` — **not** gitignored, committed.

---

## 10. Commands (currently available)

```bash
# Enter container
cd ~/Projects/Solana/zkpool-solana
docker compose -f infra/docker-compose.yml exec solana bash

# Or one-shot command inside container
docker compose -f infra/docker-compose.yml exec solana bash -ic '<command>'
```

Full Makefile will arrive at Stage 9.

---

## 11. Known pitfalls (from v2)

- `anchor init --name <name>` does not create a subdirectory — it initializes in cwd. Use `mkdir X && cd X && nargo init --name X` for circuits.
- `sunspot --version` is not supported — use `sunspot --help`.
- `sunspot deploy` requires `GNARK_VERIFIER_BIN` — the Sunspot repo must be cloned. In v3, it is cloned in the Dockerfile.
- `anchor build` only uses an existing keypair if present in `target/deploy/`. Restore it from `.secrets/` before build to keep the Program ID stable.
- `@solana/kit`, `@solana/program-client-core`, `@codama/*` — must be pinned to **exact** versions (no `^`), otherwise pnpm installs incompatible versions.
- Codama path in `codama.json` must be `../onchain/target/idl/zk_pool.json` (from `web/`).
- `Prover.toml` is gitignored; regenerate it from `nargo test test_generate_valid_inputs --show-output`.

---

## 12. Next task for a new assistant

**If you are starting a new chat:**

1. Read this file completely.
2. Read `docs/notes/00-checkpoints.md` for checkpoint rules.
3. The last completed stage is **Stage 1**.
4. The next task is **Stage 2 — ZK circuits on Noir**.
5. Do **not** jump ahead. One task at a time. After each task, update `docs/PROJECT_CONTEXT.md` and `docs/notes/NN-name.md`.
6. All artifacts must be checkpointed with SHA-256 in `.checkpoints/NN-name/`.
