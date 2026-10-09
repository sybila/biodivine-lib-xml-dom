"""Book chapter "Thread safety" — the Python half.

The point of this file is that it terminates: one document, several threads writing and reading,
and no deadlock - because a document owns exactly one lock, no operation holds two, and the long
operations release the GIL.
"""

import concurrent.futures

import biodivine_lib_xml_dom as xml

document = xml.Document()
root = document.create_element("root")
document.set_root(root)


def writer(index: int) -> None:
    for item in range(25):
        node = document.create_element(f"item{index}_{item}")
        node.append_child(document.create_text("payload"))
        root.append_child(node)


def reader() -> int:
    total = 0
    for _ in range(50):
        total += len(root)
        xml.write(document)
    return total


with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
    writers = [pool.submit(writer, index) for index in range(4)]
    readers = [pool.submit(reader) for _ in range(4)]
    for future in writers:
        future.result(timeout=30)
    assert all(future.result(timeout=30) > 0 for future in readers)

assert len(root) == 4 * 25
assert document.is_valid()
print(f"{len(root)} children, document valid")
