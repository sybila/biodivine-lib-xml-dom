# Python bindings: build and test transcript

Every command below was run in this sandbox; the outputs are verbatim. The pytest run covers both
test modules (`test_xml_dom.py` for behaviour, `test_public_surface.py` for the exported surface).

Toolchain and pins:

```
$ python3 --version
Python 3.11.2
$ .venv/bin/maturin --version
maturin 1.15.0
$ .venv/bin/python -m pytest --version
pytest 9.1.1
$ grep -n 'pyo3' biodivine-lib-xml-dom-py-sys/Cargo.toml
17:extension-module = ["pyo3/extension-module"]
21:pyo3 = { version = "0.29", features = ["macros"] }
25:pyo3 = { version = "0.29", features = ["auto-initialize"] }
```

## Build (maturin develop --release)

$ cd biodivine-lib-xml-dom-py-sys && ../.venv/bin/maturin develop --release
```
🍹 Building a mixed python/rust project
🐍 Found CPython 3.11 at /sandbox/biodivine-lib-xml-dom/.venv/bin/python
🔗 Found pyo3 bindings
📡 Using build options features from pyproject.toml
   Compiling biodivine-lib-xml-dom-py-sys v0.1.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `release` profile [optimized] target(s) in 0.67s
📦 Built wheel for CPython 3.11 to /tmp/.tmpDPDRfb/biodivine_lib_xml_dom-0.1.0-cp311-cp311-linux_x86_64.whl
✏️ Setting installed package as editable
🛠 Installed biodivine-lib-xml-dom-0.1.0
```

## Python test suite

$ .venv/bin/python -m pytest biodivine-lib-xml-dom-py-sys/tests-python
```
============================= test session starts ==============================
platform linux -- Python 3.11.2, pytest-9.1.1, pluggy-1.6.0
rootdir: /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys
configfile: pyproject.toml
collected 32 items

biodivine-lib-xml-dom-py-sys/tests-python/test_public_surface.py ....... [ 21%]
                                                                         [ 21%]
biodivine-lib-xml-dom-py-sys/tests-python/test_xml_dom.py .............. [ 65%]
...........                                                              [100%]

============================== 32 passed in 0.06s ==============================
```

## Workspace gates on both toolchains

$ cargo test --workspace
```
test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom-4df520ea80dac325)
     Running tests/cloning.rs (target/debug/deps/cloning-0beb9c154f17fe8a)
     Running tests/concurrency.rs (target/debug/deps/concurrency-e840ef759e868816)
     Running tests/element.rs (target/debug/deps/element-6e93e1a1f40c4a10)
     Running tests/errors.rs (target/debug/deps/errors-b6eaedf3db463830)
     Running tests/io.rs (target/debug/deps/io-81923a9d8b390184)
     Running tests/properties.rs (target/debug/deps/properties-7959541fe3fb2956)
     Running tests/structure.rs (target/debug/deps/structure-ad3fa8bd84bfbee0)
     Running tests/validation.rs (target/debug/deps/validation-5bebe8e350287266)
     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom_sys-7020e76c8dafe61f)
   Doc-tests biodivine_lib_xml_dom
   Doc-tests biodivine_lib_xml_dom_sys
```

$ cargo +1.95.0 test --workspace
```
test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

$ cargo clippy --workspace --all-targets
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s

```

$ cargo fmt --check
```
(no output: clean)
```

$ RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p biodivine-lib-xml-dom -p biodivine-lib-xml-dom-py-sys
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
   Generated /sandbox/biodivine-lib-xml-dom/target/doc/biodivine_lib_xml_dom/index.html and 1 other file

```

## Proof that the core crate has no PyO3 dependency

$ cargo tree -p biodivine-lib-xml-dom --edges normal
```
biodivine-lib-xml-dom v0.1.0 (/sandbox/biodivine-lib-xml-dom)
├── parking_lot v0.12.5
│   ├── lock_api v0.4.14
│   │   └── scopeguard v1.2.0
│   └── parking_lot_core v0.9.12
│       ├── cfg-if v1.0.4
│       ├── libc v0.2.186
│       └── smallvec v1.15.2
├── quick-xml v0.41.0
│   └── memchr v2.8.2
└── thiserror v2.0.18
    └── thiserror-impl v2.0.18 (proc-macro)
        ├── proc-macro2 v1.0.106
        │   └── unicode-ident v1.0.24
        ├── quote v1.0.46
        │   └── proc-macro2 v1.0.106 (*)
        └── syn v2.0.118
            ├── proc-macro2 v1.0.106 (*)
            ├── quote v1.0.46 (*)
            └── unicode-ident v1.0.24
```

$ cargo tree -p biodivine-lib-xml-dom --edges normal | grep -c pyo3
```
0
```

$ grep -rn "pyo3" src/ biodivine-lib-xml-dom-py-sys/src/../../src 2>/dev/null | wc -l
```
0
```

$ grep -n "pyo3" Cargo.toml
```
# The Python bindings live in a separate crate (requirement (6): the core crate contains no PyO3
# code at all). The core crate stays the workspace *root package*, so `src/` and every existing
```
(The only occurrences in the root manifest are the two comment lines above; there is no `pyo3`
dependency entry. The `pyo3` dependency exists solely in `biodivine-lib-xml-dom-py-sys/Cargo.toml`,
and the workspace member is the only crate that can see it.)
