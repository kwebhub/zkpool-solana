# DEMO NOTICE

**This repository is a demonstration / educational project.**

It is **NOT** production-ready. It is **NOT** safe for real funds. It is **NOT** audited.

The project exists to reproduce the architecture of `solana-zk-pool` v2 with a checkpoint-based process, in order to eliminate a class of bugs around public-input layout mismatch between three layers (circuit, on-chain program, frontend).

Everything below is **intentionally missing**, simplified, or omitted.

---

## 1. Cryptography

### 1.1. Groth16 trusted setup — NO MPC CEREMONY

`sunspot setup target/withdrawal.ccs` generates `withdrawal.pk` and `withdrawal.vk` in a **single-party trusted setup**. Whoever ran the setup **saw the toxic waste**. If they kept it, they can forge proofs.

**What is missing:**
- Multi-party computation (MPC) ceremony.
- Public ceremony transcript.
- Independent verification of the setup.

**For production:** use the [reilabs trusted-setup](https://github.com/reilabs/trusted-setup) or an equivalent MPC ceremony with public participation.

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

**What is missing:**
- On-chain verification that `new_root` is the correct result of inserting `commitment` into the current tree.
- Bounds on `commitment` (must be a valid BN254 field element).

**For production:** either verify on-chain, or make the tree updateable only by a trusted indexer with strong input validation.

### 2.2. Upgrade authority — single key

The program and verifier are upgradeable. The upgrade authority is a single keypair. If it leaks, the entire pool can be drained.

**What is missing:**
- Multisig authority (e.g. Squads).
- Timelock on upgrades.
- On-chain governance.

**For production:** Squads multisig + timelock.

### 2.3. No rate limiting on-chain

The program does not limit deposits or withdrawals per epoch. A whale can front-run, back-run, or grief.

**For production:** rate limits, per-epoch caps, or deposit-cooldown.

### 2.4. No fee mechanism

No protocol fee. No relayer fee. No incentive for anyone to run infrastructure.

**For production:** optional fee, or a public-goods funding model.

---

## 3. Backend / services

### 3.1. Prover is single-threaded and has no queue

`services/prover` calls `sunspot prove` synchronously. Parallel requests **race** on `withdrawal.proof` / `withdrawal.pw` files. No timeout. No queue.

**For production:**
- Redis Streams (or Kafka) queue.
- Per-request temporary directories.
- Timeout and cancellation.
- Horizontal scaling.

### 3.2. Backend sees commitments and nullifiers

`services/backend` indexes on-chain events and stores commitments and nullifiers in Postgres. It **cannot** see the witness (nullifier, secret, note_secret), but it sees the timing and amounts of deposits and withdrawals. This is a metadata-leak surface.

**For production:** consider TEEs, threshold decryption, or splitting the indexer across jurisdictions.

### 3.3. Merkle service is trusted

The frontend asks the merkle service for the Merkle proof. If the service lies, the proof will not verify — but the frontend cannot detect this **before** submitting the transaction. The user pays fees for nothing.

**For production:** have the frontend verify the Merkle proof locally (it already has all commitments from `/api/commitments`).

### 3.4. No DDoS protection

Backend and merkle services have no auth, no rate limiting beyond a simple Redis-based limiter. Anyone can hammer them.

**For production:** Cloudflare, WAF, per-IP limits, auth for the prover.

### 3.5. No monitoring / alerting

Prometheus and Grafana are provided as dev conveniences. There are no alerts, no on-call, no runbooks.

**For production:** alerting on prover queue depth, indexer lag, DB size, error rates.

---

## 4. Frontend

### 4.1. Secrets in `localStorage`

Deposit notes (including secrets) are stored in `localStorage` in plaintext. Any XSS reads them.

**For production:**
- Encrypt notes with a key derived from the user's wallet signature.
- Or use WebAuthn / secure enclave.
- Strict CSP, no third-party scripts.

### 4.2. No CSP

The Vite dev server has no Content-Security-Policy. Any injected script runs with full privileges.

**For production:** strict CSP, Subresource Integrity, no inline scripts.

### 4.3. No offline mode

If the frontend cannot reach the backend, the user cannot withdraw — even though the proof could be generated locally.

**For production:** allow fully client-side witness generation and direct RPC submission.

---

## 5. Operations

### 5.1. No CI/CD hardening

GitHub Actions workflows will be added at Stage 10. They do not:
- Sign artifacts.
- Use OIDC federation.
- Pin action SHAs (only versions).

**For production:** pin SHAs, use OIDC, sign releases.

### 5.2. No secrets management

The Solana keypair is a plaintext file in a Docker volume. `.secrets/` is gitignored, but it is not encrypted.

**For production:**
- Hardware wallet for authority.
- KMS / HSM for signing keys.
- No plaintext keypairs on disk.

### 5.3. No backups

Postgres and Redis have named Docker volumes but no snapshot schedule, no replication, no restore drills.

**For production:** automated backups, off-site replicas, tested restores.

### 5.4. No disaster recovery

If the prover, backend, or merkle service goes down, there is no documented recovery path.

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

## 7. What IS in the project (for completeness)

- Correct Groth16 circuit with `recipient_binding` for front-running protection.
- Poseidon2-based commitments and nullifiers.
- `spec.json` as single source of truth + `validate-spec` CLI.
- `sync-circuits` CLI with SHA-256 check/apply.
- 41 tests across 4 Noir circuits.
- LiteSVM E2E test (planned, Stage 4.5).
- Backend, merkle, prover, frontend (planned, Stages 5–8).
- Threat model (planned, Stage 11).

---

## 8. Bottom line

**Do not use this with real funds.**

If you want to use it in production:
1. Run an MPC trusted setup.
2. Audit the circuit and the program.
3. Fix the deposit trust model.
4. Multisig + timelock upgrade authority.
5. Fix the frontend secret storage.
6. Add queueing to the prover.
7. Legal review.

This document will be updated as the project grows.
