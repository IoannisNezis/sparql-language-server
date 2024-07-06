use log::warn;
use tree_sitter::{Node, Tree, TreeCursor};

use crate::lsp::{
    textdocument::{TextDocumentItem, TextEdit},
    FormattingOptions,
};

use super::utils::KEYWORDS;

pub(super) fn format_query(
    document: &TextDocumentItem,
    tree: &Tree,
    options: &FormattingOptions,
) -> Vec<TextEdit> {
    let range = document.get_full_range();
    let indent_string = match options.insert_spaces {
        true => " ".repeat(options.tab_size as usize),
        false => "\t".to_string(),
    };
    let text = format_helper(&document.text, &mut tree.walk(), 0, &indent_string);
    vec![TextEdit::new(range, text)]
}
pub(super) fn format_helper(
    text: &String,
    cursor: &mut TreeCursor,
    indentation: usize,
    indent_base: &str,
) -> String {
    let mut result = String::new();
    let indent_str = &indent_base.repeat(indentation);
    let indent_str_small = match indentation {
        0 => String::new(),
        i => (&indent_base).repeat(i - 1),
    };
    let line_break = "\n".to_string() + &indent_str;
    let line_break_small = "\n".to_string() + &indent_str_small;

    match cursor.node().kind() {
        "unit" => {
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                "\n\n",
                0,
                indent_base,
            ));
        }
        "Prologue" | "GroupOrUnionGraphPattern" | "MinusGraphPattern" => {
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                &line_break,
                indentation,
                indent_base,
            ));
        }
        "BaseDecl"
        | "PrefixDecl"
        | "SubSelect"
        | "DatasetClause"
        | "DefaultGraphClause"
        | "NamedGraphClause"
        | "TriplesSameSubject"
        | "property"
        | "WhereClause"
        | "OptionalGraphPattern"
        | "GraphGraphPattern"
        | "ServiceGraphPattern"
        | "Filter"
        | "binary_expression"
        | "Bind"
        | "InlineData"
        | "ValuesClause"
        | "DataBlock"
        | "GroupClause"
        | "GroupCondition"
        | "HavingClause"
        | "HavingCondition"
        | "OrderClause"
        | "OrderCondition"
        | "LimitOffsetClauses"
        | "LimitClause"
        | "OffsetClause"
        | "ExistsFunc"
        | "NotExistsFunc" => {
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                " ",
                indentation,
                indent_base,
            ));
        }
        "Aggregate" | "SelectClause" => {
            result.push_str(
                &separate_children_by(text, &cursor.node(), " ", indentation, indent_base)
                    .replace("( ", "(")
                    .replace(" )", ")"),
            );
        }

        "ConstructQuery" => {
            result.push_str(
                &separate_children_by(text, &cursor.node(), "\n", indentation, indent_base)
                    .replace("\n{", " {"),
            );
        }
        "SelectQuery" | "DescribeQuery" | "AskQuery" => {
            result.push_str(
                &separate_children_by(text, &cursor.node(), " ", indentation, indent_base)
                    .replace("} \n", "}\n"),
            );
        }
        "ObjectList" | "ExpressionList" | "SubstringExpression" | "RegexExpression" | "ArgList" => {
            result.push_str(
                &separate_children_by(text, &cursor.node(), "", 0, indent_base).replace(",", ", "),
            );
        }
        // TODO: indent propertys properly
        "PropertyListNotEmpty" => {
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                &line_break,
                indentation,
                indent_base,
            ));
        }
        "GroupGraphPattern" | "BrackettedExpression" | "ConstructTemplate" => {
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                "",
                indentation + 1,
                indent_base,
            ));
        }
        "BuildInCall" | "FunctionCall" => {
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                "",
                indentation,
                indent_base,
            ));
        }
        "GroupGraphPatternSub" | "ConstructTriples" => {
            result.push_str(&line_break);
            result.push_str(
                &separate_children_by(text, &cursor.node(), &line_break, indentation, indent_base)
                    .replace(&(line_break + "."), " ."),
            );
            result.push_str(&line_break_small);
        }
        "SolutionModifier" => {
            result.push_str(&line_break);
            result.push_str(&separate_children_by(
                text,
                &cursor.node(),
                &line_break,
                indentation,
                indent_base,
            ));
        }
        "TriplesBlock" => {
            result.push_str(
                &separate_children_by(text, &cursor.node(), " ", indentation, indent_base)
                    .replace(". ", &(".".to_string() + &line_break)),
            );
        }
        keyword if (KEYWORDS.contains(&keyword)) => {
            result.push_str(&cursor.node().kind().to_string().to_uppercase());
        }
        "PNAME_NS" | "IRIREF" | "VAR" | "INTEGER" | "DECIMAL" | "String" | "NIL"
        | "BLANK_NODE_LABEL" | "RdfLiteral" | "PrefixedName" | "(" | ")" | "{" | "}" | "."
        | "," | ";" | "*" | "+" | "-" | "/" | "<" | ">" | "=" | ">=" | "<=" | "!=" | "||"
        | "&&" | "path_element" => {
            result.push_str(&extract_node(text, &cursor.node()));
        }
        other => {
            warn!("found unknown node kind while formatting: {}", other);
            result.push_str(&extract_node(text, &cursor.node()));
        }
    }
    return result;
}

fn separate_children_by(
    text: &String,
    node: &Node,
    seperator: &str,
    indentation: usize,
    indent_base: &str,
) -> String {
    node.children(&mut node.walk())
        .map(|node| format_helper(text, &mut node.walk(), indentation, indent_base))
        .collect::<Vec<String>>()
        .join(seperator)
}

fn extract_node(source_code: &String, node: &Node) -> String {
    source_code
        .get(node.start_byte()..node.end_byte())
        .unwrap()
        .to_string()
}
