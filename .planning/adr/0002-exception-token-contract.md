# ADR 0002 — Exception token contract

- Status: accepted
- Date: 2026-10-09
- Supersedes: the implicit "5-digit decimal" token contract in `docs/architecture.md`

## Context

Exception tokens let a user authorize a scoped bypass of a guardrail. The original
implementation derived a **5-digit decimal** token (`10_000..=99_999`, ~16.5 bits) from a
keyed BLAKE3 hash over `(path, guard, reason, date)`.

The audit (`/root/slugrails-audit/FINDINGS.md`, SR-S05 / SR-H01) found:

- the token space is small and the format is ambiguous with ordinary 5-digit numbers;
- the working tree had already switched to a 128-bit hex token while the tests, docs and
  storage validator still accepted the legacy 5-digit form, leaving two credential formats live;
- nothing recorded which format was canonical, so the compatibility policy was accidental.

## Decision

1. **Canonical token format is `sr_tok_` + 32 lowercase hex characters** (39 bytes total) —
   the first 128 bits of the keyed-BLAKE3 MAC over `(normalized_path, guard_id, reason, grant_date)`.
   `grant_date` is the `YYYY-MM-DD` prefix of the entry's `granted_at` field.
2. **The key** is the local 256-bit salt (`~/.slugthuglabs/slugrails/.secret.key`, mode `0600`,
   overridable by `SLUGRAILS_SECRET_SALT` for tests/CI). The construction is a keyed BLAKE3 MAC,
   **not** HMAC-SHA256; documentation must say so.
3. **Legacy 5-digit tokens are rejected.** They are not accepted as valid credentials by
   `ProjectExceptions::is_exception_valid[_with_ttl]`, and the legacy generator helpers are removed
   so the weak format cannot be reintroduced by accident.
4. **Validation** re-derives the expected token from the stored entry's own fields and requires
   both `entry.token == presented` and `presented == expected`, so a tampered store entry cannot be
   used to validate an unrelated token.
5. **Scope remains `(file, guard, reason)` with a 30-day default TTL**; the token is deliberately
   *not* bound to a line number (see SR-S09 in the audit for the known consequence).

## Compatibility / migration

Pre-cutover exceptions are invalid after this change: they stop matching, the underlying violation
reappears, and the user re-issues the exception with `slugrails exception add` (which must now be run
in an interactive terminal — ADR 0003). This is a deliberate security-over-compatibility choice:
keeping the 5-digit form alive would preserve a weak credential format indefinitely.

## Verification

- `tests/token_collision_test.rs` asserts the canonical format, determinism, date binding,
  separation across file/guard/reason, idempotent replacement, and rejection of legacy,
  malformed, truncated and tampered tokens.
- `tests/exception_tests.rs`, `tests/foundation_tests.rs`, `tests/mcp_handlers_suite_tests.rs`
  assert the 39-character `sr_tok_` shape.

## Consequences

- `src/util/crypto.rs` exposes only `compute_exception_token[_with_date]`.
- `src/storage/mod.rs` recomputes a single expected token using the entry's grant date.
- Documentation (`docs/architecture.md`, `docs/cli-reference.md`, `docs/troubleshooting.md`)
  states the canonical format and the keyed-BLAKE3 construction.
