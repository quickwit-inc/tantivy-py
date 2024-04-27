# Tutorials

## Building an index and populating it

```python
import tempfile
import pathlib
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
tmpdir = tempfile.TemporaryDirectory()
index_path = pathlib.Path(tmpdir.name) / "index"
index_path.mkdir()
persistent_index = tantivy.Index(schema, path=str(index_path))
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
schema_builder_tok = tantivy.SchemaBuilder()
schema_builder_tok.add_text_field("body",  stored=True,  tokenizer_name='en_stem')
```

## Adding one document.

```python
writer = index.writer()
writer.add_document(tantivy.Document(
	doc_id=1,
    title=["The Old Man and the Sea"],
    body=["""He was an old man who fished alone in a skiff in the Gulf Stream and he had gone eighty-four days now without taking a fish."""],
))
# ... and committing
writer.commit()
writer.wait_merging_threads()
```

Note that `wait_merging_threads()` must come at the end, because
the `writer` object will not be usable after this call.

## Building and Executing Queries with the Query Parser

With the Query Parser, you can easily build simple queries for your index.

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
```

The `parse_query` method takes in a query string (visit [reference](reference.md#valid-query-formats) for more details on the syntax) and create a `Query` object that can be used to search the index.

In Tantivy, hit documents during search will return a `DocAddress` object that can be used to retrieve the document from the searcher, rather than returning the document directly.

## Building and Executing Queries with Query Objects

> *This is an advanced topic. Only consider this if you need very fine-grained control over your queries, or existing query parsers do not meet your needs.*

If you have a Lucene / ElasticSearch background, you might be more comfortable building nested queries programmatically. Also, some queries (e.g. ConstQuery, DisjunctionMaxQuery) are not supported by the query parser due to their complexity in expression.

Consider the following query in ElasticSearch:

```json
{
    "query": {
        "bool": {
            "must": [
                {
                    "dis_max": {
                        "queries": [
                            {
                                "match": {
                                    "title": {
                                        "query": "fish",
                                        "boost": 2
                                    }
                                }
                            },
                            {
                                "match": {
                                    "body": {
                                        "query": "eighty-four days",
                                        "boost": 1.5
                                    }
                                }
                            }
                        ],
                        "tie_breaker": 0.3
                    }
                }
            ]
        }
    }
}
```

It is impossible to express this query using the query parser. Instead, you can build the query programmatically mixing with the query parser:

```python
from tantivy import Query, Occur, Index

...

complex_query = Query.boolean_query(
    [
        (
            Occur.Must,
            Query.disjunction_max_query(
                [
                    Query.boost_query(
                        # by default, only the query parser will analyze
                        # your query string
                        index.parse_query("fish", ["title"]), 
                        2.0
                    ),
                    Query.boost_query(
                        index.parse_query("eighty-four days", ["body"]), 
                        1.5
                    ),
                ],
                0.3,
            ),
        )
    ]
)

```

<!--TODO: Update the reference link to the query parser docs when available.-->

## Using the snippet generator

Let's revisit the query `"fish days"` in our [example](#building-and-executing-queries-with-the-query-parser):

```python
hit_text = best_doc["body"][0]
print(f"{hit_text=}")
assert hit_text == (
    "He was an old man who fished alone in a skiff in the "
    "Gulf Stream and he had gone eighty-four days now "
    "without taking a fish."
)

from tantivy import SnippetGenerator
snippet_generator = SnippetGenerator.create(
    searcher, query, schema, "body"
)
snippet = snippet_generator.snippet_from_doc(best_doc)
```

The snippet object provides the hit ranges. These are the marker
offsets in the text that match the query.

```python
highlights = snippet.highlighted()
first_highlight = highlights[0]
assert first_highlight.start == 93
assert first_highlight.end == 97
assert hit_text[first_highlight.start:first_highlight.end] == "days"
```

The snippet object can also generate a marked-up HTML snippet:

```python
html_snippet = snippet.to_html()
assert html_snippet == (
    "He was an old man who fished alone in a skiff in the "
    "Gulf Stream and he had gone eighty-four <b>days</b> now "
    "without taking a <b>fish</b>"
)
```


