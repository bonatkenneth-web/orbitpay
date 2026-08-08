# OrbitPay – Drips Wave Contribution Plan

OrbitPay is a non-custodial recurring payments protocol on Stellar Soroban.
The core contract is stable. The Wave program opens the surrounding
infrastructure to community contributors through scoped, sprint-sized issues.

---

## How Issues Are Structured

Each issue is tagged with a difficulty label (`good-first-issue`, `intermediate`,
`advanced`) and the acceptance criteria a PR must satisfy to be merged.
Every issue has a clear definition of done.

---

## Categories of Work

### 1. Bug Fixes
Small, well-scoped issues that fix confirmed problems in the contract or tooling.

| # | Title | Difficulty |
|---|-------|------------|
| B-1 | `execute_pay` should return `InsufficientAllowance` error instead of propagating raw token error | intermediate |
| B-2 | `deploy.sh` fails silently when `stellar` CLI is not on PATH | good-first-issue |
| B-3 | TTL bump constants not configurable at deploy time | intermediate |

---

### 2. New Features
Additive work that extends the protocol without breaking existing behaviour.

| # | Title | Difficulty |
|---|-------|------------|
| F-1 | **Keeper Bot** – Node.js service that polls `is_payment_due` and executes charges | advanced |
| F-2 | **Merchant Dashboard** – Next.js app streaming `pay_exec` events to show subscribers and revenue | advanced |
| F-3 | **Subscriber Portal** – React UI to view and cancel active subscriptions | intermediate |
| F-4 | Add `max_payments` field to `Subscription` so subscriptions auto-expire after N charges | intermediate |
| F-5 | Add `pause_sub` / `resume_sub` so subscribers can freeze billing without cancelling | intermediate |
| F-6 | Helper view returning all active subscriptions for a given subscriber | good-first-issue |

---

### 3. Testing
Expanding coverage beyond the 25 existing unit tests.

| # | Title | Difficulty |
|---|-------|------------|
| T-1 | Fuzz test `create_sub` with random amount and interval values using `proptest` | intermediate |
| T-2 | Integration test: full subscribe → charge × 3 → cancel cycle against a local Stellar node | advanced |
| T-3 | Test that `upgrade` correctly migrates storage across a simulated contract version bump | advanced |
| T-4 | Add test asserting `pause` blocks all 3 state-mutating entry points, not just `execute_pay` | good-first-issue |

---

### 4. Documentation
Improvements that make the project easier to understand and integrate.

| # | Title | Difficulty |
|---|-------|------------|
| D-1 | Write `CONTRIBUTING.md` covering branch naming, commit style, and PR checklist | good-first-issue |
| D-2 | Add inline sequence diagram (Mermaid) to README showing the full payment lifecycle | good-first-issue |
| D-3 | Document all contract entry points with parameter types and error codes in a `SPEC.md` | intermediate |
| D-4 | Write keeper bot setup guide (env vars, cron vs event-driven, error handling) | intermediate |

---

### 5. DevOps / Tooling
Infrastructure that makes the project easier to run and ship.

| # | Title | Difficulty |
|---|-------|------------|
| O-1 | GitHub Actions CI: run `cargo test` and `cargo clippy` on every PR | good-first-issue |
| O-2 | Add `cargo audit` step to CI to catch known CVEs in dependencies | good-first-issue |
| O-3 | Automated testnet deploy on merge to `main` using `deploy.sh` in CI | intermediate |
| O-4 | Add `wasm-opt` size-report comment to PRs that touch contract code | intermediate |

---

## Sprint Cycle

Issues are batched into two-week sprints. Maintainers label issues `sprint-active`
at the start of each sprint. Contributors comment to claim; maintainers assign
within 24 hours. PRs inactive for 5 days are unassigned so others can pick them up.

---

## Acceptance Criteria (all issues)

- Code follows existing style (`cargo fmt`, zero `clippy` warnings)
- New logic is covered by at least one test
- PR description references the issue number and describes how it was tested
