"""Argument coercion and result wrapping, shared by the wrapper classes.

The pure-Python layer never exposes a ``_sys`` object: every method that returns a document, node,
element, name or namespace wraps it in the corresponding Python class first, so users only ever see
the Pythonic types and ``isinstance`` works as expected.

The wrapper classes register themselves here at import time (:func:`register`), which keeps this
module free of imports of its siblings and therefore keeps the package's import graph acyclic.
"""

from __future__ import annotations

import os
from typing import Any, Dict, Iterable, List, Optional, Union

PathLike = Union[str, "os.PathLike[str]"]

#: ``_sys`` class -> Python wrapper class. Filled in by the modules themselves.
_WRAPPERS: Dict[type, type] = {}


def register(sys_class: type, wrapper_class: type) -> None:
    """Registers the Python wrapper for a native class."""
    _WRAPPERS[sys_class] = wrapper_class


def unwrap_node(value: Any) -> Any:
    """Returns the native node behind a :class:`~biodivine_lib_xml_dom.Node` (or ``Element``)."""
    inner = getattr(value, "_node", None)
    if inner is None:
        raise TypeError(f"expected a Node or an Element, got {type(value).__name__}")
    return inner


def unwrap_element(value: Any) -> Any:
    """Returns the native element behind an :class:`~biodivine_lib_xml_dom.Element`."""
    inner = getattr(value, "_element", None)
    if inner is not None:
        return inner
    element = unwrap_node(value).as_element()
    if element is None:
        raise TypeError("this node is not an element")
    return element


def unwrap_document(value: Any) -> Any:
    """Returns the native document behind a :class:`~biodivine_lib_xml_dom.Document`."""
    inner = getattr(value, "_document", None)
    if inner is None:
        raise TypeError(f"expected a Document, got {type(value).__name__}")
    return inner


def unwrap_name(value: Any) -> Any:
    """Returns the native qualified name behind ``value``.

    Three spellings are accepted wherever a name is expected:

    * ``QualifiedName`` - the explicit form;
    * ``str`` - a name with no namespace, which is by far the common case;
    * ``(local_name, namespace)`` - a tuple, where the namespace may be a :class:`Namespace`, a
      URI string, or ``None`` for no namespace::

          document.create_element(("child", Namespace("http://example.com", "ex")))
          document.create_element(("child", "http://example.com"))
          element.set_attribute(("class", None), "main")
    """
    inner = getattr(value, "_name", None)
    if inner is not None:
        return inner
    from . import _sys

    if isinstance(value, tuple):
        if len(value) != 2:
            raise TypeError(
                "a name given as a tuple must be (local_name, namespace_or_uri), got "
                f"{len(value)} elements"
            )
        from .name import QualifiedName

        return QualifiedName(value[0], value[1])._name
    if isinstance(value, str):
        return _sys.QualifiedName.without_namespace(value)
    raise TypeError(
        f"expected a QualifiedName, a str or a (local_name, namespace) tuple, "
        f"got {type(value).__name__}"
    )


def to_path(value: PathLike) -> str:
    """Coerces ``value`` to a filesystem path (``str`` or any `os.PathLike`)."""
    return os.fspath(value)


def wrap(value: Any) -> Any:
    """Wraps a native result in its Python counterpart."""
    wrapper_class = _WRAPPERS.get(type(value))
    if wrapper_class is None:
        raise TypeError(f"no Python wrapper for {type(value).__name__}")
    return wrapper_class._wrap(value)  # type: ignore[attr-defined]


def wrap_many(values: Iterable[Any]) -> List[Any]:
    """Wraps every element of ``values``."""
    return [wrap(value) for value in values]


def optional(value: Optional[Any]) -> Any:
    """Wraps ``value`` unless it is ``None``."""
    return None if value is None else wrap(value)
