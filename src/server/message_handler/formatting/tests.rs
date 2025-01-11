use indoc::indoc;
use tree_sitter::Parser;

use crate::server::{
    configuration::FormatSettings,
    lsp::{textdocument::TextDocumentItem, FormattingOptions},
    message_handler::formatting::format_textdoument,
};

fn format_and_compare(ugly_query: &str, pretty_query: &str) {
    let format_settings = FormatSettings::default();
    let format_options = FormattingOptions {
        tab_size: 2,
        insert_spaces: true,
    };
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_sparql::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(ugly_query, None).unwrap();
    let mut document = TextDocumentItem::new("testdocument", ugly_query);
    let changes = format_textdoument(&document, &tree, &format_settings, &format_options);
    document.apply_text_edits(changes);
    assert_eq!(document.text, pretty_query);
}
#[test]
fn format_prologue() {
    let ugly_query = indoc!(
        "
              PReFIX   namespace:  <uri>

            prefix namespace:  <uri>
            SELECT * {}\n"
    );
    let pretty_query = indoc!(
        "PREFIX namespace: <uri>
         PREFIX namespace: <uri>
         SELECT * {}
         "
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn format_nesting_indentation() {
    let ugly_query = "SELECT * {{{SELECT * {?a ?a ?a}}}}\n";
    let pretty_query = indoc!(
        "SELECT * {
           {
             {
               SELECT * {
                 ?a ?a ?a
               }
             }
           }
         }\n"
    );
    format_and_compare(ugly_query, pretty_query);
}
#[test]
fn format_alternating_group_graph_pattern() {
    let ugly_query = indoc!(
        "SELECT * {
             ?a ?c ?b .{
             } ?a ?b ?c
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           ?a ?c ?b .
           {}
           ?a ?b ?c
         }\n"
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn format_union() {
    let ugly_query = indoc!(
        "SELECT * {
           {} UNION { {} UNION {}}
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           {}
           UNION {
             {}
             UNION {}
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_select_clause() {
    let ugly_query = indoc!("SELECT ( <> as ?a) ?a  *{}\n");
    let pretty_query = indoc!("SELECT (<> AS ?a) ?a * {}\n");
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_optional() {
    let ugly_query = indoc!(
        "SELECT * {
             Optional
             {
             ?a ?c ?c}
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           OPTIONAL {
             ?a ?c ?c
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_minus() {
    let ugly_query = indoc!(
        "SELECT * {
             {} MINUS {{} MINUS {}}
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           {}
           MINUS {
             {}
             MINUS {}
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_graph() {
    let ugly_query = indoc!(
        "SELECT * {
             {} Graph ?a { ?a ?b  ?c}
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           {}
           GRAPH ?a {
             ?a ?b ?c
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_filter() {
    let ugly_query = indoc!("SELECT * {filter   (1>0)}\n");
    let pretty_query = indoc!(
        "SELECT * {
           FILTER (1 > 0)
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_binary_expression() {
    let ugly_query = indoc!(
        "SELECT * {
            filter (1 = 3+2-2.9*10/0 && 
                    1 > 2 ||
                    1 < 2 ||
                    1 <= 2 &&
                    1 >= 9 ||
                    1 != 3 ||
                    5 in (1,2,3) &&
                    6 not in (4,5,6+3))}\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           FILTER (1 = 3 + 2 - 2.9 * 10 / 0 && 1 > 2 || 1 < 2 || 1 <= 2 && 1 >= 9 || 1 != 3 || 5 IN (1, 2, 3) && 6 NOT IN (4, 5, 6 + 3))
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_bind() {
    let ugly_query = indoc!("SELECT * {Bind (1 as ?var )}\n");
    let pretty_query = indoc!(
        "SELECT * {
           BIND (1 AS ?var)
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_inline_data() {
    let ugly_query = indoc!("SELECT * {values ?a { 1 2 3}}\n");
    let pretty_query = indoc!(
        "SELECT * {
           VALUES ?a { 1 2 3 }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_values_clause() {
    let ugly_query = indoc!("SELECT * {}values ?a { 1 2 3}\n");
    let pretty_query = indoc!(
        "SELECT * {}
         VALUES ?a { 1 2 3 }\n"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_solution_modifier() {
    let ugly_query = indoc!(
        "SELECT * WHERE {}
           GROUP by ( 2 AS ?a )
          HAVING (2 > 2) (1 > 2)
            order BY ASC (?c)
         OFfSET 3 LiMIT 3"
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {}
         GROUP BY (2 AS ?a)
         HAVING (2 > 2) (1 > 2)
         ORDER BY ASC(?c)
         OFFSET 3
         LIMIT 3
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_dataset_clause() {
    let ugly_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         SELECT  ?name ?x FROM    <http://example.org/foaf/aliceFoaf> FROM    <http://example.org/foaf/aliceFoaf>
         WHERE   { ?x foaf:name ?name }\n"
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         SELECT ?name ?x
         FROM <http://example.org/foaf/aliceFoaf>
         FROM <http://example.org/foaf/aliceFoaf>
         WHERE {
           ?x foaf:name ?name
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_construct() {
    let ugly_query = indoc!(
        "PREFIX foaf:  <http://xmlns.com/foaf/0.1/>
         PREFIX vcard:  <http://www.w3.org/2001/vcard-rdf/3.0#>
         CONSTRUCT   { <http://example.org/person#Alice> vcard:FN ?name }
         WHERE       { ?x foaf:name ?name } LIMIT 10"
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         PREFIX vcard: <http://www.w3.org/2001/vcard-rdf/3.0#>
         CONSTRUCT {
           <http://example.org/person#Alice> vcard:FN ?name
         }
         WHERE {
           ?x foaf:name ?name
         }
         LIMIT 10
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_describe() {
    let ugly_query = indoc!(
        "PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
         DESCRIBE ?x ?y <http://example.org/>
         WHERE    {?x foaf:knows ?y}\n"
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         DESCRIBE ?x ?y <http://example.org/> WHERE {
           ?x foaf:knows ?y
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_ask() {
    let ugly_query = indoc!(
        r#"PREFIX foaf:    <http://xmlns.com/foaf/0.1/>
           ASK  { ?x foaf:name  "Alice" }
           "#
    );
    let pretty_query = indoc!(
        r#"PREFIX foaf: <http://xmlns.com/foaf/0.1/>
           ASK {
             ?x foaf:name "Alice"
           }
           "#
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_graph_management() {
    let ugly_query = indoc!(
        "load SIlENT <a> INTO graph <c> ;
              LOAD    <b>; Clear Graph <b>          ;
          drop   graph<c>  ; ADD SILENT GRAPH <c> to DEFAULT ;MOVE default TO GRAPH <a> ;
                  create graph <d>"
    );
    let pretty_query = indoc!(
        "LOAD SILENT <a> INTO GRAPH <c> ;
         LOAD <b> ;
         CLEAR GRAPH <b> ;
         DROP GRAPH <c> ;
         ADD SILENT GRAPH <c> TO DEFAULT ;
         MOVE DEFAULT TO GRAPH <a> ;
         CREATE GRAPH <d>
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_insert_data() {
    let ugly_query = indoc!(
        "Insert   data
         {
            ?a ?b ?c.
             graph <a> {
         ?c ?b ?a }.
         ?d ?e ?f
         graph  ?d 
         {
         ?a ?d ?c
         }
         ?d ?e ?f
         }\n"
    );
    let pretty_query = indoc!(
        "INSERT DATA {
           ?a ?b ?c .
           GRAPH <a> {
             ?c ?b ?a
           } .
           ?d ?e ?f
           GRAPH ?d {
             ?a ?d ?c
           }
           ?d ?e ?f
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_delete_data() {
    let ugly_query = indoc!(
        "delete   data
         {
            ?a ?b ?c.
             graph <a> {
         ?c ?b ?a }.
         ?d ?e ?f
         graph  ?d 
         {
         ?a ?d ?c
         }
         ?d ?e ?f
         }\n"
    );
    let pretty_query = indoc!(
        "DELETE DATA {
           ?a ?b ?c .
           GRAPH <a> {
             ?c ?b ?a
           } .
           ?d ?e ?f
           GRAPH ?d {
             ?a ?d ?c
           }
           ?d ?e ?f
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_delete_where() {
    let ugly_query = indoc!(
        "delete   where
         {
            ?a ?b ?c.
             graph <a> {
         ?c ?b ?a }.
         ?d ?e ?f
         graph  ?d 
         {
         ?a ?d ?c
         }
         ?d ?e ?f
         }\n"
    );
    let pretty_query = indoc!(
        "DELETE WHERE {
           ?a ?b ?c .
           GRAPH <a> {
             ?c ?b ?a
           } .
           ?d ?e ?f
           GRAPH ?d {
             ?a ?d ?c
           }
           ?d ?e ?f
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_modify() {
    let ugly_query = indoc!(
        "With <a> delete
         { 
         ?a  ?b   ?C   
          } insert { ?x ?y ?z } using <a> using named <b> where
             {
         { ?a ?b ?c  .  }
         }\n"
    );
    let pretty_query = indoc!(
        "WITH <a>
         DELETE {
           ?a ?b ?C
         }
         INSERT {
           ?x ?y ?z
         }
         USING <a>
         USING NAMED <b>
         WHERE {
           {
             ?a ?b ?c .
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_property_paths() {
    let ugly_query = indoc!(
        "SELECT *
         WHERE  { ?P foaf:givenName ?G ; foaf:surname ?S }\n"
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?P foaf:givenName ?G ;
              foaf:surname ?S
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_property_list_paths() {
    let ugly_query = indoc!(
        "SELECT * WHERE {
           ?a          <iri>/^a/(!<>?)+   |           (<iri> 
         | ^a |  a) ?b .
         }\n"
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?a <iri>/^a/(!<>?)+ | (<iri> | ^a | a) ?b .
         }
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_comments() {
    let ugly_query = indoc!(
        "# unit comment 1
         PREFIX test: <test>
           # prolouge comment
         PREFIX test: <test>  # unit comment 2
         SELECT ?a WHERE {
         # GroupGraphPattern comment 1
           ?c <> ?a . # Triples comment
           ?d <> ?b .
           ?b <> ?a .
           # GroupGraphPatternSub comment
           {} # GroupGraphPattern comment 2
         }


         # unit comment 3"
    );
    let pretty_query = indoc!(
        "# unit comment 1
         PREFIX test: <test>
         # prolouge comment
         PREFIX test: <test>
         # unit comment 2
         SELECT ?a WHERE {
           # GroupGraphPattern comment 1
           ?c <> ?a .
           # Triples comment
           ?d <> ?b .
           ?b <> ?a .
           # GroupGraphPatternSub comment
           {}
           # GroupGraphPattern comment 2
         }
         # unit comment 3
         "
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_function_like_keywords() {
    let ugly_query = indoc!(
        r#"SELECT (MAX (?a)  AS ?max_a ) WHERE {
           BIND (  "A" AS  ?a )
           FILTER ( ?a = "A")
           FILTER YEAR ( ?a)
           FILTER <>  (2)
         }
         GROUP BY(2 AS ?d)
         HAVING (?a > 2)
         ORDER BY DESC (?d)
         LIMIT 12 OFFSET 20
        "#
    );
    let pretty_query = indoc!(
        r#"SELECT (MAX(?a) AS ?max_a) WHERE {
             BIND ("A" AS ?a)
             FILTER (?a = "A")
             FILTER YEAR(?a)
             FILTER <>(2)
           }
           GROUP BY (2 AS ?d)
           HAVING (?a > 2)
           ORDER BY DESC(?d)
           LIMIT 12
           OFFSET 20
           "#
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn format_full_select_querry() {
    let ugly_query = indoc!(
        "PREFIX namespace: <iri>
         SELECT ?var From <dataset> FROM <dataset> WHERE {
         ?s ?p ?o
         }
         GROUP BY ?s
         HAVING (?p > 0)
         ORDER BY DESC(?o)
         LIMIT 12 OFFSET 20"
    );
    let pretty_query = indoc!(
        "PREFIX namespace: <iri>
         SELECT ?var
         FROM <dataset>
         FROM <dataset>
         WHERE {
           ?s ?p ?o
         }
         GROUP BY ?s
         HAVING (?p > 0)
         ORDER BY DESC(?o)
         LIMIT 12
         OFFSET 20
         "
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn format_blank_node_property_list_path() {
    let ugly_query_1 = indoc!(
        "SELECT * WHERE {
            wd:Q11571 p:P166 [ps:P166 ?entity ;
                      pq:P585 ?date ]
         }
         "
    );
    let pretty_query_1 = indoc!(
        "SELECT * WHERE {
           wd:Q11571 p:P166 [
             ps:P166 ?entity ;
             pq:P585 ?date
           ]
         }
         "
    );
    format_and_compare(ugly_query_1, pretty_query_1);
    let ugly_query_2 = indoc!(
        "SELECT * WHERE {
            wd:Q11571 p:P166 [
                      pq:P585 ?date ]
         }
         "
    );
    let pretty_query_2 = indoc!(
        "SELECT * WHERE {
           wd:Q11571 p:P166 [ pq:P585 ?date ]
         }
         "
    );
    format_and_compare(ugly_query_2, pretty_query_2);
}

#[test]
fn format_anon() {
    let ugly_query = indoc!(
        "SELECT * WHERE {
            ?s ?p
            [
            ]
         }
         "
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?s ?p []
         }
         "
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn format_comments_property_lists() {
    let ugly_query = indoc!(
        "SELECT * WHERE {
           ?rettore p:P106 [
             pq:P642 wd:Q193510 ;
           # of Padua Univerity
             pq:P580 ?starttime ;
           ]
         }"
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?rettore p:P106 [
             pq:P642 wd:Q193510 ;
             # of Padua Univerity
             pq:P580 ?starttime ;
           ]
         }
         "
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn format_commas() {
    let ugly_query = indoc!(
        r#"SELECT * WHERE {
           ?a ?b ",,," .
           FILTER (1 IN (1,2,3))
         }
         "#
    );
    let pretty_query = indoc!(
        r#"SELECT * WHERE {
            ?a ?b ",,," .
            FILTER (1 IN (1, 2, 3))
          }
          "#
    );
    format_and_compare(ugly_query, pretty_query);
}
