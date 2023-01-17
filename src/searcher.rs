#![allow(clippy::new_ret_no_self)]

use std::collections::HashMap;
use crate::{document::Document, get_field, query::Query, to_pyerr};
use pyo3::{exceptions::PyValueError, prelude::*};
use tantivy as tv;
use tantivy::collector::{Count, MultiCollector, TopDocs};
use tv::collector::{FacetCollector};
use tv::fastfield::FastFieldReader;
use tv::{SegmentReader, Score, DocId};

/// Tantivy's Searcher class
///
/// A Searcher is used to search the index given a prepared Query.
#[pyclass]
pub(crate) struct Searcher {
    pub(crate) inner: tv::LeasedItem<tv::Searcher>,
}

#[derive(Clone)]
enum Fruit {
    Score(f32),
    Order(u64),
}

impl std::fmt::Debug for Fruit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fruit::Score(s) => f.write_str(&format!("{}", s)),
            Fruit::Order(o) => f.write_str(&format!("{}", o)),
        }
    }
}

impl ToPyObject for Fruit {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            Fruit::Score(s) => s.to_object(py),
            Fruit::Order(o) => o.to_object(py),
        }
    }
}

#[pyclass]
/// Object holding a results successful search.
pub(crate) struct SearchResult {
    hits: Vec<(Fruit, DocAddress)>,
    
    #[pyo3(get)]
    /// How many documents matched the query. Only available if `count` was set
    /// to true during the search.
    count: Option<usize>,

    #[pyo3(get)]
    /// Results of facets using using `count_facets_by_field` parameter
    facet_counts: Option<HashMap<String, u64>>,
}

#[pymethods]
impl SearchResult {
    fn __repr__(&self) -> PyResult<String> {
        if let Some(count) = self.count {
            Ok(format!(
                "SearchResult(hits: {:?}, count: {})",
                self.hits, count
            ))
        } else {
            Ok(format!("SearchResult(hits: {:?})", self.hits))
        }
    }

    #[getter]
    /// The list of tuples that contains the scores and DocAddress of the
    /// search results.
    fn hits(&self, py: Python) -> PyResult<Vec<(PyObject, DocAddress)>> {
        let ret: Vec<(PyObject, DocAddress)> = self
            .hits
            .iter()
            .map(|(result, address)| (result.to_object(py), address.clone()))
            .collect();
        Ok(ret)
    }
}

#[pymethods]
impl Searcher {
    /// Search the index with the given query and collect results.
    ///
    /// Args:
    ///     query (Query): The query that will be used for the search.
    ///     limit (int, optional): The maximum number of search results to
    ///         return. Defaults to 10.
    ///     count (bool, optional): Should the number of documents that match
    ///         the query be returned as well. Defaults to true.
    ///     count_facets_by_field (Field, optional): Return grouped number of
    ///         documents by the given facet field. Defaults to false
    ///     order_by_field (Field, optional): A schema field that the results
    ///         should be ordered by. The field must be declared as a fast field
    ///         when building the schema. Note, this only works for unsigned
    ///         fields.
    ///     weight_by_field (Field, optional): A schema field increases the
    ///         score of the document by the given value. It should be a fast
    ///         field of float data type
    ///     offset (Field, optional): The offset from which the results have
    ///         to be returned.
    ///
    /// Returns `SearchResult` object.
    ///
    /// Raises a ValueError if there was an error with the search.
    #[args(limit = 10, offset = 0, count = true)]
    fn search(
        &self,
        _py: Python,
        query: &Query,
        limit: usize,
        count: bool,
        count_facets_by_field: Option<&str>,
        order_by_field: Option<&str>,
        weight_by_field: Option<&str>,
        offset: usize,
    ) -> PyResult<SearchResult> {
        let mut multicollector = MultiCollector::new();

        let count_handle = if count {
            Some(multicollector.add_collector(Count))
        } else {
            None
        };

        let facet_handle = if let Some(facet_name) = count_facets_by_field {
            let field = get_field(&self.inner.index().schema(), facet_name)?;
            let mut facet_collector = FacetCollector::for_field(field);
            facet_collector.add_facet("/");
            Some(multicollector.add_collector(facet_collector))
        } else {
            None
        };

        let (mut multifruit, hits) = {

            if let Some(weight_by) = weight_by_field {

                let field = get_field(&self.inner.index().schema(), weight_by)?;
                let collector = TopDocs::with_limit(limit)
                    .and_offset(offset)
                    .tweak_score(move |segment_reader: &SegmentReader| {
                        let weight_reader = segment_reader.fast_fields().f64(field).unwrap();
                        return move |doc: DocId, original_score: Score| {
                            let weight: f64 = weight_reader.get(doc);
                            let new_score = original_score + weight as f32;
                            return new_score
                        }
                    });

                let top_docs_handle = multicollector.add_collector(collector);
                let ret = self.inner.search(query.get(), &multicollector);

                match ret {
                    Ok(mut r) => {
                        let top_docs = top_docs_handle.extract(&mut r);
                        let result: Vec<(Fruit, DocAddress)> = top_docs
                            .iter()
                            .map(|(f, d)| {
                                (Fruit::Score(*f), DocAddress::from(d))
                            })
                            .collect();
                        (r, result)
                    }
                    Err(e) => return Err(PyValueError::new_err(e.to_string())),
                }

            } else if let Some(order_by) = order_by_field {

                let field = get_field(&self.inner.index().schema(), order_by)?;
                let collector = TopDocs::with_limit(limit)
                    .and_offset(offset)
                    .order_by_u64_field(field);
                let top_docs_handle = multicollector.add_collector(collector);
                let ret = self.inner.search(query.get(), &multicollector);

                match ret {
                    Ok(mut r) => {
                        let top_docs = top_docs_handle.extract(&mut r);
                        let result: Vec<(Fruit, DocAddress)> = top_docs
                            .iter()
                            .map(|(f, d)| {
                                (Fruit::Order(*f), DocAddress::from(d))
                            })
                            .collect();
                        (r, result)
                    }
                    Err(e) => return Err(PyValueError::new_err(e.to_string())),
                }
            } else {
                let collector = TopDocs::with_limit(limit).and_offset(offset);
                let top_docs_handle = multicollector.add_collector(collector);
                let ret = self.inner.search(query.get(), &multicollector);

                match ret {
                    Ok(mut r) => {
                        let top_docs = top_docs_handle.extract(&mut r);
                        let result: Vec<(Fruit, DocAddress)> = top_docs
                            .iter()
                            .map(|(f, d)| {
                                (Fruit::Score(*f), DocAddress::from(d))
                            })
                            .collect();
                        (r, result)
                    }
                    Err(e) => return Err(PyValueError::new_err(e.to_string())),
                }
            }
        };

        let count = match count_handle {
            Some(h) => Some(h.extract(&mut multifruit)),
            None => None,
        };

        let facet_counts:Option<HashMap<String, u64>> = match facet_handle {
            Some(h) => {
                let facet_counts_obj = h.extract(&mut multifruit);

                let collection: Vec<(&tv::schema::Facet, u64)> = facet_counts_obj
                    .get("/")
                    .collect();

                let mut facet_counts:HashMap<String, u64> = HashMap::new();

                for (facet, count) in collection.iter() {
                    facet_counts.insert(facet.to_path_string(), *count);
                }

                Some(facet_counts)
            },
            None => None,
        };

        Ok(SearchResult { hits, count, facet_counts})
    }

    /// Returns the overall number of documents in the index.
    #[getter]
    fn num_docs(&self) -> u64 {
        self.inner.num_docs()
    }

    /// Fetches a document from Tantivy's store given a DocAddress.
    ///
    /// Args:
    ///     doc_address (DocAddress): The DocAddress that is associated with
    ///         the document that we wish to fetch.
    ///
    /// Returns the Document, raises ValueError if the document can't be found.
    fn doc(&self, doc_address: &DocAddress) -> PyResult<Document> {
        let doc = self.inner.doc(doc_address.into()).map_err(to_pyerr)?;
        let named_doc = self.inner.schema().to_named_doc(&doc);
        Ok(Document {
            field_values: named_doc.0,
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Searcher(num_docs={}, num_segments={})",
            self.inner.num_docs(),
            self.inner.segment_readers().len()
        ))
    }
}

/// DocAddress contains all the necessary information to identify a document
/// given a Searcher object.
///
/// It consists in an id identifying its segment, and its segment-local DocId.
/// The id used for the segment is actually an ordinal in the list of segment
/// hold by a Searcher.
#[pyclass]
#[derive(Clone, Debug)]
pub(crate) struct DocAddress {
    pub(crate) segment_ord: tv::SegmentOrdinal,
    pub(crate) doc: tv::DocId,
}

#[pymethods]
impl DocAddress {
    /// The segment ordinal is an id identifying the segment hosting the
    /// document. It is only meaningful, in the context of a searcher.
    #[getter]
    fn segment_ord(&self) -> u32 {
        self.segment_ord
    }

    /// The segment local DocId
    #[getter]
    fn doc(&self) -> u32 {
        self.doc
    }
}

impl From<&tv::DocAddress> for DocAddress {
    fn from(doc_address: &tv::DocAddress) -> Self {
        DocAddress {
            segment_ord: doc_address.segment_ord,
            doc: doc_address.doc_id,
        }
    }
}

impl Into<tv::DocAddress> for &DocAddress {
    fn into(self) -> tv::DocAddress {
        tv::DocAddress {
            segment_ord: self.segment_ord(),
            doc_id: self.doc(),
        }
    }
}
