# Security Policy

SlugThug Labs takes the security of SlugRails and the environments it monitors seriously.

---

## Supported Versions

Only the latest release receives active security patches.

| Version | Supported |
| :--- | :---: |
| 0.2.x | :white_check_mark: |
| < 0.2.0 | :x: |

---

## Reporting a Vulnerability

If you discover a security vulnerability or sandbox escape in SlugRails, **please do NOT report it via a public GitHub issue.**

Instead, please send a responsible disclosure report via email to:

**`security@slugthuglabs.dev`**  
*(cc: `admin@slugthuglabs.dev`)*

### Please Include:
1. Description of the vulnerability and potential impact.
2. Steps or proof-of-concept (PoC) to reproduce the behavior.
3. Affected versions and operating environments (Windows, Linux, macOS).
4. Any proposed remediations or patches if available.

---

## Vulnerability Handling SLA

- **Initial Response:** We will acknowledge receipt of your vulnerability report within **48 hours**.
- **Assessment & Triage:** We will assess severity and confirm reproducibility within **5 business days**.
- **Fix & Disclosure:** We will work with you to patch the issue and coordinate public release with appropriate attribution.

---

## Architectural Security Invariants

SlugRails enforces several built-in security baselines by design:
- `#![forbid(unsafe_code)]` at all crate roots.
- Canonical path resolution with strict sandbox bounds to prevent path traversal outside workspace roots.
- Bounded memory buffers (1MB maximum per source file read) to prevent denial-of-service via memory exhaustion.
- Complete stream isolation: stdio MCP JSON-RPC transport operates exclusively on standard output, with all diagnostic and tracing telemetry strictly directed to standard error.
