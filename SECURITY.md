# Security Policy

## Supported versions

This project is a **demo / educational build**. Only the latest commit on `main` is maintained.

| Version | Supported |
|---------|-----------|
| `main`  | ✅ |
| `0.2.x` | latest patch only |
| `< 0.2.0` | ❌ |

---

## ⚠️ Important context

**zkpool-solana is not production software.** Before reporting, please read:

- [`docs/DEMO-NOTICE.md`](docs/DEMO-NOTICE.md) — what this project is.
- [`docs/threat-model.md`](docs/threat-model.md) — 12 modeled attack scenarios.

**Some attack scenarios are documented as accepted limitations** (not bugs). If your finding is already listed in the threat model under "Not prevented" or "Partial", it's known — please reference the threat-model entry in your report.

**Known accepted limitations:**

- **A1** — `deposit` does not verify `new_root`. A malicious depositor can corrupt the tree.
- **A5** — Phishing frontend is not mitigated (user responsibility).
- **A6** — The prover sees the full witness. Running the prover as a service means the operator can steal funds.
- **A9** — Commitment forgery (self-harm only).
- **A10** — `ROOT_HISTORY_SIZE = 10` is small; spam-deposits can evict a legitimate root.
- **Trusted setup for Groth16** — no MPC ceremony.
- **Single-keypair upgrade authority.**
- **Backend sees commitments + nullifiers** — no privacy from the operator.
- **Solflare withdraw is unsupported** — Solflare's `signAndSendTransaction` requires a `@solana/web3.js@1.x` `Transaction` instance; our `@solana/kit`-wire object fails with `JsonRpcError: Internal error`. See `docs/notes/16-phantom-compat.md` §9.
- **Backpack wallet is untested.**

If you have a report that is **not** in this list, we want to know.

---

## How to report a vulnerability

### Private disclosure (preferred)

Use [GitHub Security Advisories](https://github.com/kwebhub/zkpool-solana/security/advisories/new) — this creates a private channel visible only to maintainers.

**Please include:**

1. **Summary** — one sentence on what the issue is.
2. **Impact** — what an attacker can do. Which asset is at risk.
3. **Affected component** — on-chain program, backend, prover, merkle service, frontend.
4. **Reproduction** — exact steps, with commands if applicable.
5. **Suggested mitigation** — if you have one.
6. **Threat-model cross-reference** — if it's an existing scenario, cite the ID (A1–A12).
7. **Credit** — how you'd like to be acknowledged (or "anonymous").

### Response timeline

| Stage | Target |
|---|---|
| Acknowledgment | 3 business days |
| Initial assessment | 7 business days |
| Fix or mitigation plan | 14 business days |
| Public disclosure | after fix ships, or 90 days from report |

**Best-effort only.** This is a solo-maintained educational project, not a company with a security team.

---

## Scope

### In scope

- **On-chain programs:**
  - `zk_pool` — `8cGzkFK9H15mcpndAaY7ApCJhkHcujttR4E2D8rS6LCm`
  - Sunspot verifier — `5t51iu6apRxgLbt91eVZ6YYzHsnmBCVnLGqJtqdfMFWJ`
- **Backend** — `services/backend/` (HTTP API, indexer, DB).
- **Prover** — `services/prover/`.
- **Merkle service** — `services/merkle/`.
- **Frontend** — `web/`.
- **Circuits** — `circuits/`.
- **Scripts** — `scripts/`.

### Out of scope

- **Solana L1** — report to [Solana Foundation](https://solana.com/security).
- **Anchor framework** — report to [Coral XYZ](https://github.com/coral-xyz/anchor/security).
- **Noir / nargo** — report to [Noir Lang](https://github.com/noir-lang/noir/security).
- **Sunspot / gnark-solana** — report upstream.
- **`@solana/kit`, `@noir-lang/noir_js`, Codama, Fastify** — report upstream.
- **Third-party crate advisories** — handled by Dependabot + `cargo audit` in CI.

### Minimum bar

Not accepted as vulnerabilities:

- **Theoretical attacks without a demonstration.**
- **"Best practice" suggestions** (e.g. "add CSP") — open a regular issue instead.
- **Denial of service via legitimate high load** — rate limits exist; bypassing them may be in scope if trivial.
- **Anything listed under "accepted limitations" above** — open a discussion, not a security report.
- **Broken links, typos, cosmetic issues.**
- **Solflare withdraw failure** — documented, accepted, architectural (not a security issue).

---

## What we do

### Automated

- **`cargo audit`** on every PR + weekly (`security.yml`).
- **`pnpm audit`** on every PR + weekly.
- **`cargo deny`** — advisories, licenses, bans, sources.
- **Dependabot** — weekly PRs for Rust (7 crates), Node (2 dirs), GitHub Actions (monthly).

### Manual

- **Threat model** — [`docs/threat-model.md`](docs/threat-model.md).
- **Adversarial tests** — [`tests/src/test_adversarial.rs`](tests/src/test_adversarial.rs).
- **Split-deposit tests** — [`tests/src/test_deposit_split.rs`](tests/src/test_deposit_split.rs).
- **Backend input validation** — every endpoint validates shape, length, encoding.
- **On-chain constraints** — Anchor account validation, PDA seeds, `require_keys_eq!`.

---

## Disclosure policy

Once a vulnerability is fixed:

1. A GitHub Security Advisory is published.
2. A note is added to `CHANGELOG.md` under "Security".
3. Credit is given (unless requested otherwise).

**We will not:**
- Sue you for good-faith research.
- Demand silence after the fix ships.
- Ignore reports because they're "not severe enough".

**We will:**
- Acknowledge every report.
- Tell you honestly if we can't fix it.
- Credit your work.

---

## Bug bounty

**No bug bounty.** This is a demo project without funding.

Credit in `CHANGELOG.md` and the security advisory is the only reward we can offer.

---

## Contact

- **Security reports** — [GitHub Security Advisories](https://github.com/kwebhub/zkpool-solana/security/advisories/new).
- **General questions** — open a regular issue.
- **Maintainer** — [@kwebhub](https://github.com/kwebhub).
