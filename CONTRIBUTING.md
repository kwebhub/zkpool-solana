# Contributing to zkpool-solana

Thanks for your interest. This project is a **demo / educational** build — contributions are welcome, but please read this first.

> ⚠️ **Not for production.** See [`docs/DEMO-NOTICE.md`](docs/DEMO-NOTICE.md) and [`docs/threat-model.md`](docs/threat-model.md).

---

## Before you start

1. **Read the [PROJECT_CONTEXT.md](docs/PROJECT_CONTEXT.md).** It's the single source of truth for project rules, structure, and known pitfalls. Every decision in this repo traces back to it.
2. **Skim [docs/notes/](docs/notes/).** The notes are organized by stage — one file per stage, in Russian. They explain the *why* behind the code.
3. **Run `make up`** to verify the environment works. If it doesn't, open an issue before submitting a PR.

---

## Setup

```bash
git clone https://github.com/kwebhub/zkpool-solana
cd zkpool-solana
make up
```

That's it. All services run in Docker. No local Rust/Node/Solana install required.

### Verify

```bash
make status
```

Expected output: 5 containers up, 3 app services healthy (backend 4001, prover 4002, merkle 4003).

---

## Workflow

### 1. Branch

```bash
git checkout -b feat/short-description
# or: fix/, docs/, chore/, refactor/, test/, ci/
```

### 2. Small, focused changes

**One concern per PR.** Not "fix bug + add feature + refactor". Split them.

### 3. Run tests before pushing

```bash
# Full test suite (all layers)
make exec-c CMD='cd /home/ubuntu/onchain && cargo test -p zk_pool --lib'
make exec-c CMD='cd /home/ubuntu/tests && cargo test'
make exec-c CMD='cd /home/ubuntu/services/backend && cargo test --lib'
make exec-c CMD='cd /home/ubuntu/services/prover && cargo test --lib'
make exec-c CMD='cd /home/ubuntu/services/merkle && pnpm test'
make exec-c CMD='cd /home/ubuntu/web && pnpm typecheck && pnpm build'
```

CI runs these on every push. Red CI = no merge.

### 4. Commit messages

**Conventional Commits:**

```
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

**Types:**
- `feat` — new feature
- `fix` — bug fix
- `docs` — documentation
- `refactor` — code change that neither fixes nor adds
- `test` — adding tests
- `chore` — build, deps, config
- `ci` — CI/CD changes
- `perf` — performance

**Scopes** (examples): `onchain`, `backend`, `prover`, `merkle`, `web`, `circuits`, `infra`, `ci`, `e2e`, `security`.

**Examples:**

```
feat(web): wallet connect via window.phantom.solana
fix(prover): nargo execute fails on names with dots
docs(threat-model): add A11 verifier substitution
ci(security): upgrade sqlx to 0.8 (RUSTSEC-2024-0363)
```

### 5. Open a PR

The [PR template](.github/PULL_REQUEST_TEMPLATE.md) has a checklist. Fill it.

---

## Code style

### Rust

- `cargo fmt` before commit (default rustfmt, 4-space indent).
- `cargo clippy -- -D warnings` — no warnings.
- Follow existing patterns. Read adjacent files.

### TypeScript / Vue

- `pnpm typecheck` must pass.
- Prettier defaults (Vite + Vue).
- Composables live in `web/src/composables/use*.ts`.

### Python / shell

- Only used in CI. No strict rules.

---

## What gets merged

### ✅ Welcome

- **Bug fixes** — with a test that reproduces the bug.
- **Tests** — more coverage, especially adversarial cases.
- **Documentation** — improvements to `README.md`, `docs/*.md`, `docs/notes/*.md`.
- **Small features** — that fit the existing architecture.
- **CI improvements** — faster, more reliable workflows.

### ⚠️ Discuss first

Open an issue before starting work on:

- **New services** — architectural decision.
- **Circuit changes** — breaks deployed verifier.
- **Public inputs layout change** — breaks every layer.
- **New dependencies** — supply-chain implications.
- **Refactors that touch more than 3 files.**

### ❌ Not accepted

- **Changes that break the demo.** The project must remain runnable via `make up`.
- **Removing tests.** Ever.
- **"Simplifying" by deleting information from docs.** See rule §0.11 in `PROJECT_CONTEXT.md`.
- **Changing license.** MIT stays.

---

## Pitfalls

Common issues — see [PROJECT_CONTEXT.md §8](docs/PROJECT_CONTEXT.md#8-known-pitfalls). Highlights:

- **`anchor init` doesn't create a subdirectory.**
- **`sunspot --version` is not supported.**
- **`nargo execute -p <name>` requires `<name>` with no dots.**
- **`Prover.toml` hex must be `0x`-prefixed.**
- **Anchor Borsh: `Vec<u8>` needs a `u32 LE` length prefix.**
- **Groth16 verification uses ~182k CU; default 200k is not enough.**
- **`NULLIFIER_RECORD_SEED = b"nullifier_record"`** (not `b"nullifier3"`).

---

## Reporting security issues

**Do not open a public issue.** See [SECURITY.md](SECURITY.md).

---

## Getting help

- **Open an issue** — bugs, questions, feature discussions.
- **Read the notes** — `docs/notes/` has stage-by-stage reasoning.
- **Read `PROJECT_CONTEXT.md`** — most "why is it like this" questions are answered there.

---

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
