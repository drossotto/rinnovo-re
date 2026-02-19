# RNB Format v0.1.0

## Overview

RNB (Rinnovo Binary) is a single-file container format for biological datasets. 

It is designed for:

- Memory-mapped access
- Segment-based storage
- Forward-compatible evolution

---

## File Layout

[RNB HEADER][SEGMENT DATA ...][SEGMENT DIRECTORY]

---

## Header

- Magic bytes: `RNB\0`
- Version: u16 (major, minor)
- Endianness: little-endian
- Directory offset: u64
- Directory length: u64

---

## Segment Directory

Each entry:
- segment_id (u32)
- segment_type (u32)
- offset (u64)
- length (u64)
- checksum (u64)

---

## Design Rules

- Unknown segment types MUST be skippable
- All data is stored in typed segments
- No JSON blobs in core storage