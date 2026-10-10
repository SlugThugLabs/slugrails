# Commercial Retail Software Audit Baseline & Rubric

Perform a 100% honest, comprehensive, evidence-based audit of this codebase as if you were a principal engineer performing due diligence before approving a commercial production release.

Do **not** assume the code is correct. Attempt to prove where it will fail.

Base every finding on concrete evidence from the codebase. For every issue include:
* **Severity:** (Critical, High, Medium, Low)
* **Confidence:** (High, Medium, Low)
* **File(s):** Exact path and lines
* **Function/Class:** Exact symbol name
* **Root cause:** Technical explanation
* **User impact:** Direct impact on end-user or system
* **Recommended fix:** Precise, actionable resolution (no generic advice)

---

# Audit Sections & Evaluation Criteria

## Executive Summary
- Overall Score (0–10)
- Production Readiness (0–100%)
- Commercial Readiness (0–100%)
- Maintainability (0–100%)
- Technical Debt (Low / Medium / High / Severe)
- Estimated effort to reach production quality
- Summary of strengths, weaknesses, highest-risk areas, and top 5 priorities.

## 1. Architecture Review
- Pattern, Layering, Module boundaries, Dependency direction, Coupling & Cohesion.
- Core principles: Separation of Concerns, SRP, DIP, OCP, ISP, DRY, KISS.
- Concrete violations: Circular dependencies, God objects, tight coupling, hidden dependencies, leaky abstractions, incorrect ownership, missing layers.

## 2. Code Quality & Physical Limits
- Physical constraints: File <= 200 lines, function <= 50 lines, nesting <= 4, params <= 5.
- Worst code smells (Rank top 10 offenders): Duplicate code, dead code, long functions, poor naming, magic numbers, side effects, global state, inconsistent patterns.
- Descriptive naming: No unapproved single-letter variables.

## 3. Complexity Analysis
- Estimated cyclomatic & cognitive complexity rankings.
- Identification of high-risk maintenance hot spots.

## 4. Reliability & Robustness
- Production failure risks: unwrap(), expect(), panic paths, resource leaks, lock contention, deadlocks, error propagation, unhandled partial failures.
- Zero panic paths in production (`src/`).

## 5. Security Review
- Concrete vulnerabilities: Input validation, path traversal, command injection, secret leakage, unsafe deserialization, dependency vulnerabilities, permissions. Realistic exploit scenarios required.

## 6. Performance Review
- Memory allocations, excessive cloning, buffering, I/O efficiency, startup latency.
- Flag `.clone()` on types or avoidable heap churn.

## 7. User Experience Review
- CLI usability: Error messages, recovery from failures, responsiveness, help text clarity, exit codes.

## 8. UI ↔ Backend Integration
- Verify commands are fully wired to real engines (no stub handlers, fake success messages, placeholder logic).
- Measure: % features that actually work end-to-end.

## 9. Testing & Negative Fixtures
- Measured coverage (target >= 85%).
- Negative fixture fidelity: Does the test harness verify rejection of intentional architectural violations?
- Boundary cases: exact 200 vs 201 line files, 50 vs 51 line functions.

## 10. Maintainability
- Onboarding friction for a new senior engineer: Documentation clarity (`///` on all public symbols), module organization, build ergonomics.

## 11. Platform Compatibility
- Windows, Linux, and macOS compatibility: path separators (`std::path::PathBuf` vs hardcoded slashes), line endings, shell assumptions, terminal escape codes.

## 12. Production Readiness
- Logging, structured diagnostics, configuration safety, exit codes (0 = PASS, 1 = VIOLATION, 2 = ERROR), versioning.

## 13. Technical Debt
- Ranked backlog of debt items with severity, cost if ignored, and effort to resolve.

## 14. Final Verdict & Scorecard
- Pass/Fail principal engineer verdict.
- Scorecard (0–10 across all 13 dimensions).
- Overall Grade: A / B / C / D / F.
- Prioritized remediation roadmap (Critical, High, Medium, Low, Future).
