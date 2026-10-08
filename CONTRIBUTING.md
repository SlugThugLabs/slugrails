# Contributing to CodeGuards

Thank you for your interest in contributing to CodeGuards! We welcome bug reports, documentation improvements, guard templates, and code enhancements from the community.

---

## Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all contributors. Please treat fellow community members with respect, constructive feedback, and professional communication.

---

## Contributor License Agreement (CLA) & Licensing

CodeGuards is distributed under the **Functional Source License, Version 1.1, Apache 2.0 Change License (`FSL-1.1-Apache-2.0`)**.

To ensure that CodeGuards can remain sustainable and offer enterprise governance solutions while converting to Apache 2.0 after two years:

1. **Inbound License:** By submitting a pull request, patch, or contribution, you agree that your contribution is provided under the terms of the FSL-1.1-Apache-2.0 license.
2. **Commercial Rights:** You grant **SlugThug Labs** a non-exclusive, perpetual, irrevocable, worldwide license to relicense, commercialize, or sublicense your contributions as part of official CodeGuards commercial editions, hosted services, or enterprise packages, without restricting your rights under FSL-1.1.
3. **Original Work:** You represent that each contribution is your original creation or that you have the full legal right to submit it.

---

## Architectural & Quality Invariants

All contributions must adhere to the project's strict quality and architectural standards:

1. **Zero Unsafe:** All crates enforce `#![forbid(unsafe_code)]`. No `unsafe` blocks are permitted.
2. **File Size Budget:** No source file in `src/` may exceed **200 lines of code**. Large files must be cleanly decomposed into focused submodules.
3. **Zero Test Pollution in `src/`:** All test code, mocks, and integration harnesses must reside exclusively in `tests/`. Do not introduce `#[cfg(test)]` modules inside `src/`.
4. **Zero Lingering Prints or Panics:** Production code must use typed errors via `thiserror` and telemetry via `tracing`. No unhandled `unwrap()`, `expect()`, or `println!` statements are allowed in `src/`.
5. **Passing Gates:** Pull requests must pass all checks:
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   codeguards-mcp check --all
   ```

---

## Submitting Pull Requests

1. Fork the repository and create your branch from `master`.
2. Ensure your changes follow the architectural invariants above.
3. Add corresponding integration tests under `tests/`.
4. Submit your pull request with a clear description of the problem solved.

Questions or commercial inquiries? Reach out to `admin@slugthuglabs.dev`.
