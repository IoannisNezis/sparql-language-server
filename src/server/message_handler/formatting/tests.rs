use indoc::indoc;
use tree_sitter::Parser;

use crate::server::{
    configuration::FormatSettings,
    lsp::{
        textdocument::{TextDocumentItem, TextEdit},
        FormattingOptions,
    },
    message_handler::formatting::format_document,
};

fn check_collision(edits: &Vec<TextEdit>) {
    for idx1 in 0..edits.len() {
        for idx2 in idx1 + 1..edits.len() {
            let a = edits.get(idx1).unwrap();
            let b = edits.get(idx2).unwrap();
            assert!(!a.overlaps(&b));
        }
    }
}

fn format_and_compare(ugly_query: &str, pretty_query: &str, format_settings: &FormatSettings) {
    let format_options = FormattingOptions {
        tab_size: 2,
        insert_spaces: true,
    };
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_sparql::LANGUAGE.into())
        .unwrap();
    // let tree = parser.parse(ugly_query, None).unwrap();
    let mut document = TextDocumentItem::new("testdocument", ugly_query);
    let tree = parser
        .parse(ugly_query.as_bytes(), None)
        .expect("could not parse");
    let edits = format_document(&document, &tree, &format_options, format_settings).unwrap();
    check_collision(&edits);
    document.apply_text_edits(edits);
    assert_eq!(document.text, pretty_query);
}

#[test]
fn format_basic() {
    let ugly_query = indoc!(
        "SELECT * WHERE{
           ?a ?c ?b .
           ?a ?b ?c
         }
        "
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?a ?c ?b .
           ?a ?b ?c
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}
#[test]
fn format_prologue() {
    let ugly_query = indoc!(
        "
              PReFIX   namespace:  <uri>

            prefix namespace:  <uri>
            SELECT * {}
        "
    );
    let pretty_query = indoc!(
        "PREFIX namespace: <uri>
         PREFIX namespace: <uri>
         SELECT * {}
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}
#[test]
fn format_alternating_group_graph_pattern() {
    let ugly_query = indoc!("SELECT  *  {  ?a  ?c  ?b  .    {   }  ?a  ?b   ?c   }");
    let pretty_query = indoc!(
        "SELECT * {
           ?a ?c ?b .
           {}
           ?a ?b ?c
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_union() {
    let ugly_query = indoc!(
        "SELECT * { ?a ?b ?c
           {} UNION { {} UNION {} . ?a ?b ?c}
             }
        "
    );
    let pretty_query = indoc!(
        "SELECT * {
           ?a ?b ?c
           {}
           UNION {
             {}
             UNION {} .
             ?a ?b ?c
           }
         }
        "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_surouding_whitespace() {
    let ugly_query = indoc!(
        "    
          
            
           SELECT * WHERE {}
    
            
                "
    );
    let pretty_query = indoc!("SELECT * WHERE {}\n");
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_select_clause() {
    let ugly_query = indoc!("SELECT (   <>    as   ?a  )    ?a   {  }  \n");
    let pretty_query = indoc!("SELECT (<> AS ?a) ?a {}\n");
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_optional() {
    let ugly_query = indoc!(
        "SELECT * {
         ?s ?p ?o
             Optional
             {
             ?a ?c ?c}
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           ?s ?p ?o
           OPTIONAL {
             ?a ?c ?c
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_service() {
    let ugly_query = indoc!(
        "SELECT * {
         ?s ?p ?o
             SERVICE <iri>
             {
             ?a ?c ?c}
             }\n"
    );
    let pretty_query = indoc!(
        "SELECT * {
           ?s ?p ?o
           SERVICE <iri> {
             ?a ?c ?c
           }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_minus() {
    let ugly_query = indoc!(
        "SELECT * {
             {} MINUS {{} MINUS {}}
             }
        "
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_graph() {
    let ugly_query = indoc!(
        "SELECT * {
             {} Graph ?a { ?a ?b  ?c}
             }
        "
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_bind() {
    let ugly_query = indoc!("SELECT * {?s ?p ?o Bind (1 as ?var )}\n");
    let pretty_query = indoc!(
        "SELECT * {
           ?s ?p ?o
           BIND (1 AS ?var)
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_values_clause() {
    let ugly_query = indoc!("SELECT * {}values ?a { 1 2 3}\n");
    let pretty_query = indoc!(
        "SELECT * {}
         VALUES ?a { 1 2 3 }\n"
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_solution_modifier() {
    let ugly_query = indoc!(
        "SELECT * WHERE {
            {
              SELECT * WHERE {
              }
             GROUP by ( 2 AS ?a )
           HAVING (2 > 2) (1 > 2)
            order BY ASC (?c)
          OFfSET 3 LiMIT 3
            }
         }
           GROUP by ( 2 AS ?a )
          HAVING (2 > 2) (1 > 2)
            order BY ASC (?c)
         OFfSET 3 LiMIT 3"
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           {
             SELECT * WHERE {}
             GROUP BY (2 AS ?a)
             HAVING (2 > 2) (1 > 2)
             ORDER BY ASC(?c)
             OFFSET 3
             LIMIT 3
           }
         }
         GROUP BY (2 AS ?a)
         HAVING (2 > 2) (1 > 2)
         ORDER BY ASC(?c)
         OFFSET 3
         LIMIT 3
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_dataset_clause() {
    let ugly_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         SELECT  ?name ?x FROM    <http://example.org/foaf/aliceFoaf> FROM    <http://example.org/foaf/aliceFoaf>
         WHERE   { ?x foaf:name ?name }
        "
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_construct() {
    let ugly_query = indoc!(
        "PREFIX foaf:  <http://xmlns.com/foaf/0.1/>
         PREFIX vcard:  <http://www.w3.org/2001/vcard-rdf/3.0#>
         CONSTRUCT   { <http://example.org/person#Alice> vcard:FN ?name }
         WHERE       { ?x foaf:name ?name } LIMIT 10
         "
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_describe() {
    let ugly_query = indoc!(
        "PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
         DESCRIBE ?x ?y <http://example.org/>
         WHERE    {?x foaf:knows ?y}
         "
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         DESCRIBE ?x ?y <http://example.org/>
         WHERE {
           ?x foaf:knows ?y
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_graph_management() {
    let ugly_query = indoc!(
        "PREFIX a: <> load SIlENT <a> INTO graph <c> ;
              LOAD    <b>; Clear Graph <b>          ;
          drop   graph<c>  ; ADD SILENT GRAPH <c> to DEFAULT ;MOVE default TO GRAPH <a> ;
                  create graph <d>
        "
    );
    let pretty_query = indoc!(
        "PREFIX a: <>
         LOAD SILENT <a> INTO GRAPH <c> ;
         LOAD <b> ;
         CLEAR GRAPH <b> ;
         DROP GRAPH <c> ;
         ADD SILENT GRAPH <c> TO DEFAULT ;
         MOVE DEFAULT TO GRAPH <a> ;
         CREATE GRAPH <d>
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_insert_data() {
    let ugly_query = indoc!(
        "INSERT { ?v <a> <b> } WHERE { VALUES ?v { 1 2 } }
         "
    );
    let pretty_query = indoc!(
        "INSERT {
           ?v <a> <b>
         }
         WHERE {
           VALUES ?v { 1 2 }
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_delete_data() {
    let ugly_query = indoc!(
        "delete   data
         {
            ?a ?b ?c. 
             graph <a> {
         ?c ?b ?a.
             ?c ?b ?a   }.
         ?d ?e ?f
         graph  ?d 
         {
         ?a ?d ?c
         }
         ?d ?e ?f
         }
         "
    );
    let pretty_query = indoc!(
        "DELETE DATA {
           ?a ?b ?c .
           GRAPH <a> {
             ?c ?b ?a .
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
         }
         "
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_property_paths() {
    let ugly_query1 = indoc!(
        "SELECT *
         WHERE  { ?P foaf:givenName ?G ; foaf:surname ?S;?p ?o;<><> }
        "
    );
    let pretty_query1 = indoc!(
        "SELECT * WHERE {
           ?P foaf:givenName ?G ;
              foaf:surname ?S ;
              ?p ?o ;
              <> <>
         }
         "
    );
    format_and_compare(ugly_query1, pretty_query1, &FormatSettings::default());
    let ugly_query2 = indoc!(
        "SELECT *
         WHERE  { ?P foaf:givenName ?G ; foaf:surname ?S; }
        "
    );
    let pretty_query2 = indoc!(
        "SELECT * WHERE {
           ?P foaf:givenName ?G ;
              foaf:surname ?S ;
         }
         "
    );
    format_and_compare(ugly_query2, pretty_query2, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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


         # unit comment 3
         "
    );
    let pretty_query = indoc!(
        "# unit comment 1
         PREFIX test: <test>
         # prolouge comment
         PREFIX test: <test> # unit comment 2
         SELECT ?a WHERE {
           # GroupGraphPattern comment 1
           ?c <> ?a . # Triples comment
           ?d <> ?b .
           ?b <> ?a .
           # GroupGraphPatternSub comment
           {} # GroupGraphPattern comment 2
         }
         # unit comment 3
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_aggregate() {
    let ugly_query = indoc!(
        r#"SELECT (MAX (?a + 2)  AS ?max_a ) (SAMPLE (DISTINCT ?x) as ?dx) WHERE {
         }
        "#
    );
    let pretty_query = "SELECT (MAX(?a + 2) AS ?max_a) (SAMPLE(DISTINCT ?x) AS ?dx) WHERE {}\n";
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query_1, pretty_query_1, &FormatSettings::default());
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
    format_and_compare(ugly_query_2, pretty_query_2, &FormatSettings::default());
}

#[test]
fn format_anon() {
    let ugly_query = indoc!(
        "SELECT * WHERE {
         ?s ?p[]
         }
         "
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?s ?p []
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_comments_with_dots() {
    let ugly_query = indoc!(
        "SELECT * WHERE {
           ?s # First comment sentence. Second comment sentence.
           ?p
           ?o
           .
           ?s ?p ?o
         }
         "
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
           ?s # First comment sentence. Second comment sentence.
           ?p ?o .
           ?s ?p ?o
         }
         "
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
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
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_filter_inline() {
    let ugly_query = indoc!(
        r#"SELECT * WHERE {
           FILTER (?a)
           ?a ?b ",,," .
           FILTER (?a)
           ?a ?b ?c . FILTER (?a)
           ?a ?b ?c FILTER (?a)
         }
         "#
    );
    let pretty_query = indoc!(
        r#"SELECT * WHERE {
             FILTER (?a)
             ?a ?b ",,," .
             FILTER (?a)
             ?a ?b ?c . FILTER (?a)
             ?a ?b ?c FILTER (?a)
           }
           "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_leading_comments() {
    let ugly_query = indoc!(
        r#"     # comment 1
                
            #comment 2
           SELECT * WHERE {}
         "#
    );
    let pretty_query = indoc!(
        r#"# comment 1
           #comment 2
           SELECT * WHERE {}
         "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_trailing_comments() {
    let ugly_query = indoc!(
        r#"SELECT *          # trailing comment
           WHERE {}

               # non trailing comment
         "#
    );
    let pretty_query = indoc!(
        r#"SELECT * # trailing comment
           WHERE {}
           # non trailing comment
         "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_object_list() {
    let ugly_query = indoc!(
        r#"SELECT * WHERE {
             <a>
             <b>
             <c>
             ,
             <d>
           }
         "#
    );
    let pretty_query = indoc!(
        r#"SELECT * WHERE {
             <a> <b> <c>, <d>
           }
         "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_comments_in_empty_ggp() {
    let ugly_query = indoc!(
        r#"SELECT * WHERE {
           #a
           }
          "#
    );
    let pretty_query = indoc!(
        r#"SELECT * WHERE {
             #a
           }
          "#
    );

    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}
#[test]
fn format_comments_in_strange_positions() {
    let ugly_query = indoc!(
        r#"#asd
           SELECT           # trailing comment
            * 
            # c1
           WHERE #c2
           {#3
           }#c3

               # non trailing comment
         "#
    );
    let pretty_query = indoc!(
        r#"#asd
           SELECT # trailing comment
           *
           # c1
           WHERE #c2
           { #3
           } #c3
           # non trailing comment
         "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_group_concat() {
    let ugly_query = indoc!(
        r#"SELECT (  GROUP_CONCAT  (   ?a   ;   SEPARATOR  =  "bar"  )  AS ?x) WHERE {}
          "#
    );
    let pretty_query = "SELECT (GROUP_CONCAT(?a; SEPARATOR=\"bar\") AS ?x) WHERE {}\n";
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_setting_align_prefixes() {
    let mut format_settings = FormatSettings::default();
    format_settings.align_prefixes = true;
    let ugly_query = indoc!(
        "PREFIX namespace123: <iri> PREFIX namespace12: <iri> PREFIX namespace1: <iri>
         SELECT * WHERE {}
         "
    );
    let pretty_query1 = indoc!(
        "PREFIX namespace123: <iri>
         PREFIX namespace12:  <iri>
         PREFIX namespace1:   <iri>
         SELECT * WHERE {}
         "
    );
    let pretty_query2 = indoc!(
        "PREFIX namespace123: <iri>
         PREFIX namespace12: <iri>
         PREFIX namespace1: <iri>
         SELECT * WHERE {}
         "
    );
    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.align_prefixes = false;
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_setting_align_predicates() {
    let mut format_settings = FormatSettings::default();
    format_settings.align_predicates = true;
    let ugly_query = indoc!(
        "SELECT * WHERE {
           ?adlasjsalkdjaldasjd <> <> ; <> <>
         }
         "
    );
    let pretty_query1 = indoc!(
        "SELECT * WHERE {
           ?adlasjsalkdjaldasjd <> <> ;
                                <> <>
         }
         "
    );
    let pretty_query2 = indoc!(
        "SELECT * WHERE {
           ?adlasjsalkdjaldasjd <> <> ;
             <> <>
         }
         "
    );

    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.align_predicates = false;
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_setting_separate_prolouge() {
    let mut format_settings = FormatSettings::default();
    format_settings.separate_prolouge = true;
    let ugly_query = indoc!(
        "PREFIX namespace: <iri>
         SELECT * WHERE {}
         "
    );
    let pretty_query1 = indoc!(
        "PREFIX namespace: <iri>

         SELECT * WHERE {}
         "
    );
    let pretty_query2 = indoc!(
        "PREFIX namespace: <iri>
         SELECT * WHERE {}
         "
    );

    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.separate_prolouge = false;
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_setting_capitalize_keywords() {
    let mut format_settings = FormatSettings::default();
    format_settings.capitalize_keywords = true;
    let ugly_query = indoc!(
        "prefix namespace: <iri>
         select * where {}
         "
    );
    let pretty_query1 = indoc!(
        "PREFIX namespace: <iri>
         SELECT * WHERE {}
         "
    );
    let pretty_query2 = indoc!(
        "prefix namespace: <iri>
         select * where {}
         "
    );

    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.capitalize_keywords = false;
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_setting_insert_spaces() {
    let mut format_settings = FormatSettings::default();
    format_settings.insert_spaces = Some(true);
    let ugly_query = indoc!(
        "SELECT * WHERE { ?a ?b ?c }
         "
    );
    let pretty_query1 = indoc!(
        "SELECT * WHERE {
           ?a ?b ?c
         }
         "
    );
    let pretty_query2 = indoc!(
        "SELECT * WHERE {
         \t?a ?b ?c
         }
         "
    );

    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.insert_spaces = Some(false);
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_setting_tab_size() {
    let mut format_settings = FormatSettings::default();
    format_settings.tab_size = Some(4);
    format_settings.insert_spaces = Some(true);
    let ugly_query = indoc!(
        "SELECT * WHERE { ?a ?b ?c }
         "
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {
             ?a ?b ?c
         }
         "
    );

    format_and_compare(ugly_query, pretty_query, &format_settings);
}

#[test]
fn format_setting_where_new_line() {
    let mut format_settings = FormatSettings::default();
    format_settings.where_new_line = true;
    let ugly_query = indoc!(
        "SELECT * WHERE { ?a ?b ?c }
         "
    );
    let pretty_query1 = indoc!(
        "SELECT *
         WHERE {
           ?a ?b ?c
         }
         "
    );
    let pretty_query2 = indoc!(
        "SELECT * WHERE {
           ?a ?b ?c
         }
         "
    );

    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.where_new_line = false;
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_setting_filter_same_line() {
    let mut format_settings = FormatSettings::default();
    format_settings.filter_same_line = true;
    let ugly_query = indoc!(
        "SELECT * WHERE { ?a ?b ?c FILTER (?a)}
         "
    );
    let pretty_query1 = indoc!(
        "SELECT * WHERE {
           ?a ?b ?c FILTER (?a)
         }
         "
    );
    let pretty_query2 = indoc!(
        "SELECT * WHERE {
           ?a ?b ?c
           FILTER (?a)
         }
         "
    );

    format_and_compare(ugly_query, pretty_query1, &format_settings);
    format_settings.filter_same_line = false;
    format_and_compare(ugly_query, pretty_query2, &format_settings);
}

#[test]
fn format_comments_1() {
    let ugly_query = indoc!(
        r#"SELECT   #1
               #2
           * WHERE {}
          "#
    );
    let pretty_query = indoc!(
        r#"SELECT #1
           #2
           * WHERE {}
          "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_comments_and_tss() {
    let ugly_query = indoc!(
        r#"SELECT * WHERE {
             <> <> <> ;  #c1
                <> <> ;
                <> <> ;     #c2
           }
          "#
    );
    let pretty_query = indoc!(
        r#"SELECT * WHERE {
             <> <> <> ; #c1
                <> <> ;
                <> <> ; #c2
           }
          "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_utf8() {
    let ugly_query = indoc!(
        r#"SELECT * WHERE {
             ?väriable <öther> "😀"
           }
          "#
    );
    let pretty_query = indoc!(
        r#"SELECT * WHERE {
             ?väriable <öther> "😀"
           }
          "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}

#[test]
fn format_comments_2() {
    let ugly_query = indoc!(
        r#"PREFIX namespace: <iri>
           SELECT#1
           #2
           * WHERE {}
          "#
    );
    let pretty_query = indoc!(
        r#"PREFIX namespace: <iri>
           SELECT #1
           #2
           * WHERE {}
          "#
    );
    format_and_compare(ugly_query, pretty_query, &FormatSettings::default());
}
