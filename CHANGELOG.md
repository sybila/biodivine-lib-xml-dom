# Changelog

Notable changes to this project, in the format of [Keep a Changelog](https://keepachangelog.com/).

The library has not been released yet, so there is a single entry. It describes the shape of the
first release rather than a diff against a previous one.

## Unreleased

### Added

* Documents, nodes and elements with full namespace awareness: expanded names throughout, validation
  of names and content at construction, and one whole-document validation pass that reports every
  problem at once.
* An arena-backed storage model with copy-on-write-free handles that can be shared between threads;
  one lock per document and no operation that holds two.
* Garbage collection: nodes live while a handle or the document root refers to them, unreachable
  nodes are reclaimed by a mark-and-sweep collection, and the capacity policy is configurable.
* Parsing and serialization as `Document` methods, XML 1.0 with UTF-8, with typed errors for every
  way an input can fail.
* Python bindings in two layers: a thin PyO3 mirror of the Rust API and a Pythonic package on top of
  it, with the interpreter lock released around whole-document work.
* Documentation: rustdoc for the API, the XML rule summaries reproduced as `xml_spec::rule_set`, the
  rule coverage table, design notes, and a book with Rust and Python examples.
