# SlugRails (`slugrails-mcp`)

> Architectural guardrails, anti-drift enforcement, and integrated SlugPlan for AI coding agents.

SlugRails keeps autonomous AI coding agents on structural rails by enforcing architectural boundaries declared in `.planning/ARCHITECTURE.md`. It provides integrated planning via **SlugPlan**, real-time bouncer checking via **SlugRails**, and pairs seamlessly with **SlugAudit** for retrospective codebase evidence.

### Two-Tier Storage Architecture:
- **Global Hub:** `~/.slugthuglabs/slugrails/tests/` (reusable guard templates and secret keys)
- **Local Project Hub:**
  - `<project>/.planning/ARCHITECTURE.md` (the core architectural contract)
  - `<project>/.planning/slugplan/` (intent declarations and project milestones)
  - `<project>/.planning/slugrails/` (exceptions and anti-drift tokens)
  - `<project>/.planning/slugaudit/` (evidence database when auditing)

---

## AI Agent Integration (Hermes, Claude Code, Cursor, Antigravity)

SlugRails is built to be driven directly by AI agents through MCP tools and CLI execution.

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

## Agent Workflow

### 1. Authoring & Architecture Setup
During conversational planning, the agent creates or updates `.planning/ARCHITECTURE.md` with TOML frontmatter:

```toml
+++
modules = ["server", "domain", "storage"]
layers = ["transport", "domain", "storage"]
enforce = [
  "source_limits",
  "no_unwrap",
  "no_debug_prints",
  "layer_dependencies",
]

[allowed_dependencies]
server = ["domain"]
domain = ["storage"]
storage = []

[guard_settings.source_limits]
max_lines = 400
+++

# Architecture & System Design
```

The agent validates the contract via MCP:
* **`validate_architecture`**: Checks that all declared `enforce` rules exist in `~/.slugthug/codeguards/tests/`.
* **`list_guard_tests`**: Inspects available guard tests across categories (`structural`, `complexity`, `hygiene`, `languages/rust`).
* **`create_guard_test`**: Authors a new reusable test definition when a project requires custom invariants.

---

## 2. Enforcement & Coding Loop

During normal coding, the agent runs standard implementation tasks. Verification happens via CLI triggers:

```bash
# Check modified / staged files in active git diff (<5ms)
slugrails check

# Run full project scan across all files
slugrails check --all

# Validate architecture contract
slugrails validate
```

### Git Hook Integration
Install automated pre-commit enforcement gate with one command:
```bash
slugrails hook install
```
*(To uninstall: `slugrails hook uninstall`)*

---

## 3. User-Authorized Exceptions

When a file genuinely requires an exemption (e.g. file size limit for generated code), agents cannot self-authorize. The user grants an exception token:

```bash
# User grants exception
slugrails exception add src/server/transport.rs complexity/source-limits --reason="Combined transport scaffolding"

# List active exceptions
slugrails exception list

# Revoke an exception
slugrails exception revoke <token>
```

The file adds the verified header:
```rust
// codeguard-exception: token=23954; guard=complexity/source-limits; reason="Combined transport scaffolding"
```

---

## Build & Install

```bash
cargo build --release
cp target/release/codeguards-mcp /root/.slugthug/bin/codeguards-mcp
```

---

## License & Commercial Terms

CodeGuards is licensed under the **Functional Source License, Version 1.1, Apache 2.0 Change License (`FSL-1.1-Apache-2.0`)**.

- **100% Free for Developers & Internal Teams:** You are free to use CodeGuards internally to develop, analyze, audit, test, and maintain your own software—including commercial products and services. You own 100% of the code and products you build with it.
- **Protection Against Commercial Resale & AI Bundling:** Third parties and AI platforms may not offer the functionality of CodeGuards as a competing commercial product or service, nor bundle or embed it into a paid commercial offering without an agreement from SlugThug Labs.
- **Automatic Apache 2.0 Conversion:** Every release automatically converts to the fully permissive **Apache License 2.0** two years after its release date.

For full tier comparisons, OEM embedding, and enterprise fleet governance, see **[Commercial Solutions & Licensing](docs/commercial-licensing.md)** or contact **`admin@slugthuglabs.dev`**.

---

## Governance & Security

- **[Security Policy](SECURITY.md)**: Responsible vulnerability disclosure and security SLAs.
- **[Contributing Guide](CONTRIBUTING.md)**: Architectural standards, invariants, and Contributor License Agreement.

