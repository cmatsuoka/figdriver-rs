# Font Encodings Implementation Plan

## Current State

**What works:**
- Control file parsing (`h`/`j`/`b`/`u`/`g` encoding commands are parsed into `InputEncoding` enum)
- ISO 2022 settings parsing (`Iso2022Settings` struct)
- Character code transformations via `t`/`f` commands
- Font lookups via `char` → `i32` cast (works for BMP Unicode only)

**What is missing:**
- Actual encoding decoders (all encoding modes are parsed but not *executed*)
- The input pipeline assumes UTF-8 strings throughout

## The Problem

The current input flow is:

```
stdin bytes → UTF-8 String → char iteration → font.get(char) → i32 code
```

Per the spec, the correct flow should be:

```
stdin bytes → [encoding decoder] → i32 code → [control file transform] → font lookup
```

The encoding decoder depends on the control file's `h`/`j`/`b`/`u` command. Without any encoding command, the default is single-byte Latin-1 mode.

## Scope And Priorities

| Encoding | Priority | Notes |
|----------|----------|-------|
| UTF-8 (`u`) | **High** | Most relevant for modern usage |
| Latin-1 default | **High** | Required for spec compliance |
| DBCS (`b`) | **Medium** | Used by some East Asian fonts |
| HZ (`h`) | **Low** | Legacy Chinese encoding |
| Shift-JIS (`j`) | **Low** | Legacy Japanese encoding |
| ISO 2022 (`g`) | **Low** | Complex, legacy |

## Implementation Steps

### Step 1: Add encoding decoders (`src/flc.rs`)

Add an `EncodingDecoder` struct that converts raw bytes into `i32` character codes. Each encoding variant needs a stateful decoder that yields codes one at a time.

```rust
pub struct EncodingDecoder<'a> {
    encoding: InputEncoding,
    bytes: &'a [u8],
    pos: usize,
    // state for HZ, Shift-JIS, ISO 2022
}

impl<'a> Iterator for EncodingDecoder<'a> {
    type Item = i32;
    fn next(&mut self) -> Option<i32> { ... }
}
```

**UTF-8 decoder** (`u` command):
- Standard UTF-8 decoding (1-4 bytes per character)
- Invalid sequence yields code 128 (per spec: "An incorrectly formatted sequence is interpreted as the character 128")
- Can reuse Rust's `std::str::from_utf8` or manual byte-by-byte decoding

**DBCS decoder** (`b` command):
- Bytes 0x00-0x7F: single-byte characters
- Bytes 0x80-0xFF: high-order byte of a two-byte character
- Two-byte value = `high * 256 + low`

**Latin-1 default** (no encoding command):
- Each byte is one character code (0-255)

**Shift-JIS decoder** (`j` command):
- Bytes 0x80-0x9F and 0xE0-0xEF: high-order byte of two-byte character
- All other bytes: single-byte character
- Two-byte value = `high * 256 + low`

**HZ decoder** (`h` command):
- State machine: `~{` enters two-byte mode, `~}` exits
- `~~` becomes single `~`, other `~X` sequences are stripped
- In two-byte mode: value = `high * 256 + low`
- Outside two-byte mode: single-byte character

### Step 2: Add `get_code(i32)` to FIGfont (`src/figfont.rs`)

`FIGfont::get(char)` currently casts `char` to `i32`, which is limited to Unicode range (0..=0x10FFFF). The spec allows codes from -2147483648 to +2147483647 (full i32 range).

Add `FIGfont::get_code(code: i32) -> Option<&FIGchar>` for direct i32 lookup. Keep `get(char)` as a convenience wrapper for library API compatibility.

### Step 3: Change Smusher to accept `i32` codes (`src/smusher/`)

`Smusher` currently takes `&str` and iterates chars. Change to accept `&[i32]` codes directly. This removes the UTF-8 assumption from the rendering core.

### Step 4: Change Wrapper to accept `i32` codes (`src/wrapper.rs`)

`Wrapper::wrap_str` currently takes `&str`. Change to `wrap_codes(&[i32])`. Space detection for word wrapping becomes `code == 32`. Newline detection becomes `code == 10` or `code == 13`.

### Step 5: Wire encoding in CLI (`src/bin/figlet/main.rs`)

When stdin is the input source:
1. Read stdin as raw bytes
2. Determine encoding from control file (`Control::encoding()`)
3. Decode bytes → `i32` codes via `EncodingDecoder`
4. Apply control file transformations: `control.apply(code)`
5. Feed transformed codes to Wrapper/Smusher

For CLI arguments (`OsString`), continue using UTF-8 path (args are always UTF-8 in practice).

### Step 6: Handle ISO 2022 (`g` command)

ISO 2022 is a stateful encoding with escape sequences. This is complex and can be a follow-up. Requires:
- Parsing ESC sequences to switch G-registers
- Tracking left-half/right-half active character set
- Computing final code as `byte_value + 65536 * designating_byte` (for non-default sets)

## Data Flow After Changes

```
stdin bytes (Raw)
    |
    v
[Encoding Decoder]  <--- controlled by InputEncoding (from control file)
    |
    v
i32 codes (raw character codes)
    |
    v
[Control File Transform]  <--- Control::apply(code)
    |
    v
i32 codes (transformed)
    |
    v
[FIGfont::get_code(code)]  <--- HashMap<i32, FIGchar> lookup
    |
    v
FIGchar glyphs -> Smusher -> Wrapper -> output
```

## Deviations From Original Plan

The following implementation decisions differ from the original plan:

1. **`FIGfont::get` changed directly** — Instead of adding a new `get_code(i32)` and keeping `get(char)` as a wrapper, `get` was changed to take `i32` directly. No stable API existed to preserve.

2. **Smusher kept `push(char)` and `push_str(&str)`** — In addition to `push_codes(&[i32])`, the original `push` and `push_str` methods were retained for ergonomic string-based usage. Internal `push_code(i32)` serves as the shared implementation.

3. **Wrapper kept `push(char)`, `push_str(&str)`, `wrap_str(&str)`** — Same dual-API pattern as Smusher. Core methods (`push_code`, `push_codes`, `wrap_codes`) accept `i32` codes. Convenience methods (`push`, `push_str`, `wrap_str`) delegate to the core methods via `codes_from_str`. Internal buffer changed from `String` to `Vec<i32>`.

4. **Decoder moved to `src/control/` module** — The `EncodingDecoder` lives in `src/control/decoder.rs` alongside `flc.rs` under `src/control/mod.rs`, rather than being added to `src/flc.rs` directly.

5. **Invalid codes skip silently** — Codes outside the valid range (negative or > 0x10FFFF) are skipped during rendering, rather than falling back to the original character.

6. **HZ decoder spec compliance fix** — The HZ decoder was corrected to discard stray `~X` sequences and lone trailing tilde silently, matching reference `figlet` behavior. The initial tests encoded the wrong behavior.

## Key Design Decisions

1. **`i32` throughout** - Use `i32` for character codes everywhere between decoding and font lookup. This matches the spec's range and avoids `char`'s Unicode limitations.

2. **Decoder as iterator** - The encoding decoder yields `i32` codes one at a time, allowing streaming without materializing the entire input.

3. **CLI args vs stdin** - CLI arguments remain UTF-8 (the OS provides UTF-8 strings). Only stdin needs encoding-aware reading.

4. **Ergonomic API preserved** — Keep `char`/`&str` methods on Smusher and Wrapper for ergonomics; add `i32` code methods internally and expose `push_codes`/`wrap_codes` for encoding-aware paths.

## Files To Modify

| File | Changes |
|------|---------|
| `src/flc.rs` | Add `EncodingDecoder` struct, implement decoders for each `InputEncoding` variant |
| `src/figfont.rs` | Add `get_code(i32)` method, keep `get(char)` as wrapper |
| `src/smusher/` | Change to accept `&[i32]` codes instead of `&str` |
| `src/wrapper.rs` | Change `wrap_str` to `wrap_codes(&[i32])` |
| `src/bin/figlet/main.rs` | Add encoding-aware stdin reading, wire decoder -> transform -> codes pipeline |
| `src/lib.rs` | Export `EncodingDecoder` if needed |

## Tests

- UTF-8 decoder: multi-byte sequences, invalid sequences yield code 128
- DBCS decoder: high-byte detection, two-byte composition
- Latin-1: byte-to-code identity mapping
- Shift-JIS: correct high-byte range detection
- HZ: state machine transitions, escape sequences
- Integration: pipe encoded input through figlet with control file, verify output matches reference `figlet`
- End-to-end: compare figdriver-rs output against `figlet` reference for UTF-8, DBCS, and Latin-1 input
