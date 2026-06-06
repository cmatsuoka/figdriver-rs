<!--
  Compliance report against FIGfont Version 2 standard (doc/figfont.txt).
  Generated: 2026-06-05
-->

# FIGdriver Standard Compliance Report

This document records figdriver-rs compliance against the FIGfont Version 2 standard
(specified in `doc/figfont.txt`).

## 1. Required Features

The standard states (line 1545):

> Future FIGdrivers must read and process FIGfont files as described in this document,
> but are not necessarily expected to process control files, smush, perform fitting or
> kerning, perform vertical operations, or even produce multiple lines in output FIGures.

### 1.1 FIGfont file reading — COMPLIANT

| Feature | Status | Notes |
|---|:---:|---|
| `flf2a` signature | ✔ | Also supports `tlf2a` (TOIlet) |
| Hardblank character | ✔ | Tracked and replaced with space on output |
| Height / Baseline / Max_Length | ✔ | Parsed and used |
| Old_Layout parameter | ✔ | Parsed and used for horizontal layout |
| Comment lines | ✔ | Skipped during loading |
| Print_Direction (LTR/RTL) | ✔ | Parsed and used |
| Full_Layout parameter | ✔ | Parsed; horizontal bits used; vertical bits ignored (see §3) |
| Codetag_Count | ✔ | Parsed (informational) |
| Required characters (32-126 + 7 Deutsch) | ✔ | Loaded in fixed order |
| Code-tagged characters | ✔ | Decimal, hex (`0x`), octal (`0`), negative codes, `-1` terminator |
| Missing character (code 0) | ✔ | Used as fallback for unknown characters |
| Tab fallback to space | ✔ | Implemented in renderer |

### 1.2 Control file processing — COMPLIANT

| Feature | Status | Notes |
|---|---|---|
| `flc2a` signature | ✔ | Optional, accepted if present |
| Comment lines (`#` prefix) | ✔ | Skipped during parsing |
| Blank lines | ✔ | Skipped during parsing |
| `t` command (single char mapping) | ✔ | |
| `t` command (range mapping) | ✔ | e.g., `t A-Z a-z` |
| `number number` (numeric mapping) | ✔ | Unicode mapping table format |
| `f` command (freeze / new stage) | ✔ | Creates new transformation stage |
| Escape sequences (`\n`, `\t`, `\r`, etc.) | ✔ | Full set supported |
| Numeric escapes (`\65`, `\0x41`, `\0101`) | ✔ | Decimal, hex, octal |
| Negative character codes | ✔ | |
| First-match rule per stage | ✔ | Only first matching command applies |
| Multi-file pipeline | ✔ | Multiple `-C` flags chain transformations |
| `h` command (HZ mode declaration) | ✔ | Parsed; HZ decoder implemented (`EncodingDecoder`) |
| `j` command (Shift-JIS declaration) | ✔ | Parsed; Shift-JIS decoder implemented (`EncodingDecoder`) |
| `b` command (DBCS declaration) | ✔ | Parsed; generic DBCS decoder implemented (`EncodingDecoder`) |
| `u` command (UTF-8 declaration) | ✔ | Parsed; UTF-8 decoder implemented (`EncodingDecoder`) |
| Latin-1 encoding | ✔ | Decoder implemented (`EncodingDecoder`) |
| `g` command (ISO 2022 G-register) | ⚠ | Parsed and stored, but ISO 2022 decoder not implemented |

## 2. Horizontal Layout Modes

All horizontal layout modes are fully implemented.

| Mode | Full_Layout bit | Status |
|---|---|:---:|
| Full width | (no bit 64 or 128) | ✔ |
| Fitting (kerning) | 64 | ✔ |
| Smushing (controlled) | 128 + rule bits | ✔ |
| Smushing (universal) | 128 (no rule bits) | ✔ |

## 3. Horizontal Smushing Rules

All six horizontal smushing rules are implemented.

| Rule | Bit | Name | Status |
|---|---|---|:---:|
| 1 | 1 | Equal Character Smushing | ✔ |
| 2 | 2 | Underscore Smushing | ✔ |
| 3 | 4 | Hierarchy Smushing | ✔ |
| 4 | 8 | Opposite Pair Smushing | ✔ |
| 5 | 16 | Big X Smushing | ✔ |
| 6 | 32 | Hardblank Smushing | ✔ |

## 4. Vertical Layout Modes — NOT IMPLEMENTED

Vertical fitting and smushing are **not implemented**. The relevant bits in Full_Layout
(8192, 16384) are parsed from the font header but have no effect on rendering.

This is consistent with the reference implementation: FIGlet 2.2 also does not support
vertical operations (per spec line 1689). Only FIGWin 1.0 supports them.

| Mode | Full_Layout bit | Status |
|---|---|:---:|
| Vertical fitting | 8192 | ✘ |
| Vertical smushing | 16384 | ✘ |

## 5. Vertical Smushing Rules — NOT IMPLEMENTED

None of the five vertical smushing rules are implemented.

| Rule | Bit | Name | Status |
|---|---|---|:---:|
| 1 | 256 | Equal Character Smushing | ✘ |
| 2 | 512 | Underscore Smushing | ✘ |
| 3 | 1024 | Hierarchy Smushing | ✘ |
| 4 | 2048 | Horizontal Line Smushing | ✘ |
| 5 | 4096 | Vertical Line Supersmushing | ✘ |

## 6. Word Wrapping — COMPLIANT

Per spec lines 1606-1634:

| Behavior | Status | Notes |
|---|:---:|---|
| Wrap on input blanks | ✔ | |
| Respect explicit linebreaks | ✔ | LF, CR, CRLF normalized |
| Collapse consecutive blanks at wrap | ✔ | Discarded until next non-blank |
| Preserve leading blanks (start / after linebreak) | ✔ | |
| Preserve trailing blanks (before linebreak / end) | ✔ | |
| RTL reversal of blank rules | ✔ | |
| Left / Center / Right alignment | ✔ | |

## 7. Print Direction — COMPLIANT

| Direction | Status | Notes |
|---|:---:|---|
| Left-to-right (default) | ✔ | |
| Right-to-left | ✔ | Character-level smush prefers left char in RTL |
| Font default | ✔ | Uses Print_Direction from header |

## 8. File Compression — COMPLIANT

Per spec lines 1654-1671:

| Feature | Status | Notes |
|---|---|---|
| ZIP archive detection | ✔ | PK magic bytes |
| Deflate decompression | ✔ | |
| Single-entry handling | ✔ | Subsequent entries ignored |
| `.flf` compressed fonts | ✔ | |
| `.flc` compressed control files | ✔ | |
| Decompression size limit | ✔ | 10 MB (zip bomb protection) |

## 9. Layout Parameter Consistency — COMPLIANT

The implementation correctly interprets the relationship between Old_Layout and
Full_Layout per spec lines 743-828:

- Old_Layout `-1` → full width
- Old_Layout `0` → fitting (or universal smush when Full_Layout bit 128 is set with no rule bits)
- Old_Layout positive → controlled smush with specified rule bits
- Full_Layout bit 64 → horizontal fitting default
- Full_Layout bit 128 → horizontal smush default
- Full_Layout bits 1-32 → controlled smush rule selection

## 10. Summary of Non-Compliant or Incomplete Features

| Feature | Gap | Priority |
|---|---|---|
| Vertical fitting and smushing | Not implemented (all 7 bits: 256-4096, 8192, 16384) | Low — FIGlet 2.2 also omits this |
| ISO 2022 decoder | FLC `g` commands parsed but G-registers not applied | Low — legacy East Asian encoding |

### Notes on ISO 2022 decoder

The ISO 2022 decoder requires tracking G-register state and interpreting escape
sequences in the input stream to switch character sets dynamically. This is distinct
from the other encoding decoders (HZ, Shift-JIS, DBCS, Latin-1, UTF-8), which are
now fully implemented in `EncodingDecoder`. The `g` command settings are parsed and
stored, but the actual runtime decoding of ISO 2022 escape sequences is not yet wired
into the rendering pipeline. Implementing it would require an additional decoder mode
that processes ESC sequences in the input byte stream to assign and switch G-registers.

### Notes on encoding decoders

HZ, Shift-JIS, DBCS, Latin-1, and UTF-8 decoders are now fully implemented in
`EncodingDecoder` (src/control/decoder.rs). The CLI feeds stdin through the decoder
when no positional arguments are provided, using the encoding specified by the
control file. Modern usage routes through UTF-8 (`u` command), which works correctly
because the renderer operates on Unicode scalar values. The legacy decoders are useful
for processing historical input files still encoded in those formats.

### Notes on vertical operations

The spec explicitly notes (line 225):

> Not all FIGdrivers do vertical fitting or smushing. At present, FIGWin 1.0 does,
> but FIGlet 2.2 does not.

The reference implementation (figlet 2.2.2) does not support vertical fitting or
smushing. Implementing these features would make figdriver-rs more capable than
the reference, but would require a significant new rendering subsystem to handle
vertical character stacking and line-level supersmushing.
