# Kernel Overview

This document summarizes the *current* kernel surface in the Rinnovo
codebase. It is intentionally descriptive rather than aspirational so
that future work on sparse matrices / bio‑level views has a clear
baseline.

## 1. RNB format kernels (`rnb_format`)

### 1.1 Segment types

Defined in `crates/kernel/rnb_format/src/segment.rs` as `SegmentType`:

- `Manifest` – semantic description of the artifact and its contracts.
- `StringDict` – deduplicated UTF‑8 strings (type labels, names, etc.).
- `ObjectTable` – rows of logical objects with basic metadata.
- `AttributeTable` – sparse key/value metadata attached to objects.
- `RelationTable` – edges between objects.
- `NumericMatrix` – dense numeric matrix payload.

These are *physical* kernels: they describe what kind of segments are
stored on disk and how they are identified in the directory.

### 1.2 Query kernels

Also in `segment.rs`, `QueryKernel` is the first logical kernel enum
that can be advertised by the `Manifest`:

- `GetObjectById`
- `ObjectsByType`

The `Manifest` holds a `supported_kernels: Vec<QueryKernel>`. When it
is non‑empty, the runtime `Artifact.execute` API enforces that a
requested kernel is declared there before running it.

## 2. Runtime artifact kernels (`rnb_engine`)

All runtime kernels are expressed on top of the high‑level `Artifact`
wrapper in `crates/runtime/rnb_engine`.

### 2.1 Object kernels

Implemented in `object.rs`:

- `Artifact::object_count()`  
  Returns the number of objects if an `ObjectTable` is present.

- `Artifact::get_object(id: u32) -> Option<Object>`  
  Reads a single row from the `ObjectTable` and materializes it as an
  `Object` view (`id`, `type_sid`, `name_sid`, `flags`).

- `Artifact::execute(kernel: QueryKernel, arg: u32) -> io::Result<Vec<Object>>`  
  Generic executor for format‑level object kernels:
  - `GetObjectById` – `arg` is `object_id`; result is empty or a
    single `Object`.
  - `ObjectsByType` – `arg` is `type_sid`; result is all objects with
    that type.

- `Artifact::objects_by_type(type_sid: u32)`  
  Convenience wrapper over `execute(ObjectsByType, ...)`.

These are the core *object kernels* and currently the only ones wired
to the `Manifest.supported_kernels` mechanism.

### 2.2 Attribute and relation helpers

In `artifact.rs`:

- `Artifact::attributes_for_object(object_id)`  
  Returns an iterator over `AttributeRecord` for a single object.

- `Artifact::relations_from(src_id, rel_type_sid)`  
- `Artifact::relations_to(dst_id, rel_type_sid)`  

These behave like kernels but are currently helper methods, not part of
the `QueryKernel` enum.

### 2.3 Numeric matrix kernel (dense)

At the format level `NumericMatrix` is defined in
`numeric_matrix.rs` as a dense, row‑major `f32` matrix:

- Metadata: `rows`, `cols`, `elem_type` (`NumericType::F32`).
- Payload: `values: Vec<f32>` of length `rows * cols`.
- Read/write methods: `write_to`, `read_from` with a small magic +
  version header.

The runtime `Artifact` currently exposes this as:

- `Artifact::numeric_matrix() -> Option<&NumericMatrix>`

There is *no* higher‑level matrix kernel yet (no slicing, no sparse
encoding, no bio‑axis binding). It is essentially a typed payload with
validation.

## 3. Bio‑level kernels (`BioView`)

Defined in `crates/runtime/rnb_engine/src/bio.rs` as `BioView<'a>`:

- Construction: `BioView::from_artifact(&Artifact) -> Option<BioView>`
  - Requires both a `StringDict` and an `ObjectTable`.

- Internal helper: `type_sid_for(label: &str)`  
  Finds the `StringDict` index for a type label such as `"cell"` or
  `"gene"`.

- Kernels:
  - `cells() -> io::Result<Vec<Object>>` – all objects whose
    `type_sid` corresponds to `"cell"`.
  - `genes() -> io::Result<Vec<Object>>` – all objects whose
    `type_sid` corresponds to `"gene"`.

These functions are thin, semantic wrappers over the object kernels
(`objects_by_type`) and the `StringDict`. They currently assume the
presence of canonical labels `"cell"` and `"gene"` in the dictionary.

## 4. HTTP kernels (`rnb_engine_http`)

Exposed in `crates/runtime/rnb_engine_http/src/main.rs` via Axum:

- `GET /health`  
  Returns `{ status: "ok", version: <pkg version> }`.

- `GET /engine/v1/artifact/summary?path=...`  
  Uses `Artifact` to report:
  - `path`
  - `object_count`
  - `has_string_dict`
  - `has_attribute_table`
  - `has_relation_table`

- `GET /engine/v1/artifact/bio/cells?path=...`  
- `GET /engine/v1/artifact/bio/genes?path=...`  
  Both use `BioView` and serialize a `BioObjectsResponse`:
  - `path`
  - `kind` (`"cells"` / `"genes"`)
  - `object_ids: Vec<u32>`

These endpoints are the first *remote* kernels for the virtual layer:
they expose object + bio kernels over HTTP and are used by tests and
will eventually be used by the SDK.

## 5. Gaps and future work

- No sparse matrix representation yet (CSR/CSC). `NumericMatrix` is
  dense and in‑memory only.
- No matrix‑aware kernels (e.g. block fetches, projections, pooled
  views) wired into `Artifact` or `BioView`.
- Attribute/relation helpers are not yet formalized as `QueryKernel`
  variants.
- The daemon currently just loads config; it does not yet expose its
  own public kernel surface (status, block cache, multi‑engine
  orchestration).

This document should be updated whenever new kernels are added or
existing ones grow significant new behavior so that the overall
execution surface of the virtual layer stays easy to reason about.

