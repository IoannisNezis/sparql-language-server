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

#[derive(Debug)]
struct CommentMarker {
    text: String,
    position: Position,
    indentation_level: usize,
    inline: bool,
}

impl CommentMarker {
    fn to_edit(&self) -> TextEdit {
        let prefix = match (
            self.position.line == 0 && self.position.character == 0,
            self.inline,
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
) -> Vec<TextEdit> {
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
    edits = consolidate_edits(edits);
    edits = merge_comments(edits, comments, &document.text);
    edits = consolidate_edits(edits);
    edits = remove_redundent_edits(edits, document);
    return edits;
}

fn merge_comments(
    edits: Vec<TextEdit>,
    comments: Vec<CommentMarker>,
    text: &String,
) -> Vec<TextEdit> {
    let mut comment_iter = comments.into_iter().rev().peekable();
    let mut merged_edits = edits
        .into_iter()
        .fold(vec![], |mut acc: Vec<TextEdit>, mut edit| {
            while comment_iter
                .peek()
                .map(|comment| comment.position > edit.range.start)
                .unwrap_or(false)
            {
                // NOTE: remove all **consecutive** whitespace edits after comment edit.
                let comment = comment_iter
                    .next()
                    .expect("comment itterator should not be empty");
                let mut comment_edit = comment.to_edit();

                // NOTE: In some Edgecase the comment is in the middle of a (consolidated)
                // edit. For Example
                // Select #comment
                // * {}
                // In this case this edits needs to be split into two edits.
                if edit.range.contains(comment.position) {
                    let (before, after) = edit.new_text.split_at(
                        // BUG: This is very questionable!
                        // What happens if the charakters not 1 utd8 m
                        (comment.position.character - edit.range.start.character) as usize,
                    );
                    acc.push(TextEdit::new(
                        Range {
                            start: comment.position,
                            end: edit.range.end,
                        },
                        after,
                    ));
                    edit.new_text = before.to_string();
                    edit.range.end = comment.position;
                }

                let mut iter = acc.iter().rev().zip(acc.iter().rev().skip(1));
                let mut prune_end = acc.len();
                let mut prune_end_index = 0;
                while let Some((next, nextnext)) = iter.next() {
                    match next.new_text.as_str() {
                        whitespace
                            if whitespace.chars().all(|c| c.is_whitespace() && c != '\n') =>
                        {
                            //NOTE: Pruning this edit.
                            //All Whitespace after a comment has to be removed.
                        }
                        x => {
                            // NOTE: found non whitespace edit.
                            // Stop pruning edits.
                            // Add linebreak if edit does not lead with a newline.
                            // WARNING: This could cause issues.
                            // The amout of chars is neccesarily equal to the amout of
                            // utf-8 bytes. Here i assume that all whispace is 1 utf8 byte long.
                            prune_end_index = next
                                .new_text
                                .chars()
                                .take_while(|c| c.is_whitespace() && c.is_ascii() && *c != '\n')
                                .count();
                            comment_edit.range.end = next.range.start.clone();
                            if x.chars().next().map(|char| char != '\n').unwrap_or(false) {
                                comment_edit.new_text +=
                                    &get_linebreak(&comment.indentation_level, "  ");
                            }
                            break;
                        }
                    };
                    prune_end -= 1;
                    if !(next.range.end == nextnext.range.start
                        || (nextnext.range.start != nextnext.range.end
                            && next.range.start == nextnext.range.start))
                    {
                        // NOTE: found non consecutive edits.
                        // Stop pruning edits.
                        // Add linebreak.
                        let indent = match text.get(
                            Range {
                                start: next.range.end,
                                end: nextnext.range.start,
                            }
                            .to_byte_index_range(text)
                            .unwrap(),
                        ) {
                            Some("}") => comment.indentation_level.checked_sub(1).unwrap_or(0),
                            _ => comment.indentation_level,
                        };
                        comment_edit.new_text += &get_linebreak(&indent, "  ");
                        comment_edit.range.end = next.range.end.clone();
                        break;
                    }
                }
                acc = acc.split_at(prune_end).0.to_vec();
                if let Some(last) = acc.last_mut() {
                    if last.range.start == comment_edit.range.end {
                        // NOTE: Merge comment into last edit.
                        last.new_text = comment_edit.new_text + &last.new_text[prune_end_index..];
                    } else {
                        // NOTE: trim leading whitspace of last edit
                        last.new_text = last.new_text[prune_end_index..].to_string();
                        acc.push(comment_edit);
                    }
                } else {
                    acc.push(comment_edit);
                }
            }
            acc.push(edit);
            return acc;
        });
    // NOTE: all remaining comments are attached to 0:0.
    comment_iter.for_each(|comment| {
        let comment_edit = comment.to_edit();
        merged_edits.push(TextEdit::new(Range::new(0, 0, 0, 0), "\n"));
        merged_edits.push(comment_edit);
    });

    return merged_edits;
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
        inline: attach.end_position().row == comment_node.start_position().row,
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
    if let Some(edits) = post_node_augmentation(node, indentation, indent_base) {
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
        "AS" => Some(" ".to_string()),
        _ => None,
    }?;
    Some(TextEdit::new(
        Range::from_ts_positions(node.start_position(), node.start_position()),
        &insert,
    ))
}

fn post_node_augmentation(node: &Node, indentation: usize, indent_base: &str) -> Option<TextEdit> {
    let insert = match node.kind() {
        "CONSTRUCT" | "UNION" => Some(" ".to_string()),
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
        "AS" => Some(" ".to_string()),
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
