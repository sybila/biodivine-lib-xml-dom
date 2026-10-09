# Miri evidence

Miri was attempted as promised in `PLAN.md` §3.4. Outcome: **the library's own tests pass under Miri; the lock-contending tests cannot run under it**, for a reason that is about `parking_lot` rather than about this crate.

Toolchain: `rustc 1.101.0-nightly (a30aa9064 2026-10-08)` with the `miri` component, installed with `rustup toolchain install nightly --profile minimal --component miri` followed by `cargo +nightly miri setup`.

## 1. The library test suite passes under Miri

$ MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib
```
test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.21s
```

(`-Zmiri-disable-isolation` is needed only because one *test helper* reads the rule files from `specification/rules/`; without it Miri aborts on `Path::is_file` with "unsupported operation: filesystem access". No library code touches the filesystem.)

That covers the arena bookkeeping (attach/detach/replace/capacity/snapshots), the interner, namespace and name resolution, validation, the document lock's re-entrancy guard and the `Send`/`Sync` assertions.

## 2. The lock-contending tests cannot run under Miri

$ MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --test concurrency
```
test a_node_handle_remains_valid_after_being_moved_by_another_thread ... ok
test handles_can_be_moved_between_threads ... warning: integer-to-pointer cast
error: Undefined Behavior: incorrect c-variadic argument type for `syscall(SYS_futex, ...)`: expected argument #2 to have type `*mut u32` but got incompatible type `&std::sync::atomic::Atomic<i32>`
error: aborting due to 1 previous error
```

The abort happens inside `parking_lot_core`'s futex call, i.e. in a dependency: Miri's shim for `SYS_futex` does not accept the argument type `parking_lot` passes. This is a known Miri/`parking_lot` interaction, not a finding about this crate, and it means Miri cannot contribute evidence about the concurrency design.

## What Miri adds, and what it does not

| claim | Miri evidence |
| --- | --- |
| no undefined behaviour in the data structures (arena, interner, snapshot, validation) | **yes**: 84/84 lib tests pass |
| no undefined behaviour in the parser/serializer | **partial**: \`tests/io.rs\` is an integration test and was not run under Miri |
| the single-lock protocol is race-free | **no**: the contending tests abort inside `parking_lot`; this rests on the design (one lock, never re-entered) plus the native stress tests and the debug re-entrancy guard |

There is no `unsafe` code in `biodivine-lib-xml-dom` (the workspace `forbid`-style check is the absence of any `unsafe` block), so Miri's main value here is exercising dependencies and catching aliasing mistakes in the arena's index bookkeeping.
