# Threat Model — zkpool-solana

> **Status:** Stage 13.1, 2026-09-29
> **Scope:** on-chain program, backend, merkle service, prover, frontend.
> **Not in scope:** the ZK circuit's soundness (Sunspot + gnark-solana), Solana L1 itself.

---

## TL;DR

zkpool-solana is a **demo / educational** project. The threat model identifies what an attacker could do and what the current implementation does and does **not** protect against. Read alongside `docs/DEMO-NOTICE.md`.

**Key limitations (by design, documented):**
- Trusted setup for Groth16 — no MPC.
- `new_root` on deposit is **not verified** on-chain — a malicious depositor can corrupt the tree.
- Backend sees all commitments and nullifiers (no privacy from the operator).
- Frontend stores notes in `localStorage` — XSS risk.

---

## 1. Assets

| Asset | Where | Loss impact |
|---|---|---|
| **SOL in vault** | On-chain `vault` PDA | Direct financial loss |
| **Deposit notes** (nullifier, secret, note_secret) | User's browser / file | Loss = funds unrecoverable |
| **PoolState** (roots ring buffer, counters) | On-chain `PoolState` PDA | Corrupted pool |
| **Nullifier records** (double-spend guard) | On-chain PDAs | False positives block withdrawals |
| **Backend DB** (commitments, roots, nullifiers) | Postgres | Indexer state |
| **Merkle tree state** | Redis | Rebuildable from events |
| **Prover key** (`withdrawal.pk`) | `circuits/withdrawal/target/` | Doesn't leak secrets — but heavy compute |
| **Verifier program ID** | On-chain | Fixed |

---

## 2. Actors

| Actor | Trust level | Capabilities |
|---|---|---|
| **User (depositor)** | Trusted for own funds | Can deposit, has own note |
| **User (withdrawer)** | Trusted for own funds | Has note; can withdraw anywhere |
| **Pool operator (deployer)** | **Trusted** | Can pause? (no — no pause function). Can't steal funds (no admin keys). |
| **Malicious depositor** | **Untrusted** | Can craft arbitrary `commitment` + `new_root` |
| **Malicious withdrawer** | **Untrusted** | Can craft arbitrary witness; can't double-spend (nullifier record) |
| **Frontend host** | Partially trusted | Could inject JS — XSS risk |
| **Backend operator** | **Semi-trusted** | Sees commitments + nullifiers → links deposits to withdrawals (privacy leak) |
| **Prover** | Semi-trusted | Sees full witness (nullifier, secret) — can steal notes if malicious |
| **Merkle service** | Semi-trusted | Same as prover |
| **Devnet RPC** | Untrusted | Can censor, can't forge |
| **Sunspot / gnark-solana** | Trusted | Verifier correctness |
| **Network (P2P)** | Untrusted | Standard Solana assumptions |

---

## 3. Invariants (I)

| ID | Invariant | Enforced by |
|---|---|---|
| **I1** | Vault balance ≥ sum of unspent commitments | `deposit` transfers in; `withdraw` checks `vault.lamports() >= amount` |
| **I2** | Each nullifier used at most once | Anchor `init` on `NullifierRecord` PDA |
| **I3** | Root used for withdrawal is in history | `pool.is_known_root(&root)` |
| **I4** | Recipient in proof matches `to` account | `require_keys_eq!(recipient, ctx.accounts.to.key())` |
| **I5** | Proof verifies against public inputs | CPI to verifier program |
| **I6** | Public inputs match: `[root, nullifier_hash, recipient, recipient_binding, amount]` | `encode_public_inputs` + verifier |
| **I7** | `commitment = hash_3(nullifier, secret, amount)` | Circuit constraint C3 |

---

## 4. Attack scenarios (A)

### A1. Malicious depositor corrupts tree — **NOT PREVENTED**

**Scenario:** Attacker deposits 1 lamport with a garbage `new_root`. `deposit` accepts it (only checks `new_root != current_root`), adds to roots history. Subsequent honest deposits build on top of a corrupted root. All withdrawals against the corrupted root succeed only if the attacker knows the nullifier/secret for the fake commitment.

**Impact:** Tree integrity lost. Honest users' withdrawals may fail (`root` mismatch). Funds in vault are still safe (withdrawals still require a valid proof against a known root).

**Root cause:** No on-chain verification of `new_root = insert(commitment, old_tree)`. Documented in `deposit.rs`, §Trust model.

**Mitigation (not implemented):** verify `new_root` on-chain via Merkle path + hash. Expensive (SPL compute budget). Deferred.

**Severity:** High for tree integrity, medium for funds (funds still protected by proof requirement).

---

### A2. Double-spend — **PREVENTED**

**Scenario:** Withdrawer submits the same nullifier twice.

**Mitigation:** `NullifierRecord` PDA with seeds `[NULLIFIER_RECORD_SEED, pool, nullifier_hash]`. Anchor `init` fails if the account exists.
Verified in Stage 10.3: Error: Allocate: account FKA2Z... already in use

**Severity:** N/A (protected).

---

### A3. Front-running a withdrawal — **PREVENTED**

**Scenario:** Attacker sees a withdrawal tx in mempool, races to withdraw to their own address.

**Mitigation:** `recipient_binding = hash_2(note_secret, recipient)` is a public input. The proof is bound to a specific recipient. An attacker with the same proof can't change `recipient` — the verifier rejects.

**Severity:** N/A (protected).

---

### A4. Proof replay against different root — **PREVENTED**

**Scenario:** Attacker takes a valid proof for root R1, tries to use it when current roots are R2, R3.

**Mitigation:** `pool.is_known_root(&root)` checks the root in history (ring buffer of last 10 roots). Old roots fall out — proofs for very old roots fail.

**Severity:** N/A (protected).

---

### A5. Malicious frontend steals notes — **NOT PREVENTED**

**Scenario:** Phishing site impersonating zkpool. User enters note → attacker sends it to their server → withdraws to attacker's address.

**Impact:** Full loss of user funds.

**Mitigation (not implemented):** out of scope for demo. Users should verify the domain, use bookmarked URLs.

**Severity:** High (user responsibility).

---

### A6. Malicious backend steals notes — **NOT PREVENTED**

**Scenario:** Backend operator logs all `/api/withdraw` requests → sees `nullifier`, `secret`, `note_secret` → can construct a competing withdrawal tx.

**Impact:** Full loss of user funds.

**Mitigation (architectural):** The prover sees the witness. For production, ZK proving must happen **client-side** (or in a trusted execution environment). Currently the prover is a service.

**Severity:** Critical for production; **documented as demo limitation**.

---

### A7. XSS in note display — **PARTIALLY PREVENTED**

**Scenario:** Attacker crafts a note with HTML in a field. `DepositForm` renders it as `{{ ... }}` (Vue's text interpolation, auto-escapes). Safe.

**But:** "Copy note as JSON" — the user is expected to save the JSON somewhere. If they paste it into an unsafe context (e.g., a chat that renders HTML), the attacker could steal it.

**Mitigation:** Vue's template escaping covers the UI. Not our code's concern after the user copies the note.

**Severity:** Low.

---

### A8. Rate-limit bypass on `/api/withdraw` — **PARTIALLY PREVENTED**

**Scenario:** Attacker sends 1000 `/api/withdraw` requests to DoS the prover.

**Mitigation:** Backend rate-limits at 5/min per IP. Prover is single-threaded (mutex).

**But:** IP can be spoofed behind proxies. **Not tested under load.**

**Severity:** Medium (DoS, not funds).

---

### A9. Commitment forgery (deposit) — **NOT PREVENTED**

**Scenario:** Attacker submits a `commitment` that is not `hash_3(nullifier, secret, amount)` for any known secrets. `deposit` accepts it. The leaf exists in the tree, but no one can withdraw it (no valid witness). Funds are stuck in vault.

**Impact:** Funds effectively donated to pool. No loss to others (their leaves still work).

**Mitigation:** Not our concern — attacker harms only themselves. **However:** it grows the tree, potentially hitting `MAX_LEAVES = 2^20`.

**Severity:** Low (self-harm + tree spam).

---

### A10. Root history exhaustion — **PREVENTED**

**Scenario:** Attacker does 10 deposits in a row, pushing legitimate roots out of the ring buffer.

**Mitigation:** `ROOT_HISTORY_SIZE = 10`. A user who withdraws within 10 deposits of their own deposit is safe. Beyond that — the root is no longer known, and the withdrawal fails.

**Impact:** Time-limited withdrawals for infrequent users. **Not funds loss** — the user could re-deposit or wait for a new root history cycle... actually no, once out — the note is unspendable. This is a **real DoS vector** if an attacker spams deposits.

**Severity:** Medium. Documented limitation.

**Mitigation (not implemented):** increase `ROOT_HISTORY_SIZE`. Trivial. Currently hardcoded to 10.

---

### A11. Verifier program substitution — **PREVENTED**

**Scenario:** Attacker deploys a fake verifier that always returns success. Modifies the `verifier_program` account in the withdraw tx.

**Mitigation:** `#[account(address = VERIFIER_PROGRAM_ID)]` — Anchor enforces the exact verifier program ID.

**Severity:** N/A (protected).

---

### A12. Vault drainage via lamport manipulation — **PROTECTED**

**Scenario:** Attacker tries to withdraw more than the vault balance.

**Mitigation:** `require!(vault_lamports >= amount, ZkPoolError::InsufficientVaultBalance)`. Also, the withdrawal transfers via `try_borrow_mut_lamports` — only this program can do that (vault is a PDA of this program).

**But:** If the attacker's commitment encodes a huge `amount`, the proof would need to verify — which it can't, because the circuit binds `amount` into the commitment. The only way to withdraw a large amount is to have deposited that amount.

**Severity:** N/A (protected).

---

## 5. Summary table

| ID | Attack | Status | Severity |
|---|---|---|---|
| A1 | Corrupt tree via `new_root` | ❌ Not prevented | High (tree integrity) |
| A2 | Double-spend | ✅ Prevented | — |
| A3 | Front-running | ✅ Prevented | — |
| A4 | Replay against old root | ✅ Prevented | — |
| A5 | Phishing frontend | ❌ Not prevented | High (user) |
| A6 | Malicious prover sees witness | ❌ Not prevented | Critical (production) |
| A7 | XSS in note display | ⚠️ Partially | Low |
| A8 | Rate-limit bypass | ⚠️ Partially | Medium |
| A9 | Commitment forgery | ❌ Not prevented | Low (self-harm) |
| A10 | Root history exhaustion | ⚠️ Design limit | Medium |
| A11 | Verifier substitution | ✅ Prevented | — |
| A12 | Vault drainage | ✅ Prevented | — |

**Protected:** 5. **Partially:** 3. **Not prevented:** 4.

Of the 4 unprotected:
- **A1** — architectural (needs on-chain Merkle verification — out of scope)
- **A5** — user behavior
- **A6** — architectural (needs client-side prover)
- **A9** — self-harm

---

## 6. Trust assumptions

1. **Solana L1 works as specified.** No chain-level attacks modeled.
2. **Verifier program is correct.** Sunspot / gnark-solana audited by the community. Not re-audited here.
3. **Circuit is correct.** Soundness + completeness assumed.
4. **User protects their note.** Loss = funds loss.
5. **User trusts the frontend they use.** DNS / domain ownership matters.
6. **Poseidon2 is collision-resistant.** Standard assumption.

---

## 7. Recommendations for production

1. **Verify `new_root` on-chain.** Add Merkle path verification in `deposit` (extra account `merkle_proof`, `is_even`, verified against `pool.current_root()`).
2. **Move proving client-side.** `noir_js` in browser (already used for `computeHashes`). Full Groth16 in browser via WASM — heavy but possible.
3. **Increase `ROOT_HISTORY_SIZE`.** From 10 to at least 100. Cost: `100 * 32 = 3.2 KB` extra account space.
4. **MPC trusted setup.** Current key from a single ceremony.
5. **Upgrade authority review.** Currently a single keypair. Should be a multisig.
6. **Frontend SRI + CSP.** Prevent CDN compromise.
7. **Rate limiting by wallet, not just IP.** Currently IP-based; behind a proxy, easily bypassed.
8. **Client-side note storage.** Local file, hardware wallet, or encrypted keystore. Not `localStorage`.

---

## 8. References

- `docs/DEMO-NOTICE.md`
- `onchain/programs/zk_pool/src/instructions/deposit.rs` — Trust model section.
- `onchain/programs/zk_pool/src/instructions/withdraw.rs` — CPI layout, double-spend protection.
- `circuits/withdrawal/spec.json` — public inputs layout.
- `docs/PROJECT_CONTEXT.md` §7 — architectural decisions.
