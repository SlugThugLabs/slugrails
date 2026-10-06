# Engineering Workspace Standard (.planning)

This repository follows the **SlugPlan** convention: `.planning/` serves as the in-repo Confluence, Jira, and Tool Workspace.

---

## 🎯 Core Mission: Architectural Invariant & Black-Box Test Governance

CodeGuards is **NOT a simple line-counting linter**. It is an Architectural Invariant Compiler and Black-Box Enforcement Engine for autonomous AI agents and engineers.

CodeGuards operates across two distinct phases:
1. **Planning Phase (Interactive MCP):** Agent and user interactively define system boundaries, module DAGs, and formal behavioral contracts (`[[invariants]]`) in `.planning/ARCHITECTURE.md`.
2. **Enforcement Phase (Black-Box & Native Triggers):** CodeGuards compiles and enforces those invariants via native build triggers (`cargo test`, git hooks, and CLI checks). Invariants are enforced as black-box verification so autonomous agents cannot locate, weaken, or bypass test assertions.

---

## 🔒 Strict Rules of Engagement for AI Agents

1. **Never Weaken Test Assertions:**
   - Under no circumstances may an agent edit, relax, or remove test assertions to make failing builds pass.
   - Guard test runners are black-box verifications. Attempting to tamper with test runners, alter thresholds, or mock out invariant checks is strictly forbidden.
   - When an invariant test fails, the **only valid action is to fix the underlying implementation in `src/`**.

2. **Never Guess Architecture:**
   - Always read `.planning/ARCHITECTURE.md` first.
   - Module boundaries and `allowed_dependencies` are strict DAG constraints.

3. **Cross-Platform Parity (Windows & Unix):**
   - CodeGuards must run seamlessly on both Windows and Linux/macOS.
   - Home directory resolution must inspect `SLUGTHUG_HOME`, `HOME`, and `USERPROFILE`.
   - Executable binaries must respect platform extensions (`codeguards-mcp.exe` on Windows).
   - Paths must be normalized without hardcoded Unix-only root assumptions (`/root/`).

4. **Stay Scoped:**
   - Pure product code lives in `src/`.
   - Internal planning belongs strictly in `.planning/`.
   - Work only on the active task assigned in `.planning/jira/PROJECT_PLAN.md`.