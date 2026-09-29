# DEMO NOTICE

**This repository is a demonstration / educational project.**

It is **NOT** production-ready. It is **NOT** safe for real funds. It is **NOT** audited.

The project exists to reproduce the architecture of `solana-zk-pool` v2 with a checkpoint-based process, in order to eliminate a class of bugs around public-input layout mismatch between three layers (circuit, on-chain program, frontend).

**See also:**
- [`threat-model.md`](threat-model.md) — 12 attack scenarios (A1–A12), 7 invariants.
- [`SECURITY.md`](../SECURITY.md) — how to report vulnerabilities.
- [`CHANGELOG.md`](../CHANGELOG.md) — release history.

Everything below is **intentionally missing**, simplified, or omitted.

---

## 1. Cryptography

### 1.1. Groth16 trusted setup — NO MPC CEREMONY

`sunspot setup target/withdrawal.ccs` generates `withdrawal.pk` and `withdrawal.vk` in a **single-party trusted setup**. Whoever ran the setup **saw the toxic waste**. If they kept it, they can forge proofs.

**What is missing:**
- Multi-party computation (MPC) ceremony.
- Public ceremony transcript.
- Independent verification of the setup.

**For production:** use [reilabs/trusted-setup](https://github.com/reilabs/trusted-setup) or an equivalent MPC ceremony with public participation.

### 1.2. Verifying key — not pinned on-chain

The `withdrawal.vk` is used to build the verifier program, but there is **no separate on-chain commitment** to the VK. If the VK is regenerated, the verifier program must be redeployed — its Program ID changes. Any client with a cached Program ID will silently submit to a stale verifier.

**For production:**
- Pin the VK hash in the main program (`zk_pool`), so verifier swap is visible.
- Add a governance mechanism for verifier updates.

### 1.3. No circuit audit

The circuit is not audited. Constraints may be missing, or may be too weak.

**For production:** at least two independent audits by different firms.

---

## 2. On-chain program

### 2.1. Deposit trust model — not verified on-chain

`deposit` accepts `commitment` and `new_root` **from the client** without verifying that they are consistent. A malicious depositor can submit a garbage `new_root` and corrupt the pool for everyone.

**Threat-model reference:** A1 (corrupt tree via `new_root`).

**What is missing:**
- On-chain verification that `new_root` is the correct result of inserting `commitment` into the current tree.
- Bounds on `commitment` (must be a valid BN254 field element).

**For production:** either verify on-chain, or make the tree updateable only by a trusted indexer with strong input validation.

### 2.2. Commitment forgery possible

A depositor can submit an arbitrary 32-byte `commitment` that does not correspond to any valid `(nullifier, secret, amount)` triple. Funds are then stuck in the vault.

**Threat-model reference:** A9 (self-harm only, but grows the tree).

**For production:** verify `commitment = hash_3(...)` on-chain, or restrict deposits to a whitelist.

### 2.3. Root history — only 10 entries

`ROOT_HISTORY_SIZE = 10`. A user who does not withdraw within 10 deposits of their own deposit will find their root evicted, and their proof rejected.

**Threat-model reference:** A10 (root history exhaustion).

**For production:** increase `ROOT_HISTORY_SIZE` to ≥ 100.

### 2.4. Upgrade authority — single key

The program and verifier are upgradeable. The upgrade authority is a single keypair. If it leaks, the entire pool can be drained.

**What is missing:**
- Multisig authority (e.g. Squads).
- Timelock on upgrades.
- On-chain governance.

**For production:** Squads multisig + timelock.

### 2.5. No rate limiting on-chain

The program does not limit deposits or withdrawals per epoch. A whale can front-run, back-run, or grief.

**For production:** rate limits, per-epoch caps, or deposit-cooldown.

### 2.6. No fee mechanism

No protocol fee. No relayer fee. No incentive for anyone to run infrastructure.

**For production:** optional fee, or a public-goods funding model.

---

## 3. Backend / services

### 3.1. Prover is single-threaded and has no queue

`services/prover` calls `sunspot prove` **serialized by an async mutex** — no parallel proofs, no queue.

**Why the mutex:** `sunspot prove` always writes `withdrawal.proof` / `withdrawal.pw` (fixed names). Concurrent requests would clobber each other. Verified in Stage 7.4.

**What is missing:**
- Redis Streams (or Kafka) queue.
- Per-request temporary directories.
- Timeout and cancellation.
- Horizontal scaling.

### 3.2. Prover sees the full witness

The prover receives `nullifier`, `secret`, `note_secret` — everything needed to steal the funds. A malicious prover operator can construct a competing withdrawal transaction.

**Threat-model reference:** A6 (malicious prover).

**For production:** move proving client-side (browser Groth16 via WASM). This is the only architectural fix; a TEE is a mitigation, not a solution.

### 3.3. Backend sees commitments and nullifiers

`services/backend` indexes on-chain events and stores commitments and nullifiers in Postgres. It cannot see the witness, but it sees the timing and amounts of deposits and withdrawals. This is a metadata-leak surface.

**For production:** consider TEEs, threshold decryption, or splitting the indexer across jurisdictions.

### 3.4. Merkle service is trusted

The frontend asks the merkle service for the Merkle proof. If the service lies, the proof will not verify — but the frontend cannot detect this **before** submitting the transaction. The user pays fees for nothing.

**For production:** have the frontend verify the Merkle proof locally (it already has all commitments from `/api/commitments`).

### 3.5. No DDoS protection

Backend and merkle services have no auth, no rate limiting beyond a simple Redis-based limiter (60/min read, 5/min withdraw per IP).

**Threat-model reference:** A8 (rate-limit bypass).

**For production:** Cloudflare, WAF, per-IP limits, auth for the prover.

### 3.6. No monitoring / alerting

Prometheus and Grafana are provided as dev conveniences (`make up` starts them). Dashboard `zkpool-backend` with 6 panels. **But:** no alerts, no on-call, no runbooks.

**For production:** alerting on prover queue depth, indexer lag, DB size, error rates.

---

## 4. Frontend

### 4.1. Notes must be saved by the user — no recovery

Deposit notes are **not stored in the browser**. No `localStorage`, no `sessionStorage`, no cookies. The user must copy the note JSON and save it elsewhere.

**Positive:** no XSS surface for note theft.
**Negative:** closing the browser without saving the note = loss of funds. No recovery.

**For production:** offer encrypted note storage (key derived from user's wallet signature), but keep manual export available.

### 4.2. No CSP

The Vite dev server has no Content-Security-Policy. For production:

```
default-src 'self';
connect-src 'self' https://api.devnet.solana.com;
script-src 'self' 'wasm-unsafe-eval';
```

### 4.3. No offline mode

If the frontend cannot reach the backend, the user cannot withdraw — even though the witness could be generated locally and submitted directly to RPC.

**For production:** allow fully client-side witness generation and direct RPC submission.

### 4.4. Phishing frontend not mitigated

A phishing site impersonating zkpool can capture the note and withdraw to any address.

**Threat-model reference:** A5 (phishing frontend).

**For production:** domain with verified ownership, browser extensions, or hardware wallet integration.

---

## 5. Operations

### 5.1. CI/CD — minimal hardening

GitHub Actions workflows exist:
- `ci.yml` — 7 jobs (validate-spec, onchain, litesvm, backend, prover, merkle, web).
- `docker.yml` — build + publish `solana` image to GHCR.
- `release.yml` — tagged releases with artifacts + checksums.
- `security.yml` — `cargo audit`, `pnpm audit`, `cargo deny`.

**What is missing:**
- Actions pinned by SHA (currently by tag).
- OIDC federation for signing.
- Artifact signing (SLSA provenance).

**For production:** pin action SHAs, use OIDC, sign releases.

### 5.2. No secrets management

The Solana keypair is a plaintext file at `~/.config/solana/id.json`. `.secrets/` is gitignored, but not encrypted.

**For production:**
- Hardware wallet for authority.
- KMS / HSM for signing keys.
- No plaintext keypairs on disk.

### 5.3. No backups

Postgres and Redis have named Docker volumes but no snapshot schedule, no replication, no restore drills.

**For production:** automated backups, off-site replicas, tested restores.

### 5.4. No disaster recovery

If the prover, backend, or merkle service goes down, there is no documented recovery path beyond `make up`.

**For production:** documented runbooks, on-call rotation, SLOs.

---

## 6. Legal / compliance

- No KYC / AML.
- No sanctions screening.
- No terms of service.
- No privacy policy.
- No jurisdiction analysis.

**For production:** full legal review.

---

## 7. What IS in the project

See [`CHANGELOG.md`](../CHANGELOG.md) and [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md).

**Highlights:**

- **Circuit:** Groth16 (BN254) with `recipient_binding` for front-running protection, 5 public inputs, `TREE_DEPTH=20`. 41 Noir tests.
- **On-chain:** Anchor program `zk_pool` with `pool`, `deposit`, `withdraw`. Nullifier records prevent double-spend (verified in Stage 10.3). 37 unit + 15 LiteSVM tests (7 adversarial).
- **Backend:** Rust + axum. HTTP API + indexer (Postgres + Redis). Prometheus metrics.
- **Merkle service:** Node.js + Fastify, Poseidon2 via `@noir-lang/noir_js`. 23 tests.
- **Prover:** Rust + axum, Groth16 via `nargo` + `sunspot`. Mutex-serialized.
- **Frontend:** Vue 3 + TypeScript + Pug + SCSS. Wallet connect, browser-side `noir_js`, Codama client.
- **E2E:** full cycle verified twice on devnet (deposit → withdraw). Double-spend protection verified.
- **Security:** [`threat-model.md`](threat-model.md) — 12 scenarios, 7 invariants.
- **CI/CD:** 4 workflows. Dependabot (7 cargo + 2 npm + gh-actions).
- **Infrastructure:** `make up` — one-command start. Prometheus + Grafana provisioned.

**Statistics:**
- ~140 tests across all layers.
- 4 deployed programs/PDAs on devnet.
- 2 full E2E cycles with Groth16 verification on-chain.

---

## 8. Bottom line

**Do not use this with real funds.**

If you want to use it in production:

1. Run an MPC trusted setup.
2. Audit the circuit and the program.
3. Fix the deposit trust model (**A1**).
4. Move proving client-side (**A6**).
5. Increase `ROOT_HISTORY_SIZE` (**A10**).
6. Multisig + timelock upgrade authority.
7. Rate limiting by wallet, not IP (**A8**).
8. Legal review.

This document is updated as the project grows — see [`CHANGELOG.md`](../CHANGELOG.md).
