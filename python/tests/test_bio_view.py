from __future__ import annotations

from dataclasses import dataclass
from typing import List

from rinnovo_sdk import BioView


@dataclass
class _FakeArtifact:
    _strings: List[str]
    _by_type: dict[int, list[object]]

    def strings(self) -> List[str]:
        return list(self._strings)

    def objects_by_type(self, type_sid: int) -> list[object]:
        return list(self._by_type.get(type_sid, []))


def test_bio_view_from_artifact_requires_strings():
    art = _FakeArtifact(_strings=[], _by_type={})
    assert BioView.from_artifact(art) is None

    art2 = _FakeArtifact(_strings=["cell"], _by_type={0: ["c1"]})
    bio = BioView.from_artifact(art2)
    assert bio is not None


def test_bio_view_cells_and_genes_by_label():
    # StringDict: 0 -> "cell", 1 -> "gene"
    art = _FakeArtifact(
        _strings=["cell", "gene"],
        _by_type={
            0: ["cell-0", "cell-1"],
            1: ["gene-0"],
        },
    )

    bio = BioView.from_artifact(art)
    assert bio is not None

    cells = bio.cells()
    genes = bio.genes()

    assert cells == ["cell-0", "cell-1"]
    assert genes == ["gene-0"]

