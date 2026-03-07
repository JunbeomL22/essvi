---
phase: 9
slug: data-schema-and-directory-structure
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-07
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Shell commands (no code framework — this phase is filesystem + docs only) |
| **Config file** | none |
| **Quick run command** | `test -d data/ && test -f data/README.md && echo PASS` |
| **Full suite command** | `test -d data/ && test -d data/cboe/spx && test -d data/eurex/sx5e && test -d data/sample && test -f data/README.md && grep -q "quote_date" data/README.md && grep -q "strike" data/README.md && grep -q "YYYY-MM-DD" data/README.md && echo ALL_PASS` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run `test -d data/ && test -f data/README.md && echo PASS`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | STOR-01 | structural | `test -d data/cboe/spx && test -d data/eurex/sx5e && test -d data/sample` | N/A (creates dirs) | pending |
| 09-01-02 | 01 | 1 | STOR-02, STOR-03 | content | `test -f data/README.md && grep -q "quote_date" data/README.md && grep -q "expiry" data/README.md && grep -q "strike" data/README.md && grep -q "option_type" data/README.md && grep -q "bid" data/README.md && grep -q "ask" data/README.md && grep -q "underlying_price" data/README.md` | N/A (creates file) | pending |
| 09-01-03 | 01 | 1 | STOR-03 | content | `grep -q "YYYY-MM-DD" data/README.md && grep -q "forward" data/README.md && grep -q "discount_factor" data/README.md` | N/A (part of README) | pending |

*Status: pending -- all pending until execution*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No test framework installation needed — this phase produces directories and a markdown file, not code. Validation uses shell existence checks and grep.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| README content quality | STOR-03 | Semantic correctness of documentation cannot be automated | Read data/README.md and verify: (1) all 7 required columns defined with type and description, (2) optional columns listed, (3) file naming convention explained, (4) directory hierarchy explained |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
