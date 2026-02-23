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

    # On an empty artifact, object helpers should behave safely.
    assert art.get_object(0) is None
    objs = art.objects_by_type(1)
    assert isinstance(objs, list)
    assert len(objs) == 0


def test_sdk_attributes_and_relations(tmp_path: pathlib.Path):
    path = tmp_path / "sdk_attrs_rels.rnb"

    # For now, reuse the empty-artifact helper; this artifact will
    # not contain attribute or relation tables.
    rinnovo.write_empty(str(path))

    art = rinnovo_sdk.Artifact.open(path)

    # Attribute helper should behave safely and return an empty list.
    attrs = art.attributes(1)
    assert isinstance(attrs, list)
    assert attrs == []

    # Relations helper on an artifact without a RelationTable should
    # also return an empty list for all filters.
    rels = art.relations(src_id=0)
    assert isinstance(rels, list)
    assert rels == []

    rels_filtered = art.relations(dst_id=1, rel_type_sid=10)
    assert isinstance(rels_filtered, list)
    assert rels_filtered == []
