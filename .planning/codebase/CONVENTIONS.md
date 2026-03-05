# Coding Conventions

**Analysis Date:** 2026-03-05

## Naming Patterns

**Files:**
- Use snake_case for all Rust source files (standard Rust convention)
- Library root: `src/lib.rs`

**Functions:**
- Use snake_case for all function names: `add()`, `nelder_mead_bounded()`, `solve_theta()`
- Use descriptive mathematical names where appropriate: `project()`, `squared_error()`
- Mark small utility functions with `#[inline]`

**Variables:**
- Use snake_case for all variables: `f_best`, `f_worst`, `f_second_worst`, `x_spread`
- Mathematical variable names are acceptable for domain-specific code: `xr`, `xe`, `xc`, `fr`, `fe`, `fc`
- Use descriptive names for loop indices when semantic meaning exists: use `j` for dimension index, `i` for vertex index

**Types:**
- Use PascalCase for structs: `NelderMeadConfig`, `NelderMeadResult`
- Derive common traits: `#[derive(Debug, Clone)]` on data structs
- Use `Default` trait implementation for configuration structs

## Code Style

**Formatting:**
- Standard `rustfmt` (no custom `.rustfmt.toml` detected -- use Rust defaults)
- 4-space indentation
- No trailing whitespace

**Linting:**
- No custom `clippy.toml` detected -- use default Clippy rules
- Run `cargo clippy` before committing

## Rust Edition

- **Edition 2024** as specified in `Cargo.toml`
- Use edition-appropriate idioms

## Import Organization

**Order:**
1. Standard library imports (`std::`)
2. External crate imports
3. Local module imports (`use super::*`, `use crate::`)

**Pattern:**
- Use `use super::*` in test modules to import parent module items
- Prefer explicit imports over glob imports in non-test code

## Error Handling

**Patterns:**
- The codebase is numerical/mathematical -- functions return concrete values (`f64`, result structs) rather than `Result<T, E>`
- Use `.unwrap()` on `partial_cmp` for `f64` comparisons (acceptable when NaN is not expected in well-formed input)
- Convergence status is communicated via a `converged: bool` field in result structs rather than error types
- Bound violations are handled by projection (clamping) rather than returning errors

**Guidance for new code:**
- For I/O operations or fallible external calls, use `Result<T, E>` with a project-specific error type
- For pure numerical computation, returning result structs with status fields is preferred
- Avoid `panic!` in library code; use it only in tests

## Logging

**Framework:** Not yet established

**Guidance:**
- No logging dependencies currently present
- For numerical debugging, prefer returning iteration counts and convergence info in result structs
- If logging is needed later, consider `log` crate with `env_logger`

## Comments

**When to Comment:**
- Comment algorithmic steps with numbered phases (e.g., `// Sort by function value`, `// Reflection`, `// Expansion`)
- Comment mathematical formulas inline when translating from equations to code
- Use `//` single-line comments, not `/* */` block comments

**Doc Comments:**
- Not yet established; use `///` doc comments on all public functions and types
- Include parameter descriptions and brief mathematical context

## Function Design

**Size:**
- Core algorithm functions can be longer (the Nelder-Mead function is ~140 lines) when the algorithm is a single cohesive unit
- Extract helper functions for reusable operations: `project()` is extracted as a utility

**Parameters:**
- Use slices (`&[f64]`) for input arrays rather than `Vec<f64>`
- Use references to config structs (`&NelderMeadConfig`) rather than passing by value
- Use generic function parameters with trait bounds for callables: `F: Fn(&[f64]) -> f64`

**Return Values:**
- Return dedicated result structs (not tuples) for multi-value returns
- Include metadata (iteration count, convergence status) in result structs

## Module Design

**Exports:**
- Use `pub` on types and functions intended for external use
- Keep helper functions private (no `pub`) unless needed externally
- Place tests in `#[cfg(test)] mod tests` within the same file

**Barrel Files:**
- Not applicable yet (single-file crate)
- When multiple modules are added, re-export public API from `lib.rs`

## Numeric Conventions

**Floating Point:**
- Use `f64` throughout (not `f32`) for numerical precision
- Use scientific notation for tolerances: `1e-12`, `1e-10`, `1e-6`
- Use `f64::INFINITY` and `f64::NEG_INFINITY` for sentinel values
- Use `.clamp()` for bound projection

**Vectors:**
- Use `Vec<f64>` for dynamically-sized arrays
- Use `&[f64]` slices for function parameters
- Pre-allocate with `Vec::with_capacity()` when size is known

## Dependencies Policy

- Minimize external dependencies (zero dependencies currently)
- Prefer implementing small algorithms directly over pulling in crates
- This is explicitly stated in `documents/guideline.md`: "dependency 없이 가장 깔끔한 접근"

---

*Convention analysis: 2026-03-05*
