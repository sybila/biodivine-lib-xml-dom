# Documentation build transcript

Every command below was run in this sandbox; the outputs are verbatim. The single entry point is
`docs/build_docs.sh` (also `make docs`), which builds the extension if it is missing, so a docs
build cannot silently document a stale one.

```
$ python3 -V
Python 3.11.2
$ .venv/bin/python -m sphinx --version
sphinx-build 9.0.4
$ .venv/bin/python -c 'import sphinx, sphinx_design, myst_parser; print(sphinx.__version__)'
9.0.4
$ cargo --version
cargo 1.99.0 (5f94df478 2026-08-27)
```

## The docs gate

$ docs/build_docs.sh
```

=== tool versions
python 3.11.2
sphinx 9.0.4
pytest 9.1.1
cargo 1.99.0 (5f94df478 2026-08-27)
rustc 1.99.0 (b940084d7 2026-09-28)
maturin 1.15.0

=== 1. cargo doc (Rust API, warnings denied)
 Documenting biodivine-lib-xml-dom v0.1.0 (/sandbox/biodivine-lib-xml-dom)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
   Generated /sandbox/biodivine-lib-xml-dom/target/doc/biodivine_lib_xml_dom/index.html and 1 other file
rustdoc: 8 index pages under target/doc

=== 2. documentation sections (# Errors / # Panics)
the core crate: inspected 145 public functions in src
the binding crate: inspected 156 public functions in biodivine-lib-xml-dom-py-sys/src
all public items follow the `# Errors` / `# Panics` conventions

=== 3. book sources (structure and language pairs)
book sources: 12 chapters, 19 language pairs
the book is well-formed

=== 4. native extension (needed by Sphinx autodoc)
biodivine_lib_xml_dom is importable

=== 5. Python API reference (Sphinx autodoc)
Running Sphinx v9.0.4
loading translations [en]... done
making output directory... done
loading intersphinx inventory 'python' from https://docs.python.org/3/objects.inv ...
myst v5.1.0: MdParserConfig(commonmark_only=False, gfm_only=False, enable_extensions={'colon_fence', 'deflist'}, disable_syntax=[], all_links_external=False, links_external_new_tab=False, url_schemes=('http', 'https', 'mailto', 'ftp'), ref_domains=None, fence_as_directive=set(), number_code_blocks=[], title_to_header=False, heading_anchors=0, heading_slug_func=None, html_meta={}, footnote_sort=True, footnote_transition=True, words_per_minute=200, substitutions={}, linkify_fuzzy_links=True, dmath_allow_labels=True, dmath_allow_space=True, dmath_allow_digits=True, dmath_double_inline=False, update_mathjax=True, mathjax_classes='tex2jax_process|mathjax_process|math|output_area', enable_checkboxes=False, strikethrough_single_tilde=False, colon_fence_exact_match=False, suppress_warnings=[], highlight_code_blocks=True)
building [mo]: targets for 0 po files that are out of date
writing output... 
building [html]: targets for 2 source files that are out of date
updating environment: [new config] 2 added, 0 changed, 0 removed
reading sources... [ 50%] api
reading sources... [100%] index

looking for now-outdated files... none found
pickling environment... done
checking consistency... done
preparing documents... done
copying assets... 
copying static files... 
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/basic.css
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/documentation_options.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/language_data.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/alabaster.css
copying static files: done
copying extra files... 
copying extra files: done
copying assets: done
writing output... [ 50%] api
writing output... [100%] index

generating indices... genindex py-modindex done
writing additional pages... search done
dumping search index in English (code: en)... done
dumping object inventory... done
build succeeded.

The HTML pages are in biodivine-lib-xml-dom-py-sys/docs/_build.
python api docs: 5 HTML files

=== 6. the book
Running Sphinx v9.0.4
loading translations [en]... done
making output directory... done
myst v5.1.0: MdParserConfig(commonmark_only=False, gfm_only=False, enable_extensions={'colon_fence', 'deflist', 'attrs_inline'}, disable_syntax=[], all_links_external=False, links_external_new_tab=False, url_schemes=('http', 'https', 'mailto', 'ftp'), ref_domains=None, fence_as_directive=set(), number_code_blocks=[], title_to_header=False, heading_anchors=3, heading_slug_func=None, html_meta={}, footnote_sort=True, footnote_transition=True, words_per_minute=200, substitutions={}, linkify_fuzzy_links=True, dmath_allow_labels=True, dmath_allow_space=True, dmath_allow_digits=True, dmath_double_inline=False, update_mathjax=True, mathjax_classes='tex2jax_process|mathjax_process|math|output_area', enable_checkboxes=False, strikethrough_single_tilde=False, colon_fence_exact_match=False, suppress_warnings=[], highlight_code_blocks=True)
building [mo]: targets for 0 po files that are out of date
writing output... 
building [html]: targets for 12 source files that are out of date
updating environment: [new config] 12 added, 0 changed, 0 removed
reading sources... [  8%] building-documents
reading sources... [ 17%] design-notes
reading sources... [ 25%] getting-started
reading sources... [ 33%] index
reading sources... [ 42%] limitations
reading sources... [ 50%] migration
reading sources... [ 58%] namespaces
reading sources... [ 67%] parsing-and-serializing
reading sources... [ 75%] python-usage
reading sources... [ 83%] thread-safety
reading sources... [ 92%] traversing-and-editing
reading sources... [100%] validation

looking for now-outdated files... none found
pickling environment... done
checking consistency... done
preparing documents... done
copying assets... 
copying static files... 
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/basic.css
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/documentation_options.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/language_data.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/alabaster.css
copying static files: done
copying extra files... 
copying extra files: done
copying assets: done
writing output... [  8%] building-documents
writing output... [ 17%] design-notes
writing output... [ 25%] getting-started
writing output... [ 33%] index
writing output... [ 42%] limitations
writing output... [ 50%] migration
writing output... [ 58%] namespaces
writing output... [ 67%] parsing-and-serializing
writing output... [ 75%] python-usage
writing output... [ 83%] thread-safety
writing output... [ 92%] traversing-and-editing
writing output... [100%] validation

generating indices... genindex done
writing additional pages... search done
dumping search index in English (code: en)... done
dumping object inventory... done
build succeeded.

The HTML pages are in docs/book/_build.
book: 14 HTML files

=== 7. built book (chapter titles and language tabs)
book sources: 12 chapters, 19 language pairs
built book: 14 pages, 57 tab-set occurrences
the book is well-formed

=== summary
all documentation built and checked
  rustdoc            target/doc/biodivine_lib_xml_dom/index.html
  python api docs    biodivine-lib-xml-dom-py-sys/docs/_build/index.html
  book               docs/book/_build/index.html
```
