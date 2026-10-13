"""Pythonic wrapper for an element handle."""

from __future__ import annotations

from typing import TYPE_CHECKING, Dict, List, Optional, Tuple, Union

from . import _convert, _sys
from .name import QualifiedName
from .namespace import Namespace
from .node import Node

if TYPE_CHECKING:  # pragma: no cover - import cycle is broken on purpose
    from .document import Document

NameLike = Union[QualifiedName, str, Tuple[str, Union[Namespace, str, None]]]


class Element(Node):
    """A handle to an element node.

    Inherits the whole tree API from :class:`Node` (so ``element.children()``, ``element.detach()``
    and friends work directly) and adds the element-specific surface: the expanded name, attributes
    and namespace declarations. Elements are created by :meth:`Document.create_element`; the class
    has no public constructor.

    Attribute access is Pythonic on top of the expanded-name API: ``element.get("class")`` looks up
    an attribute with that local name and no namespace, and ``element.attributes`` is a plain
    ``dict``.
    """

    __slots__ = ("_element",)

    @classmethod
    def _wrap(cls, element: "_sys.Element", node: Optional["_sys.Node"] = None) -> "Element":
        wrapper = object.__new__(Element)
        wrapper._element = element
        wrapper._node = element.node() if node is None else node
        return wrapper

    # -- name -----------------------------------------------------------------------------
    @property
    def qualified_name(self) -> QualifiedName:
        """The expanded name of this element."""
        return QualifiedName._wrap(self._element.qualified_name())

    @qualified_name.setter
    def qualified_name(self, name: NameLike) -> None:
        self.set_qualified_name(name)

    @property
    def local_name(self) -> str:
        """The local name of this element.

        Assigning to it renames the element and keeps its namespace.
        """
        return self._element.local_name()

    @local_name.setter
    def local_name(self, value: str) -> None:
        namespace = self._element.namespace()
        if namespace is None:
            name = _sys.QualifiedName.without_namespace(value)
        else:
            name = _sys.QualifiedName.with_namespace(value, namespace)
        self._element.set_qualified_name(name)

    @property
    def namespace(self) -> Optional[Namespace]:
        """The namespace of this element, or ``None``."""
        inner = self._element.namespace()
        return None if inner is None else Namespace._wrap(inner)

    def set_qualified_name(self, name: NameLike) -> None:
        """Changes the name of this element. No namespace declaration is touched."""
        self._element.set_qualified_name(_convert.unwrap_name(name))

    # -- attributes -----------------------------------------------------------------------
    @property
    def attributes(self) -> Dict[QualifiedName, str]:
        """The attributes, as a ``{QualifiedName: value}`` dict."""
        return {
            QualifiedName._wrap(name): value for name, value in self._element.attributes()
        }

    def attribute(self, name: NameLike) -> Optional[str]:
        """The value of the attribute with the given expanded name, or ``None``.

        Accepts the same three spellings as every other name argument: a ``QualifiedName``, a plain
        ``str`` (meaning "no namespace"), or a ``(local_name, namespace_or_uri)`` tuple.
        """
        return self._element.attribute(_convert.unwrap_name(name))

    def get(self, name: NameLike, default: Optional[str] = None) -> Optional[str]:
        """Like :meth:`attribute`, but with a default instead of ``None``."""
        value = self.attribute(name)
        return default if value is None else value

    def has_attribute(self, name: NameLike) -> bool:
        """Whether an attribute with that expanded name exists."""
        return self._element.has_attribute(_convert.unwrap_name(name))

    def set_attribute(self, name: NameLike, value: str) -> None:
        """Sets an attribute, replacing any previous attribute with the same expanded name.

        Raises `XmlSyntaxError` if the value contains characters that are not legal in XML.
        """
        self._element.set_attribute(_convert.unwrap_name(name), value)

    def remove_attribute(self, name: NameLike) -> Optional[str]:
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

    def namespaces_in_scope(self) -> List[Tuple[Optional[str], Optional[Namespace]]]:
        """All bindings visible to this element, innermost first."""
        return [
            (prefix, None if namespace is None else Namespace._wrap(namespace))
            for prefix, namespace in self._element.namespaces_in_scope()
        ]

    def declare_namespace(
        self, namespace: Union[Namespace, str], prefix: Optional[str] = None
    ) -> None:
        """Declares a namespace on this element, overwriting any binding for that prefix.

        ``namespace`` is a :class:`Namespace`, or a URI together with an optional ``prefix``.
        """
        self._element.declare_namespace(_namespace(namespace, prefix))

    def declare_namespace_checked(
        self, namespace: Union[Namespace, str], prefix: Optional[str] = None
    ) -> None:
        """Like :meth:`declare_namespace`, but refuses to *change* an existing binding.

        Raises `XmlNamespaceError` if the prefix is already declared on this element with a
        different URI.
        """
        self._element.declare_namespace_checked(_namespace(namespace, prefix))

    def undeclare_default_namespace(self) -> None:
        """Declares ``xmlns=""``, removing the default namespace from this element's scope."""
        self._element.undeclare_default_namespace()

    def remove_namespace_declaration(self, prefix: Optional[str] = None) -> Optional[Namespace]:
        """Removes the declaration of ``prefix`` (``None``: the default namespace) from this element.

        Returns the namespace that was declared, or ``None`` if the removed declaration was the empty
        ``xmlns=""``. Raises ``KeyError`` if this element has no declaration for ``prefix``, like
        ``dict.pop``.
        """
        removed, previous = self._element.remove_namespace_declaration(prefix)
        if not removed:
            raise KeyError(prefix)
        return None if previous is None else Namespace._wrap(previous)

    def lookup_namespace(self, prefix: Optional[str] = None) -> Optional[Namespace]:
        """The namespace bound to ``prefix`` in this element's scope, or ``None``.

        The predefined ``xml`` prefix needs no declaration and is not resolved here; use
        :meth:`resolve_element_name`/:meth:`resolve_attribute_name` for that.
        """
        inner = self._element.lookup_namespace(prefix)
        return None if inner is None else Namespace._wrap(inner)

    def resolve_element_name(self, name: str) -> QualifiedName:
        """Resolves ``"prefix:local"``/``"local"`` against the scope of this element."""
        return QualifiedName._wrap(self._element.resolve_element_name(name))

    def resolve_attribute_name(self, name: str) -> QualifiedName:
        """Resolves an attribute name against the scope of this element.

        The default namespace never applies to attributes.
        """
        return QualifiedName._wrap(self._element.resolve_attribute_name(name))

    # -- cloning, typed as Element ----------------------------------------------------------
    def deep_clone(self) -> "Element":
        """A detached deep copy of this element, in the same document."""
        return Element._wrap(self._element.deep_clone())

    def shallow_clone(self) -> "Element":
        """A detached copy of this element without children, in the same document."""
        return Element._wrap(self._element.shallow_clone())

    def deep_clone_into(self, target: "Document") -> "Element":
        """A detached deep copy of this element in ``target``."""
        return Element._wrap(self._element.deep_clone_into(_convert.unwrap_document(target)))

    def shallow_clone_into(self, target: "Document") -> "Element":
        """A detached copy of this element (no children) in ``target``."""
        return Element._wrap(
            self._element.shallow_clone_into(_convert.unwrap_document(target))
        )

    def __repr__(self) -> str:
        return f"Element({str(self.qualified_name)!r}, id={int(self.id)})"


def _namespace(namespace: Union[Namespace, str], prefix: Optional[str]) -> "_sys.Namespace":
    """The native namespace for a ``(namespace_or_uri, prefix)`` argument pair."""
    if isinstance(namespace, Namespace):
        if prefix is not None:
            raise TypeError("`prefix` can only be given together with a URI, not a Namespace")
        return namespace._namespace
    return Namespace(namespace, prefix)._namespace


_convert.register(_sys.Element, Element)
