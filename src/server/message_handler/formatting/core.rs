use lazy_static::lazy_static;
use std::{collections::HashSet, str::CharIndices, usize};
use streaming_iterator::StreamingIterator;

use log::{error, info, warn};
use tree_sitter::{ffi::TSPoint, Node, Parser, Point, Query, QueryCursor, TreeCursor};

use crate::server::{
    configuration::FormatSettings,
    lsp::{
        textdocument::{Range, TextDocumentItem, TextEdit},
        FormattingOptions,
    },
};

use super::utils::KEYWORDS;

pub(super) fn format_document(
    document: &TextDocumentItem,
    parser: &mut Parser,
    options: &FormattingOptions,
    settings: &FormatSettings,
) -> Vec<TextEdit> {
    // TODO: Throw error dont panic!
    let indent_string = match settings.insert_spaces.unwrap_or(options.insert_spaces) {
        true => " ".repeat(settings.tab_size.unwrap_or(options.tab_size) as usize),
        false => "\t".to_string(),
    };
    let tree = parser.parse(document.text.as_bytes(), None).unwrap();
    let mut edits = collect_format_edits(
        &document.text,
        &mut tree.walk(),
        0,
        &indent_string,
        "",
        settings,
    );
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    let consolidated = consolidate_edits(edits);
    // for x in &consolidated {
    //     println!("{:?}", x);
    // }
    consolidated
}

fn consolidate_edits(edits: Vec<TextEdit>) -> Vec<TextEdit> {
    let accumulator: Vec<TextEdit> = Vec::new();
    edits.into_iter().fold(accumulator, |mut acc, edit| {
        if edit.is_empty() {
            return acc;
        }
        match acc.last_mut() {
            Some(prev) if prev.range.start == edit.range.end => {
                prev.new_text.insert_str(0, &edit.new_text);
                prev.range.start = edit.range.start;
            }
            _ => {
                acc.push(edit);
            }
        };
        acc
    })
}

lazy_static! {
    static ref BRACKETS_OPEN: HashSet<&'static str> = HashSet::from(["[", "(", "{"]);
    static ref BRACKETS_CLOSE: HashSet<&'static str> = HashSet::from(["]", ")", "}"]);
    static ref EXIT_EARLY: HashSet<&'static str> =
        HashSet::from(["STRING_LITERAL", "PN_LOCAL", ":"]);
    static ref INC_INDENTATION: HashSet<&'static str> = HashSet::from([
        "BlankNodePropertyListPath",
        "GroupGraphPattern",
        "BrackettedExpression",
        "ConstructTemplate",
        "QuadData",
        "QuadsNotTriples"
    ]);
}

pub(super) fn collect_format_edits(
    text: &String,
    cursor: &mut TreeCursor,
    mut indentation: usize,
    indent_base: &str,
    extra_indent: &str,
    settings: &FormatSettings,
) -> Vec<TextEdit> {
    let node = cursor.node();

    // NOTE: Early exit
    if EXIT_EARLY.contains(node.kind()) {
        return vec![];
    }

    let indent_str = &indent_base.repeat(indentation);
    let indent_str_big = (&indent_base).repeat(indentation + 1);
    // let indent_str_small = match indentation {
    //     0 => String::new(),
    //     i => (&indent_base).repeat(i - 1),
    // };
    let line_break = format!("\n{}", indent_str);

    // let line_break_small = format!("\n{}", indent_str_small);
    let line_break_big = format!("\n{}", indent_str_big);

    let separator = match node.kind() {
        "unit"
        | "Prologue"
        | "GroupOrUnionGraphPattern"
        | "Modify"
        | "TriplesTemplateBlock"
        | "GroupGraphPatternSub"
        | "Quads"
        | "SolutionModifier"
        | "LimitOffsetClauses" => &line_break,

        // "namespace"
        "ExpressionList"
        | "GroupGraphPattern"
        | "BrackettedExpression"
        | "ConstructTemplate"
        | "QuadData"
        | "ObjectList"
        | "SubstringExpression"
        | "RegexExpression"
        | "ArgList"
        | "OrderCondition"
        | "Aggregate"
        | "BuildInCall"
        | "FunctionCall"
        | "PathSequence"
        | "PathEltOrInverse"
        | "PathElt"
        | "PathPrimary"
        | "PN_PREFIX"
        | "BlankNodePropertyListPath"
        | "TriplesBlock"
        | "TriplesTemplate"
        | "ConstructTriples"
        | "ConstructQuery"
        | "assignment" => "",

        "BaseDecl"
        | "SelectClause"
        | "PropertyListPathNotEmpty"
        | "PrefixDecl"
        | "SelectQuery"
        | "SubSelect"
        | "WhereClause"
        | "DatasetClause"
        | "MinusGraphPattern"
        | "DefaultGraphClause"
        | "NamedGraphClause"
        | "TriplesSameSubject"
        | "property"
        | "OptionalGraphPattern"
        | "GraphGraphPattern"
        | "ServiceGraphPattern"
        | "binary_expression"
        | "InlineData"
        | "ValuesClause"
        | "DataBlock"
        | "GroupClause"
        | "GroupCondition"
        | "HavingClause"
        | "HavingCondition"
        | "OrderClause"
        | "LimitClause"
        | "OffsetClause"
        | "ExistsFunc"
        | "NotExistsFunc"
        | "Filter"
        | "Bind"
        | "Load"
        | "Clear"
        | "Drop"
        | "Add"
        | "Move"
        | "Copy"
        | "Create"
        | "InsertData"
        | "DeleteData"
        | "DeleteWhere"
        | "GraphRef"
        | "GraphRefAll"
        | "GraphOrDefault"
        | "DeleteClause"
        | "InsertClause"
        | "UsingClause"
        | "PropertyListNotEmpty"
        | "Path"
        | "TriplesSameSubjectPath" => " ",

        "SELECT" | "WHERE" | "PNAME_NS" | "IRIREF" | "VAR" | "INTEGER" | "DECIMAL" | "String"
        | "NIL" | "BLANK_NODE_LABEL" | "RdfLiteral" | "PrefixedName" | "PathMod" | "(" | ")"
        | "{" | "}" | "." | "," | ";" | "*" | "+" | "-" | "/" | "<" | ">" | "=" | ">=" | "<="
        | "!=" | "||" | "&&" | "|" | "^" | "[" | "]" => "",
        _ => {
            log::warn!("unknown node: {}", node.kind());
            " "
        }
    };

    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    let seperation_edits = children
        .iter()
        .zip(children.iter().skip(1))
        .map(|(node1, node2)| {
            // Add sparator between nodes
            TextEdit::new(
                Range::new(
                    node1.end_position().row as u32,
                    node1.end_position().column as u32,
                    node2.start_position().row as u32,
                    node2.start_position().column as u32,
                ),
                separator,
            )
        });

    if INC_INDENTATION.contains(node.kind()) {
        indentation += 1;
    }
    let recursive_edits = children.iter().flat_map(|node| {
        // collect edits from childs
        collect_format_edits(
            text,
            &mut node.walk(),
            indentation,
            indent_base,
            extra_indent,
            settings,
        )
    });
    let details_edits: Vec<TextEdit> = match node.kind() {
        "comment" => vec![TextEdit::new(
            Range::new(
                node.end_position().row as u32,
                node.end_position().column as u32,
                node.end_position().row as u32,
                node.end_position().column as u32,
            ),
            &line_break,
        )],
        "unit" => {
            let mut edits = vec![];
            if let Some(child) = children.first() {
                edits.push(TextEdit::new(
                    Range::from_ts_positions(Point { row: 0, column: 0 }, child.start_position()),
                    "",
                ));
            }
            if let Some(child) = children.last() {
                edits.push(TextEdit::new(
                    Range::from_ts_positions(child.end_position(), node.end_position()),
                    "",
                ));
            }

            edits
        }

        "BlankNodePropertyListPath" => match children.get(1).map(|child| child.child_count() > 3) {
            Some(false) => children
                .iter()
                .filter_map(|child| match child.kind() {
                    "[" => Some(TextEdit::new(Range::from_node(*child), "[ ")),
                    "]" => Some(TextEdit::new(Range::from_node(*child), " ]")),
                    _ => None,
                })
                .collect(),
            Some(true) => children
                .iter()
                .filter_map(|child| match child.kind() {
                    "[" => Some(TextEdit::new(
                        Range::from_node(*child),
                        &format!("[{}", line_break_big),
                    )),
                    "]" => Some(TextEdit::new(
                        Range::from_node(*child),
                        &format!("{}]", line_break),
                    )),
                    _ => None,
                })
                .collect(),
            None => vec![],
        },
        "PropertyListPathNotEmpty" => match node.parent() {
            Some(parent) => match parent.kind() {
                "BlankNodePropertyListPath" => {
                    let mut edits: Vec<TextEdit> = children
                        .iter()
                        .filter_map(|child| match child.kind() {
                            ";" => Some(TextEdit::new(
                                Range::from_node(*child),
                                &format!(";{}", &line_break[..line_break.len() - 1]),
                            )),
                            _ => None,
                        })
                        .collect();
                    edits
                }
                "TriplesSameSubjectPath" => children
                    .iter()
                    .filter_map(|child| match child.kind() {
                        ";" => Some(TextEdit::new(
                            Range::from_node(*child),
                            &format!(";{}", &line_break[..line_break.len() - 1]),
                        )),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            },
            None => vec![],
        },
        // "SolutionModifier" => vec![TextEdit::new(
        //     Range::from_ts_positions(node.start_position(), node.start_position()),
        //     "",
        // )],
        "ConstructTemplate" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "{" => Some(TextEdit::new(
                    Range::from_node(*child),
                    &format!(" {{{}{}", line_break, indent_base),
                )),
                "}" => Some(TextEdit::new(
                    Range::from_node(*child),
                    &format!("{}}}{}", line_break, line_break),
                )),
                _ => None,
            })
            .collect(),
        "TriplesBlock" | "ConstructTriples" => children
            .iter()
            .enumerate()
            .filter_map(|(idx, child)| match child.kind() {
                "." => match idx {
                    x if x < children.len() - 1 => Some(TextEdit::new(
                        Range::from_node(*child),
                        &format!(" .{}", line_break),
                    )),
                    _ => Some(TextEdit::new(Range::from_node(*child), " .")),
                },
                _ => None,
            })
            .collect(),

        "GroupGraphPattern" if children.len() > 2 => {
            children
                .iter()
                .filter_map(|child| match child.kind() {
                    open if BRACKETS_OPEN.contains(open) => Some(TextEdit::new(
                        Range::from_node(*child),
                        &format!("{}{}{}", open, line_break, indent_base),
                    )),
                    close if BRACKETS_CLOSE.contains(close) => Some(TextEdit::new(
                        Range::from_node(*child),
                        &format!("{}{}", line_break, close),
                    )),
                    _ => None,
                })
                .collect()

            // let mut edits = vec![];
            // if let Some(child) = children.first() {
            //     if BRACKETS_OPEN.contains(child.kind()) {
            //         edits.push(TextEdit::new(
            //             Range::from_node(*child),
            //             &format!("{}{}{}", child.kind(), line_break, indent_base),
            //         ));
            //     }
            // }
            // if let Some(child) = children.last() {
            //     if BRACKETS_CLOSE.contains(child.kind()) {
            //         edits.push(TextEdit::new(
            //             Range::from_node(*child),
            //             &format!("{}{}", line_break, child.kind(),),
            //         ));
            //     }
            // }
            // edits
        }
        "ExpressionList" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "," => Some(TextEdit::new(Range::from_node(*child), ", ")),
                _ => None,
            })
            .collect(),
        "AS" => vec![
            TextEdit::new(
                Range::new(
                    node.start_position().row as u32,
                    node.start_position().column as u32,
                    node.end_position().row as u32,
                    node.end_position().column as u32,
                ),
                " AS ",
            ),
            // TextEdit::new(
            //     Range::new(
            //         node.end_position().row as u32,
            //         node.end_position().column as u32,
            //         node.end_position().row as u32,
            //         node.end_position().column as u32,
            //     ),
            //     " ",
            // ),
        ],
        "ANON" => vec![TextEdit::new(Range::from_node(node), "[]")],
        keyword if (KEYWORDS.contains(&keyword)) => match settings.capitalize_keywords {
            true => vec![TextEdit::new(Range::from_node(node), keyword)],
            false => vec![TextEdit::new(
                Range::from_node(node),
                node.utf8_text(text.as_bytes()).unwrap(),
            )],
        },
        _ => vec![],
    };

    if node.kind() == "ERROR" {
        return recursive_edits.collect();
    }
    let x = recursive_edits
        .chain(details_edits.into_iter())
        .chain(seperation_edits)
        .collect();
    return x;
}

pub(super) fn format_parse1(
    text: &String,
    cursor: &mut TreeCursor,
    indentation: usize,
    indent_base: &str,
    extra_indent: &str,
    settings: &FormatSettings,
) -> String {
    let indent_str = &indent_base.repeat(indentation);
    let indent_str_small = match indentation {
        0 => String::new(),
        i => (&indent_base).repeat(i - 1),
    };
    let line_break = "\n".to_string() + &indent_str;
    let line_break_small = "\n".to_string() + &indent_str_small;

    match cursor.node().kind() {
        "unit" => separate_children_by(text, &cursor.node(), &line_break, 0, indent_base, settings)
            .replace("→", "")
            .replace("←", ""),
        "Update" => separate_children_by(text, &cursor.node(), " ", 0, indent_base, settings)
            .replace("; ", ";\n"),
        "Prologue" => {
            let mut formatted_string = separate_children_by(
                text,
                &cursor.node(),
                &line_break,
                indentation,
                indent_base,
                settings,
            );
            if settings.align_prefixes {
                formatted_string = align_prefixes(formatted_string, &cursor.node(), text);
            }
            formatted_string.replace("→", "").replace("←", "")
                + match settings.separate_prolouge {
                    true => "\n",
                    false => "",
                }
        }
        "GroupOrUnionGraphPattern" => separate_children_by(
            text,
            &cursor.node(),
            &line_break,
            indentation,
            indent_base,
            settings,
        )
        .replace(&("UNION".to_string() + &line_break), "UNION "),
        "Modify" => separate_children_by(
            text,
            &cursor.node(),
            &line_break,
            indentation,
            indent_base,
            settings,
        )
        .replace("WITH\n", "WITH ")
        .replace("WHERE\n{", "WHERE {"),
        "BaseDecl"
        | "PrefixDecl"
        | "SelectClause"
        | "SubSelect"
        | "DatasetClause"
        | "MinusGraphPattern"
        | "DefaultGraphClause"
        | "NamedGraphClause"
        | "TriplesSameSubject"
        | "property"
        | "OptionalGraphPattern"
        | "GraphGraphPattern"
        | "ServiceGraphPattern"
        | "binary_expression"
        | "InlineData"
        | "ValuesClause"
        | "DataBlock"
        | "GroupClause"
        | "GroupCondition"
        | "HavingClause"
        | "HavingCondition"
        | "OrderClause"
        | "LimitClause"
        | "OffsetClause"
        | "ExistsFunc"
        | "NotExistsFunc"
        | "Filter"
        | "Bind"
        | "Load"
        | "Clear"
        | "Drop"
        | "Add"
        | "Move"
        | "Copy"
        | "Create"
        | "InsertData"
        | "DeleteData"
        | "DeleteWhere"
        | "GraphRef"
        | "GraphRefAll"
        | "GraphOrDefault"
        | "DeleteClause"
        | "InsertClause"
        | "UsingClause"
        | "PropertyListNotEmpty"
        | "Path" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation,
            indent_base,
            settings,
        ),
        "assignment" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation,
            indent_base,
            settings,
        )
        .replace("( ", "(")
        .replace(" )", ")"),
        "WhereClause" => {
            (match settings.where_new_line {
                true => line_break,
                false => String::new(),
            } + &separate_children_by(
                text,
                &cursor.node(),
                " ",
                indentation,
                indent_base,
                settings,
            ))
        }

        "ConstructQuery" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation,
            indent_base,
            settings,
        )
        .replace(" WHERE", "\nWHERE")
        .replace("} ", "}"),
        "DescribeQuery" | "AskQuery" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation,
            indent_base,
            settings,
        )
        .replace("} \n", "}\n"),
        "SelectQuery" => {
            let seperator = match &cursor.node().child(1) {
                Some(node) if node.kind() == "DatasetClause" => &line_break,
                _ => " ",
            };
            separate_children_by(
                text,
                &cursor.node(),
                seperator,
                indentation,
                indent_base,
                settings,
            )
            .replace("} \n", "}\n")
        }

        "ExpressionList" => {
            separate_children_by(text, &cursor.node(), "", indentation, indent_base, settings)
                .replace(",", ", ")
        }
        "TriplesSameSubjectPath" => match cursor.node().child_count() {
            2 => {
                let subject = cursor
                    .node()
                    .child(0)
                    .expect("The first node has to exist, i just checked if there are 2.");
                let range = subject.range();
                let predicate = cursor
                    .node()
                    .child(1)
                    .expect("The second node has to exist, i just checked if there are 2.");
                let indent_str = match settings.align_predicates {
                    true => " ".repeat(range.end_point.column - range.start_point.column + 1),
                    false => indent_base.to_string(),
                };

                subject.utf8_text(text.as_bytes()).unwrap().to_string()
                    + " "
                    + &format_parse1(
                        text,
                        &mut predicate.walk(),
                        indentation,
                        indent_base,
                        &indent_str,
                        settings,
                    )
            }
            _ => separate_children_by(
                text,
                &cursor.node(),
                " ",
                indentation,
                indent_base,
                settings,
            ),
        },
        "BlankNodePropertyListPath" => {
            let ret = separate_children_by(
                text,
                &cursor.node(),
                " ",
                indentation + 1,
                indent_base,
                settings,
            );
            // Check of property list contains more than one member
            // n -> n/2 entries in the list
            if cursor.node().named_child(0).unwrap().named_child_count() > 2 {
                format!(
                    "[{}{}{}{}]",
                    &line_break,
                    &indent_base,
                    &ret[2..ret.len() - 2],
                    &line_break
                )
            } else {
                ret
            }
        }
        "PropertyListPathNotEmpty" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation,
            indent_base,
            settings,
        ) // TODO: Only break on ";" if there is a value afterwards
        .replace("; ", &format!(";{}{}", &line_break, &extra_indent))
        .replace("→", "")
        .replace("← ", &line_break),
        "GroupGraphPattern" | "BrackettedExpression" | "ConstructTemplate" | "QuadData" => {
            separate_children_by(
                text,
                &cursor.node(),
                "",
                indentation + 1,
                indent_base,
                settings,
            )
            .replace("{→", &("{".to_string() + &line_break + indent_base))
            .replace("→", indent_base)
            .replace("←}", &(line_break + "}"))
            .replace("←", "")
        }
        "ObjectList"
        | "SubstringExpression"
        | "RegexExpression"
        | "ArgList"
        | "OrderCondition"
        | "Aggregate"
        | "BuildInCall"
        | "FunctionCall"
        | "PathSequence"
        | "PathEltOrInverse"
        | "PathElt"
        | "PathPrimary" => {
            separate_children_by(text, &cursor.node(), "", indentation, indent_base, settings)
        }
        "QuadsNotTriples" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation + 1,
            indent_base,
            settings,
        ),
        "TriplesTemplateBlock" => separate_children_by(
            text,
            &cursor.node(),
            &line_break,
            indentation,
            indent_base,
            settings,
        )
        .replace(&(line_break + "}"), &(line_break_small + "}")),
        "GroupGraphPatternSub" | "ConstructTriples" | "Quads" => {
            line_break.clone()
                + &separate_children_by(
                    text,
                    &cursor.node(),
                    &line_break,
                    indentation,
                    indent_base,
                    settings,
                )
                .replace(&(line_break + "."), " .")
                .replace("→", "")
                .replace("←", "")
                + &line_break_small
        }
        "SolutionModifier" => {
            // NOTE: If the Query contains a DatasetClause the childs are speparated by a newline
            // in this case no line break is required here
            match &cursor
                .node()
                .parent()
                .map(|parent| parent.child(1))
                .flatten()
            {
                Some(node) if node.kind() == "DatasetClause" => "",
                _ => &line_break,
            }
            .to_string()
                + &separate_children_by(
                    text,
                    &cursor.node(),
                    &line_break,
                    indentation,
                    indent_base,
                    settings,
                )
        }
        "LimitOffsetClauses" => separate_children_by(
            text,
            &cursor.node(),
            &line_break,
            indentation,
            indent_base,
            settings,
        ),
        "TriplesBlock" | "TriplesTemplate" => separate_children_by(
            text,
            &cursor.node(),
            " ",
            indentation,
            indent_base,
            settings,
        )
        .replace("→", "")
        .replace("← ", &line_break),
        // NOTE: Here a marker chars (→ and ←) are used to mark the start and end of a comment node.
        // Later these markers are removed.
        // This assumes that the marker char does not appear in queries.
        // TODO: solve this better
        "comment" => "→".to_string() + cursor.node().utf8_text(text.as_bytes()).unwrap() + "←",
        keyword if (KEYWORDS.contains(&keyword)) => match settings.capitalize_keywords {
            true => keyword.to_string(),
            false => cursor
                .node()
                .utf8_text(text.as_bytes())
                .unwrap()
                .to_string(),
        },
        "ANON" => "[]".to_string(),
        "PNAME_NS" | "IRIREF" | "VAR" | "INTEGER" | "DECIMAL" | "String" | "NIL"
        | "BLANK_NODE_LABEL" | "RdfLiteral" | "PrefixedName" | "PathMod" | "(" | ")" | "{"
        | "}" | "." | "," | ";" | "*" | "+" | "-" | "/" | "<" | ">" | "=" | ">=" | "<=" | "!="
        | "||" | "&&" | "|" | "^" | "[" | "]" => cursor
            .node()
            .utf8_text(text.as_bytes())
            .unwrap()
            .to_string(),
        // "ERROR" => {
        //     cursor.node().child
        //     if let Some(child) = cursor.node().child(0) {
        //         format_helper(
        //             text,
        //             &mut child.walk(),
        //             indentation,
        //             indent_base,
        //             extra_indent,
        //         )
        //     } else {
        //         String::new()
        //     }
        // }
        other => {
            warn!("found unknown node kind while formatting: {}", other);
            cursor
                .node()
                .utf8_text(text.as_bytes())
                .unwrap()
                .to_string()
        }
    }
}

fn separate_children_by(
    text: &String,
    node: &Node,
    seperator: &str,
    indentation: usize,
    indent_base: &str,
    settings: &FormatSettings,
) -> String {
    node.children(&mut node.walk())
        .map(|node| {
            format_parse1(
                text,
                &mut node.walk(),
                indentation,
                indent_base,
                "",
                settings,
            )
        })
        .collect::<Vec<String>>()
        .join(seperator)
}

fn align_prefixes(mut formatted_string: String, node: &Node, text: &String) -> String {
    if let Ok(query) = Query::new(
        &tree_sitter_sparql::LANGUAGE.into(),
        "(PrefixDecl (PNAME_NS) @prefix)",
    ) {
        // Step 1: Get all prefix strs and their lengths.
        // NOTE: Here a `HashSet` is used to avoid douplication of prefixes.
        //
        let mut query_cursor = QueryCursor::new();
        let mut captures = query_cursor.captures(&query, *node, text.as_bytes());
        let mut namespaces_set: HashSet<(&str, usize)> = HashSet::new();
        while let Some((mat, capture_index)) = captures.next() {
            let node = mat.captures[*capture_index].node;
            namespaces_set.insert((
                node.utf8_text(text.as_bytes()).unwrap(),
                node.end_position().column - node.start_position().column,
            ));
        }
        // Step 2: Get the length of the longest prefix.
        let max_prefix_length = namespaces_set
            .iter()
            .fold(0, |old_max, (_, length)| old_max.max(*length));
        // Step 3: Insert n spaces after each prefix, where n is the length difference to
        //         the longest prefix
        for (prefix, length) in namespaces_set {
            formatted_string = formatted_string.replace(
                &format!(" {} ", prefix),
                &format!(" {}{}", prefix, " ".repeat(max_prefix_length - length + 1)),
            );
        }
    } else {
        error!(
            "Query string to retrieve prefixes in invalid!\nIndentation of Prefixes was aborted."
        );
    }
    return formatted_string;
}
