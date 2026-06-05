# Publish Readiness Report

Date: 2026-06-04
Crate: figdriver v0.2.0

---

## Summary

| Check | Status |
| --- | --- |
| Tests (unit + integration + doc-tests) | PASS (94 passed, 0 failed) |
| Clippy (`-D warnings`) | PASS (clean) |
| `cargo doc` | PASS (no warnings) |
| `cargo publish --dry-run` | PASS (compiles, 92 files, 839 KiB) |
| Dependency hygiene | ISSUE |
| API surface | ISSUE |
| Documentation | ISSUE |

---

## Issues to Fix Before Publishing

### 1. Binary-only dependencies pollute library consumers [HIGH]

`pico-args` and `terminal_size` are listed as regular `[dependencies]` but are only used by the `figlet` binary (`src/bin/figlet/main.rs` and `cli.rs`). Library consumers will unnecessarily pull in `terminal_size` -> `rustix` -> `linux-raw-sys`.

**Fix:** Move them to target-specific dependencies in `Cargo.toml`:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
pico-args = { version = "0.5", features = ["combined-flags", "short-space-opt", "eq-separator"] }
terminal_size = "0.4"
```

### 2. `Error::Cli` is CLI-specific, not library-API [HIGH]

The `Cli(String)` variant in `lib.rs` is only used by the binary. It shouldn't be part of the public library API exposed to downstream crates.

**Fix:** Move CLI error handling into the binary crate, or use a separate error type for the binary.

### 3. Inconsistent indentation in `lib.rs` [LOW]

Line 9 has a leading space: ` pub use self::smusher::{...}`

**Fix:** Remove the leading space on line 9 of `src/lib.rs`.

### 4. Public fields on `FIGfont` without documentation [MEDIUM]

Fields `hardblank`, `height`, `old_layout`, `right_to_left`, `layout` are `pub` but undocumented.

**Fix:** Either add doc comments or make them private with accessor methods (more idiomatic Rust).

### 5. Missing documentation on FLC types [MEDIUM]

Public types `FlcCommand`, `TransformationStage`, `InputEncoding`, `Iso2022CharSetSize`, `Iso2022CharSet`, `Iso2022Settings` lack doc comments.

**Fix:** Add doc comments explaining each type's purpose.

### 6. `SMUSH_*` constants lack documentation [MEDIUM]

The constants `SMUSH_EQUAL`, `SMUSH_UNDERLINE`, `SMUSH_HIERARCHY`, `SMUSH_PAIR`, `SMUSH_BIGX`, `SMUSH_HARDBLANK`, `SMUSH_KERN`, `SMUSH_ENABLE` are re-exported publicly but undocumented.

**Fix:** Add doc comments explaining each smush mode flag.

---

## What's Good

- **Test coverage:** 94 tests pass across unit tests, integration tests, and doc-tests
- **Linting:** Clippy runs clean with `-D warnings`
- **Documentation:** `cargo doc` generates without warnings
- **Packaging:** `cargo publish --dry-run` compiles and packages successfully (92 files, 839 KiB compressed to 157 KiB)
- **API design:** The `FIGfont` -> `Smusher` -> `Wrapper` pipeline is clear and coherent
- **Examples:** Three examples (`hello.rs`, `simple.rs`, `showfonts.rs`) demonstrate the main API patterns
- **Readme:** Comprehensive with feature list, installation, library usage, CLI options, and examples
- **License:** MIT, clearly stated in both Cargo.toml and LICENSE file
- **Categories/keywords:** Appropriate crates.io metadata (`text-processing`, `ascii`, `ascii-art`, `terminal`)
- **LTO:** Release profile enables LTO for optimized builds

---

## Verification Commands Used

```bash
cargo clippy -- -D warnings
cargo doc --no-deps
cargo test
cargo publish --dry-run
```
