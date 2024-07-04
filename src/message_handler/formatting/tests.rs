use indoc::indoc;
use tree_sitter::Parser;

use crate::message_handler::formatting::format_helper;

fn check_formatting(ugly_query: &str, pretty_query: &str) {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_sparql::language())
        .unwrap();
    let tree = parser.parse(ugly_query, None).unwrap();
    let formatted_query = format_helper(
        &ugly_query.to_string(),
        &mut tree.root_node().walk(),
        0,
        "  ",
    );
    assert_eq!(formatted_query, pretty_query);
}
#[test]
fn prologue() {
    let ugly_query = indoc!(
        "
              PReFIX   namespace:  <uri>

            prefix namespace:  <uri>
            SELECT * {}"
    );
    let pretty_query = indoc!(
        "PREFIX namespace: <uri>
             PREFIX namespace: <uri>

             SELECT * {}"
    );
    check_formatting(ugly_query, pretty_query);
}

#[test]
fn nesting_indentation() {
    let ugly_query = "SELECT * {{{SELECT * {?a ?a ?a}}}}";
    let pretty_query = indoc!(
        "SELECT * {
               {
                 {
                   SELECT * {
                     ?a ?a ?a
                   }
                 }
               }
             }"
    );
    check_formatting(ugly_query, pretty_query);
}
#[test]
fn alternating_group_graph_pattern() {
    let ugly_query = indoc!(
        "SELECT * {
             ?a ?c ?b .{
             } ?a ?b ?c
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
               ?a ?c ?b .
               {}
               ?a ?b ?c
             }"
    );
    check_formatting(ugly_query, pretty_query);
}

#[test]
fn union() {
    let ugly_query = indoc!(
        "SELECT * {
             {} UNION {}
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
               {}
               UNION
               {}
             }"
    );
    check_formatting(ugly_query, pretty_query)
}

#[test]
fn optional() {
    let ugly_query = indoc!(
        "SELECT * {
             Optional
             {
             ?a ?c ?c}
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
               OPTIONAL {
                 ?a ?c ?c
               }
             }"
    );
    check_formatting(ugly_query, pretty_query)
}
