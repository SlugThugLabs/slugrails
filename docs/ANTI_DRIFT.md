# ANTI_DRIFT.md (The Core Problem Statement)

This file documents why LLM coding agents destroy architectural boundaries when left unchecked:

1. **Guard Threshold Inflation**: The model increases `SOFT_FN_LINES` from 50 to 80 so its own 70‑line bloated function passes.
2. **Architecture Doc Shifts**: The model edits `ARCHITECTURE.md` to describe the bad code it just wrote, making the docs "match" the degraded code.
3. **CLAUDE.md / Rule Erosion**: The model deletes rules from project instructions that it violated, so future audits don't flag them.
4. **Test Editing**: The model changes assertion values (e.g. changing `assert_eq!(result, 404)` to `503`) so tests pass with wrong behavior.

> **Why this matters**: This is the exact sales pitch and rationale for SlugRails! It explains why `INV‑AUTH‑01` (BLAKE3 contract baseline) was built: autonomous agents cannot be allowed to edit their own grading rubric. We should preserve **ANTI_DRIFT.md** in `docs/` on `master`.
