#![allow(clippy::new_ret_no_self)]

use pyo3::{exceptions, prelude::*};

use tantivy::schema::{self, FacetOptions};

use crate::schema::Schema;
use std::sync::{Arc, RwLock};
use tantivy::schema::INDEXED;

/// Tantivy has a very strict schema.
/// You need to specify in advance whether a field is indexed or not,
/// stored or not.
///
/// This is done by creating a schema object, and
/// setting up the fields one by one.
///
/// Examples:
///
///     >>> builder = tantivy.SchemaBuilder()
///
///     >>> title = builder.add_text_field("title", stored=True)
///     >>> body = builder.add_text_field("body")
///
///     >>> schema = builder.build()
#[pyclass]
#[derive(Clone)]
pub(crate) struct SchemaBuilder {
    pub(crate) builder: Arc<RwLock<Option<schema::SchemaBuilder>>>,
}

const TOKENIZER: &str = "default";
const RECORD: &str = "position";

#[pymethods]
impl SchemaBuilder {
    #[new]
    fn new() -> Self {
        SchemaBuilder {
            builder: Arc::new(From::from(Some(schema::Schema::builder()))),
        }
    }

    /// Add a new text field to the schema.
    ///
    /// Args:
    ///     name (str): The name of the field.
    ///     stored (bool, optional): If true sets the field as stored, the
    ///         content of the field can be later restored from a Searcher.
    ///         Defaults to False.
    ///     tokenizer_name (str, optional): The name of the tokenizer that
    ///         should be used to process the field. Defaults to 'default'
    ///     index_option (str, optional): Sets which information should be
    ///         indexed with the tokens. Can be one of 'position', 'freq' or
    ///         'basic'. Defaults to 'position'. The 'basic' index_option
    ///         records only the document ID, the 'freq' option records the
    ///         document id and the term frequency, while the 'position' option
    ///         records the document id, term frequency and the positions of
    ///         the term occurrences in the document.
    ///
    /// Returns the associated field handle.
    /// Raises a ValueError if there was an error with the field creation.
    #[args(
        stored = false,
        tokenizer_name = "TOKENIZER",
        index_option = "RECORD"
    )]
    fn add_text_field(
        &mut self,
        name: &str,
        stored: bool,
        tokenizer_name: &str,
        index_option: &str,
    ) -> PyResult<Self> {
        let builder = &mut self.builder;
        let options = SchemaBuilder::build_text_option(
            stored,
            tokenizer_name,
            index_option,
        )?;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_text_field(name, options);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Add a new signed integer field to the schema.
    ///
    /// Args:
    ///     name (str): The name of the field.
    ///     stored (bool, optional): If true sets the field as stored, the
    ///         content of the field can be later restored from a Searcher.
    ///         Defaults to False.
    ///     indexed (bool, optional): If true sets the field to be indexed.
    ///     fast (str, optional): Set the u64 options as a single-valued fast
    ///         field. Fast fields are designed for random access. Access time
    ///         are similar to a random lookup in an array. If more than one
    ///         value is associated to a fast field, only the last one is kept.
    ///         Can be one of 'single' or 'multi'. If this is set to 'single,
    ///         the document must have exactly one value associated to the
    ///         document. If this is set to 'multi', the document can have any
    ///         number of values associated to the document. Defaults to None,
    ///         which disables this option.
    ///
    /// Returns the associated field handle.
    /// Raises a ValueError if there was an error with the field creation.
    #[args(stored = false, indexed = false)]
    fn add_integer_field(
        &mut self,
        name: &str,
        stored: bool,
        indexed: bool,
        fast: Option<&str>,
    ) -> PyResult<Self> {
        let builder = &mut self.builder;

        let opts = SchemaBuilder::build_int_option(stored, indexed, fast)?;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_i64_field(name, opts);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Add a new float64 field to the schema.
    /// Note: When adding value to the index, make sure that it is type-casted to float
    /// Adding integers or other values may produce false result
    ///
    /// Args:
    ///     name (str): The name of the field.
    ///     stored (bool, optional): If true sets the field as stored, the
    ///         content of the field can be later restored from a Searcher.
    ///         Defaults to False.
    ///     indexed (bool, optional): If true sets the field to be indexed.
    ///     fast (str, optional): Set the f64 options as a single-valued fast
    ///         field. Fast fields are designed for random access. Access time
    ///         are similar to a random lookup in an array. If more than one
    ///         value is associated to a fast field, only the last one is kept.
    ///         Can be one of 'single' or 'multi'. If this is set to 'single,
    ///         the document must have exactly one value associated to the
    ///         document. If this is set to 'multi', the document can have any
    ///         number of values associated to the document. Defaults to None,
    ///         which disables this option.
    ///
    /// Returns the associated field handle.
    /// Raises a ValueError if there was an error with the field creation.
    #[args(stored = false, indexed = false)]
    fn add_float_field(
        &mut self,
        name: &str,
        stored: bool,
        indexed: bool,
        fast: Option<&str>,
    ) -> PyResult<Self> {
        let builder = &mut self.builder;

        let opts = SchemaBuilder::build_float_option(stored, indexed, fast)?;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_f64_field(name, opts);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Add a new unsigned integer field to the schema.
    ///
    /// Args:
    ///     name (str): The name of the field.
    ///     stored (bool, optional): If true sets the field as stored, the
    ///         content of the field can be later restored from a Searcher.
    ///         Defaults to False.
    ///     indexed (bool, optional): If true sets the field to be indexed.
    ///     fast (str, optional): Set the u64 options as a single-valued fast
    ///         field. Fast fields are designed for random access. Access time
    ///         are similar to a random lookup in an array. If more than one
    ///         value is associated to a fast field, only the last one is kept.
    ///         Can be one of 'single' or 'multi'. If this is set to 'single,
    ///         the document must have exactly one value associated to the
    ///         document. If this is set to 'multi', the document can have any
    ///         number of values associated to the document. Defaults to None,
    ///         which disables this option.
    ///
    /// Returns the associated field handle.
    /// Raises a ValueError if there was an error with the field creation.
    #[args(stored = false, indexed = false)]
    fn add_unsigned_field(
        &mut self,
        name: &str,
        stored: bool,
        indexed: bool,
        fast: Option<&str>,
    ) -> PyResult<Self> {
        let builder = &mut self.builder;

        let opts = SchemaBuilder::build_int_option(stored, indexed, fast)?;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_u64_field(name, opts);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Add a new date field to the schema.
    ///
    /// Args:
    ///     name (str): The name of the field.
    ///     stored (bool, optional): If true sets the field as stored, the
    ///         content of the field can be later restored from a Searcher.
    ///         Defaults to False.
    ///     indexed (bool, optional): If true sets the field to be indexed.
    ///     fast (str, optional): Set the u64 options as a single-valued fast
    ///         field. Fast fields are designed for random access. Access time
    ///         are similar to a random lookup in an array. If more than one
    ///         value is associated to a fast field, only the last one is kept.
    ///         Can be one of 'single' or 'multi'. If this is set to 'single,
    ///         the document must have exactly one value associated to the
    ///         document. If this is set to 'multi', the document can have any
    ///         number of values associated to the document. Defaults to None,
    ///         which disables this option.
    ///
    /// Returns the associated field handle.
    /// Raises a ValueError if there was an error with the field creation.
    #[args(stored = false, indexed = false)]
    fn add_date_field(
        &mut self,
        name: &str,
        stored: bool,
        indexed: bool,
        fast: Option<&str>,
    ) -> PyResult<Self> {
        let builder = &mut self.builder;

        let opts = SchemaBuilder::build_int_option(stored, indexed, fast)?;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_date_field(name, opts);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Add a new json field to the schema.
    ///
    /// Args:
    ///     name (str): the name of the field.
    ///     stored (bool, optional): If true sets the field as stored, the
    ///         content of the field can be later restored from a Searcher.
    ///         Defaults to False.
    ///     tokenizer_name (str, optional): The name of the tokenizer that
    ///         should be used to process the field. Defaults to 'default'
    ///     index_option (str, optional): Sets which information should be
    ///         indexed with the tokens. Can be one of 'position', 'freq' or
    ///         'basic'. Defaults to 'position'. The 'basic' index_option
    ///         records only the document ID, the 'freq' option records the
    ///         document id and the term frequency, while the 'position' option
    ///         records the document id, term frequency and the positions of
    ///         the term occurrences in the document.
    ///
    /// Returns the associated field handle.
    /// Raises a ValueError if there was an error with the field creation.
    #[args(
        stored = false,
        tokenizer_name = "TOKENIZER",
        index_option = "RECORD"
    )]
    fn add_json_field(
        &mut self,
        name: &str,
        stored: bool,
        tokenizer_name: &str,
        index_option: &str,
    ) -> PyResult<Self> {
        let builder = &mut self.builder;
        let options = SchemaBuilder::build_text_option(
            stored,
            tokenizer_name,
            index_option,
        )?;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_json_field(name, options);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }

        Ok(self.clone())
    }

    /// Add a Facet field to the schema.
    /// Args:
    ///     name (str): The name of the field.
    #[args(stored = false, indexed = false)]
    fn add_facet_field(&mut self, name: &str) -> PyResult<Self> {
        let builder = &mut self.builder;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_facet_field(name, FacetOptions::default());
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Add a fast bytes field to the schema.
    ///
    /// Bytes field are not searchable and are only used
    /// as fast field, to associate any kind of payload
    /// to a document.
    ///
    /// Args:
    ///     name (str): The name of the field.
    fn add_bytes_field(&mut self, name: &str) -> PyResult<Self> {
        let builder = &mut self.builder;

        if let Some(builder) = builder.write().unwrap().as_mut() {
            builder.add_bytes_field(name, INDEXED);
        } else {
            return Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ));
        }
        Ok(self.clone())
    }

    /// Finalize the creation of a Schema.
    ///
    /// Returns a Schema object. After this is called the SchemaBuilder cannot
    /// be used anymore.
    fn build(&mut self) -> PyResult<Schema> {
        let builder = self.builder.write().unwrap().take();
        if let Some(builder) = builder {
            let schema = builder.build();
            Ok(Schema { inner: schema })
        } else {
            Err(exceptions::PyValueError::new_err(
                "Schema builder object isn't valid anymore.",
            ))
        }
    }
}

impl SchemaBuilder {
    fn build_int_option(
        stored: bool,
        indexed: bool,
        fast: Option<&str>,
    ) -> PyResult<schema::NumericOptions> {
        let opts = schema::NumericOptions::default();

        let opts = if stored { opts.set_stored() } else { opts };
        let opts = if indexed { opts.set_indexed() } else { opts };

        let fast = match fast {
            Some(f) => {
                let f = f.to_lowercase();
                match f.as_ref() {
                    "single" => Some(schema::Cardinality::SingleValue),
                    "multi" => Some(schema::Cardinality::MultiValues),
                    _ => return Err(exceptions::PyValueError::new_err(
                        "Invalid index option, valid choices are: 'multivalue' and 'singlevalue'"
                    )),
                }
            }
            None => None,
        };

        let opts = if let Some(f) = fast {
            opts.set_fast(f)
        } else {
            opts
        };

        Ok(opts)
    }

    fn build_float_option(
        stored: bool,
        indexed: bool,
        fast: Option<&str>,
    ) -> PyResult<schema::NumericOptions> {
        let opts = schema::NumericOptions::default();

        let opts = if stored { opts.set_stored() } else { opts };
        let opts = if indexed { opts.set_indexed() } else { opts };

        let fast = match fast {
            Some(f) => {
                let f = f.to_lowercase();
                match f.as_ref() {
                    "single" => Some(schema::Cardinality::SingleValue),
                    "multi" => Some(schema::Cardinality::MultiValues),
                    _ => return Err(exceptions::PyValueError::new_err(
                        "Invalid index option, valid choices are: 'multivalue' and 'singlevalue'"
                    )),
                }
            }
            None => None,
        };

        let opts = if let Some(f) = fast {
            opts.set_fast(f)
        } else {
            opts
        };

        Ok(opts)
    }

    fn build_text_option(
        stored: bool,
        tokenizer_name: &str,
        index_option: &str,
    ) -> PyResult<schema::TextOptions> {
        let index_option = match index_option {
            "position" => schema::IndexRecordOption::WithFreqsAndPositions,
            "freq" => schema::IndexRecordOption::WithFreqs,
            "basic" => schema::IndexRecordOption::Basic,
            _ => return Err(exceptions::PyValueError::new_err(
                "Invalid index option, valid choices are: 'basic', 'freq' and 'position'"
            ))
        };

        let indexing = schema::TextFieldIndexing::default()
            .set_tokenizer(tokenizer_name)
            .set_index_option(index_option);

        let options =
            schema::TextOptions::default().set_indexing_options(indexing);
        let options = if stored {
            options.set_stored()
        } else {
            options
        };

        Ok(options)
    }
}
