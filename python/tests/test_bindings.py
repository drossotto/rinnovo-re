import os
import pathlib

import rinnovo


def test_hello():
    assert rinnovo.hello() == "rinnovo ready"


def test_write_and_open_empty(tmp_path: pathlib.Path):
    path = tmp_path / "empty.rnb"

    rinnovo.write_empty(str(path))
    assert path.exists()

    f = rinnovo.open(str(path))

    # Header invariants
    assert bytes(f.header.magic) == b"RNB\0"
    assert f.header.dir_len > 0

    # Manifest invariants
    assert f.manifest.max_chunk_bytes > 0
    assert rinnovo.SEGMENT_MANIFEST in f.manifest.required_segments


def test_read_manifest_bytes(tmp_path: pathlib.Path):
    path = tmp_path / "manifest_bytes.rnb"
    rinnovo.write_empty(str(path))

    raw = rinnovo.read_manifest_bytes(str(path))
    assert isinstance(raw, (bytes, bytearray))
    assert b"MNF\0" in raw[:16]
