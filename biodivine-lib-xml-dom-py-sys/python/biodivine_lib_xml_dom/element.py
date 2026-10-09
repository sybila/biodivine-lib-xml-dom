"""Pythonic wrapper for an element handle."""

from __future__ import annotations

from typing import TYPE_CHECKING, Dict, List, Optional, Union

from . import _convert, _sys
from .name import QualifiedName
from .namespace import Namespace
from .node import Node

if TYPE_CHECKING:  # pragma: no cover - import cycle is broken on purpose
    from .document import Document


class Element(Node):
    """A handle to an element node.

    Inherits the whole tree API from :class:`Node` (so ``element.children()``, ``element.detach()``
    and friends work directly) and adds the element-specific surface: the expanded name, attributes
    and namespace declarations.

    Attribute access is Pythonic on top of the expanded-name API:
    ``element.get("class")`` looks up an attribute with that local name and no namespace,
    ``element.set("class", "main")`` does the same for writing, and ``element.attributes`` is a
    plain ``dict``.
    """

    __slots__ = ("_element",)

    def __init__(self, element: "_sys.Element", node: Optional["_sys.Node"] = None) -> None:
        self._element = element
        super().__init__(node if node is not None else element.node())

    @classmethod
    def _wrap(cls, element: "_sys.Element", node: Optional["_sys.Node"] = None) -> "Element":
        return cls(element, node)

    @property
    def node(self) -> Node:
        """This element as a plain :class:`Node` handle.

        An element *is* a node, so this returns the element itself; it exists so that code written
        against the native API (``element.node().children()``) also reads naturally in Python.
        """
        return self

    # -- name -----------------------------------------------------------------------------
    @property
    def qualified_name(self) -> QualifiedName:
        """The expanded name of this element."""
        return QualifiedName._wrap(self._element.qualified_name())

    @property
    def local_name(self) -> str:
        """The local name of this element."""
        return self._element.local_name()

    @property
    def namespace(self) -> Optional[Namespace]:
        """The namespace of this element, or ``None``."""
        inner = self._element.namespace()
        return None if inner is None else Namespace._wrap(inner)

    @local_name.setter
    def local_name(self, value: str) -> None:
        self._element.set_qualified_name(_sys.QualifiedName.without_namespace(value))

    def set_qualified_name(self, name: Union[QualifiedName, str]) -> None:
        """Changes the name of this element. No namespace declaration is touched."""
        self._element.set_qualified_name(_convert.unwrap_name(name))

    # -- attributes -----------------------------------------------------------------------
    @property
    def attributes(self) -> Dict[QualifiedName, str]:
        """The attributes, as a ``{QualifiedName: value}`` dict."""
        return {
            QualifiedName._wrap(name): value for name, value in self._element.attributes()
        }

    def attribute(self, name: Union[QualifiedName, str, tuple]) -> Optional[str]:
        """The value of the attribute with the given expanded name, or ``None``.

        Accepts the same three spellings as every other name argument: a ``QualifiedName``, a plain
        ``str`` (meaning "no namespace"), or a ``(local_name, namespace_or_uri)`` tuple.
        """
        return self._element.attribute(_convert.unwrap_name(name))

    def get(self, name: Union[QualifiedName, str, tuple], default: Optional[str] = None) -> Optional[str]:
        """Like :meth:`attribute`, but with a default instead of ``None``."""
        value = self.attribute(name)
        return default if value is None else value

    def has_attribute(self, name: Union[QualifiedName, str, tuple]) -> bool:
        """Whether an attribute with that expanded name exists."""
        return self.attribute(name) is not None

    def set_attribute(self, name: Union[QualifiedName, str, tuple], value: str) -> None:
        """Sets an attribute, overwriting any previous value with the same expanded name.

        Raises `XmlSyntaxError` if the value contains characters that are not legal in XML.
        """
        self._element.set_attribute(_convert.unwrap_name(name), value)

    def remove_attribute(self, name: Union[QualifiedName, str, tuple]) -> Optional[str]:
        """Removes an attribute and returns its previous value, or ``None``."""
        return self._element.remove_attribute(_convert.unwrap_name(name))

    def clear_attributes(self) -> None:
        """Removes all attributes."""
        self._element.clear_attributes()

    # -- namespaces -----------------------------------------------------------------------
    @property
    def namespace_declarations(self) -> Dict[Optional[str], Optional[Namespace]]:
        """The declarations written on *this* element, without inheritance.

        Keys are prefixes (``None`` for the default namespace); a value of ``None`` is an empty
        declaration (``xmlns=""``).
        """
        return {
            prefix: None if namespace is None else Namespace._wrap(namespace)
            for prefix, namespace in self._element.namespace_declarations()
        }

    def namespaces_in_scope(self) -> List[tuple]:
        """All bindings visible to this element, innermost first."""
        return [
            (prefix, None if namespace is None else Namespace._wrap(namespace))
            for prefix, namespace in self._element.namespaces_in_scope()
        ]

    def declare_namespace(self, namespace: Union[Namespace, str], prefix: Optional[str] = None) -> None:
        """Declares a namespace on this element, overwriting any binding for that prefix."""
        if isinstance(namespace, str):
            namespace = Namespace(namespace, prefix)
        self._element.declare_namespace(namespace._namespace)

    def declare_namespace_checked(self, namespace: Union[Namespace, str]) -> None:
        """Declares a namespace, refusing to *change* an existing binding for the prefix."""
        if isinstance(namespace, str):
            namespace = Namespace(namespace)
        self._element.declare_namespace_checked(namespace._namespace)

    def undeclare_default_namespace(self) -> None:
        """Declares ``xmlns=""``, removing the default namespace from this element's scope."""
        self._element.undeclare_default_namespace()

    def remove_namespace_declaration(self, prefix: Optional[str] = None) -> Optional[Optional[Namespace]]:
        """Removes a declaration (``None`` = the default namespace) and returns its previous value."""
        previous = self._element.remove_namespace_declaration(prefix)
        if previous is None:
            return None
        return None if previous is None else Namespace._wrap(previous)

    def get_namespace(self, prefix: Optional[str] = None) -> Optional[Namespace]:
        """The namespace bound to ``prefix`` in this element's scope, or ``None``.

        The predefined ``xml`` prefix needs no declaration and is not resolved here; use
        :meth:`resolve_attribute_name`/:meth:`resolve_qualified_name` for that.
        """
        inner = self._element.get_namespace(prefix)
        return None if inner is None else Namespace._wrap(inner)

    def resolve_qualified_name(self, name: str) -> QualifiedName:
        """Resolves ``"prefix:local"``/``"local"`` against the scope of this element."""
        return QualifiedName._wrap(self._element.resolve_qualified_name(name))

    def resolve_attribute_name(self, name: str) -> QualifiedName:
        """Resolves an attribute name against the scope of this element (the default namespace
        never applies to attributes)."""
        return QualifiedName._wrap(self._element.resolve_attribute_name(name))

    # -- element-specific tree operations --------------------------------------------------
    def append_child(self, child: Union[Node, Element, "_sys.Node", "_sys.Element"]) -> None:
        """Appends ``child`` to this element (see :meth:`Node.append_child`)."""
        self._element.append_child(_convert.unwrap_node(child))

    def insert_child(self, index: int, child: Union[Node, Element, "_sys.Node", "_sys.Element"]) -> None:
        """Inserts ``child`` as the ``index``-th child of this element."""
        self._element.node().insert_child(index, _convert.unwrap_node(child))

    def deep_clone(self) -> "Element":
        """A detached deep copy of this element, in the same document."""
        return Element._wrap(self._element.deep_clone())

    def shallow_clone(self) -> "Element":
        """A detached copy of this element without children, in the same document."""
        return Element._wrap(self._element.shallow_clone())

    def deep_clone_into(self, target: "Document") -> "Element":
        """A detached deep copy of this element in ``target``."""
        return Element._wrap(self._element.deep_clone_into(target._document))

    def shallow_clone_into(self, target: "Document") -> "Element":
        """A detached copy of this element (no children) in ``target``."""
        return Element._wrap(self._element.shallow_clone_into(target._document))

    def __repr__(self) -> str:
        return f"Element({str(self.qualified_name)!r}, id={int(self.id)})"



_convert.register(_sys.Element, Element)
