use crate::to_pyerr;
use pyo3::prelude::*;
use std::ops::Bound;
use tantivy as tv;
use tv::schema::{Field, Term};

/// Tantivy's Query
#[pyclass]
pub(crate) struct Query {
    pub(crate) inner: Box<dyn tv::query::Query>,
}

impl Query {
    pub(crate) fn get(&self) -> &dyn tv::query::Query {
        &self.inner
    }
}

#[pymethods]
impl Query {
    #[staticmethod]
    fn term(field_id: u32, text: &str) -> Query {
        let term = Term::from_field_text(Field::from_field_id(field_id), text);
        Query {
            inner: Box::new(tv::query::TermQuery::new(
                term,
                tv::schema::IndexRecordOption::Basic,
            )),
        }
    }

    #[staticmethod]
    fn fuzzy_term(field_id: u32, distance: u8, text: &str) -> Query {
        let ftq = tv::query::FuzzyTermQuery::new(
            Term::from_field_text(Field::from_field_id(field_id), text),
            distance,
            true,
        );
        Query {
            inner: (Box::new(ftq)),
        }
    }

    #[staticmethod]
    fn regex(field_id: u32, pattern: &str) -> PyResult<Query> {
        let rq = tv::query::RegexQuery::from_pattern(
            pattern,
            Field::from_field_id(field_id),
        )
        .map_err(to_pyerr)?;
        Ok(Query {
            inner: Box::new(rq),
        })
    }

    #[staticmethod]
    fn phrase(field_id: u32, words: Vec<&str>) -> Query {
        let terms = words
            .iter()
            .map(|&w| Term::from_field_text(Field::from_field_id(field_id), w))
            .collect::<Vec<_>>();
        Query {
            inner: Box::new(tv::query::PhraseQuery::new(terms)),
        }
    }

    #[staticmethod]
    fn boost(q: &Query, boost: f32) -> Query {
        let bq = tv::query::BoostQuery::new(q.get().box_clone(), boost);
        Query {
            inner: Box::new(bq),
        }
    }

    #[staticmethod]
    fn and_q(qs : Vec<PyRef<Query>>) -> Query {
        Query {
            inner: Box::new(tv::query::BooleanQuery::intersection(
                qs.iter().map(|q| q.get().box_clone()).collect::<Vec<_>>()
            ))
        }
    }

    #[staticmethod]
    fn or_q(qs : Vec<PyRef<Query>>) -> Query {
        Query {
            inner: Box::new(tv::query::BooleanQuery::union(
                qs.iter().map(|q| q.get().box_clone()).collect::<Vec<_>>()
            ))
        }
    }

    #[staticmethod]
    fn range_q(field_id: u32, left: &str, right: &str) -> Query {
        Query {
            inner: Box::new(tv::query::RangeQuery::new_str_bounds(
                Field::from_field_id(field_id),
                Bound::Included(left),
                Bound::Included(right),
            )),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self.get()))
    }
}
