use lazy_static::lazy_static;
use std::{collections::HashSet, usize, vec};

use tree_sitter::{Node, Point, Tree, TreeCursor};

use crate::server::{
    configuration::FormatSettings,
    lsp::{
        textdocument::{Position, Range, TextDocumentItem, TextEdit},
        FormattingOptions,
    },
};

use super::utils::KEYWORDS;

pub(super) fn format_document(
    document: &TextDocumentItem,
    tree: &Tree,
    options: &FormattingOptions,
    settings: &FormatSettings,
) -> Vec<TextEdit> {
    // TODO: Throw error dont panic!
    let indent_string = match settings.insert_spaces.unwrap_or(options.insert_spaces) {
        true => " ".repeat(settings.tab_size.unwrap_or(options.tab_size) as usize),
        false => "\t".to_string(),
    };
    let (mut edits, comments) = collect_format_edits(
        &document.text,
        &mut tree.walk(),
        0,
        &indent_string,
        "",
        settings,
    );
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    // log::info!("-------------Raw edits  ------------");
    // for x in &edits {
    //     log::info!("{}", x);
    // }

    log::info!("{}", edits.len());
    edits = insert_comments(edits, comments);
    edits = consolidate_edits(edits);
    edits = remove_redundent_edits(edits, document);
    return edits;
}

fn insert_comments(edits: Vec<TextEdit>, comments: Vec<CommentMarker>) -> Vec<TextEdit> {
    let mut comment_iter = comments.into_iter().peekable();
    let mut drop_whitespace = false;
    edits.into_iter().rev().fold(vec![], |mut acc, edit| {
        while comment_iter
            .peek()
            .map(|comment| comment.position <= edit.range.start)
            .unwrap_or(false)
        {
            log::info!("{:?}", edit);
            log::info!(
                "{:?} <- {}",
                acc.last(),
                comment_iter.peek().unwrap().position
            );
            acc.push(
                comment_iter
                    .next()
                    .expect("comment itterator should not be empty")
                    .to_edit(),
            );
            drop_whitespace = true;
        }
        acc.push(edit);
        return acc;
    })
}

struct CommentMarker {
    text: String,
    position: Position,
    indentation_level: usize,
    inline: bool,
}

impl CommentMarker {
    fn to_edit(&self) -> TextEdit {
        let prefix = match self.inline {
            true => " ",
            false => &get_linebreak(&self.indentation_level, "  "),
        };
        TextEdit::new(
            Range::new(
                self.position.line,
                self.position.character,
                self.position.line,
                self.position.character,
            ),
            &format!("{}{}", prefix, &self.text),
        )
    }
}

lazy_static! {
    static ref BRACKETS_OPEN: HashSet<&'static str> = HashSet::from(["[", "(", "{"]);
    static ref BRACKETS_CLOSE: HashSet<&'static str> = HashSet::from(["]", ")", "}"]);
    static ref EXIT_EARLY: HashSet<&'static str> =
        HashSet::from(["STRING_LITERAL", "PN_LOCAL", ":"]);
    static ref INC_INDENTATION: HashSet<&'static str> = HashSet::from([
        "BlankNodePropertyListPath",
        "GroupGraphPattern",
        "TriplesTemplateBlock",
        "BrackettedExpression",
        "ConstructTemplate",
        "QuadData",
    ]);
}
pub(self) fn collect_format_edits(
    text: &String,
    cursor: &mut TreeCursor,
    indentation: usize,
    indent_base: &str,
    extra_indent: &str,
    settings: &FormatSettings,
) -> (Vec<TextEdit>, Vec<CommentMarker>) {
    let node = cursor.node();

    // NOTE: Early exit
    if EXIT_EARLY.contains(node.kind()) {
        return (vec![], vec![]);
    }

    // NOTE: Extract comments
    let (children, mut comments): (Vec<Node>, Vec<CommentMarker>) =
        node.children(cursor)
            .fold((vec![], vec![]), |mut acc, node| {
                match node.kind() {
                    "comment" => acc.1.push(comment_marker(&node, text, indentation)),
                    _ => acc.0.push(node),
                };
                return acc;
            });

    // NOTE: Step 1: Separation
    let separator = get_separator(node.kind());
    let seperation_edits =
        children
            .iter()
            .zip(children.iter().skip(1))
            .filter_map(move |(node1, node2)| match separator {
                Seperator::LineBreak => Some(TextEdit::new(
                    Range::between(node1, node2),
                    &get_linebreak(&indentation, indent_base),
                )),
                Seperator::Space => Some(TextEdit::new(Range::between(node1, node2), " ")),
                Seperator::Empty => Some(TextEdit::new(Range::between(node1, node2), "")),
                Seperator::Unknown => None,
            });

    // NOTE: Step 2: Augmentation
    let augmentation_edits =
        node_augmentation(&node, &children, indentation, indent_base, settings, text);

    // NOTE: Step 3: Recursion
    let recursive_edits = children.iter().flat_map(|node| {
        let (edits, mut x) = collect_format_edits(
            text,
            &mut node.walk(),
            match INC_INDENTATION.contains(node.kind()) {
                true => indentation + 1,
                false => indentation,
            },
            indent_base,
            extra_indent,
            settings,
        );
        comments.append(&mut x);
        return edits;
    });

    let edits = seperation_edits
        .into_iter()
        .chain(recursive_edits)
        .chain(augmentation_edits)
        .collect();

    return (edits, comments);
}

fn comment_marker(comment_node: &Node, text: &String, indentation: usize) -> CommentMarker {
    assert_eq!(comment_node.kind(), "comment");
    let attach = comment_node
        .prev_sibling()
        .or(comment_node.parent())
        .expect("all comment nodes should have a parent");
    CommentMarker {
        text: comment_node
            .utf8_text(text.as_bytes())
            .expect("TSNode range should have a valid utf8 string")
            .to_string(),
        position: Position::from_point(attach.end_position()),
        inline: attach.end_position().row == comment_node.start_position().row,
        indentation_level: indentation,
    }
}

fn remove_redundent_edits(edits: Vec<TextEdit>, document: &TextDocumentItem) -> Vec<TextEdit> {
    edits
        .into_iter()
        .filter(|edit| {
            if let Some(slice) = document.get_range(&edit.range) {
                if edit.new_text == slice {
                    return false;
                }
            }
            return true;
        })
        .collect()
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
            Some(prev)
                if prev.range.start == prev.range.end && prev.range.start == edit.range.start =>
            {
                prev.new_text.push_str(&edit.new_text);
                prev.range.end = edit.range.end;
            }
            _ => {
                acc.push(edit);
            }
        };
        acc
    })
}

fn node_augmentation(
    node: &Node,
    children: &Vec<Node>,
    indentation: usize,
    indent_base: &str,
    settings: &FormatSettings,
    text: &String,
) -> Vec<TextEdit> {
    let mut augmentations =
        in_node_augmentation(node, children, indentation, indent_base, settings, text);
    augmentations.push(pre_node_augmentation(
        node,
        indentation,
        indent_base,
        settings,
    ));
    augmentations.push(post_node_augmentation(node, indentation, indent_base));

    // NOTE: Capitalize keywords
    if KEYWORDS.contains(&node.kind()) && settings.capitalize_keywords {
        augmentations.push(TextEdit::new(Range::from_node(&node), node.kind()));
    }
    return augmentations;
}

fn in_node_augmentation(
    node: &Node,
    children: &Vec<Node>,
    indentation: usize,
    indent_base: &str,
    settings: &FormatSettings,
    text: &String,
) -> Vec<TextEdit> {
    match node.kind() {
        "unit" => match (children.first(), children.last()) {
            (Some(first), Some(last)) => vec![
                TextEdit::new(
                    Range::from_ts_positions(Point { row: 0, column: 0 }, first.start_position()),
                    "",
                ),
                TextEdit::new(
                    Range::from_ts_positions(last.end_position(), node.end_position()),
                    "\n",
                ),
            ],
            _ => vec![],
        },
        "ERROR" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "comment" => Some(TextEdit::new(Range::from_node(child), "")),
                _ => None,
            })
            .collect(),
        "comment" => {
            let mut edits = Vec::new();
            if let Some(prev) = node.prev_sibling() {
                let insert = match prev.end_position().row == node.start_position().row {
                    true => " ",
                    false => &get_linebreak(&indentation, indent_base),
                };
                edits.append(&mut vec![
                    TextEdit::new(Range::from_node(node), ""),
                    TextEdit::new(
                        Range::from_ts_positions(prev.end_position(), prev.end_position()),
                        &format!("{}{}", insert, node.utf8_text(text.as_bytes()).unwrap()),
                    ),
                ]);
            }
            if let Some(next) = node.next_sibling() {
                edits.append(&mut vec![
                    TextEdit::new(Range::from_node(node), ""),
                    TextEdit::new(
                        Range::from_ts_positions(next.start_position(), next.start_position()),
                        &format!("{}", get_linebreak(&indentation, indent_base)),
                    ),
                ]);
            }
            return edits;
        }
        "PropertyListPathNotEmpty" => match node.parent() {
            Some(parent) => match parent.kind() {
                "BlankNodePropertyListPath" | "TriplesSameSubjectPath" => children
                    .iter()
                    .filter_map(|child| match child.kind() {
                        ";" => {
                            let linebreak = get_linebreak(&indentation, indent_base);
                            Some(TextEdit::new(
                                Range::from_node(&child),
                                &format!(";{}", &linebreak[..linebreak.len() - 1]),
                            ))
                        }
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            },
            None => vec![],
        },
        "TriplesSameSubjectPath" => {
            let subject = children.first();
            let prop_list = children.last();
            match (subject, prop_list) {
                (Some(subject), Some(prop_list))
                    if subject.kind() == "VAR"
                        && prop_list.kind() == "PropertyListPathNotEmpty"
                        && prop_list.child_count() > 3 =>
                {
                    let insert = match settings.align_predicates {
                        true => &format!("{}", " ".repeat(get_column_width(subject) + 1)),
                        false => "  ",
                    };
                    let mut cursor = prop_list.walk();
                    prop_list
                        .children(&mut cursor)
                        .skip(1)
                        .filter_map(|child| match child.kind() {
                            "Path" => Some(TextEdit::new(
                                Range::from_ts_positions(
                                    child.start_position(),
                                    child.start_position(),
                                ),
                                insert,
                            )),
                            _ => None,
                        })
                        .collect()
                }
                _ => vec![],
            }
        }
        "TriplesBlock"
        | "TriplesTemplate"
        | "ConstructTriples"
        | "Quads"
        | "GroupGraphPatternSub" => children
            .iter()
            .enumerate()
            .filter_map(|(idx, child)| match child.kind() {
                "." => match idx {
                    x if x < children.len() - 1 => Some(TextEdit::new(
                        Range::from_node(&child),
                        &format!(" .{}", get_linebreak(&indentation, indent_base)),
                    )),
                    _ => Some(TextEdit::new(Range::from_node(&child), " .")),
                },
                _ => None,
            })
            .collect(),
        "ExpressionList" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "," => Some(TextEdit::new(Range::from_node(child), ", ")),
                _ => None,
            })
            .collect(),
        "DescribeQuery" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "DESCRIBE" | "WhereClause" => None,
                _ => Some(TextEdit::new(
                    Range::from_ts_positions(child.start_position(), child.start_position()),
                    " ",
                )),
            })
            .collect(),
        "Modify" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "WITH" => None,
                "IRIREF" => Some(vec![TextEdit::new(
                    Range::from_ts_positions(child.start_position(), child.start_position()),
                    " ",
                )]),
                "DeleteClause" | "InsertClause" | "UsingClause" => Some(vec![TextEdit::new(
                    Range::from_ts_positions(child.start_position(), child.start_position()),
                    &get_linebreak(&indentation, indent_base),
                )]),
                "WHERE" => Some(vec![
                    TextEdit::new(
                        Range::from_ts_positions(child.start_position(), child.start_position()),
                        &get_linebreak(&indentation, indent_base),
                    ),
                    TextEdit::new(
                        Range::from_ts_positions(child.end_position(), child.end_position()),
                        " ",
                    ),
                ]),
                _ => None,
            })
            .flatten()
            .collect(),
        "Update" => children
            .iter()
            .filter_map(|child| match child.kind() {
                ";" => Some(TextEdit::new(
                    Range::from_ts_positions(child.start_position(), child.start_position()),
                    " ",
                )),
                _ => None,
            })
            .collect(),
        "ANON" => vec![TextEdit::new(Range::from_node(node), "[]")],
        _ => vec![],
    }
}

fn pre_node_augmentation(
    node: &Node,
    indentation: usize,
    indent_base: &str,
    settings: &FormatSettings,
) -> TextEdit {
    let insert = match node.kind() {
        "GroupGraphPatternSub"
        | "ConstructTriples"
        | "SolutionModifier"
        | "Quads"
        | "DatasetClause"
        | "UNION" => &get_linebreak(&indentation, indent_base),

        "Filter" => match node.prev_sibling() {
            Some(prev)
                if prev.kind() == "TriplesBlock"
                    && prev.end_position().row == node.start_position().row =>
            {
                " "
            }
            Some(_prev) => &get_linebreak(&indentation, indent_base),
            None => "",
        },
        "QuadsNotTriples"
        | "GroupOrUnionGraphPattern"
        | "OptionalGraphPattern"
        | "MinusGraphPattern"
        | "GraphGraphPattern"
        | "ServiceGraphPattern"
        | "Bind"
        | "InlineData"
        | "Update"
        | "Update1"
            if node.prev_sibling().is_some() =>
        {
            &get_linebreak(&indentation, indent_base)
        }
        "PropertyListPathNotEmpty" => match node.parent().map(|parent| parent.kind()) {
            Some("BlankNodePropertyListPath") if node.child_count() > 3 => {
                &get_linebreak(&indentation, indent_base)
            }
            Some("BlankNodePropertyListPath") if node.child_count() <= 3 => " ",
            _ => "",
        },
        "TriplesTemplate" | "TriplesBlock" => match node.prev_sibling().map(|parent| parent.kind())
        {
            // Some("{") => &get_linebreak(&indentation, indent_base),
            Some(x) if x != "." => &get_linebreak(&indentation, indent_base),
            _ => "",
        },
        "WhereClause" => {
            match settings.where_new_line
                || node
                    .parent()
                    .map(|parent| parent.kind() == "ConstructQuery")
                    .unwrap_or(false)
                || node
                    .parent()
                    .map(|parent| parent.kind() == "DescribeQuery")
                    .unwrap_or(false)
                || node
                    .prev_sibling()
                    .map(|sibling| sibling.kind() == "DatasetClause")
                    .unwrap_or(false)
            {
                true => &get_linebreak(&indentation, indent_base),
                false => " ",
            }
        }
        "AS" => " ",
        _ => "",
    };
    TextEdit::new(
        Range::from_ts_positions(node.start_position(), node.start_position()),
        insert,
    )
}

fn post_node_augmentation(node: &Node, indentation: usize, indent_base: &str) -> TextEdit {
    let insert = match node.kind() {
        "CONSTRUCT" | "UNION" => " ",
        "PropertyListPathNotEmpty" => match node.parent().map(|parent| parent.kind()) {
            Some("BlankNodePropertyListPath") if node.child_count() > 3 => {
                &get_linebreak(&indentation.checked_sub(1).unwrap_or(0), indent_base)
            }
            Some("BlankNodePropertyListPath") if node.child_count() <= 3 => " ",
            _ => "",
        },
        "TriplesTemplate" => match node.parent().map(|parent| parent.kind()) {
            Some("TriplesTemplateBlock") => {
                &get_linebreak(&indentation.checked_sub(1).unwrap_or(0), indent_base)
            }
            _ => "",
        },
        "GroupGraphPatternSub" | "ConstructTriples" | "Quads" => {
            &get_linebreak(&indentation.checked_sub(1).unwrap_or(0), indent_base)
        }
        "AS" => " ",
        _ => "",
    };
    TextEdit::new(
        Range::from_ts_positions(node.end_position(), node.end_position()),
        insert,
    )
}

fn get_linebreak(indentation: &usize, indent_base: &str) -> String {
    format!("\n{}", indent_base.repeat(*indentation))
}

enum Seperator {
    LineBreak,
    Space,
    Empty,
    Unknown,
}

fn get_separator(kind: &str) -> Seperator {
    match kind {
        "unit" | "Prologue" | "SolutionModifier" | "LimitOffsetClauses" => Seperator::LineBreak,

        "ExpressionList"
        | "GroupGraphPattern"
        | "GroupGraphPatternSub"
        | "GroupOrUnionGraphPattern"
        | "TriplesTemplateBlock"
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
        | "Quads"
        | "ConstructTriples"
        | "ConstructQuery"
        | "SelectQuery"
        | "SubSelect"
        | "AskQuery"
        | "assignment"
        | "DescribeQuery"
        | "Modify"
        | "Update"
        | "Update1" => Seperator::Empty,

        "BaseDecl"
        | "SelectClause"
        | "PropertyListPathNotEmpty"
        | "PrefixDecl"
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
        | "TriplesSameSubjectPath"
        | "QuadsNotTriples" => Seperator::Space,

        "SELECT" | "WHERE" | "PNAME_NS" | "IRIREF" | "VAR" | "INTEGER" | "DECIMAL" | "String"
        | "NIL" | "BLANK_NODE_LABEL" | "RdfLiteral" | "PrefixedName" | "PathMod" | "(" | ")"
        | "{" | "}" | "." | "," | ";" | "*" | "+" | "-" | "/" | "<" | ">" | "=" | ">=" | "<="
        | "!=" | "||" | "&&" | "|" | "^" | "[" | "]" => Seperator::Empty,

        _ => {
            log::warn!("unknown node: {}", kind);
            Seperator::Unknown
        }
    }
}

fn get_column_width(node: &Node) -> usize {
    let range = node.range();
    range
        .end_point
        .column
        .checked_sub(range.start_point.column)
        .unwrap_or(0)
}
