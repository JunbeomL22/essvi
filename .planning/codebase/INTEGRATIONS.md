# External Integrations

**Analysis Date:** 2026-03-05

## APIs & External Services

None. This is a self-contained computational library with no external API calls.

## Data Storage

**Databases:**
- None. No database connectivity.

**File Storage:**
- No file I/O detected in current code.
- Input data (market total variance, ATM log-moneyness) is expected to be passed programmatically.

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- Not applicable. Pure library, no auth required.

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- No logging framework. The guideline example uses `println!` for output.

## CI/CD & Deployment

**Hosting:**
- Not applicable (library crate, not a deployed service)

**CI Pipeline:**
- None detected (no `.github/workflows/`, no CI config files)

## Environment Configuration

**Required env vars:**
- None

**Secrets location:**
- Not applicable

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Integration Notes

This project has **zero external integrations by design**. Per `documents/guideline.md`, the explicit architectural decision is to avoid external crate dependencies (e.g., `argmin`, `nlopt`) in favor of a self-contained implementation. Future integrations to consider as the library grows:

- **Data input**: Market data feeds or file parsers (CSV/JSON) for strike/volatility data
- **FFI**: C-compatible interface if the library needs to be called from Python/other languages
- **Serialization**: `serde` if calibration results need to be serialized

---

*Integration audit: 2026-03-05*
