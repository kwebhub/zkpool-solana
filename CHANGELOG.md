# Changelog

All notable changes to this project are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added — Stage 16

- **16.1 — Lazy-load `@noir-lang/noir_js`.** `web/src/noir/poseidon.ts` and `web/src/noir/hashes.ts` now use `import("@noir-lang/noir_js")` inside `loadHash2()` / `loadHashes()`. The ~3.84 MB WASM (acvm_js + noirc_abi_wasm) is no longer part of the initial bundle; it downloads on the first proof-relevant action. `vite-plugin-top-level-await` removed from `web/vite.config.ts` — Vite handles TLA in the emitted chunk natively. See `docs/notes/17-wasm-lazy-load.md`.
- **16.2 — Solflare wallet connect.** `web/src/stores/wallet.ts` now reads `provider.publicKey` after `connect()` resolves, supporting both Phantom (`{ publicKey }`) and Solflare (`true`). `web/src/wallet/types.ts` updated accordingly.

### Changed — Stage 16

- **`dist/assets/` split into two JS chunks** (main + lazy chunk containing `noir_js`). WASM assets are emitted but requested on demand.

### Known limitations — Stage 16

- **Solflare withdraw is not supported.** `provider.signAndSendTransaction({ serialize, message: { version: 0 } })` opens the Solflare signing dialog (devnet blockhash passes the network check) but fails at the Approve step with `JsonRpcError: Internal error at chrome-extension://bhhhlbepdkbapadjdnnojkbgioiodbic/inpage.js`. Solflare expects a `Transaction`/`VersionedTransaction` instance from `@solana/web3.js@1.x`, not a `@solana/kit`-wire object. **Only Phantom is supported for withdrawal.** Deposit works with both.
- **Backpack untested.**

---

## [0.2.1] — 2026-10-02

Patch release on top of `v0.2.0`. Adds Stage 16.1 (lazy-load `noir_js`) and 16.2 (Solflare wallet connect). No breaking changes relative to `v0.2.0`. See `[Unreleased]` above for the full description of Stage 16 — the entries are identical and this tag is intended for a future release-cut.

**Assets (this release):** same set as `v0.2.0` — `zk_pool.so`, `zk_pool.json`, `zkpool-backend`, `zkpool-prover`, `zkpool-web.tar.gz`, `SHA256SUMS`.

---
## [0.2.0] — 2026-10-01

Second tagged release. **Split deposit — breaking change.** Notes created by v0.1.0 are **not** compatible with the v0.2.0 circuit and cannot be withdrawn. Demo / educational build. See [DEMO-NOTICE.md](docs/DEMO-NOTICE.md) and [threat-model.md](docs/threat-model.md).

### Added — Split deposit (Stage 15)

Split a single deposit into three unequal commitments in one transaction. Breaks the on-chain amount correlation between deposit and withdrawal. See [`docs/notes/15-split-deposit.md`](docs/notes/15-split-deposit.md).

- **Circuit** (`circuits/withdrawal/`):
  - New public input `total_amount` (aggregate deposit amount). Public inputs: 5 → 6.
  - New private inputs `splits[3]` (split vector) and `note_index` (u32). Private inputs: 5 → 7.
  - New constraints:
    - **C4** — `splits[0] + splits[1] + splits[2] == total_amount`.
    - **C5** — `splits[note_index] == amount`.
  - `SPLIT_COUNT = 3` (const-generic, same pattern as `TREE_DEPTH`).
  - Public witness: 172 → 204 bytes. Groth16 proof: 324 → 388 bytes (Sunspot 1.0.0 toolchain change).
  - New ACIR hash: `a49bc877135ae75713d2ab7cbe39a69151a606c328f48fe8163c0629787d3262`.
- **Spec** (`circuits/withdrawal/spec.json`):
  - `circuit.split_count = 3`, `circuit.nr_public_inputs = 6`.
  - `witness_layout.total_bytes = 204`.
  - `validate-spec` extended to **17 rules** (`rule_split_count`, `rule_private_inputs_lengths` added).
- **Anchor `zk_pool` program:**
  - New instruction **`deposit_split`** — creates 3 commitments, emits 3 `DepositEvent`s, one final root stored, `total_deposits += 1`, `next_leaf_index += 3`. Args struct: `DepositSplitArgs { commitments, new_roots, amounts, total_amount }`.
  - `withdraw` gains a `total_amount: u64` argument.
  - Constants: `NR_PUBLIC_INPUTS = 6`, `PUBLIC_INPUTS_BYTES = 204`, `PROOF_LEN = 388`, `SPLIT_COUNT = 3`.
  - New errors: `SplitSumMismatch`, `NotEnoughRoom`.
- **Verifier** upgraded in place (same Program ID `5t51iu6a…`). New `.so` 197 056 B.
- **Backend:**
  - `WithdrawRequest` gains `total_amount`, `splits[3]`, `note_index`.
  - Validation: `splits.len() == 3`, `note_index < 3`.
  - `WithdrawResponse` doc updated (388 B proof, 204 B public witness).
  - Indexer already handles N `DepositEvent`s per transaction — no code change required.
- **Prover:**
  - `WitnessInputs` gains `total_amount`, `splits`, `note_index`.
  - `to_toml()` writes all 13 fields in the order produced by `test_witness.nr`.
- **Frontend:**
  - `SPLIT_COUNT = 3` in `constants.ts`.
  - `WithdrawRequest` and `WithdrawResult` updated.
  - `parseNote.ts` reads `splits` and `note_index`.
  - `buildWitness.ts` computes `total_amount = Σ splits[i]`.
  - `useWithdraw.ts`: `PROOF_LEN = 388`, `total_amount` passed to the instruction.
  - `generateNote.ts`: new `generateSplitNotes(amounts)` for 3 notes.
  - `useDeposit.ts`: new `depositSplit(amounts)` with 3 cumulative root previews.
  - `DepositForm.vue`: split mode with 3 amount inputs; displays 3 notes for saving.
  - Codama client regenerated (new instruction `depositSplit`, new arg `totalAmount` on `withdraw`).
- **Scripts:**
  - New `scripts/e2e-deposit-split/` — Rust CLI for a full split-deposit cycle.
  - `scripts/e2e-withdraw/` updated for the new note format and 388-byte proof.
- **Tests:** LiteSVM suite extended with 5 split tests (`tests/src/test_deposit_split.rs`). Total `tests/`: **20**.
- **Devnet E2E verified:** 1 SOL → 3 notes (0.5 + 0.3 + 0.2) → 3 withdrawals to the same recipient → double-spend attempt rejected.

### Changed

- **Groth16 proof length** — 324 → 388 bytes. Driven by the Sunspot 1.0.0 rebuild against the new toolchain (`solana-program 3.0.0`, `solana-bn254 3.1.2`). Verified by `sunspot verify`.
- **Verifier `.so` size** — 87 312 → 197 056 bytes. The increase is greater than the constraint delta (+58); attributed to the new toolchain's bundle composition.
- **Verifier CU cost** — ~182 000 → ~382 000 CU. `ComputeBudgetInstruction::SetComputeUnitLimit(1_400_000)` now required in `scripts/e2e-withdraw/` and `web/src/composables/useWithdraw.ts`. 400 000 was insufficient.
- **Devnet verifier upgrade** was performed **in place** (Program ID `5t51iu6a…` unchanged) via `solana program deploy --program-id <existing>`. Recorded in Stage 15.4. See §4.1 of `docs/notes/15-split-deposit.md` for the `sunspot deploy` keypair-overwrite pitfall.
- **`WithdrawEvent`** unchanged — the indexer already parses one event per log line.

### Security

- **`sunspot deploy` overwrites the program keypair.**
  - Symptom: running `sunspot deploy` unconditionally replaces `withdrawal-keypair.json`, changing the Program ID.
  - Cause: `sunspot deploy` always `os.Rename`s the freshly built `verifier_bin-keypair.json` on top of the target.
  - Fix: back up the original keypair **before** deploy, restore it **after**, verify with `solana-keygen pubkey`. Documented in `docs/notes/15-split-deposit.md` §15.4.

### Fixed

- **Noir `Field` comparison.** `assert((note_index as Field) < (SPLIT_COUNT as Field))` failed with `Fields cannot be compared`. Fixed by comparing as `u32` directly.
- **Prover/backend stale binaries.** `cargo build --release` + restart is required after every wire-format change. Documented in `docs/PROJECT_CONTEXT.md` §8.11.
- **Backend 400 on `/api/withdraw`** masked a prover 500. Diagnosed by tailing **both** logs.
- **fix 16 — Phantom-compatible `signAndSendTransaction`.** The first UI interaction with Phantom revealed that Phantom's `signAndSendTransaction` expects an object `{ serialize, message: { version: 0 } }`, not a base64 string. Applied in `web/src/composables/useDeposit.ts` and `web/src/composables/useWithdraw.ts`. Symptom: `TypeError: Cannot use 'in' operator to search for 'version' in AQAAAA…` with `chrome-extension://bfnaelmomeimhlpmgjnjophhpkkoljpa/solana.js` in the stack.
- **fix 16 — legacy single-notes in a Stage 15 circuit.** The circuit hard-codes `splits: [Field; 3]`; legacy single-notes carry `splits: [amount]` (length 1). Client pads to `[amount, 0, 0]`, `note_index = 0`, `total_amount = amount`. Backend and prover accept `splits.len()` ∈ `{1, SPLIT_COUNT}`. No circuit / verifier / on-chain changes. Applied in `web/src/withdraw/buildWitness.ts`, `web/src/withdraw/parseNote.ts`, `services/backend/src/api_types.rs`, `services/prover/src/witness.rs`.

### Known limitations

In addition to the v0.1.0 limitations (see below), Stage 15 introduces:

- **Path 2 split deposit** adds two new public/private inputs but does **not** change the trusted setup assumptions. No MPC ceremony.
- **Intermediate roots** are not stored in `PoolState.roots` — only the final root is. A user can only withdraw a split note against the final root. By design (documented in `docs/notes/15-split-deposit.md` §15.5).
- **`splits` leak = split privacy lost.** If the witness leaks (A6), the on-chain privacy gain is nullified.
- **Phantom wallet format.** `signAndSendTransaction` requires `{ serialize, message: { version: 0 } }`, not a base64 string. Verified with Phantom only; Solflare and Backpack are untested (see `docs/notes/16-phantom-compat.md` §9).
- **`@noir-lang/noir_js` WASM is 3.84 MB in the initial bundle.** Static import chain reaches `App.vue`. Deferred dynamic-import optimization (see `docs/PROJECT_CONTEXT.md` §8.12).

Documented in [`docs/threat-model.md`](docs/threat-model.md):

- **A1** — `deposit` does not verify `new_root` on-chain.
- **A5** — Phishing frontend is out of scope.
- **A6** — The prover sees the full witness; for production, proving must run client-side.
- **A9** — Commitment forgery is self-harm only.
- **A10** — `ROOT_HISTORY_SIZE = 10` allows root eviction by spam-deposits.
- **Trusted setup** for Groth16 — no MPC ceremony.
- **Single-keypair** upgrade authority.

### Breaking change — v0.1.0 notes are not withdrawable

The circuit changed shape (6 public inputs, new C4/C5 constraints). Any note generated against the v0.1.0 circuit will be rejected by the v0.2.0 verifier. This is intentional — see `docs/notes/15-split-deposit.md` §4.3.

The two real deposits made on devnet during Stage 10 remain in the Merkle tree with their old roots (leaf 0, leaf 1). They cannot be withdrawn under v0.2.0.

### Statistics

- **~144 tests** across all layers (circuits 41, on-chain 39, LiteSVM 20, backend 5, merkle 23, prover 16).
- **4 workflows** in CI/CD.
- **5 containers** in `docker compose up`.
- **4 deployed programs/PDAs** on devnet (verifier, zk_pool, pool, vault) — Program IDs unchanged since v0.1.0.
- **1 full split-deposit E2E** cycle verified on devnet (deposit + 3 withdrawals + double-spend rejection).

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
- Stage 14: finalization (release `v0.1.0`).
- Stage 15: split deposit (release `v0.2.0`).

Every stage has:
- A checkpoint in `.checkpoints/NN-*/` with SHA-256 of artifacts.
- A stage note in `docs/notes/NN-*.md` (Russian).
- A record in `docs/PROJECT_CONTEXT.md`.

---

[Unreleased]: https://github.com/kwebhub/zkpool-solana/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/kwebhub/zkpool-solana/releases/tag/v0.2.1
[0.2.0]: https://github.com/kwebhub/zkpool-solana/releases/tag/v0.2.0
[0.1.0]: https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0
