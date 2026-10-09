# SlugRails (`slugrails-mcp`)

> Architectural rail builder and contract scaffolding for software projects and AI coding agents.

SlugRails lays down the architectural rails for your repository. It doesn't need to run as an intrusive background daemon or active process—instead, it scaffolds machine-enforceable contracts (`.planning/ARCHITECTURE.md`) and native test harnesses (`tests/slugrails.rs`) so that your existing toolchain (**`cargo test`**, **`git commit`**, and CI/CD pipelines) triggers and enforces architectural boundaries automatically.

---

## Core Philosophy: The Architectural Rail Builder

> [!IMPORTANT]
> **SlugRails is NOT an active background bouncer or linter replacement—it is the Rail Builder.**

Linters (Clippy, ESLint, Ruff) enforce syntax conventions, formatting preferences, and localized code idioms. **SlugRails operates at a completely different tier of the engineering lifecycle**:

1. **Lays Down the Rails:** SlugRails establishes machine-enforceable boundaries at the repository level: layer isolation DAGs, file volume ceilings (≤ 200 LOC), control-flow depth, security policies, and unhandled errors. Once laid down, standard developer tools like `cargo test` trigger enforcement automatically.
2. **Intent Stabilization:** Autonomous agents work best when human intent is established early and reinforced continuously. Through **SlugPlan**, SlugRails helps discover domain terms before coding starts, converging fuzzy design ideas into hard, verifiable contracts.
3. **Tamper-Proof Invariance:** Agents cannot weaken, mock, or comment out in-repo unit assertions to pass checks. Guard definitions and verification run black-box outside agent-tamperable code paths with BLAKE3 HMAC cryptographic integrity.

---

## Two-Tier Storage Architecture

- **Global Hub:** `~/.slugthuglabs/slugrails/tests/` (reusable guard templates, global catalogs, and secret keys)
- **Local Project Hub:**
  - `<project>/.planning/ARCHITECTURE.md` (the core architectural contract with TOML frontmatter)
  - `<project>/.planning/slugplan/` (intent declarations and project milestones)
  - `<project>/.planning/slugrails/` (exceptions and anti-drift tokens)
  - `<project>/.planning/slugaudit/` (evidence database when auditing)

---

## SlugPlan: Intent Convergence Engine & Architectural Interview

Autonomous coding begins with intent convergence. SlugPlan interviews the developer or agent, establishes domain boundaries, and scaffold architectural contracts.

### CLI Architectural Interview (`slugrails plan`)
```bash
# Conduct an architectural interview and scaffold contracts
slugrails plan --intent "Payment gateway integration with idempotent webhook processing" --style "hexagonal"

# Supported architectural styles:
#   - layered (domain -> application -> infrastructure)
#   - hexagonal / ports-and-adapters (domain -> ports -> adapters -> driver)
#   - cli / tool (domain -> engine -> cli)
```

Running `slugrails plan`:
1. Generates `.planning/ARCHITECTURE.md` populated with dependency matrices and recommended guards.
2. Records `.planning/slugplan/intent.json` with high-level architectural goals.
3. Automatically scaffolds `tests/slugrails.rs` for daemon-free `cargo test` enforcement.

### Agent Workflow via MCP (`slugplan_probe`)
AI agents (Hermes, Claude Code, Cursor, Antigravity) use the `slugplan_probe` tool during high-level planning:
```json
{
  "description": "High-throughput telemetry ingestion pipeline",
  "language": "rust",
  "style": "layered",
  "auto_scaffold": true
}
```
When `auto_scaffold` is `true`, SlugRails establishes the complete contract and test harness in a single turn, enabling the agent to immediately code within hard bounds.

---

## Standalone Test Harness: Native `cargo test` Enforcement

SlugRails provides a zero-daemon integration via `slugrails scaffold-tests`:

```bash
slugrails scaffold-tests
```

This generates `tests/slugrails.rs`:
```rust
#[test]
fn test_architecture_contract_enforced() {
    // Triggers SlugRails check on all source files
    // Fails `cargo test` with compiler-grade ANSI reports if rules are breached
}
```

### Why Native `cargo test` Matters:
- **No Background Processes:** Does not require a running MCP server, background daemon, or containerized service. Standard `cargo test` or `cargo test --test slugrails` triggers the rails natively.
- **CI/CD Integration:** Plugs directly into standard GitHub Actions, GitLab CI, or pre-push gates with zero setup overhead.
- **Tamper Resistance:** Agents executing `cargo test` cannot bypass architectural constraints by modifying source code in `src/`.

---

## Active Guard Tests Catalog

SlugRails ships with built-in, pure-Rust tokenizer guard checks:

| Guard Identifier | Category | Purpose & Behavior |
| :--- | :--- | :--- |
| **`complexity/deep-nesting`** | `complexity` | **Newly Active.** Flags functions exceeding maximum control flow nesting depth (default: `<= 4`). Prevents arrow anti-patterns and forces early returns or decomposition. |
| **`hygiene/swallowed-errors`** | `hygiene` | **Newly Active.** Flags discarded results, suppressed errors, and empty error catch/match arms (`Err(_) => {}`). Enforces explicit `?` propagation or logging. |
| **`hygiene/no-secrets`** | `hygiene` | **Newly Active.** Entropy- and pattern-based scanner detecting hardcoded API keys, bearer tokens, private keys, and credential strings before commit. |
| **`languages/rust/unsafe-policy`** | `languages/rust` | **Newly Active.** Strictly enforces crate-level `#![forbid(unsafe_code)]` and flags unexempted `unsafe` blocks or functions in production code. |
| `complexity/source-limits` | `complexity` | Enforces production code line budget per file (default: 400 lines; strict internal repo ceiling: 200 LOC). |
| `complexity/function-limits` | `complexity` | Enforces maximum function/method length (default: 80 lines). |
| `quality/no-stubs` | `quality` | Rejects `todo!`, `unimplemented!`, or unfinished placeholder stubs in production paths. |
| `hygiene/no-debug-prints` | `hygiene` | Rejects leftover `println!`, `dbg!`, or `console.log` statements in production code. |
| `languages/rust/no-unwrap` | `languages/rust` | Prohibits bare `.unwrap()` and `.expect()` calls in production Rust source code. |
| `structural/layer-dependencies`| `structural` | Enforces strict DAG import hierarchy based on `[allowed_dependencies]` in `ARCHITECTURE.md`. |
| `structural/docs-drift` | `structural` | Bidirectional verification between declared module map and filesystem reality. |

---

## Repository Invariants Verification

SlugRails enforces 4 absolute quality and architectural invariants on its own codebase, verified via automated script:

```powershell
pwsh -File .\scripts\verify_invariants.ps1
```

1. **File Line Budget:** Every source file in `src/` has `<= 200 LOC` (hard ceiling).
2. **Zero Unsafe:** Zero occurrences of `unsafe` code in `src/`, enforced by crate-level `#![forbid(unsafe_code)]`.
3. **Zero Test Pollution:** Zero `#[cfg(test)]` or `#[test]` attributes in `src/` (tests isolated exclusively to `tests/`).
4. **Module Disk Alignment:** All modules declared in `src/guards/checks/mod.rs` exist on disk.

---

## AI Agent Integration (Hermes, Claude Code, Cursor, Antigravity)

### Registering the MCP Server

```bash
# Hermes Agent
hermes mcp add slugrails --command "slugrails serve"
```

For clients configuring via `mcp.json` (Cursor, Claude Desktop, Antigravity, OpenCode):
```json
{
  "mcpServers": {
    "slugrails": {
      "command": "slugrails",
      "args": ["serve"]
    }
  }
}
```

---

## Agent Enforcement & Coding Loop

During normal coding, verification happens via fast CLI triggers:

```bash
# Check modified / staged files in active git diff (<5ms)
slugrails check

# Run full project scan across all files
slugrails check --all

# Validate architecture contract
slugrails validate

# Install automated git pre-commit hook
slugrails hook install
```

### User-Authorized Exceptions
When a file genuinely requires an exemption, agents cannot self-authorize. The user grants an exception token:

```bash
# User grants exception
slugrails exception add src/server/transport.rs complexity/source-limits --reason="Combined transport scaffolding"

# List active exceptions
slugrails exception list

# Revoke an exception
slugrails exception revoke <token>
```

---

## Build & Install

```bash
cargo build --release
cp target/release/slugrails /root/.slugthug/bin/slugrails
```

---

## License & Commercial Terms

SlugRails is licensed under the **Functional Source License, Version 1.1, Apache 2.0 Change License (`FSL-1.1-Apache-2.0`)**.

- **100% Free for Developers & Internal Teams:** You are free to use SlugRails internally to develop, analyze, audit, test, and maintain your own software—including commercial products and services. You own 100% of the code and products you build with it.
- **Protection Against Commercial Resale & AI Bundling:** Third parties and AI platforms may not offer the functionality of SlugRails as a competing commercial product or service, nor bundle or embed it into a paid commercial offering without an agreement from SlugThug Labs.
- **Automatic Apache 2.0 Conversion:** Every release automatically converts to the fully permissive **Apache License 2.0** two years after its release date.

For full tier comparisons, OEM embedding, and enterprise fleet governance, see **[Commercial Solutions & Licensing](docs/commercial-licensing.md)** or contact **`admin@slugthuglabs.dev`**.

---

## Governance & Security

- **[Security Policy](SECURITY.md)**: Responsible vulnerability disclosure and security SLAs.
- **[Contributing Guide](CONTRIBUTING.md)**: Architectural standards, invariants, and Contributor License Agreement.
