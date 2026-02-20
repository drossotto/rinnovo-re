import pathlib

import rinnovo_sdk
import rinnovo


def test_sdk_artifact_open(tmp_path: pathlib.Path):
    path = tmp_path / "sdk_artifact.rnb"

    rinnovo.write_empty(str(path))
    assert path.exists()

    art = rinnovo_sdk.Artifact.open(path)

    # Header invariants via SDK
    assert bytes(art.header.magic) == b"RNB\0"
    assert art.header.dir_len > 0

    # Manifest invariants via SDK, using both direct access and helper
    assert rinnovo.SEGMENT_MANIFEST in art.required_segments
    assert art.has_segment_type(rinnovo.SEGMENT_MANIFEST)

