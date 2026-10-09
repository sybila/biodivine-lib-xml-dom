//! Reading and writing XML documents.
//!
//! The two halves live in separate internal modules: one turns bytes into a [`crate::Document`],
//! the other turns a document back into bytes. Both are documented at length in their own module
//! documentation, including which specification rules they enforce and which parts of the data
//! model they deliberately do not represent.
//!
//! The entry points are re-exported both here and from the crate root, so the usual spelling is
//! `biodivine_lib_xml_dom::parse_string` and `biodivine_lib_xml_dom::io::write_string`.

mod parse;
mod write;

pub use parse::{parse_bytes, parse_file, parse_reader, parse_string};
pub(crate) use write::write_element_to_string;
pub use write::{
    DeclarationStyle, EmptyElementStyle, WriteOptions, write_file, write_file_with, write_string,
    write_string_with, write_writer, write_writer_with,
};
