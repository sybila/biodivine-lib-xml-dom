//! The XML parser and the XML serializer.
//!
//! Both halves are implementation details of this crate: the public entry points are the
//! `Document::parse_*` and `Document::write_*` methods, and that is where the contract they
//! implement is documented. This module keeps the code, the option types the serializer accepts,
//! and the reasoning that is only interesting to someone changing it.
//!
//! * [`parse`] turns bytes into a [`Document`](crate::Document);
//! * [`write`] turns a document back into bytes.
//!
//! The two option enums and [`WriteOptions`] are re-exported from the crate root, next to the
//! methods that take them.

pub(crate) mod parse;
pub(crate) mod write;

pub(crate) use parse::{parse_bytes, parse_file, parse_reader, parse_string};
pub use write::{DeclarationStyle, EmptyElementStyle, WriteOptions};
pub(crate) use write::{
    write_element_to_string, write_file, write_file_with, write_string, write_string_with,
    write_writer, write_writer_with,
};
