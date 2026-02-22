# `open_rnb` flow

This diagram documents the control flow and validation logic inside `rnb_format::open_rnb`, which reconstructs an `RnbFile` from a `.rnb` artifact on disk.

```mermaid
flowchart TD
    A[open_rnb(path)] --> B[File::open(path)]
    B --> C[RnbHeader::read_from]
    C --> D[seek to header.dir_offset]
    D --> E[RnbDirectory::read_from(len = header.dir_len)]

    E --> F[find manifest entry in directory]
    F --> G[read_manifest_bytes via read_segment_bytes]
    G --> H[checksum64_fnv1a(manifest_bytes)]

    H -->|mismatch| X[Error: InvalidData\n\"manifest checksum mismatch\"]
    H -->|ok| I[Manifest::read_from]

    I --> J[for each SegmentType in manifest.required_segments]
    J -->|missing| Y[Error: InvalidData\n\"missing required segment\"]
    J -->|all present| K[load optional segments]

    K --> S[StringDict?]
    S --> S1{entry present?}
    S1 -->|no| O1[next]
    S1 -->|yes| S2[read bytes + checksum\nStringDict::from_bytes]

    O1 --> O[ObjectTable?]
    O --> O2{entry present?}
    O2 -->|no| A1[next]
    O2 -->|yes| O3[read bytes + checksum\nObjectTable::read_from]

    A1 --> AT[AttributeTable?]
    AT --> AT2{entry present?}
    AT2 -->|no| R1[next]
    AT2 -->|yes| AT3[read bytes + checksum\nAttributeTable::read_from]

    R1 --> RT[RelationTable?]
    RT --> RT2{entry present?}
    RT2 -->|no| N1[next]
    RT2 -->|yes| RT3[read bytes + checksum\nRelationTable::read_from]

    N1 --> NM[NumericMatrix?]
    NM --> NM2{entry present?}
    NM2 -->|no| Q[construct RnbFile]
    NM2 -->|yes| NM3[read bytes + checksum\nNumericMatrix::read_from]

    S2 --> O1
    O3 --> A1
    AT3 --> R1
    RT3 --> N1
    NM3 --> Q

    Q --> R[return RnbFile {
      header,
      directory,
      manifest,
      string_dict,
      object_table,
      attribute_table,
      relation_table,
      numeric_matrix
    }]
```

