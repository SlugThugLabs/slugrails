# COACH_NOT_COP.md (Enforcement Philosophy)

## Overview
The "Coach‑Not‑Cop" philosophy provides a tiered enforcement model that balances developer productivity with strict architectural safety. It is designed for SlugRails’ CI/CD pipelines and marketing material.

### 1. Fast Mode – Coaching Tips Only
* **Goal:** Accelerate development while surfacing potential issues.
* **Behavior:** The system emits **non‑blocking suggestions** (e.g., “Consider extracting this function to reduce nesting”).
* **Use‑case:** Early‑stage PRs, exploratory branches, or local `cargo check` runs.

### 2. Balanced Mode – Random Sampling Review
* **Goal:** Provide quick, high‑value feedback without full‑scale analysis.
* **Behavior:** Randomly selects **2–5 changed files** per PR/commit and runs the full suite of guards on them. Results are shown as **informational warnings**.
* **Use‑case:** Feature branches approaching merge, CI runs where fast feedback is essential.

### 3. Strict Mode – Comprehensive Gatekeeper
* **Goal:** Guarantee architectural invariants before production deployment.
* **Behavior:** Executes **all CodeGuards checks** on the full diff, failing the CI job on any violation.
* **Use‑case:** Release branches, `main`/`master` merges, CI pipelines for production.

## Coaching Reports
In addition to raw guard output, the system generates **positive reinforcement reports**:
* “✅ Clean architectural streak: 5 commits without violations.”
* “🎉 3 new modules added while respecting layer dependencies.”

These reports are intended to celebrate good practices, reinforcing the coach‑style mindset.

---
*Place this file in `docs/COACH_NOT_COP.md` on `master`.*
