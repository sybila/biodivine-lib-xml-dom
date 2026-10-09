"""Module-level parsing and serialization helpers.

These are the Pythonic entry points: ``parse`` accepts ``str`` or ``bytes``, and the file variants
accept anything `os.fspath` understands (``str``, ``pathlib.Path``, ...).
"""

from __future__ import annotations

from typing import Optional, Union

from . import _convert, _sys
from .document import Document
from .write_options import WriteOptions

__all__ = ["parse", "parse_file", "write", "write_file"]


def parse(source: Union[str, bytes]) -> Document:
    """Parses an XML document from a string or from bytes (which must be valid UTF-8).

    Raises `XmlSyntaxError`/`XmlNamespaceError` for malformed input.
    """
    if isinstance(source, bytes):
        return Document._wrap(_sys.parse_bytes(source))
    return Document._wrap(_sys.parse_string(source))


def parse_file(path: "_convert.PathLike") -> Document:
    """Parses an XML document from a file (``str``, ``pathlib.Path``, or any ``os.PathLike``)."""
    return Document._wrap(_sys.parse_file(_convert.to_path(path)))


def write(document: Document, options: Optional[WriteOptions] = None) -> str:
    """Serializes ``document`` into a string.

    ``options`` selects the declaration and empty-element style; the defaults reproduce a parsed
    document as closely as the data model allows.
    """
    return _sys.write_string_with(document._document, options)


def write_file(
    document: Document,
    path: "_convert.PathLike",
    options: Optional[WriteOptions] = None,
) -> None:
    """Writes ``document`` to a file."""
    _sys.write_file_with(document._document, _convert.to_path(path), options)
