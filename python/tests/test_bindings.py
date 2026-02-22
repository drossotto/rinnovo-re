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


def test_object_kernels_on_empty_artifact(tmp_path: pathlib.Path):
    path = tmp_path / "objects_empty.rnb"
    rinnovo.write_empty(str(path))

    # No object table is present, so kernels should behave gracefully.
    obj = rinnovo.get_object(str(path), 0)
    assert obj is None

    objs = rinnovo.objects_by_type(str(path), 1)
    assert isinstance(objs, list)
    assert len(objs) == 0

    # Kernel constants should be exposed.
    assert isinstance(rinnovo.KERNEL_GET_OBJECT_BY_ID, int)
    assert isinstance(rinnovo.KERNEL_OBJECTS_BY_TYPE, int)
