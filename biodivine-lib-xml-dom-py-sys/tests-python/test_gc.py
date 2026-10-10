"""Garbage collection from Python: ids, collection and the handle lifetime."""

from __future__ import annotations

import threading

import biodivine_lib_xml_dom as xml


def test_an_id_is_opaque_until_the_document_resolves_it() -> None:
    document = xml.Document()
    node = document.create_text("kept")

    id = node.id
    assert isinstance(id, xml.NodeId)
    assert xml.NodeId(int(id)) == id
    assert document.node(id).ptr_eq(node)
    assert document.node(int(id)).ptr_eq(node)

    # An id that no document handed out simply does not resolve.
    assert document.node(xml.NodeId(4_000_000)) is None


def test_a_handle_keeps_a_node_alive_and_dropping_it_does_not() -> None:
    document = xml.Document()
    node = document.create_text("temporary")
    id = node.id

    assert document.node_count == 1
    assert document.collect_garbage().collected == 0

    del node
    stats = document.collect_garbage()
    assert stats.collected == 1
    assert stats.live == 0
    assert document.node_count == 0
    assert document.node(id) is None


def test_the_root_keeps_its_tree_alive() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    for index in range(5):
        child = document.create_element("child")
        child.set_attribute("n", str(index))
        child.append_child(document.create_text(f"text {index}"))
        root.append_child(child)

    del root
    stats = document.collect_garbage()
    assert stats.collected == 0
    assert stats.live == document.node_count == 11
    assert document.root.child_elements()[4].local_name == "child"


def test_a_reclaimed_slot_is_reused() -> None:
    document = xml.Document()
    first = document.create_text("first")
    first_id = first.id
    del first
    assert document.collect_garbage().collected == 1

    second = document.create_text("second")
    assert second.id == first_id
    assert document.node(first_id).ptr_eq(second)


def test_stats_describe_the_arena() -> None:
    document = xml.Document()
    document.create_text("a")
    stats = document.collect_garbage()
    assert stats.capacity == document.capacity
    assert stats.capacity == stats.live + stats.free
    assert 0.0 <= stats.free_fraction() <= 1.0
    assert repr(stats).startswith("GcStats(")


def test_nodes_lists_only_live_nodes() -> None:
    document = xml.Document()
    kept = document.create_text("kept")
    discarded = document.create_text("discarded")
    assert len(document.nodes()) == 2

    del discarded
    assert document.collect_garbage().collected == 1
    live = document.nodes()
    assert len(live) == 1
    assert live[0].ptr_eq(kept)


def test_handles_released_by_another_thread_are_safe_while_collecting() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)

    def churn() -> None:
        for index in range(300):
            node = document.create_text(f"temporary {index}")
            parent = document.create_element("temporary")
            parent.append_child(node)

    workers = [threading.Thread(target=churn) for _ in range(4)]
    for worker in workers:
        worker.start()
    for _ in range(200):
        document.collect_garbage()
    for worker in workers:
        worker.join()

    # The rooted tree is untouched, and once the workers have stopped only the root is left: the
    # final collection cannot find anything that an earlier cycle missed.
    assert [child.local_name for child in root.child_elements()] == []
    assert document.node_count == 1
    assert document.collect_garbage().collected == 0
    assert document.is_valid()
