#![allow(clippy::new_ret_no_self)]

use pyo3::{exceptions, prelude::*, types::PyAny};

use crate::{
    document::{extract_value, Document},
    get_field,
    parser_error::QueryParserErrorIntoPy,
    query::Query,
    schema::Schema,
    searcher::Searcher,
    to_pyerr,
};
use tantivy as tv;
use tantivy::{
    directory::MmapDirectory,
    schema::{NamedFieldDocument, Term, Value, Field},
    tokenizer::{
        Language, LowerCaser, RemoveLongFilter, SimpleTokenizer, Stemmer,
        TextAnalyzer,
    },
};


/// IndexWriter is the user entry-point to add documents to the index.
///
/// To create an IndexWriter first create an Index and call the writer() method
/// on the index object.
#[pyclass]
pub(crate) struct Geocoding {
    attr1: u64,
}

impl Geocoding {
    pub(crate) fn private_method(&self) -> u64 {
        32
    }
}

#[pymethods]
impl Geocoding {
    #[staticmethod]
    pub fn static_meth() -> u64 {
        42
    }
}