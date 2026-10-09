"""Argument coercion and result wrapping, shared by the wrapper classes.

The pure-Python layer never exposes a ``_sys`` object directly: every method that returns a
document, node or element wraps it in the corresponding Python class first, so users only ever see
the Pythonic types and ``isinstance`` works as expected.

The wrapper classes register themselves here at import time (:func:`register`), which keeps this
module free of imports of its siblings and therefore keeps the package's import graph acyclic.
Coercion is duck-typed on the private ``_node``/``_element``/``_name``/``_namespace`` attributes, so
it works for the wrapper classes and for their subclasses alike.
"""

from __future__ import annotations

import os
from typing import Any, Dict, Iterable, Optional, Tuple, Type, Union

PathLike = Union[str, "os.PathLike[str]"]

#: ``_sys`` class -> Python wrapper class. Filled in by the modules themselves.
_WRAPPERS: Dict[type, type] = {}


def register(sys_class: type, wrapper_class: type) -> None:
    """Registers the Python wrapper for a native class."""
    _WRAPPERS[sys_class] = wrapper_class


def unwrap_node(value: Any) -> Any:
    """Returns the native node/element behind ``value``.

    Accepts a :class:`~biodivine_lib_xml_dom.Node`, an
    :class:`~biodivine_lib_xml_dom.Element`, or an already-unwrapped native object.
    """
    inner = getattr(value, "_node", None)
    return value if inner is None else inner


def unwrap_element(value: Any) -> Any:
    """Returns the native element behind ``value``."""
    inner = getattr(value, "_element", None)
    if inner is not None:
        return inner
    element = unwrap_node(value).as_element()
    if element is None:
        raise TypeError("this node is not an element")
    return element


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

    if isinstance(value, _sys.QualifiedName):
        # Already native: this happens when a caller mixes the two layers explicitly.
        return value
    if isinstance(value, tuple):
        if len(value) != 2:
            raise TypeError(
                "a name given as a tuple must be (local_name, namespace_or_uri), got "
                f"{len(value)} elements"
            )
        from .name import QualifiedName

        return QualifiedName(value[0], value[1])._name
    return _sys.QualifiedName.without_namespace(value)


def unwrap_namespace(value: Any) -> Any:
    """Returns the native namespace behind ``value``; a string is read as a URI."""
    inner = getattr(value, "_namespace", None)
    if inner is not None:
        return inner
    from . import _sys

    if isinstance(value, _sys.Namespace):
        return value
    return _sys.Namespace(value)


def to_path(value: PathLike) -> str:
    """Coerces ``value`` to a filesystem path (``str`` or any `os.PathLike`)."""
    return os.fspath(value)


def wrap(value: Any) -> Any:
    """Wraps a native result in its Python counterpart (leaving other values untouched)."""
    for sys_class, wrapper_class in _WRAPPERS.items():
        if isinstance(value, sys_class):
            return wrapper_class._wrap(value)  # type: ignore[attr-defined]
    return value


def wrap_many(values: Iterable[Any]) -> list:
    """Wraps every element of ``values``."""
    return [wrap(value) for value in values]


def optional(value: Optional[Any]) -> Any:
    """Wraps ``value`` unless it is ``None``."""
    return None if value is None else wrap(value)


def wrap_namespaces(values: Iterable[Tuple[Optional[str], Any]]) -> Dict[Optional[str], Any]:
    """Turns a list of ``(prefix, namespace)`` pairs into a dict, wrapping the namespaces."""
    return {prefix: optional(namespace) for prefix, namespace in values}
