# Changelog

All notable changes to this project are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

Nothing yet.

---

## [0.1.0] — 2026-09-29

First tagged release. **Demo / educational build.** See [DEMO-NOTICE.md](docs/DEMO-NOTICE.md) and [threat-model.md](docs/threat-model.md).

### Added — Core protocol (Stages 0–4)

- **Noir circuits** (`circuits/`):
  - `poseidon` library — Poseidon2 hash_1 / hash_2 / hash_3.
  - `hash2` circuit — external Poseidon2 hash of two field elements.
  - `hashes` circuit — commitment + nullifier_hash.
  - `withdrawal` circuit — Groth16 (BN254), 5 public inputs, TREE_DEPTH=20.
- **Sunspot verifier** deployed to devnet: `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`.
- **Anchor `zk_pool` program** deployed to devnet: `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`.
  - Instructions: `pool`, `deposit`, `withdraw`.
  - Vault PDA holds SOL; nullifier records prevent double-spend.
- **`scripts/validate-spec`** — 15 rules verifying `spec.json` consistency across circuit / Rust / TS.
- **`scripts/sync-circuits`** — check / apply ACIR copies into services and web.
- **LiteSVM integration tests** — 8 initial tests (pool, deposit, withdraw, double-spend).

### Added — Services (Stages 5–7)

- **Backend** (`services/backend/`) — Rust + axum, port 4001.
  - Endpoints: `/api/health`, `/api/commitments`, `/api/root`, `/api/proof`, `/api/root-preview`, `/api/withdraw`, `/metrics`.
  - Indexer: RPC polling → DepositEvent / WithdrawEvent → Postgres + Redis Merkle tree.
  - Rate limiting (5/min withdraw, 60/min read), Prometheus metrics.
- **Merkle service** (`services/merkle/`) — Node.js + Fastify, port 4003.
  - `POST /hash`, `POST /hashes`, `POST /root`, `POST /proof`, `GET /health`.
  - Poseidon2 via `@noir-lang/noir_js` — byte-identical to circuit.
  - 23 tests (`node --test`).
- **Prover** (`services/prover/`) — Rust + axum, port 4002.
  - `POST /prove` — Groth16 proof via nargo + sunspot subprocesses, serialized by mutex.
  - `GET /health`.
  - 8 unit + 3 integration + 1 ignored real-proof test.

### Added — On-chain pool deployment (Stage 8)

- `scripts/pool-init/` — Rust CLI initializing `PoolState` + `vault` PDA on devnet.
- Pool PDA: `B89Yhoecj9AKJEDXT49DfjbTJoqjovmcKgYmqdzQwBYf`.
- Vault PDA: `HYQgjKSDU8cw9Q74QBWQShGcSs5sERxAoe7PLyF5q6jq`.

### Added — Frontend (Stage 9)

- **Vue 3 + Vite + TypeScript + Pug + SCSS + Pinia** (`web/`).
- Wallet connect (Phantom, Solflare, Backpack — via `window.*` injection; no `@solana/wallet-adapter-vue`).
- **Codama-generated client** for the `zk_pool` program.
- **`@noir-lang/noir_js`** — Poseidon2 and commitment/nullifier_hash computed in browser.
- Typed API client for backend endpoints.
- **Deposit flow**: note generation → commitment → root preview → instruction → tx.
- **Withdraw flow**: note parse → witness assembly → Groth16 via backend → instruction → tx.
- BN254 field reduction (TS port of `encoding.rs::reduce_to_field`).

### Added — End-to-end demo (Stage 10)

- **`scripts/e2e-deposit/`** and **`scripts/e2e-withdraw/`** — Rust CLIs for a full devnet cycle.
- Verified **twice** on devnet: deposit → withdraw to arbitrary recipient.
- Double-spend protection verified: second withdrawal of the same note rejected on-chain.
- Groth16 proof verified by the on-chain verifier (`Proof verified successfully!`).

### Added — Infrastructure (Stage 11)

- **`Makefile`** — `up`, `down`, `reset`, `status`, `logs`, `web`, `build`, `clean`, `exec-c`, `help`.
- **Port mappings** — 4001–4003, 5173, 9090, 3000 reachable from host.
- **Prometheus** — scrapes backend `/metrics` every 15s.
- **Grafana** — provisioned datasource + dashboard (`zkpool-backend`).
- **Postgres** + **Redis** in Docker Compose.

### Added — Engineering (Stage 12)

- **CI workflow** (`.github/workflows/ci.yml`) — 7 jobs: validate-spec, onchain, litesvm, backend, prover, merkle, web.
- **Docker workflow** — build + push to GHCR (paths-filtered on `infra/docker/**`).
- **Release workflow** — tagged releases with `.so` + IDL + binaries + web + SHA256SUMS.
- **Security workflow** — `cargo audit` (7 crates), `pnpm audit` (2 dirs), `cargo deny`.
- **Templates** — `PULL_REQUEST_TEMPLATE.md`, `ISSUE_TEMPLATE/{bug,feature}`, `dependabot.yml`.
- **`CONTRIBUTING.md`**, **`SECURITY.md`**, **`README.md`**, **`README-ru.md`** (`docs/ru/`).
- **`.editorconfig`**, **`LICENSE`** (MIT).

### Added — Security (Stage 13)

- **Threat model** (`docs/threat-model.md`) — 12 attack scenarios (A1–A12), 7 invariants (I1–I7).
- **Adversarial tests** (`tests/src/test_adversarial.rs`) — 7 tests documenting A1, A9, A10 and verifying A11 rejection. Total `tests/` suite: 15.
- **Backend input validation** — hex64 checks on all field-element inputs, `leaf_index < 2^20`, `commitments < 2^20`.
- **Frontend security review** — no XSS vectors, no storage, `signAndSendTransaction` only.
- **`deny.toml`** at root + `services/backend/deny.toml`.

### Security

- **`RUSTSEC-2024-0363`** — upgraded `sqlx` from `0.7.4` to `0.8.6` (SQL injection via protocol smuggling over 4 GiB).
- **`poseidon2` mask fix** — random field elements now clear top 3 bits (`& 0x1f`), not top 2 (`& 0x3f`). Previous mask allowed values above the BN254 field modulus.

### Fixed

- **Anchor Borsh `Vec<u8>` length prefix** — `withdraw` data now includes `u32 LE` proof length. Previously caused `memory allocation failed`.
- **`NULLIFIER_RECORD_SEED`** — corrected from `b"nullifier3"` to `b"nullifier_record"`.
- **Groth16 compute budget** — `ComputeBudgetInstruction::SetComputeUnitLimit(400_000)` prepended to withdrawal txs (verifier uses ~182k CU).
- **`getDepositInstructionAsync`** in the frontend — pass `{address, role}` for the `depositor` account, no full signer needed.
- **`--experimental-strip-types` test runner** — `allowImportingTsExtensions: true` in `web/tsconfig.json`.
- **`pnpm 12` allowBuilds** — `allowBuilds` in `pnpm-workspace.yaml` (not `pnpm.onlyBuiltDependencies`).
- **Stale ACIR copies** — `sync-circuits --apply` fixed outdated `services/merkle/circuits/*.json` and `web/public/circuits/*.json`.

### Known limitations

Documented in [`docs/threat-model.md`](docs/threat-model.md):

- **A1** — `deposit` does not verify `new_root` on-chain.
- **A5** — Phishing frontend is out of scope.
- **A6** — The prover sees the full witness; for production, proving must run client-side.
- **A9** — Commitment forgery is self-harm only.
- **A10** — `ROOT_HISTORY_SIZE = 10` allows root eviction by spam-deposits.
- **Trusted setup** for Groth16 — no MPC ceremony.
- **Single-keypair** upgrade authority.

### Statistics

- **~140 tests** across all layers (circuits, on-chain, LiteSVM, backend, merkle, prover).
- **4 workflows** in CI/CD.
- **5 containers** in `docker compose up`.
- **4 deployed programs/PDAs** on devnet.
- **2 full E2E cycles** verified on devnet.

---

## How this release was built

Stage-by-stage, with checkpoints:

- Stages 0–4: repository, Docker, circuits, Sunspot verifier, Anchor program.
- Stage 5: backend.
- Stage 6: Merkle service.
- Stage 7: prover.
- Stage 8: on-chain pool deployment.
- Stage 9: frontend.
- Stage 10: full E2E on devnet.
- Stage 11: infrastructure (Makefile, Prometheus, Grafana).
- Stage 12: CI/CD.
- Stage 13: security.
- Stage 14: finalization.

Every stage has:
- A checkpoint in `.checkpoints/NN-*/` with SHA-256 of artifacts.
- A stage note in `docs/notes/NN-*.md` (Russian).
- A record in `docs/PROJECT_CONTEXT.md`.

---

[Unreleased]: https://github.com/kwebhub/zkpool-solana/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0
