[![Build Status](https://travis-ci.org/quickwit-inc/tantivy-py.svg?branch=master)](https://travis-ci.org/quickwit-inc/tantivy-py)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

tantivy-py
==========

Python bindings for [Tantivy](https://github.com/quickwit-oss/tantivy) the full-text search engine library written in Rust.


# Installation

The bindings can be installed using from pypi using pip:

    pip install tantivy

If no binary wheel is present for your operating system the bindings will be
build from source, this means that Rust needs to be installed before building
can succeed.

Note that the bindings are using [PyO3](https://github.com/PyO3/pyo3), which
only supports python3.

# Development

Setting up a development environment can be done in a virtual environment using
[`nox`](https://nox.thea.codes) or using local packages using the provided `Makefile`.

For the `nox` setup install the virtual environment and build the bindings using:

    python3 -m pip install nox
    nox

For the `Makefile` based setup run:

    make

Running the tests is done using:

    make test

Building python wheel
    pip install maturin
    maturin build

# Usage

The Python bindings have a similar API to Tantivy. To create a index first a schema
needs to be built. After that documents can be added to the index and a reader
can be created to search the index.

## Building an index and populating it

```python
import tantivy

# Declaring our schema.
schema_builder = tantivy.SchemaBuilder()
schema_builder.add_text_field("title", stored=True)
schema_builder.add_text_field("body", stored=True)
schema_builder.add_integer_field("doc_id",stored=True)
schema = schema_builder.build()

# Creating our index (in memory)
index = tantivy.Index(schema)
```

To have a persistent index, use the path
parameter to store the index on the disk, e.g:

```python
index = tantivy.Index(schema, path=os.getcwd() + '/index')
```

By default, tantivy  offers the following tokenizers
which can be used in tantivy-py:
 -  `default`
`default` is the tokenizer that will be used if you do not
 assign a specific tokenizer to your text field.
 It will chop your text on punctuation and whitespaces,
 removes tokens that are longer than 40 chars, and lowercase your text.

-  `raw`
 Does not actual tokenizer your text. It keeps it entirely unprocessed.
 It can be useful to index uuids, or urls for instance.

-  `en_stem`

 In addition to what `default` does, the `en_stem` tokenizer also
 apply stemming to your tokens. Stemming consists in trimming words to
 remove their inflection. This tokenizer is slower than the default one,
 but is recommended to improve recall.

to use the above tokenizers, simply provide them as a parameter to `add_text_field`. e.g.
```python
schema_builder.add_text_field("body",  stored=True,  tokenizer_name='en_stem')
```

### Adding one document.

```python
writer = index.writer()
writer.add_document(tantivy.Document(
	doc_id=1,
    title=["The Old Man and the Sea"],
    body=["""He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish."""],
))
# ... and committing
writer.commit()
```


## Building and Executing Queries

First you need to get a searcher for the index

```python
# Reload the index to ensure it points to the last commit.
index.reload()
searcher = index.searcher()
```

Then you need to get a valid query object by parsing your query on the index.

```python
query = index.parse_query("fish days", ["title", "body"])
(best_score, best_doc_address) = searcher.search(query, 3).hits[0]
best_doc = searcher.doc(best_doc_address)
assert best_doc["title"] == ["The Old Man and the Sea"]
print(best_doc)
```

### Valid Query Formats

tantivy-py supports the query language used in tantivy.
Some basic query Formats.


 - AND and OR conjunctions.
```python
query = index.parse_query('(Old AND Man) OR Stream', ["title", "body"])
(best_score, best_doc_address) = searcher.search(query, 3).hits[0]
best_doc = searcher.doc(best_doc_address)
```

 - +(includes) and -(excludes) operators.
```python
query = index.parse_query('+Old +Man chef -fished', ["title", "body"])
(best_score, best_doc_address) = searcher.search(query, 3).hits[0]
best_doc = searcher.doc(best_doc_address)
```
Note: in a query like above, a word with no +/- acts like an OR.

 - phrase search.
```python
query = index.parse_query('"eighty-four days"', ["title", "body"])
(best_score, best_doc_address) = searcher.search(query, 3).hits[0]
best_doc = searcher.doc(best_doc_address)
```

- integer search
```python
query = index.parse_query('"eighty-four days"', ["doc_id"])
(best_score, best_doc_address) = searcher.search(query, 3).hits[0]
best_doc = searcher.doc(best_doc_address)
```
Note: for integer search, the integer field should be indexed.

For more possible query formats and possible query options, see [Tantivy Query Parser Docs.](https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html)