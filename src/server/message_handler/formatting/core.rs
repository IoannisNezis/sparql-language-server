use core::fmt;
use lazy_static::lazy_static;
use std::{collections::HashSet, usize, vec};

use tree_sitter::{Node, Point, Tree, TreeCursor};

use crate::server::{
    configuration::FormatSettings,
    lsp::{
        errors::ResponseError,
        textdocument::{Position, Range, TextDocumentItem, TextEdit},
        FormattingOptions,
    },
};

use super::utils::KEYWORDS;

#[derive(Debug)]
struct CommentMarker {
    text: String,
    position: Position,
    indentation_level: usize,
    trailing: bool,
}

#[derive(Debug)]
struct ConsolidatedTextEdit {
    edits: Vec<TextEdit>,
}

impl fmt::Display for ConsolidatedTextEdit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "".to_string();
        for edit in &self.edits {
            s += &format!("|{}", edit);
        }
        write!(f, "{} = {}", self.fuse(), s)
    }
}

impl ConsolidatedTextEdit {
    fn fuse(&self) -> TextEdit {
        TextEdit::new(
            self.range(),
            &self
                .edits
                .iter()
                .flat_map(|edit| edit.new_text.chars())
                .collect::<String>(),
        )
    }

    fn range(&self) -> Range {
        Range {
            start: self.edits.first().unwrap().range.start,
            end: self.edits.last().unwrap().range.end,
        }
    }

    fn new(edit: TextEdit) -> Self {
        Self { edits: vec![edit] }
    }

    fn split_at(self, position: Position) -> (ConsolidatedTextEdit, ConsolidatedTextEdit) {
        let before = ConsolidatedTextEdit { edits: Vec::new() };
        let after = ConsolidatedTextEdit { edits: Vec::new() };
        self.edits
            .into_iter()
            .fold((before, after), |(mut before, mut after), edit| {
                match (edit.range.start, edit.range.end, position) {
                    (start, end, position) if start < position && position >= end => {
                        before.edits.push(edit)
                    }
                    _ => after.edits.push(edit),
                };
                (before, after)
            })
    }
}

impl CommentMarker {
    fn to_edit(&self) -> TextEdit {
        let prefix = match (
            self.position.line == 0 && self.position.character == 0,
            self.trailing,
        ) {
            (true, _) => "",
            (false, true) => " ",
            (false, false) => &get_linebreak(&self.indentation_level, "  "),
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

pub(super) fn format_document(
    document: &TextDocumentItem,
    tree: &Tree,
    options: &FormattingOptions,
    settings: &FormatSettings,
) -> Result<Vec<TextEdit>, ResponseError> {
    // TODO: Throw error dont panic!
    let indent_string = match settings.insert_spaces.unwrap_or(options.insert_spaces) {
        true => " ".repeat(settings.tab_size.unwrap_or(options.tab_size) as usize),
        false => "\t".to_string(),
    };

    let (mut edits, mut comments) = collect_format_edits(
        &document.text,
        &mut tree.walk(),
        0,
        &indent_string,
        "",
        settings,
    );
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    comments.sort_by(|a, b| a.position.cmp(&b.position));
    let consolidated_edits = consolidate_edits(edits);
    edits = merge_comments(consolidated_edits, comments, &document.text)?;
    edits = remove_redundent_edits(edits, document);
    return Ok(edits);
}

fn merge_comments(
    edits: Vec<ConsolidatedTextEdit>,
    comments: Vec<CommentMarker>,
    text: &String,
) -> Result<Vec<TextEdit>, ResponseError> {
    let mut comment_iter = comments.into_iter().rev().peekable();
    let mut merged_edits =
        edits
            .into_iter()
            .fold(vec![], |mut acc: Vec<TextEdit>, mut consolidated_edit| {
                let start_position = consolidated_edit
                    .edits
                    .first()
                    .expect("Every consolidated edit should consist of at least one edit")
                    .range
                    .start;

                while comment_iter
                    .peek()
                    .map(|comment| comment.position >= start_position)
                    .unwrap_or(false)
                {
                    let comment = comment_iter
                        .next()
                        .expect("comment itterator should not be empty");

                    // NOTE: In some Edgecase the comment is in the middle of a (consolidated)
                    // edit. For Example
                    // Select #comment
                    // * {}
                    // In this case this edits needs to be split into two edits.
                    let (mut previous_edit, mut next_edit) =
                        match consolidated_edit.split_at(comment.position) {
                            (previous_edit, next_edit) => (previous_edit, next_edit.fuse()),
                        };
                    // WARNING: This could cause issues.
                    // The amout of chars is neccesarily equal to the amout of
                    // utf-8 bytes. Here i assume that all whispace is 1 utf8 byte long.
                    match next_edit
                        .new_text
                        .chars()
                        .enumerate()
                        .find_map(|(idx, char)| {
                            (!char.is_whitespace() || char == '\n').then_some((idx, char))
                        }) {
                        Some((idx, '\n')) => {
                            next_edit.new_text = format!(
                                "{}{}",
                                comment.to_edit().new_text,
                                &next_edit.new_text[idx..]
                            )
                        }
                        Some((idx, _char)) => {
                            next_edit.new_text = format!(
                                "{}{}{}",
                                comment.to_edit().new_text,
                                get_linebreak(&comment.indentation_level, "  "),
                                &next_edit.new_text[idx..]
                            )
                        }
                        None => {
                            let indent = match next_edit.range.end.to_byte_index(&text) {
                                Some(start_next_token) => {
                                    match text.get(start_next_token..start_next_token + 1) {
                                        Some("}") => {
                                            comment.indentation_level.checked_sub(1).unwrap_or(0)
                                        }
                                        _ => comment.indentation_level,
                                    }
                                }
                                None => comment.indentation_level,
                            };
                            next_edit.new_text = format!(
                                "{}{}",
                                comment.to_edit().new_text,
                                get_linebreak(&indent, "  "),
                            )
                        }
                    };
                    previous_edit.edits.push(next_edit);
                    consolidated_edit = previous_edit;
                }

                acc.push(consolidated_edit.fuse());
                return acc;
            });
    // NOTE: all remaining comments are attached to 0:0.
    comment_iter.for_each(|comment| {
        let comment_edit = comment.to_edit();
        merged_edits.push(TextEdit::new(Range::new(0, 0, 0, 0), "\n"));
        merged_edits.push(comment_edit);
    });

    return Ok(merged_edits);
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

fn consolidate_edits(edits: Vec<TextEdit>) -> Vec<ConsolidatedTextEdit> {
    let accumulator: Vec<ConsolidatedTextEdit> = Vec::new();
    edits.into_iter().fold(accumulator, |mut acc, edit| {
        if edit.is_empty() {
            return acc;
        }
        match acc.last_mut() {
            Some(next_consolidated) => match next_consolidated.edits.first_mut() {
                Some(next_edit) if next_edit.range.start == edit.range.end => {
                    next_consolidated.edits.insert(0, edit);
                }
                Some(next_edit)
                    if next_edit.range.start == next_edit.range.end
                        && next_edit.range.start == edit.range.start =>
                {
                    next_edit.new_text.push_str(&edit.new_text);
                    next_edit.range.end = edit.range.end;
                }
                Some(_next_edit) => {
                    acc.push(ConsolidatedTextEdit::new(edit));
                }
                None => {
                    next_consolidated.edits.push(edit);
                }
            },
            None => {
                acc.push(ConsolidatedTextEdit::new(edit));
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
            .fold((vec![], vec![]), |mut acc, child| {
                match child.kind() {
                    "comment" if node.kind() != "ERROR" => {
                        acc.1.push(comment_marker(&child, text, indentation))
                    }
                    _ => acc.0.push(child),
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
                Seperator::Empty if node2.kind() == "ERROR" => {
                    Some(TextEdit::new(Range::between(node1, node2), " "))
                }
                Seperator::Empty => Some(TextEdit::new(Range::between(node1, node2), "")),
                Seperator::Unknown => None,
            });

    // NOTE: Step 2: Augmentation
    let augmentation_edits =
        node_augmentation(&node, &children, indentation, indent_base, settings);

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
        .chain(augmentation_edits)
        .chain(recursive_edits)
        .collect();

    return (edits, comments);
}

fn comment_marker(comment_node: &Node, text: &String, indentation: usize) -> CommentMarker {
    assert_eq!(comment_node.kind(), "comment");
    let mut maybe_attach = Some(*comment_node);
    while let Some(kind) = maybe_attach.map(|node| node.kind()) {
        match kind {
            "comment" => maybe_attach = maybe_attach.map(|node| node.prev_sibling()).flatten(),
            _ => break,
        }
    }
    let attach = maybe_attach
        .or(comment_node.parent())
        .expect("all comment nodes should have a parent");
    CommentMarker {
        text: comment_node
            .utf8_text(text.as_bytes())
            .expect("TSNode range should have a valid utf8 string")
            .to_string(),
        position: match attach.kind() {
            "unit" => Position::new(0, 0),
            _ => Position::from_point(attach.end_position()),
        },
        trailing: attach.end_position().row == comment_node.start_position().row,
        indentation_level: indentation,
    }
}

fn node_augmentation(
    node: &Node,
    children: &Vec<Node>,
    indentation: usize,
    indent_base: &str,
    settings: &FormatSettings,
) -> Vec<TextEdit> {
    let mut augmentations =
        in_node_augmentation(node, children, indentation, indent_base, settings);

    if let Some(edits) = pre_node_augmentation(node, indentation, indent_base, settings) {
        augmentations.push(edits);
    }
    if let Some(edits) = post_node_augmentation(node, indentation, indent_base, settings) {
        augmentations.push(edits);
    }

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
            _ => vec![TextEdit::new(
                Range::from_ts_positions(Point { row: 0, column: 0 }, node.end_position()),
                "",
            )],
        },
        "Prologue" if settings.align_prefixes => {
            let prefix_pos_and_length: Vec<(Point, usize)> = children
                .iter()
                .filter_map(|child| match (child.kind(), child.child(1)) {
                    ("PrefixDecl", Some(grandchild)) if grandchild.kind() == "PNAME_NS" => Some((
                        grandchild.end_position(),
                        grandchild.end_position().column - grandchild.start_position().column,
                    )),
                    _ => None,
                })
                .collect();
            let max_length = prefix_pos_and_length
                .iter()
                .map(|(_pos, len)| *len)
                .max()
                .unwrap_or(0);
            prefix_pos_and_length
                .iter()
                .map(|(position, length)| {
                    TextEdit::new(
                        Range::from_ts_positions(*position, *position),
                        &" ".repeat(max_length - length),
                    )
                })
                .collect()
        }
        "PropertyListPathNotEmpty" => match node.parent() {
            Some(parent) => match parent.kind() {
                "BlankNodePropertyListPath" | "TriplesSameSubjectPath" => children
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, child)| match child.kind() {
                        ";" if idx < children.len() - 1 => {
                            let linebreak = get_linebreak(&indentation, indent_base);
                            Some(TextEdit::new(
                                Range::from_ts_positions(
                                    child.end_position(),
                                    child.end_position(),
                                ),
                                &linebreak[..linebreak.len() - 1],
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
                    if prop_list.kind() == "PropertyListPathNotEmpty" =>
                {
                    let insert = match settings.align_predicates {
                        true => &format!("{}", " ".repeat(get_column_width(subject) + 1)),
                        false => "  ",
                    };
                    let mut cursor = prop_list.walk();
                    prop_list
                        .children(&mut cursor)
                        .filter(|child| child.kind() != "comment")
                        .step_by(3)
                        .skip(1)
                        .map(|child| {
                            TextEdit::new(
                                Range::from_ts_positions(
                                    child.start_position(),
                                    child.start_position(),
                                ),
                                insert,
                            )
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
                "." => {
                    let mut edits = vec![TextEdit::new(
                        Range::from_ts_positions(child.start_position(), child.start_position()),
                        " ",
                    )];
                    if idx < children.len() - 1 {
                        edits.push(TextEdit::new(
                            Range::from_ts_positions(child.end_position(), child.end_position()),
                            &get_linebreak(&indentation, indent_base),
                        ));
                    }

                    return Some(edits);
                }
                _ => None,
            })
            .flatten()
            .collect(),
        "assignment" => children
            .iter()
            .filter_map(|child| match child.kind() {
                "AS" => Some(vec![
                    TextEdit::new(
                        Range::from_ts_positions(child.end_position(), child.end_position()),
                        " ",
                    ),
                    TextEdit::new(
                        Range::from_ts_positions(child.start_position(), child.start_position()),
                        " ",
                    ),
                ]),
                _ => None,
            })
            .flatten()
            .collect(),
        "ExpressionList" | "ObjectList" => children
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
        "Aggregate" => children
            .iter()
            .filter_map(|child| match child.kind() {
                ";" => Some(TextEdit::new(
                    Range::from_ts_positions(child.end_position(), child.end_position()),
                    " ",
                )),
                _ => None,
            })
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
) -> Option<TextEdit> {
    let insert = match node.kind() {
        "GroupGraphPatternSub"
        | "ConstructTriples"
        | "SolutionModifier"
        | "Quads"
        | "DatasetClause"
        | "UNION" => Some(get_linebreak(&indentation, indent_base)),

        "Filter" => match node.prev_sibling() {
            Some(prev)
                if prev.kind() == "TriplesBlock"
                    && prev.end_position().row == node.start_position().row
                    && settings.filter_same_line =>
            {
                Some(" ".to_string())
            }
            Some(_prev) => Some(get_linebreak(&indentation, indent_base)),
            None => None,
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
            Some(get_linebreak(&indentation, indent_base))
        }
        "PropertyListPathNotEmpty" => match node.parent().map(|parent| parent.kind()) {
            Some("BlankNodePropertyListPath") if node.child_count() > 3 => {
                Some(get_linebreak(&indentation, indent_base))
            }
            Some("BlankNodePropertyListPath") if node.child_count() <= 3 => Some(" ".to_string()),
            _ => None,
        },
        "TriplesTemplate" | "TriplesBlock" => match node.prev_sibling().map(|parent| parent.kind())
        {
            // Some("{") => &get_linebreak(&indentation, indent_base),
            Some(x) if x != "." => Some(get_linebreak(&indentation, indent_base)),
            _ => None,
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
                true => Some(get_linebreak(&indentation, indent_base)),
                false => Some(" ".to_string()),
            }
        }
        _ => None,
    }?;
    Some(TextEdit::new(
        Range::from_ts_positions(node.start_position(), node.start_position()),
        &insert,
    ))
}

fn post_node_augmentation(
    node: &Node,
    indentation: usize,
    indent_base: &str,
    settings: &FormatSettings,
) -> Option<TextEdit> {
    let insert = match node.kind() {
        "CONSTRUCT" | "UNION" => Some(" ".to_string()),
        "Prologue" if settings.separate_prolouge && node.next_sibling().is_some() => {
            Some(get_linebreak(&indentation, indent_base))
        }
        "PropertyListPathNotEmpty" => match node.parent().map(|parent| parent.kind()) {
            Some("BlankNodePropertyListPath") if node.child_count() > 3 => Some(get_linebreak(
                &indentation.checked_sub(1).unwrap_or(0),
                indent_base,
            )),
            Some("BlankNodePropertyListPath") if node.child_count() <= 3 => Some(" ".to_string()),
            _ => None,
        },
        "TriplesTemplate" => match node.parent().map(|parent| parent.kind()) {
            Some("TriplesTemplateBlock") => Some(get_linebreak(
                &indentation.checked_sub(1).unwrap_or(0),
                indent_base,
            )),
            _ => None,
        },
        "GroupGraphPatternSub" | "ConstructTriples" | "Quads" => Some(get_linebreak(
            &indentation.checked_sub(1).unwrap_or(0),
            indent_base,
        )),
        _ => None,
    }?;
    Some(TextEdit::new(
        Range::from_ts_positions(node.end_position(), node.end_position()),
        &insert,
    ))
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
    if KEYWORDS.contains(&kind) {
        return Seperator::Unknown;
    }
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

        "ERROR" => Seperator::Unknown,
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
