from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Union

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

