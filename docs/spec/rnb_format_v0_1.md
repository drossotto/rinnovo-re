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

- Magic bytes: RNB\0
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

### 1. Manifest (SegmentType::Manifest = 1)

Required segment that describes global properties of the file and constrains
how readers should interpret the rest of the artifact.

Layout (little-endian):

- magic: MNF\0 (4 bytes)
- version_major: u16 (currently 0)
- version_minor: u16 (currently 1)
- flags: u32
- required_segment_count: u32
- supported_kernel_count: u32
- max_chunk_bytes: u32
- reserved: u32 (must be 0)
- required_segments: equired_segment_count × u32 (SegmentType values)
- supported_kernels: supported_kernel_count × u32 (QueryKernel values)

Notes:
- equired_segments lists segment types that MUST be present for a valid file.
  The container loader enforces that each listed SegmentType has at least one
  corresponding entry in the segment directory.
- supported_kernels advertises which query kernels the engine can serve when
  operating over this artifact.

### 2. String Dictionary (SegmentType::StringDict = 2)

Optional segment that stores a compact dictionary of UTF-8 strings used elsewhere in the file.

Layout (little-endian):

- magic: SDCT (4 bytes)
- version_major: u16 (currently 0)
- version_minor: u16 (currently 1)
- string_count: u32
- blob_len: u32 (total bytes in concatenated UTF-8 blob)
- offsets: (string_count + 1) × u32
- blob: lob_len bytes of concatenated UTF-8 string data

Semantics:
- offsets[i] and offsets[i+1] are the byte range of string i within lob.
- Offsets MUST start at 0 and be non-decreasing; offsets[string_count] MUST equal
  lob_len.
- Strings MUST be valid UTF-8.

### 3. Object Table (SegmentType::ObjectTable = 3)

Optional segment that provides a minimal, fixed-width mapping from object IDs to basic metadata.
Each row index corresponds to an implicit object_id.

Layout (little-endian):

- magic: OBT\0 (4 bytes)
- version_major: u16 (currently 0)
- version_minor: u16 (currently 1)
- object_count: u32
- reserved: u32 (must be 0)
- rows: object_count ×:
  - type_sid: u32 (StringDict ID for an object's type/kind)
  - name_sid: u32 (StringDict ID for a primary name/label)
  - flags: u32 (reserved for future use)

Semantics:

- object_id is the row index (0-based) in the table.
- All human-readable labels referenced here must come from the StringDict segment
  via 	ype_sid and 
ame_sid.

---

## Objects (logical view, v0.1)

RNB defines a simple logical object model layered on top of the Object
Table segment:

- each row in the Object Table corresponds to an `object_id` equal to
  the row index (0-based);
- the `type_sid` and `name_sid` fields are string identifiers into the
  global String Dictionary segment;
- additional optional segments (AttributeTable, RelationTable,
  NumericMatrix) may attach further metadata, relationships, or
  payloads to objects by referring to their `object_id`.

The runtime exposes this as a stable `Object` view with fields:

- `id` (u32) — the object_id;
- `type_sid` (u32) — type/kind identifier (StringDict ID);
- `name_sid` (u32) — primary label/name (StringDict ID);
- `flags` (u32) — reserved for future use.

This logical view is derived purely from the Object Table segment and
does not constrain later layout optimizations (e.g. columnar storage),
as long as the semantics above are preserved.

---

## Design Rules

- Unknown segment types MUST be skippable
- All data is stored in typed segments
- No JSON blobs in core storage
