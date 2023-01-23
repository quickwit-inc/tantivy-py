use pyo3::{exceptions, prelude::*};
use tantivy as tv;

mod document;
mod facet;
mod index;
mod query;
mod schema;
mod schemabuilder;
mod searcher;

use document::Document;
use facet::Facet;
use index::Index;
use schema::Schema;
use schemabuilder::SchemaBuilder;
use searcher::{DocAddress, Searcher};
use query::Query;

/// Python bindings for the search engine library Tantivy.
///
/// Tantivy is a full text search engine library written in rust.
///
/// It is closer to Apache Lucene than to Elasticsearch and Apache Solr in
/// the sense it is not an off-the-shelf search engine server, but rather
/// a library that can be used to build such a search engine.
/// Tantivy is, in fact, strongly inspired by Lucene's design.
///
/// Example:
///     >>> import json
///     >>> import tantivy
///
///     >>> builder = tantivy.SchemaBuilder()
///
///     >>> title = builder.add_text_field("title", stored=True)
///     >>> body = builder.add_text_field("body")
///
///     >>> schema = builder.build()
///     >>> index = tantivy.Index(schema)
///     >>> doc = tantivy.Document()
///     >>> doc.add_text(title, "The Old Man and the Sea")
///     >>> doc.add_text(body, ("He was an old man who fished alone in a "
///                             "skiff in the Gulf Stream and he had gone "
///                             "eighty-four days now without taking a fish."))
///
///     >>> writer.add_document(doc)
///
///     >>> doc = schema.parse_document(json.dumps({
///            "title": ["Frankenstein", "The Modern Prometheus"],
///            "body": ("You will rejoice to hear that no disaster has "
///                     "accompanied the commencement of an enterprise which "
///                     "you have regarded with such evil forebodings.  "
///                     "I arrived here yesterday, and my first task is to "
///                     "assure my dear sister of my welfare and increasing "
///                     "confidence in the success of my undertaking.")
///     }))
///
///     >>> writer.add_document(doc)
///     >>> writer.commit()
///
///     >>> reader = index.reader()
///     >>> searcher = reader.searcher()
///
///     >>> query = index.parse_query("sea whale", [title, body])
///
///     >>> result = searcher.search(query, 10)
///
///     >>> assert len(result) == 1
///
#[pymodule]
fn tantivy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Schema>()?;
    m.add_class::<SchemaBuilder>()?;
    m.add_class::<Searcher>()?;
    m.add_class::<Document>()?;
    m.add_class::<Index>()?;
    m.add_class::<DocAddress>()?;
    m.add_class::<Facet>()?;
    m.add_class::<Query>()?;
    Ok(())
}

pub(crate) fn to_pyerr<E: ToString>(err: E) -> PyErr {
    exceptions::PyValueError::new_err(err.to_string())
}

pub(crate) fn get_field(
    schema: &tv::schema::Schema,
    field_name: &str,
) -> PyResult<tv::schema::Field> {
    let field = schema.get_field(field_name).ok_or_else(|| {
        exceptions::PyValueError::new_err(format!(
            "Field `{}` is not defined in the schema.",
            field_name
        ))
    })?;

    Ok(field)
}
