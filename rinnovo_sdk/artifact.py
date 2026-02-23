from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Union

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

    # --- String dictionary helpers --------------------------------------------

    def strings(self) -> List[str]:
        """
        Return the underlying StringDict as a list of strings.

        If the artifact has no StringDict segment, returns an empty list.
        """
        # Use the in-memory artifact held by the bindings.
        return list(self._inner.list_strings())

    def string_index(self) -> Dict[str, int]:
        """
        Return a mapping from string label to its SID.

        This is derived from `strings()`, using the list index as the SID.
        """
        return {s: i for i, s in enumerate(self.strings())}

    # --- Virtual object helpers -------------------------------------------------

    def get_object(self, object_id: int) -> Optional[object]:
        """
        Look up a single logical object by its ID.

        Returns a `rinnovo.Object` instance or None.
        """
        return self._inner.get_object(int(object_id))

    def objects_by_type(self, type_sid: int) -> List[object]:
        """
        Return all logical objects whose `type_sid` matches the given value.

        The elements are `rinnovo.Object` instances.
        """
        return list(self._inner.objects_by_type(int(type_sid)))

    # --- Attribute and relation helpers ----------------------------------------

    def attributes(self, object_id: int) -> List[tuple[int, int, int, int]]:
        """
        Return all attribute records attached to the given object.

        Each element is a tuple ``(object_id, key_sid, value_sid, flags)``.
        """
        return list(self._inner.list_attributes(int(object_id)))

    def relations(
        self,
        src_id: int | None = None,
        dst_id: int | None = None,
        rel_type_sid: int | None = None,
    ) -> List[tuple[int, int, int, int]]:
        """
        Return relation records filtered by source, destination, and type.

        Each element is a tuple ``(src_id, dst_id, rel_type_sid, flags)``.
        """
        return list(
            self._inner.list_relations(
                None if src_id is None else int(src_id),
                None if dst_id is None else int(dst_id),
                None if rel_type_sid is None else int(rel_type_sid),
            )
        )
