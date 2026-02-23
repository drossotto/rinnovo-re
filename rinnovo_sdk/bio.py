from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, List, Optional

from .artifact import Artifact


@dataclass
class BioView:
    """
    High-level biological helpers over an Artifact.

    This mirrors the Rust `BioView` semantics in a Python-friendly way:
    it uses the artifact's StringDict to resolve type labels like
    ``"cell"`` and ``"gene"`` into SIDs, then delegates to the SDK's
    object helpers.
    """

    artifact: Artifact
    _strings: List[str]
    _index: Dict[str, int]

    @classmethod
    def from_artifact(cls, artifact: Artifact) -> Optional["BioView"]:
        """
        Construct a BioView if the artifact has a non-empty StringDict.

        Returns None if the StringDict is missing or empty.
        """
        strings = artifact.strings()
        if not strings:
            return None
        index = {s: i for i, s in enumerate(strings)}
        return cls(artifact=artifact, _strings=strings, _index=index)

    # --- Internal helpers -----------------------------------------------------

    def _type_sid_for(self, label: str) -> int:
        try:
            return self._index[label]
        except KeyError as exc:  # label not present
            raise ValueError(f"type label {label!r} not found in StringDict") from exc

    def _objects_by_type_label(self, label: str):
        sid = self._type_sid_for(label)
        return self.artifact.objects_by_type(sid)

    # --- Public biological views ---------------------------------------------

    def cells(self):
        """Return all objects whose type label is ``\"cell\"``."""
        return self._objects_by_type_label("cell")

    def genes(self):
        """Return all objects whose type label is ``\"gene\"``."""
        return self._objects_by_type_label("gene")
