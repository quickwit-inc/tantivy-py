use crate::{
    get_field, make_term, make_term_for_type, schema::FieldType, to_pyerr,
    DocAddress, Schema,
};
use core::ops::Bound as OpsBound;
use pyo3::{
    exceptions,
    prelude::*,
    types::{PyAny, PyFloat, PyString, PyTuple},
};
use tantivy as tv;

/// Custom Tuple struct to represent a pair of Occur and Query
/// for the BooleanQuery
struct OccurQueryPair(Occur, Query);

impl<'source> FromPyObject<'source> for OccurQueryPair {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        let (occur, query): (Occur, Query) = ob.extract()?;

        Ok(OccurQueryPair(occur, query))
    }
}

/// Tantivy's Occur
#[pyclass(frozen, module = "tantivy.tantivy")]
#[derive(Clone)]
pub enum Occur {
    Must,
    Should,
    MustNot,
}

impl From<Occur> for tv::query::Occur {
    fn from(occur: Occur) -> tv::query::Occur {
        match occur {
            Occur::Must => tv::query::Occur::Must,
            Occur::Should => tv::query::Occur::Should,
            Occur::MustNot => tv::query::Occur::MustNot,
        }
    }
}

/// Tantivy's Query
#[pyclass(frozen, module = "tantivy.tantivy")]
pub(crate) struct Query {
    pub(crate) inner: Box<dyn tv::query::Query>,
}

impl Clone for Query {
    fn clone(&self) -> Self {
        Query {
            inner: self.inner.box_clone(),
        }
    }
}

impl Query {
    pub(crate) fn get(&self) -> &dyn tv::query::Query {
        &self.inner
    }
}

#[pymethods]
impl Query {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Query({:?})", self.get()))
    }

    /// Construct a Tantivy's TermQuery
    #[staticmethod]
    #[pyo3(signature = (schema, field_name, field_value, index_option = "position"))]
    pub(crate) fn term_query(
        schema: &Schema,
        field_name: &str,
        field_value: &Bound<PyAny>,
        index_option: &str,
    ) -> PyResult<Query> {
        let term = make_term(&schema.inner, field_name, field_value)?;
        let index_option = match index_option {
            "position" => tv::schema::IndexRecordOption::WithFreqsAndPositions,
            "freq" => tv::schema::IndexRecordOption::WithFreqs,
            "basic" => tv::schema::IndexRecordOption::Basic,
            _ => return Err(exceptions::PyValueError::new_err(
                "Invalid index option, valid choices are: 'basic', 'freq' and 'position'"
            ))
        };
        let inner = tv::query::TermQuery::new(term, index_option);
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's TermSetQuery
    #[staticmethod]
    #[pyo3(signature = (schema, field_name, field_values))]
    pub(crate) fn term_set_query(
        schema: &Schema,
        field_name: &str,
        field_values: Vec<Bound<PyAny>>,
    ) -> PyResult<Query> {
        let terms = field_values
            .into_iter()
            .map(|field_value| {
                make_term(&schema.inner, field_name, &field_value)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let inner = tv::query::TermSetQuery::new(terms);
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's AllQuery
    #[staticmethod]
    pub(crate) fn all_query() -> PyResult<Query> {
        let inner = tv::query::AllQuery {};
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's FuzzyTermQuery
    ///
    /// # Arguments
    ///
    /// * `schema` - Schema of the target index.
    /// * `field_name` - Field name to be searched.
    /// * `text` - String representation of the query term.
    /// * `distance` - (Optional) Edit distance you are going to alow. When not specified, the default is 1.
    /// * `transposition_cost_one` - (Optional) If true, a transposition (swapping) cost will be 1; otherwise it will be 2. When not specified, the default is true.
    /// * `prefix` - (Optional) If true, prefix levenshtein distance is applied. When not specified, the default is false.
    #[staticmethod]
    #[pyo3(signature = (schema, field_name, text, distance = 1, transposition_cost_one = true, prefix = false))]
    pub(crate) fn fuzzy_term_query(
        schema: &Schema,
        field_name: &str,
        text: &Bound<PyString>,
        distance: u8,
        transposition_cost_one: bool,
        prefix: bool,
    ) -> PyResult<Query> {
        let term = make_term(&schema.inner, field_name, text)?;
        let inner = if prefix {
            tv::query::FuzzyTermQuery::new_prefix(
                term,
                distance,
                transposition_cost_one,
            )
        } else {
            tv::query::FuzzyTermQuery::new(
                term,
                distance,
                transposition_cost_one,
            )
        };
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's PhraseQuery with custom offsets and slop
    ///
    /// # Arguments
    ///
    /// * `schema` - Schema of the target index.
    /// * `field_name` - Field name to be searched.
    /// * `words` - Word list that constructs the phrase. A word can be a term text or a pair of term text and its offset in the phrase.
    /// * `slop` - (Optional) The number of gaps permitted between the words in the query phrase. Default is 0.
    #[staticmethod]
    #[pyo3(signature = (schema, field_name, words, slop = 0))]
    pub(crate) fn phrase_query(
        schema: &Schema,
        field_name: &str,
        words: Vec<Bound<PyAny>>,
        slop: u32,
    ) -> PyResult<Query> {
        let mut terms_with_offset = Vec::with_capacity(words.len());
        for (idx, word) in words.into_iter().enumerate() {
            if let Ok((offset, value)) = word.extract() {
                // Custom offset is provided.
                let term = make_term(&schema.inner, field_name, &value)?;
                terms_with_offset.push((offset, term));
            } else {
                // Custom offset is not provided. Use the list index as the offset.
                let term = make_term(&schema.inner, field_name, &word)?;
                terms_with_offset.push((idx, term));
            };
        }
        if terms_with_offset.is_empty() {
            return Err(exceptions::PyValueError::new_err(
                "words must not be empty.",
            ));
        }
        let inner = tv::query::PhraseQuery::new_with_offset_and_slop(
            terms_with_offset,
            slop,
        );
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's BooleanQuery
    #[staticmethod]
    #[pyo3(signature = (subqueries))]
    pub(crate) fn boolean_query(
        subqueries: Vec<(Occur, Query)>,
    ) -> PyResult<Query> {
        let dyn_subqueries = subqueries
            .into_iter()
            .map(|(occur, query)| (occur.into(), query.inner.box_clone()))
            .collect::<Vec<_>>();

        let inner = tv::query::BooleanQuery::from(dyn_subqueries);

        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's DisjunctionMaxQuery
    #[staticmethod]
    pub(crate) fn disjunction_max_query(
        subqueries: Vec<Query>,
        tie_breaker: Option<Bound<PyFloat>>,
    ) -> PyResult<Query> {
        let inner_queries: Vec<Box<dyn tv::query::Query>> = subqueries
            .iter()
            .map(|query| query.inner.box_clone())
            .collect();

        let dismax_query = if let Some(tie_breaker) = tie_breaker {
            tv::query::DisjunctionMaxQuery::with_tie_breaker(
                inner_queries,
                tie_breaker.extract::<f32>()?,
            )
        } else {
            tv::query::DisjunctionMaxQuery::new(inner_queries)
        };

        Ok(Query {
            inner: Box::new(dismax_query),
        })
    }

    /// Construct a Tantivy's BoostQuery
    #[staticmethod]
    #[pyo3(signature = (query, boost))]
    pub(crate) fn boost_query(query: Query, boost: f32) -> PyResult<Query> {
        let inner = tv::query::BoostQuery::new(query.inner, boost);
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's RegexQuery
    #[staticmethod]
    #[pyo3(signature = (schema, field_name, regex_pattern))]
    pub(crate) fn regex_query(
        schema: &Schema,
        field_name: &str,
        regex_pattern: &str,
    ) -> PyResult<Query> {
        let field = get_field(&schema.inner, field_name)?;

        let inner_result =
            tv::query::RegexQuery::from_pattern(regex_pattern, field);
        match inner_result {
            Ok(inner) => Ok(Query {
                inner: Box::new(inner),
            }),
            Err(e) => Err(to_pyerr(e)),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (doc_address, min_doc_frequency = Some(5), max_doc_frequency = None, min_term_frequency = Some(2), max_query_terms = Some(25), min_word_length = None, max_word_length = None, boost_factor = Some(1.0), stop_words = vec![]))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn more_like_this_query(
        doc_address: &DocAddress,
        min_doc_frequency: Option<u64>,
        max_doc_frequency: Option<u64>,
        min_term_frequency: Option<usize>,
        max_query_terms: Option<usize>,
        min_word_length: Option<usize>,
        max_word_length: Option<usize>,
        boost_factor: Option<f32>,
        stop_words: Vec<String>,
    ) -> PyResult<Query> {
        let mut builder = tv::query::MoreLikeThisQuery::builder();
        if let Some(value) = min_doc_frequency {
            builder = builder.with_min_doc_frequency(value);
        }
        if let Some(value) = max_doc_frequency {
            builder = builder.with_max_doc_frequency(value);
        }
        if let Some(value) = min_term_frequency {
            builder = builder.with_min_term_frequency(value);
        }
        if let Some(value) = max_query_terms {
            builder = builder.with_max_query_terms(value);
        }
        if let Some(value) = min_word_length {
            builder = builder.with_min_word_length(value);
        }
        if let Some(value) = max_word_length {
            builder = builder.with_max_word_length(value);
        }
        if let Some(value) = boost_factor {
            builder = builder.with_boost_factor(value);
        }
        builder = builder.with_stop_words(stop_words);

        let inner = builder.with_document(tv::DocAddress::from(doc_address));
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    /// Construct a Tantivy's ConstScoreQuery
    #[staticmethod]
    #[pyo3(signature = (query, score))]
    pub(crate) fn const_score_query(
        query: Query,
        score: f32,
    ) -> PyResult<Query> {
        let inner = tv::query::ConstScoreQuery::new(query.inner, score);
        Ok(Query {
            inner: Box::new(inner),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (schema, field_name, field_type, lower_bound, upper_bound, include_lower = true, include_upper = true))]
    pub(crate) fn range_query(
        schema: &Schema,
        field_name: &str,
        field_type: FieldType,
        lower_bound: &Bound<PyAny>,
        upper_bound: &Bound<PyAny>,
        include_lower: bool,
        include_upper: bool,
    ) -> PyResult<Query> {
        match field_type {
            FieldType::Text => {
                return Err(exceptions::PyValueError::new_err(
                    "Text fields are not supported for range queries.",
                ))
            }
            FieldType::Boolean => {
                return Err(exceptions::PyValueError::new_err(
                    "Boolean fields are not supported for range queries.",
                ))
            }
            FieldType::Facet => {
                return Err(exceptions::PyValueError::new_err(
                    "Facet fields are not supported for range queries.",
                ))
            }
            FieldType::Bytes => {
                return Err(exceptions::PyValueError::new_err(
                    "Bytes fields are not supported for range queries.",
                ))
            }
            FieldType::Json => {
                return Err(exceptions::PyValueError::new_err(
                    "Json fields are not supported for range queries.",
                ))
            }
            _ => {}
        }

        let lower_bound_term = make_term_for_type(
            &schema.inner,
            field_name,
            field_type.clone(),
            lower_bound,
        )?;
        let upper_bound_term = make_term_for_type(
            &schema.inner,
            field_name,
            field_type.clone(),
            upper_bound,
        )?;

        let lower_bound = if include_lower {
            OpsBound::Included(lower_bound_term)
        } else {
            OpsBound::Excluded(lower_bound_term)
        };

        let upper_bound = if include_upper {
            OpsBound::Included(upper_bound_term)
        } else {
            OpsBound::Excluded(upper_bound_term)
        };

        let inner = tv::query::RangeQuery::new_term_bounds(
            field_name.to_string(),
            field_type.into(),
            &lower_bound,
            &upper_bound,
        );

        Ok(Query {
            inner: Box::new(inner),
        })
    }
}
