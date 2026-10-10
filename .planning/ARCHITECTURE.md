# SlugRails — Engineering Architecture & Quality Contract

**Status:** Proposed development contract — human approval required  
**Applies to:** The SlugRails source repository and the work of every human or AI contributor  
**Purpose:** Build the product defined by `VISION.md` and the approved project plan without losing engineering quality, architectural integrity, tests, or previously approved decisions.

> **This is not a specification for the product's future enforcement technology.** It does not prescribe a customer-facing guard engine, registry, guard format, catalog, feature set, pricing model, or implementation schedule. Those questions are decided through the vision, project plan, and separately approved technical decisions when the relevant work is undertaken. Here, *check*, *verification*, and *control* mean independent development-quality checks unless explicitly stated otherwise.

## 1. Authority and scope

1. `VISION.md` defines **what product** is to be delivered and why. The approved project plan defines **scope, order, and acceptance criteria**. This document defines **how code must be engineered and verified** throughout that work.
2. Engineering changes must preserve the intended product behavior and approved requirements. Where documents conflict or an engineering standard obstructs a genuine product requirement, **stop and request a human decision**; do not weaken either silently.
3. Numeric thresholds below are strict **proposed SlugRails development standards**, not defaults for customers. They become binding on approval. Agents cannot relax them because compliance is difficult.
4. The actual repository is the starting point. **Inspect, preserve, repair, and extend existing usable implementation.** A rewrite or wholesale reorganization requires a documented technical case and human approval.
5. This document cannot approve itself. Changes to its substantive requirements, limits, exclusions, verification commands, or protected Cargo settings require explicit human review; no AI-issued token, comment, or in-file status is proof of approval.
6. Do not claim that a requirement is automatically enforced until its actual checking mechanism is installed, exercised, and shown to reject an intentionally failing example. Unimplemented checks remain **verification gaps**, not passes.

## 2. Core engineering commitments

- **Correctness over appearances:** Working behavior and reproducible evidence matter more than an impressive demonstration, a high test count, or a clean-looking architecture diagram.
- **Clarity over abstraction:** Responsibilities, ownership, data flow, and failure handling should be obvious. Reject speculative frameworks, redundant wrapper layers, and indirection without demonstrable benefit.
- **Use the best existing tool:** Use Cargo, rustc, rustfmt, Clippy, Cargo-compatible utilities, standard testing tools, and proven libraries where they enforce the real requirement. Build a focused independent checker **only where existing tooling is insufficient**. Never weaken the invariant to fit the tool.
- **Strong standards without harmful proxies:** Numerical limits are appropriate where they measure a meaningful property. Do not optimize a proxy at the expense of simpler, safer, more maintainable code.
- **No hidden shortcuts:** No abandoned temporary bypasses, silent downgrades, fabricated evidence, or production behavior replaced by a mock.
- **Verify the behavior, not merely the declaration:** A command mentioned in Cargo or CI is not evidence that it ran, passed, or caught the intended violation.

## 3. Strict Rust development profile

The following limits govern first-party Rust source. Their methods of counting and exclusions must be defined once, checked consistently, and protected from unauthorized modification.

| Requirement | Proposed bound | Verification |
|---|---:|---|
| Production Rust source file length | **200 physical lines** | Existing independent repository checker; scan all `src/**/*.rs` |
| Production function/method length | **50 physical lines**, including signature and body | Parser-aware analysis or validated independent checker |
| Control-flow nesting depth | **4** | Parser-aware analysis with known-good/bad tests |
| Function parameters | **5**, excluding receiver | Parser-aware check |
| Cyclomatic complexity | **10** | Defined, tested metric; no unexamined text-count proxy |
| Cognitive complexity | **15** | Tool with documented semantics and adversarial tests |
| Test file length | **300 physical lines** | Independent repository checker; no weakened splitting |
| Measured line coverage | **85% minimum** | Approved actual coverage instrument with exclusions disclosed |

**Rules of application:**

1. Production-vs-test classification must be correct; test-only source must not be misclassified. Generated and vendored files require specific preapproved scope rules, not improvised exclusions.
2. Limits must not encourage deceptive fragmentation. Splitting an oversized function into tangled helpers with shared mutable state fails architectural review even if every helper is short.
3. A metric that cannot reliably inspect macros, attributes, modules, or supported syntax must report its coverage limitation; a regex that merely approximates an AST invariant cannot be sold as proof.
4. Maintain `#![forbid(unsafe_code)]` in first-party Rust crate roots. Adding `unsafe` is prohibited without an approved architectural policy change and independent safety review.
5. No production `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, avoidable `panic!()`, or equivalent deliberate crash paths. Tests may use `unwrap()` and `expect()` to fail loudly when their own preconditions fail.
6. Errors must be propagated or intentionally handled. Preserve source/context at boundaries. Do not silently swallow `Result`, substitute success defaults for failures, hide critical failures behind `let _ =`, or treat unreadable inputs as clean.
7. Public library APIs should expose useful, stable error types where callers need to distinguish failures. Preserve provenance of internal failures without exposing secrets or inappropriate internal details.
8. No leftover debug prints, commented-out discarded code, unused workarounds, unfinished production TODOs, or blanket lint suppressions. Intentional CLI messages and structured diagnostics are legitimate output.
9. Avoid multiple adjacent boolean arguments, cryptic API names, unbounded recursion, global mutable state, hidden cross-module coupling, and unnecessary public visibility. Prefer explicit domain types and small cohesive interfaces.
10. Every externally visible capability needs useful documentation and realistic usage examples. Verify executable documentation examples; don't count token code blocks as working examples.
11. Keep functions and modules testable through real interfaces. Do not manufacture abstractions solely to make a trivial metric pass.

## 4. Architecture and dependencies

### Existing-code-first boundaries

The repository already contains Rust modules including `analyzer`, `cli`, `contract`, `error`, `guards`, `library`, `menu`, `server`, `storage`, `style`, `types`, and `util`. **These are the actual codebase to audit**, not a mandate to replace them with a speculative nine-module layout.

1. Before changing module boundaries, capture the **observed** module/dependency graph and responsibilities. Identify cycles, upward dependency leaks, duplicated parsing or policy logic, and misplaced concerns.
2. Establish an **approved dependency-direction allowlist** based on actual responsibilities and the product work in scope. Add a structural check that rejects forbidden edges and handles Rust module syntax correctly. An unreviewed guessed graph is not authoritative.
3. Domain models, persistence, user interfaces, protocol adapters, and execution orchestration should have clear ownership. User-interface adapters must not reimplement core decision logic; shared libraries must not depend on presentation or transport simply for convenience.
4. No cyclic dependencies, unexplained singleton state, or cross-layer helpers that conceal prohibited coupling. Distinct functions may share tested generic utilities without coupling unrelated decisions.
5. Avoid new workspace crates, services, databases, build systems, or external infrastructure without an evidence-backed need. Keep deployment and development workflows understandable.
6. Do not change architecture to conform to a template merely because a coding agent prefers that template.

### Cargo dependency policy

There is **no universal hard maximum** on direct dependency count or transitive graph depth. Those numbers should be measured and reported, not used to force an inferior reimplementation.

1. Every new direct dependency must have a written purpose, the capability it adds, and a brief comparison with already available facilities. Reuse mature, maintained tools and crates when appropriate.
2. New, removed, replaced, upgraded, downgraded, renamed, or feature-reconfigured dependencies are **governed changes**. Detect them against the approved Cargo baseline; require justification and approval where they change an established engineering decision.
3. Check duplicate functionality, dependency fitness, maintenance status, supported platform/toolchain compatibility, and unnecessary enabled features.
4. Resolve the **actual** dependency graph, including workspace inheritance, optional features, target-specific dependencies, and `Cargo.lock`. Audit resolved versions, not just strings in `Cargo.toml`.
5. Run maintained security and license policy checks. A known applicable severe security issue or prohibited license is a release blocker absent explicit documented risk acceptance. Unknown or unverified data is not a pass.
6. Report direct and transitive counts, graph depth, and significant growth as diagnostics/review signals. Do not impose `max_direct_dependencies = 25` or `max_transitive_dependency_depth = 7` without a demonstrated product-specific reason.
7. Avoid network use in routine offline code paths unless required by the specific feature being implemented; do not introduce implicit telemetry or model calls.

## 5. Cargo and tooling configuration preservation

**Explicit requirement:** Anything intentionally added to the approved Cargo development configuration must **be checked for presence, preserved against unauthorized weakening/removal, and demonstrated to work**.

This applies, as relevant, to:

- `Cargo.toml` and workspace manifests: package/workspace settings, Rust edition and minimum supported toolchain decisions, resolver choice, features, lints, targets, build profiles, dependencies and dev-dependencies.
- `Cargo.lock`, committed toolchain selection (when used), rustfmt and Clippy configuration, coverage configuration, dependency-audit policy, and justified exception lists.
- Development/CI commands and their invocation settings, including formatting, compilation, tests, coverage, dependency checking, and security scanning.

### Correct mechanism, not cargo-shaped fiction

- A Rust crate needed by source code belongs in the appropriate Cargo dependencies table.
- A Cargo subcommand such as `cargo-deny`, `cargo-audit`, or a coverage utility may be an **installed tool**, not a valid dependency in the project's `[dev-dependencies]`. Govern it through the appropriate installation/pinning, configuration, and CI execution mechanism.
- A setting such as `resolver`, `[lints]`, or `[workspace.lints]` belongs in the correct manifest location for the actual project structure and Rust edition. Do **not** hardcode an outdated resolver value simply to match a template.
- A tool declared in configuration but never invoked does **not** satisfy the requirement. A tool that runs but fails to detect a known-bad fixture is not sufficiently validated.

### Protected-change procedure

1. Record an approved **baseline of governed configuration** with the exact files, relevant semantic settings, expected commands, and permitted environment-specific differences.
2. On every applicable development/CI verification, compare current configuration with this baseline using structured parsers where possible, not superficial text searches.
3. Report missing required entries, changed severity levels, removed commands, added bypass flags, altered coverage exclusions, disabled security checks, changed features, and changed dependencies. Do not accept weakened settings because tests now pass.
4. Require explicit human authorization to update the baseline. An AI agent may prepare a proposed change and its rationale; it cannot self-authorize it.
5. Re-run the actual validation commands after an approved change. Demonstrate at least one known-bad case is rejected by each critical checking mechanism.
6. The baseline must not be quietly rewritten to match an unauthorized working tree. An external or independently managed copy plus protected CI is preferred for anti-drift assurance; a file editable by the same agent is not a hard security boundary.
7. Do not introduce custom Cargo-checking software when a proven, reliable parser or existing utility can do the job correctly; implement only the missing verification logic.

## 6. Mandatory development verification

The project must maintain an **independently runnable** verification entry point. It must not rely on unfinished SlugRails functionality to prove SlugRails is being developed correctly.

Minimum required stages, adapted only by approved changes to supported features/toolchains:

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release --locked
```

And, through correctly installed and maintained tooling:

- Coverage instrumentation reporting **at least 85% line coverage** across the approved included first-party source, with all exclusions enumerated.
- Dependency license, advisory, lockfile and policy validation; for example `cargo deny check` or an equivalent validated workflow. Network-dependent advisory refresh may run in CI/release verification rather than the offline local check path.
- Repository invariants: file lengths, test/source separation if retained as an approved project standard, module existence, and any approved dependency-direction checks.
- Security/secret scanning, basic package/install checks, and target-platform checks on the supported release matrix.
- Regression and adversarial test fixtures for each custom checker, especially cases deliberately designed to bypass naive matching.

**Verification integrity:**

1. A checker must exit nonzero when the condition is violated. Missing configuration, missing executables, skipped mandatory stages, malformed reports, or unreadable required inputs cannot produce a successful overall result.
2. Record exactly which commands ran, their versions, environment/target, results, and failed or omitted prerequisites. Preserve output that an independent reviewer can reproduce.
3. CI must execute the same intended checks on clean checkouts. Test the gate with a deliberately invalid change before claiming it protects the project.
4. A shell script that says PASS based on filenames or strings, without checking the actual invariant, is insufficient. Repair deficient checks rather than weakening the requirement.
5. When a tool does not support a requested syntax/feature/configuration, record an explicit gap and resolve it. Never silently turn off a check.
6. Passing a test suite establishes only what the suite actually verifies. It does not certify missing features, customer experience, or commercial readiness.
7. Native Cargo tools and independent scripts are the v1 development verification authority. **Do not block v1 development on building its own product mechanisms or claim that unfinished product mechanisms are enforcing this document.**

## 7. Testing, correctness, and anti-softening

1. Tests must verify externally meaningful behavior, failure behavior, and invariants—not merely implementation details or the existence of a symbol/file.
2. Each substantive behavior change should have tests appropriate to its risks: unit, integration, regression, negative, and failure-injection where relevant. Public surfaces must have meaningful behavioral coverage, not mechanical one-test-per-function counting.
3. Validate both good and bad cases. If a requirement says a condition is prohibited, prove that the corresponding negative test actually fails for the intended reason.
4. Measure coverage, but never equate **85% coverage** with 85% correctness. Do not introduce artificial calls or trivial assertions to inflate coverage.
5. Real external integration requires real integration evidence. Unit-test fakes are permissible when clearly scoped, but cannot be the sole proof of a production workflow.
6. Do not remove tests, decrease assertions, change expected outputs, approve snapshots indiscriminately, add `#[ignore]`, exclude source from coverage, or change test gates just because implementation is failing.
7. A legitimate modification to tests or quality policy must demonstrate that original intent is preserved or explicitly document the authorized behavioral change. No hidden weakening through renaming, restructuring, or replacement with easier checks.
8. Test exceptions and suppressions must be specific, justified, and reviewed. No broad temporary allowances or permanent disabling of failing stages.
9. Include targeted coverage of corrupted/missing input, denied permissions, unexpected file types, partial writes, interrupted operations, invalid configuration, and version mismatches as applicable.
10. When an existing test is wrong, prove why and repair it with stronger evidence; don't use a failed test as automatic permission to revise the requirement.
11. Document honestly which requirements are verified, partially verified, unverified, and not yet implemented.

## 8. Reliability, security, performance, and portability

### Reliability

- Use explicit error types and actionable context; never misreport a partial, failed, or skipped operation as success.
- Design durable writes with atomic replacement/recovery where required. Verify interrupted-write and restart behavior for persisted state.
- Bound retries, queues, timeouts, tasks, file sizes, and resource usage according to realistic inputs. Establish cancellation and shutdown behavior for asynchronous tasks.
- Do not block async executors with blocking I/O or sleeps; do not hold incompatible locks across `.await`. Use existing Rust lints and focused tests where effective.

### Security

- Treat external inputs, paths, serialized data, and environment configuration as untrusted until validated.
- Prevent traversal, symlink-following mistakes, command/shell injection, secret disclosure, and accidentally broad file permissions. Path safety requires actual boundary/symlink handling, not a blanket call to `canonicalize()`.
- No hardcoded credentials, sensitive values in logs, or insecure release defaults.
- Review third-party dependencies, build scripts, licenses, and advisories; resolve or explicitly accept applicable risks through the proper approval process.
- State actual authorization and filesystem trust assumptions. **Same-user files and processes are not isolated security principals**; never claim otherwise.

### Performance

- Establish representative workloads, baseline timings, and memory measurements for significant operations.
- Investigate unexpectedly expensive algorithms, repetitive full-project scans, unnecessary allocations, pathological inputs, and regressions. Prefer proven profiling over invented benchmark claims.
- Add numeric performance gates only after appropriate evidence and target-platform consideration.

### Platform compatibility

- Use `std::path` and appropriate platform APIs rather than assuming Unix separators or shells.
- Define text/line-ending canonicalization **per data contract**: normalize where semantically appropriate, preserve raw bytes where integrity or exact-byte comparison requires them.
- Test supported operating systems, supported Rust toolchains/targets, filename encodings, line endings, process behavior, and filesystem permissions. Don't claim an untested platform works.

## 9. Build hygiene and release quality

1. Keep build/toolchain requirements explicit and reproducible within supported environments. Use locked dependency resolution for validated releases; do not promise byte-identical binaries across incompatible environments without evidence.
2. Review warnings, lint suppressions, feature flags, release profiles, conditional compilation, debug behavior, and emitted logs. No accidental debug-only behavior in production paths.
3. Verify documented commands, install/uninstall and upgrade behavior, persisted-data compatibility where applicable, and actual supported platform packaging.
4. Preserve readable diagnostics and predictable exit codes; verify actual user-facing failure messages and recovery paths.
5. Maintain accurate user documentation. A feature not implemented or verified must not be described as shipped merely because a command or interface exists.
6. An independent release review must consider security, performance, maintainability, testability, UX, integrations, platform compatibility, technical debt, and commercial suitability—not simply code style.
7. The owner's commercial software audit rubric is the evaluation framework, with a **target of independently defensible 9.8+/10 overall**. The score is an ambition, not an automatic promise or a machine-checkable invariant. Critical release blockers cannot be overridden by a favorable average.

## 10. Development conduct and truthful completion

1. **Preservation-first:** Inventory existing source and behavior; classify each area as **KEEP / REPAIR / COMPLETE / REPLACE** with evidence. Prefer the smallest correct fix.
2. Work in prerequisite order. Do not claim end-to-end capability until every real prerequisite has been implemented and independently verified.
3. Track work as **PLANNED / IMPLEMENTED / VERIFIED**. A file, scaffold, mock demo, or green test that misses the intended behavior does not establish VERIFIED.
4. No fake or narrowed implementations, hidden stubs, forged test data, optimistic completion summaries, or tests rewritten to conform to faulty behavior.
5. Do not redesign the product, expand scope, add unrelated systems, or reinterpret `VISION.md` while fixing ordinary code defects. Product decisions belong in the plan and human review.
6. If a requirement cannot currently be verified, state what is missing and what would establish proof. Never claim all requirements passed when mandatory evidence is missing.
7. Major changes require a change record: problem, alternatives considered, chosen solution, affected standards/tests, compatibility consequences, and explicit approval where governed.
8. AI agents do not approve their own exemptions or relax standards to reduce work. A legitimate exception has a named rule, constrained scope, rationale, consequences, expiry/review point where appropriate, and recorded human approval.
9. Do not delete working source or test assets in pursuit of a cleaner theoretical architecture without documented benefit and approval.
10. No v1 self-hosting prerequisite: first build and validate SlugRails using **independent mature development checks**. Whether/when SlugRails should govern its own later development is a separate product/project decision, **not part of this contract**.

## 11. Definition of engineering compliance

A change is compliant only when:

- The implementation serves the currently approved product requirement without silently reducing it.
- Existing behavior is preserved or a change in behavior is explicitly approved.
- The applicable Rust, architecture, Cargo/tooling, testing, reliability, security, and portability requirements pass their **actual independent checks**.
- Required test evidence exercises the intended behavior and relevant failure modes.
- Protected development configuration and approved tests have not been removed, weakened, or silently rebaselined.
- Any unverified obligation or absent verification mechanism is disclosed as a gap; no unsupported overall PASS is claimed.
- The change is understandable, reviewable, maintainable, and proportionate to the actual engineering problem.

**Governing rule:** Build what `VISION.md` calls for, in the order the approved plan calls for, using the strongest appropriate existing engineering tools and independently verified code. No agent may exchange correctness, test integrity, or approved engineering intent for easier implementation or a more impressive-looking completion report.
