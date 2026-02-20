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

## Segment Types (v0.1.0)

### 1. Manifest (`SegmentType::Manifest = 1`)

Required segment that describes global properties of the file.

Layout (little-endian):

- magic: `MNF\0` (4 bytes)
- version_major: u16 (currently 0)
- version_minor: u16 (currently 1)
- flags: u32
- required_segment_count: u32
- supported_kernel_count: u32
- max_chunk_bytes: u32
- reserved: u32 (must be 0)
- required_segments: `required_segment_count` × u32 (`SegmentType` values)
- supported_kernels: `supported_kernel_count` × u32 (`QueryKernel` values)

Notes:
- `required_segments` lists segment types that MUST be present for a valid file.
- `supported_kernels` advertises which query kernels the engine can serve.

### 2. String Dictionary (`SegmentType::StringDict = 2`)

Optional segment that stores a compact dictionary of UTF-8 strings used elsewhere in the file.

Layout (little-endian):

- magic: `SDCT` (4 bytes)
- version_major: u16 (currently 0)
- version_minor: u16 (currently 1)
- string_count: u32
- blob_len: u32 (total bytes in concatenated UTF-8 blob)
- offsets: (`string_count + 1`) × u32
- blob: `blob_len` bytes of concatenated UTF-8 string data

Semantics:
- `offsets[i]` and `offsets[i+1]` are the byte range of string `i` within `blob`.
- Offsets MUST start at 0 and be non-decreasing; `offsets[string_count]` MUST equal `blob_len`.
- Strings MUST be valid UTF-8.

### Query Kernels

The manifest can advertise query kernels by numeric ID.

Currently defined:

- `QueryKernel::GetObjectById = 1`

These IDs are versioned independently from `SegmentType` and allow the runtime to discover which query behaviors are available for a given file.

---

## Design Rules

- Unknown segment types MUST be skippable
- All data is stored in typed segments
- No JSON blobs in core storage
