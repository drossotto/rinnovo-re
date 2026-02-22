from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional, Union

import rinnovo


PathLike = Union[str, Path]


@dataclass
class Artifact:
    """
    High-level view over an RNB artifact.

    Wraps the low-level `rinnovo.open()` result and exposes a
    convenient interface for common operations. The underlying file
    remains the same object returned by the Rust bindings.
    """

    path: Path
    _inner: object

    @classmethod
    def open(cls, path: PathLike) -> "Artifact":
        """Open an RNB artifact from the given path."""
        p = Path(path)
        inner = rinnovo.open(str(p))
        return cls(path=p, _inner=inner)

    @property
    def header(self):
        """Return the parsed header (from the Rust bindings)."""
        return self._inner.header

    @property
    def manifest(self):
        """Return the parsed manifest (from the Rust bindings)."""
        return self._inner.manifest

    @property
    def required_segments(self) -> Iterable[int]:
        """Numeric `SegmentType` identifiers that are required."""
        return list(self._inner.manifest.required_segments)

    def has_segment_type(self, segment_type_id: int) -> bool:
        """Check if a given SegmentType (by numeric id) is required."""
        return segment_type_id in self._inner.manifest.required_segments

    # --- Virtual object helpers -------------------------------------------------

    def get_object(self, object_id: int) -> Optional[object]:
        """
        Look up a single logical object by its ID.

        Returns a `rinnovo.Object` instance or None.
        """
        return rinnovo.get_object(str(self.path), int(object_id))

    def objects_by_type(self, type_sid: int) -> List[object]:
        """
        Return all logical objects whose `type_sid` matches the given value.

        The elements are `rinnovo.Object` instances.
        """
        return list(rinnovo.objects_by_type(str(self.path), int(type_sid)))
