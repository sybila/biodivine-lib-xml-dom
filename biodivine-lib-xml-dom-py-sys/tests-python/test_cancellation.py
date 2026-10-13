"""Interrupting long operations from Python.

On the main thread, a Python signal handler (the one behind Ctrl+C, or one installed with
``signal.signal``) can interrupt a whole-document operation: the operation stops early and the
exception the handler raised is propagated unchanged. The handler runs while no document lock is
held, so it may even use the document that the interrupted operation was working on.
"""

from __future__ import annotations

import signal
import subprocess
import sys
import textwrap
import time

import pytest

import biodivine_lib_xml_dom as xml

pytestmark = pytest.mark.skipif(
    not hasattr(signal, "setitimer"), reason="needs signal.setitimer (Unix)"
)

#: Large enough that every operation below takes far longer than the timer delay.
SOURCE = "<root>" + "<item a='1'>text</item>" * 400_000 + "</root>"


class Interrupted(Exception):
    """Raised by the test signal handler."""


def _raise_interrupted(signum: int, frame: object) -> None:
    raise Interrupted()


def _interrupted_after(delay: float, action) -> float:
    """Runs ``action`` once with a timer that raises `Interrupted` after ``delay`` seconds, and
    returns how long it took until the exception surfaced."""
    previous = signal.signal(signal.SIGALRM, _raise_interrupted)
    start = time.perf_counter()
    signal.setitimer(signal.ITIMER_REAL, delay)
    try:
        with pytest.raises(Interrupted):
            action()
            # Without interruption the handler runs right after `action`; fail that case below.
            time.sleep(1)
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous)
    return time.perf_counter() - start


def test_every_long_operation_can_be_interrupted() -> None:
    document = xml.Document.parse(SOURCE)
    target = xml.Document()
    for action in (
        lambda: xml.Document.parse(SOURCE),
        lambda: xml.Document.parse(SOURCE.encode()),
        document.write,
        document.validate,
        document.is_valid,
        document.validation_errors,
        document.root.deep_clone,
        lambda: document.root.deep_clone_into(target),
    ):
        started = time.perf_counter()
        action()
        duration = time.perf_counter() - started
        # Interrupted a tenth of the way in, the exception must surface well before the operation
        # could have finished.
        elapsed = _interrupted_after(duration / 10, action)
        assert elapsed < duration / 2, (action, elapsed, duration)


def test_an_interrupted_copy_creates_nothing() -> None:
    document = xml.Document.parse(SOURCE)
    target = xml.Document()
    _interrupted_after(0.01, lambda: document.root.deep_clone_into(target))
    assert target.node_count == 0


def test_a_handler_may_use_the_interrupted_document() -> None:
    # Run in a subprocess, so that a regression (a deadlock) fails the test instead of hanging it.
    script = textwrap.dedent(
        """
        import signal
        import biodivine_lib_xml_dom as xml

        document = xml.Document.parse("<root>" + "<item/>" * 400_000 + "</root>")
        def handler(signum, frame):
            document.root.append_child(document.create_element("late"))
        signal.signal(signal.SIGALRM, handler)
        signal.setitimer(signal.ITIMER_REAL, 0.01, 0.01)
        for _ in range(5):
            document.validate()
            document.collect_garbage()
        signal.setitimer(signal.ITIMER_REAL, 0)
        print("done")
        """
    )
    result = subprocess.run(
        [sys.executable, "-c", script], capture_output=True, text=True, timeout=120
    )
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == "done"
