import tantivy
import pytest

from tantivy import Document, Index, SchemaBuilder, Schema, Field, Term


def schema():
    return SchemaBuilder() \
        .add_text_field("title", stored=True) \
        .add_text_field("body") \
        .build()


@pytest.fixture(scope="class")
def ram_index():
    # assume all tests will use the same documents for now
    # other methods may set up function-local indexes
    index = Index(schema())
    writer = index.writer()

    # 2 ways of adding documents
    # 1
    doc = Document()
    # create a document instance
    # add field-value pairs
    doc.add_text("title", "The Old Man and the Sea")
    doc.add_text("body", ("He was an old man who fished alone in a skiff in"
                          "the Gulf Stream and he had gone eighty-four days "
                          "now without taking a fish."))
    writer.add_document(doc)
    # 2 use the built-in json support
    # keys need to coincide with field names
    doc = Document.from_dict({
        "title": "Of Mice and Men",
        "body": ("A few miles south of Soledad, the Salinas River drops "
                 "in close to the hillside bank and runs deep and "
                 "green. The water is warm too, for it has slipped "
                 "twinkling over the yellow sands in the sunlight "
                 "before reaching the narrow pool. On one side of the "
                 "river the golden foothill slopes curve up to the "
                 "strong and rocky Gabilan Mountains, but on the valley "
                 "side the water is lined with trees—willows fresh and "
                 "green with every spring, carrying in their lower leaf "
                 "junctures the debris of the winter’s flooding; and "
                 "sycamores with mottled, white, recumbent limbs and "
                 "branches that arch over the pool")
    })
    writer.add_document(doc)
    writer.add_json("""{
            "title": ["Frankenstein", "The Modern Prometheus"],
            "body": "You will rejoice to hear that no disaster has accompanied the commencement of an enterprise which you have regarded with such evil forebodings.  I arrived here yesterday, and my first task is to assure my dear sister of my welfare and increasing confidence in the success of my undertaking."
        }""")
    writer.commit()
    index.reload()
    return index


class TestClass(object):

    def test_simple_search(self, ram_index):
        index = ram_index
        query = index.parse_query("sea whale", ["title", "body"])

        top_docs = tantivy.TopDocs(10)

        result = index.searcher().search(query, top_docs)
        assert len(result) == 1
        _, doc_address = result[0]
        searched_doc = index.searcher().doc(doc_address)
        assert searched_doc["title"] == ["The Old Man and the Sea"]

    def test_and_query(self, ram_index):
        index = ram_index
        query = index.parse_query("title:men AND body:summer", default_field_names=["title", "body"])
        # look for an intersection of documents
        top_docs = tantivy.TopDocs(10)
        searcher = index.searcher()
        result = searcher.search(query, top_docs)

        # summer isn't present
        assert len(result) == 0

        query = index.parse_query("title:men AND body:winter", ["title", "body"])
        result = searcher.search(query, top_docs)

        assert len(result) == 1

    def test_and_query_parser_default_fields(self, ram_index):
        query = ram_index.parse_query("winter", default_field_names=["title"])
        assert repr(query) == """Query(TermQuery(Term(field=0,bytes=[119, 105, 110, 116, 101, 114])))"""

    def test_and_query_parser_default_fields_undefined(self, ram_index):
        query = ram_index.parse_query("winter")
        assert repr(query) == "Query(BooleanQuery { subqueries: [" \
                              "(Should, TermQuery(Term(field=0,bytes=[119, 105, 110, 116, 101, 114]))), " \
                              "(Should, TermQuery(Term(field=1,bytes=[119, 105, 110, 116, 101, 114])))] " \
                              "})"

    def test_query_errors(self, ram_index):
        index = ram_index
        # no "bod" field
        with pytest.raises(ValueError):
            index.parse_query("bod:men", ["title", "body"])


class TestUpdateClass(object):

    def test_delete_update(self, ram_index):
        query = ram_index.parse_query("Frankenstein", ["title"])
        top_docs = tantivy.TopDocs(10)
        result = ram_index.searcher().search(query, top_docs)
        assert len(result) == 1

        schema = ram_index.schema
        field = Field.from_schema(schema, 'title')
        term = Term.from_field_text(field, "frankenstein")
        writer = ram_index.writer()
        writer.delete_term(term)
        writer.commit();
        ram_index.reload()

        query = ram_index.parse_query("Frankenstein", ["title"])
        top_docs = tantivy.TopDocs(10)
        result = ram_index.searcher().search(query, top_docs)
        assert len(result) == 0



PATH_TO_INDEX = "tests/test_index/"


class TestFromDiskClass(object):

    def test_exists(self):
        # prefer to keep it separate in case anyone deletes this
        # runs from the root directory
        assert Index.exists(PATH_TO_INDEX)

    def test_opens_from_dir(self):
        index = Index(schema(), PATH_TO_INDEX, reuse=True)
        assert index.searcher().num_docs == 3

    def test_create_readers(self):
        # not sure what is the point of this test.
        idx = Index(schema())
        assert idx.searcher().num_docs == 0
        # by default this is manual mode
        writer = idx.writer(30000000, 1)
        writer.add_document(Document(title="mytitle", body="mybody"))
        writer.commit()
        assert idx.searcher().num_docs == 0
        # Manual is the default setting.
        # In this case, change are reflected only when
        # the index is manually reloaded.
        idx.reload()
        assert idx.searcher().num_docs == 1
        idx.config_reader("OnCommit", 4)
        writer.add_document(Document(title="mytitle2", body="mybody2"))
        writer.commit()
        import time
        for i in range(50):
            # The index should be automatically reloaded.
            # Wait for at most 5s for it to happen.
            time.sleep(0.1)
            if idx.searcher().num_docs == 2:
                return
        assert False


class TestSearcher(object):
    def test_searcher_repr(self, ram_index):
        assert repr(ram_index.searcher()) == "Searcher(num_docs=3, num_segments=1)"


class TestDocument(object):

    def test_document(self):
        doc = tantivy.Document(name="Bill", reference=[1, 2])
        assert doc["reference"] == [1, 2]
        assert doc["name"] == ["Bill"]
        assert doc.get_first("name") == "Bill"
        assert doc.get_first("reference") == 1
        assert doc.to_dict() == {"name": ["Bill"], "reference": [1, 2]}

    def test_document_with_date(self):
        import datetime
        date = datetime.datetime(2019, 8, 12, 13, 0, 0, )
        doc = tantivy.Document(name="Bill", date=date)
        assert doc["date"][0] == date

    def test_document_repr(self):
        doc = tantivy.Document(name="Bill", reference=[1, 2])
        assert repr(doc) == "Document(name=[Bill],reference=[1,2])"

    def test_document_with_facet(self):
        doc = tantivy.Document()
        facet = tantivy.Facet.from_string("/europe/france")
        doc.add_facet("facet", facet)
        assert doc["facet"][0].to_path() == ['europe', 'france']
        doc = tantivy.Document()
        facet = tantivy.Facet.from_string("/asia\\/oceania/fiji")
        doc.add_facet("facet", facet)
        assert doc["facet"][0].to_path() == ['asia/oceania', 'fiji']
        assert doc["facet"][0].to_path_str() == "/asia\\/oceania/fiji"
        assert repr(doc["facet"][0]) == "Facet(/asia\\/oceania/fiji)"
        doc = tantivy.Document(facet=facet)
        assert doc["facet"][0].to_path() == ['asia/oceania', 'fiji']

    def test_document_error(self):
        with pytest.raises(ValueError):
            tantivy.Document(name={})
