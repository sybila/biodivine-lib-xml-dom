"""Cancellation of the long-running operations from Python.

A Python interrupt (Ctrl+C / ``KeyboardInterrupt``) that arrives while a whole-document operation
is running cancels that operation instead of being ignored until it finishes. The binding registers
the interpreter's signal check as a cancellation trigger around every operation that releases the
GIL for a time proportional to the document (parsing, serializing, validating, cross-document
copies and garbage collection).
"""

import os
import signal
import threading
import time

import biodivine_lib_xml_dom as xml


def _interrupt_after(delay: float) -> None:
    time.sleep(delay)
    os.kill(os.getpid(), signal.SIGINT)


def test_python_interrupt_cancels_a_long_parse() -> None:
    # Large enough that parsing comfortably outlasts the interrupt delay below.
    payload = "<root>" + "<a/>" * 600_000 + "</root>"

    sender = threading.Thread(target=_interrupt_after, args=(0.03,))
    sender.start()
    interrupted = False
    try:
        xml.parse(payload)
        sender.join()
    except KeyboardInterrupt:
        interrupted = True
    finally:
        sender.join()

    assert interrupted, "the KeyboardInterrupt did not cancel the parse"
