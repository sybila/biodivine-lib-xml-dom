"""Parsing and serialization: the Python entry points.

Rust reaches parsing and serialization through methods of ``Document``, and the mirror layer
(``biodivine_lib_xml_dom._sys``) follows that shape. Python's own idiom is a module-level function,
so this module is where the documented entry points live: ``parse`` accepts ``str`` or ``bytes``,
``write`` returns XML text, and the ``*_file`` variants accept anything `os.fspath` understands
(``str``, ``pathlib.Path``, ``os.PathLike``). Each one is a thin wrapper around the corresponding
``Document`` method, so there is exactly one implementation.
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
        return Document._wrap(_sys.Document.parse_bytes(source))
    return Document._wrap(_sys.Document.parse_string(source))


def parse_file(path: "_convert.PathLike") -> Document:
    """Parses an XML document from a file (``str``, ``pathlib.Path``, or any ``os.PathLike``)."""
    return Document._wrap(_sys.Document.parse_file(_convert.to_path(path)))


def write(document: Document, options: Optional[WriteOptions] = None) -> str:
    """Serializes ``document`` into a string.

    ``options`` selects the declaration and empty-element style; the defaults reproduce a parsed
    document as closely as the data model allows.
    """
    return document._document.write_string(options)


def write_file(
    document: Document,
    path: "_convert.PathLike",
    options: Optional[WriteOptions] = None,
) -> None:
    """Writes ``document`` to a file."""
    document._document.write_file(_convert.to_path(path), options)
