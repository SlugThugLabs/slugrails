# SlugRails — Architectural Guardrails for AI-Assisted Software Development

**SlugRails preserves engineering intent and prevents architectural drift by turning agreed-upon software architecture into persistent, machine-enforceable constraints.**

**Core philosophy: Coach, Not Cop.** SlugRails helps developers and AI coding agents establish the right architecture, then embeds safeguards into the project's existing development workflow. It is not a background daemon, another AI coding agent, or simply a linter. It is the track layer that keeps implementation aligned with the original engineering decisions.

At its foundation, SlugRails has two responsibilities:

1. **Guide architectural discovery.** Provide the instructions and tools that help an existing AI coding agent work with the developer to understand what they actually want to build, determine appropriate engineering practices, and agree on how strictly those decisions should be enforced.

2.Make those decisions enforceable. Translate that discussion into a configurable, machine-enforceable architectural contract, create or select the necessary guards and tests, and integrate enforcement into the project's existing development and testing workflow.

After setup, routine enforcement is triggered automatically through the project's normal development workflow, such as Git hooks, test runners, build tools, or CI/CD pipelines. These triggers invoke SlugRails, which executes the required checks locally, connects to SlugRails-managed enforcement services when applicable, or combines both approaches.

Routine enforcement must not require an active AI conversation or interaction with the SlugRails MCP server. Results are returned directly to the existing testing workflow using standard pass/fail behavior and actionable diagnostics.

After setup, routine enforcement must not require reconnecting to the SlugRails MCP server or maintaining an active AI conversation. It runs through normal development and testing workflows.

## The Problem SlugRails Solves

AI coding agents can produce working code while gradually degrading the underlying software architecture. As development progresses, they may introduce unnecessary dependencies, duplicate implementations, oversized modules, hidden coupling, inconsistent patterns, or shortcuts that violate earlier design decisions.

Even when an AI understands the architecture initially, that understanding can deteriorate across long sessions, context resets, different models, and successive development tasks.

Traditional linters and test suites catch many technical mistakes, but they generally do not preserve a project's unique architectural intent.

**SlugRails makes architectural decisions durable, independently verifiable, and enforceable regardless of which human or AI writes the code.**

## How SlugRails Works

SlugRails follows a straightforward six-stage lifecycle:

**Discover → Define → Authorize → Install → Enforce → Evolve**

### 1. Discover and Define the Architecture

During project planning, SlugRails helps developers and AI agents examine requirements, identify responsibilities, establish module boundaries, and select appropriate engineering constraints.

The developer remains the decision-maker. SlugRails helps translate those decisions into a machine-readable architecture contract, typically stored in `.planning/ARCHITECTURE.md`, with supporting configuration where necessary.

Contracts are project-specific, not universal prescriptions. A small utility, web application, and distributed system should not be forced into the same architecture.

SlugRails should help developers make informed choices rather than impose arbitrary limitations that could produce worse software.

### 2. Turn Architectural Intent into Enforceable Rules

A contract explicitly identifies which rules apply. These can include:

- **Architectural boundaries:** Permitted module and layer dependencies, dependency direction, and separation of responsibilities.
- **Complexity limits:** File sizes, function lengths, nesting depth, and other measurable structural constraints when appropriate.
- **Reliability requirements:** Restrictions on unchecked panics, inappropriate `unwrap()`, swallowed errors, unfinished stubs, and debug code.
- **Dependency policies:** Allowed or forbidden packages, crates, and dependency relationships.
- **Project-specific rules:** Additional constraints selected or defined for the particular software being built.

Rules are modular and individually selectable. SlugRails should execute only the constraints explicitly enabled by the contract, and any required rule it cannot evaluate must be reported as unevaluated—not silently treated as passing.

SlugRails supports three sources of guards:

1. **SlugRails-provided guards:** Reusable rules supplied and maintained by SlugRails, including locally available free guards and optionally managed commercial guards.
2. **Developer-authored guards:** Custom guards written, imported, and maintained by developers in their own rule directories, with no artificial limits on the number they may create or execute.
3. **AI-assisted custom guards:** Project-specific guards created or refined with AI assistance. SlugRails may offer limited free allowances and paid or metered creation services. Developers retain the ability to create their own guards without using those services.

All three use a common enforcement interface and consistent evaluation semantics. Free and paid functionality must not require incompatible enforcement engines.

Guard implementations and reusable test definitions are versioned independently of individual projects. Contracts identify the required guards and their compatible versions so enforcement remains reproducible. Missing, unsupported, or unverifiable guards must never silently pass.

Enforcement records the exact guard implementations, versions, and configuration used so results can be reproduced and independently verified.

The guard architecture must accommodate local developer-owned directories, SlugRails-managed guard storage, managed updates, and optional hosted capabilities without coupling the enforcement engine to a particular commercial provider.

### 3. Authorize and Protect the Contract

Once the architecture is agreed upon, it becomes the project's authoritative engineering contract.

SlugRails separates project-specific architectural configuration from the enforcement machinery responsible for validating it.

The project may contain its contract and configuration under `.planning/`, including `.planning/ARCHITECTURE.md` and `.planning/slugrails/`. However, executable guard implementations, reusable test definitions, and enforcement infrastructure are maintained outside the governed project.

These may come from:

- A local SlugRails installation.
- Developer-owned external rule directories.
- SlugRails-managed guard storage and services.

A dedicated installation such as `~/.slugthuglabs/slugrails/` is one possible local location.

This separation keeps project repositories clean and discourages autonomous coding agents from weakening checks simply by editing nearby test files.

An autonomous agent may propose architectural changes or request exceptions, but it must not independently approve changes to the constraints governing its own work.

The security boundary must ultimately depend on filesystem permissions, trusted execution environments, independently controlled CI, or equivalent access controls—not merely on hiding the location of enforcement files.

Guard storage location is not itself a security boundary. Authoritative enforcement and contract approvals must remain protected from agents that can modify the project or their local development environment.

Commercial access controls are distinct from architectural authorization. A customer's subscription status must not determine whether an architectural violation is considered valid, whether a required check has passed, or whether an agent can approve its own changes.

### 4. Install Enforcement Into Native Developer Tools

SlugRails integrates with a project's existing build, test, and version-control workflows without scattering its internal test machinery throughout the project.

Its intended directory separation is:

```text
my-project/
├── src/
├── Cargo.toml
├── .planning/
│   ├── ARCHITECTURE.md
│   └── slugrails/
│       └── project configuration
└── .git/
    └── hooks/
        └── pre-commit

~/.slugthuglabs/slugrails/
├── guards/           # Locally available guards
├── tests/            # Reusable enforcement tests
├── contracts/        # Protected baselines / authority metadata
└── runtime/          # Enforcement tools and supporting resources

developer-owned-rules/
└── ...                # Custom guards maintained by the developer

SlugRails managed services (optional)
├── managed guards
├── guard versions
├── project guard management
└── AI-assisted guard creation
```

The paths and managed-service components are illustrative, not a finalized storage or deployment specification.

The project contains only the minimal configuration and integration necessary to activate enforcement. Actual guard evaluation takes place through SlugRails' enforcement engine or compatible managed evaluation infrastructure.

Native integrations can include:

- **Git hooks:** Invoke the SlugRails engine automatically during commits.
- **Rust workflows:** Integrate architectural verification with Cargo's normal testing or CI workflow, using minimal generated glue only where technically required.
- **Other language toolchains:** Integrate through their appropriate native hooks, test runners, or build commands.
- **CI/CD:** Run the same enforcement semantics as an independently controlled verification gate.
- **Machine-readable reporting:** Produce compiler-style diagnostics, JSON, and SARIF without adding reporting infrastructure to the governed project.

Developers should not need to invoke SlugRails manually during ordinary development. Once configured, enforcement runs through the project's existing workflow.

Local enforcement of developer-owned and locally available guards must not require cloud connectivity. Managed services may add hosted capabilities, synchronization, independent verification, or managed guard distribution.

Whether commercial guards are distributed for local execution, evaluated remotely, or supported through both approaches remains an implementation decision. This must not compromise the integrity of enforcement results.

The project remains focused on its own application code, while SlugRails retains responsibility for its enforcement mechanisms.

### 5. Evaluate Deterministically and Report Accurately

During enforcement, SlugRails collects source evidence, analyzes code structure and dependencies, and evaluates the enabled architectural rules.

Its Rust implementation uses filesystem traversal, lexical analysis, source metrics, guard implementations, and structured reporting without needing to execute the project's untrusted application code during static inspection.

Results must clearly distinguish:

- **PASS:** The authorized contract was verified, all mandatory guards were available and executed, all required evidence was inspected, and no blocking violations were found.
- **VIOLATION:** The implementation breaches an enforced architectural rule.
- **INCOMPLETE:** Required evidence or rule evaluation was unavailable.
- **ERROR:** Enforcement itself could not complete.

An incomplete scan must never masquerade as a successful one. Diagnostic output should identify the offending location, violated rule, and practical remediation.

When a required managed guard is unavailable because of service interruption, missing authorization, or other delivery failure, enforcement must not falsely report PASS. The system should use a valid previously authorized local copy where supported, or report an appropriate incomplete or error condition.

Billing and licensing decisions must remain separate from rule evaluation. The engine reports verifiable engineering results rather than modifying those results according to commercial entitlements.

### 6. Evolve Deliberately

Architectures are allowed to change. SlugRails is not intended to freeze a project permanently.

When requirements change, a developer can revisit the architectural decisions, approve adjustments, and establish a new authorized contract. Legitimate exceptions may also be approved without weakening the entire rule set.

The distinction is simple: architectural evolution is deliberate and authorized; architectural drift is accidental or unauthorized.

## Product and Commercial Model

SlugRails is designed around a **free, functional local enforcement foundation combined with optional paid management, maintained guard services, and AI-assisted guard creation**.

The business model must not depend on artificially restricting a developer's ability to create architectural rules or enforce their own decisions.

### Community / Free

The free product provides:

- Architectural discovery and contract creation.
- Core deterministic enforcement.
- A useful foundational collection of guards.
- Unlimited local evaluation.
- The ability to create, import, and execute unlimited developer-owned guards.
- Support for developer-maintained custom rule directories.
- Integration with native development and testing workflows.
- A limited allowance of SlugRails-provided AI-assisted guard creation, if offered.

A developer must be able to build and maintain a fully functional architectural enforcement system using their own rules without purchasing managed services.

### Paid / Managed

Revenue comes from services and capabilities that provide continuing value:

- **Managed guard services:** Access to SlugRails-maintained guard implementations, reusable enforcement definitions, updates, and compatible versioned packages.
- **Hosted guard management:** Protected storage, distribution, synchronization, recovery, and management of project-specific guards and enforcement configurations.
- **AI-assisted guard creation:** Paid allowances or usage-based generation, testing, refinement, and maintenance of custom guards.
- **Advanced management:** Capabilities that reduce the operational work involved in maintaining architectural protection across projects.

AI-assisted creation allowances should control usage of the generation service, not the number of developer-owned guards a customer can maintain or execute.

The service should not require a developer to pay repeatedly merely to execute their own previously created guards.

A customer may use SlugRails-managed capabilities alongside their own unlimited local rules. Both must retain consistent enforcement behavior.

### Future Cloud / Teams

An expanded managed platform may provide:

- Centralized architectural policy management.
- Shared organizational guard libraries.
- Team permissions and independent approvals.
- Protected contract authorization.
- Audit history and change tracking.
- Managed CI integrations and independently verified enforcement.
- Cross-project management and reporting.

These capabilities address coordination, trust, administration, and operational convenience rather than replacing the foundational enforcement engine.

### Infrastructure and Commercial Boundaries

SlugRails may use infrastructure providers such as Fly.io to host managed guards, project-specific enforcement resources, AI generation services, and supporting APIs.

**The product must not be architecturally dependent on a particular hosting provider.**

Local guard execution, hosted guard delivery, and remote evaluation should be treated as distinct capabilities with clear interfaces.

Normal source-code inspection and guard evaluation should remain local wherever practical. Hosted execution may be offered where its stronger service-side control or independent verification justifies the operational and privacy costs.

Licensing, billing, guard distribution, cloud communication, and AI inference must remain separate from deterministic guard evaluation.

Commercial entitlements control access to optional services and managed content, not the truthfulness or correctness of architectural verification.

The system must handle paid-service unavailability without misrepresenting enforcement results.

**Commercial principle:** SlugRails provides a free, fully functional architectural enforcement foundation with unlimited developer-authored rules. Revenue comes from managed guard services, professionally maintained enforcement capabilities, and metered AI-assisted guard creation and refinement. Paid services enhance convenience, coverage, and management without preventing developers from building and enforcing their own architectural rules.

The architecture must support commercial expansion without requiring the core enforcement system or existing project contracts to be redesigned.

## What SlugRails Is—and Is Not

SlugRails is an **architectural contract builder and enforcement system** that complements AI coding agents, linters, compilers, test frameworks, and dedicated auditing tools.

It does not replace engineering judgment, decide every architectural choice for the developer, or guarantee that passing code is defect-free.

It is not intended to force every developer into the same architectural rules, require a paid guard library, or restrict developers from creating their own enforcement systems.

Its purpose is narrower and more fundamental: **ensure that the architectural decisions made during planning remain visible, measurable, and enforceable throughout implementation.**

SlugRails is designed to remain lightweight, understandable, and independent of any particular AI provider, hosting provider, or coding environment.

Its local enforcement should work even when the AI that originally helped design the project is no longer involved.

## Current Implementation vs. Intended Design

SlugRails already has a Rust implementation with architectural contracts, modular guards, source analysis, CLI and MCP interfaces, exception-token handling, structured reports, and native-toolchain integration.

However, some intended guarantees—particularly independent human authorization, tamper-resistant contracts, complete enforcement of required rules, consistent fail-closed behavior, independent guard storage, and unified support for developer-owned and managed guard sources—remain areas requiring verification or further engineering.

Hosted guard management and the revised commercial model are intended capabilities, not claims that the underlying infrastructure has already been implemented.

These are **requirements of the intended product**, not claims that every capability is already fully implemented.

---

## The Bottom Line

**SlugRails helps humans and AI design the right software architecture, records their decisions as an enforceable contract, and installs persistent guardrails into native development tools so future code changes cannot silently undermine those decisions.**

Developers remain free to create and enforce their own rules. SlugRails earns revenue by making architectural enforcement easier to create, maintain, protect, and manage—not by preventing developers from doing it themselves.

*The goal is not merely code that works today. It is software that remains architecturally sound as people, models, requirements, and implementations change.*




## Shared SlugThugLabs Project Foundation — SlugPlan

SlugRails uses **SlugPlan**, the shared project workspace foundation for the entire SlugThugLabs ecosystem.

Every SlugThugLabs product must integrate the same SlugPlan foundation. Whichever product a developer installs or initializes first establishes the standard project workspace. Subsequent products detect and reuse it, adding only their own configuration and state without duplicating or disrupting existing information.

The shared workspace standard is:

```text
my-project/
├── AGENTS.md
├── src/
├── docs/
└── .planning/
    ├── README.md
    ├── ARCHITECTURE.md
    ├── jira/
    │   └── PROJECT_PLAN.md
    ├── adr/
    └── <product>/
```

**Workspace responsibilities:**

- `AGENTS.md` provides a common entry point for AI coding agents, with compatibility mechanisms for agents requiring different instruction files.
- `.planning/ARCHITECTURE.md` contains the authoritative system architecture, engineering decisions, and invariants.
- `.planning/jira/PROJECT_PLAN.md` contains active development plans, tasks, and milestones.
- `.planning/adr/` preserves architectural decision records and their rationale.
- `.planning/<product>/` provides isolated configuration and appropriate project-specific state for each integrated tool.
- `.planning/README.md` documents the shared workspace conventions.
- `docs/` is intended for end-user documentation, while internal engineering documentation belongs under `.planning/`.

**Implementation requirements:**

1. SlugPlan's executable functionality must be implemented entirely in **Rust**, with no Python runtime or Python implementation dependency.
2. A reusable Rust core provides workspace initialization, discovery, validation, repair, versioning, and migration.
3. Every SlugThugLabs product uses this shared implementation rather than developing competing workspace management systems.
4. Each product remains independently installable and must not require a separate SlugPlan installation or another SlugThugLabs product.
5. An optional standalone `slugplan` CLI exposes the same underlying functionality.
6. Workspace initialization must be idempotent, preserve existing developer-authored content, and never silently overwrite or destroy project information.
7. Existing repositories must be supported through inspection and deliberate, non-destructive migration.
8. Individual products own their respective namespaces. SlugPlan owns shared workspace conventions, not the architectural decisions or implementation plans documented within them.
9. Shared workspace management must function locally without mandatory cloud connectivity.

**Integration with SlugRails:**

During architectural discovery, SlugRails uses SlugPlan to initialize or recognize the project's workspace and load existing architectural context.

SlugRails then performs its own architectural discovery, contract generation, authorization, and enforcement installation.

SlugPlan manages project organization and shared agent context. SlugRails manages architectural enforcement. These responsibilities must remain distinct.

Executable guards, reusable enforcement tests, and independently protected authorization metadata remain outside the governed project's `.planning/` workspace, either in external local storage or SlugRails-managed infrastructure.

**Architectural principle:** Every SlugThugLabs product starts from the same project foundation, regardless of installation order, without introducing duplicate implementations, inconsistent conventions, or mandatory dependencies between products.