---
phase: 10
slug: data-acquisition
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-07
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | bash / shell commands (no code test framework — data-only phase) |
| **Config file** | none |
| **Quick run command** | `ls data/*/*/*csv 2>/dev/null \| wc -l` |
| **Full suite command** | `bash -c 'for f in data/*/*/*csv; do head -1 "$f" \| grep -q "quote_date.*expiry.*strike.*option_type.*bid.*ask.*underlying_price" && echo "PASS: $f" \|\| echo "FAIL: $f"; done'` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run quick run command to confirm CSV count
- **After every plan wave:** Run full suite command to verify schema conformance
- **Before `/gsd:verify-work`:** Full suite must show all PASS
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | DATA-01, DATA-02, DATA-03 | structural | `scripts/fetch_options.py` exists | N/A (new) | pending |
| 10-01-02 | 01 | 1 | DATA-01 | structural | `ls data/*/spx/*.csv \| wc -l` >= 2 | N/A (new) | pending |
| 10-01-03 | 01 | 1 | DATA-02 | structural | `ls data/*/ -d \| wc -l` >= 2 dirs with CSVs | N/A (new) | pending |
| 10-01-04 | 01 | 1 | DATA-03 | structural | `cut -d, -f4 file.csv \| sort -u` contains C and P | N/A (new) | pending |

*Status: pending*

---

## Wave 0 Requirements

- [ ] `pip install yfinance` — Python library for Yahoo Finance data access
- [ ] Verify Python 3.8+ is available

*Existing infrastructure covers all other phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Data quality review | DATA-03 | Bid/ask reasonableness requires human judgment | Spot-check 5-10 rows per CSV: bid < ask, prices > 0 for near-ATM options |
| Multi-expiry richness | DATA-03 | "Multiple expiry slices" is qualitative | Verify >= 3 distinct expiry dates per CSV |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 2s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
