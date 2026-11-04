//! Turning markdown into styled lines for the terminal.

use crate::temoc::{self, TestFile};
use markdown::ParseOptions;
use markdown::mdast::{AlignKind, List, Node, Table};
use markdown::unist::Position;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;
use std::mem;
use std::ops::Range;

/// Wide enough that a preview is never wrapped.
const PREVIEW_WIDTH: usize = 200;

/// How a piece of text is emphasized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Plain,
    Heading(u8),
    Bold,
    Italic,
    Struck,
    Code,
    /// Part of the link starting at this offset of the markdown.
    Link(usize),
    Footnote,
    Quote,
}

impl Look {
    fn name(self) -> String {
        match self {
            Self::Plain => "plain".into(),
            Self::Heading(level) => format!("heading {level}"),
            Self::Bold => "bold".into(),
            Self::Italic => "italic".into(),
            Self::Struck => "struck".into(),
            Self::Code => "code".into(),
            Self::Link(_) | Self::Footnote => "link".into(),
            Self::Quote => "quote".into(),
        }
    }

    fn style(self) -> Style {
        let style = Style::new();
        match self {
            Self::Plain => style,
            Self::Heading(1) => style
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            Self::Heading(_) => style.fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Self::Bold => style.add_modifier(Modifier::BOLD),
            Self::Italic => style.add_modifier(Modifier::ITALIC),
            Self::Struck => style.add_modifier(Modifier::CROSSED_OUT),
            Self::Code => style.fg(Color::Yellow),
            Self::Link(_) | Self::Footnote => {
                style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED)
            }
            Self::Quote => style.fg(Color::Gray),
        }
    }

    /// The look of `emphasis` inside this look. Links and headings stay as they
    /// are, so that they can still be recognized.
    fn with(self, emphasis: Self) -> Self {
        match self {
            Self::Link(_) | Self::Heading(_) => self,
            _ => emphasis,
        }
    }
}

/// Text with a single look.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Piece {
    text: String,
    look: Look,
}

impl Piece {
    fn new(text: impl Into<String>, look: Look) -> Self {
        Self {
            text: text.into(),
            look,
        }
    }
}

/// One line on screen.
type Row = Vec<Piece>;

/// How a piece of markdown appears on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// The text drawn.
    pub text: String,
    /// How the start of the text is emphasized, such as `heading 1` or `plain`.
    pub style: String,
}

/// Renders a piece of markdown by itself. Returns `None` if nothing is drawn.
///
/// # Errors
///
/// Fails when the markdown cannot be parsed.
pub fn preview(markdown: &str) -> Result<Option<Preview>, String> {
    let rows = Document::parse(markdown)?.rows(PREVIEW_WIDTH);
    Ok(rows.first().and_then(|row| {
        let style = row.first()?.look.name();
        let text = row.iter().map(|piece| piece.text.as_str()).collect();
        Some(Preview { text, style })
    }))
}

/// A heading of a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// The heading as plain text.
    pub text: String,
    /// What a link uses to point at this heading, as on GitHub.
    pub anchor: String,
    /// The line of the markdown it starts on.
    pub line: usize,
}

/// A link on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    id: usize,
    /// The text shown.
    pub text: String,
    /// Where the link points, as written in the markdown.
    pub target: String,
    /// The rows of the screen it takes up.
    pub rows: Range<usize>,
}

/// A parsed markdown document.
#[derive(Debug, Clone)]
pub struct Document {
    root: Node,
    targets: HashMap<usize, String>,
}

impl Document {
    /// Parses markdown the same way temoc does.
    ///
    /// # Errors
    ///
    /// Fails when the markdown cannot be parsed.
    pub fn parse(markdown: &str) -> Result<Self, String> {
        markdown::to_mdast(markdown, &ParseOptions::gfm())
            .map(|root| Self {
                targets: link_targets(&root),
                root,
            })
            .map_err(|message| message.to_string())
    }

    /// What temoc would find in this document.
    #[must_use]
    pub fn tests(&self) -> TestFile {
        temoc::tests_in(&self.root)
    }

    /// The top-level headings, in document order.
    #[must_use]
    pub fn headings(&self) -> Vec<Heading> {
        let mut seen: HashMap<String, usize> = HashMap::new();
        children(&self.root)
            .iter()
            .filter(|node| matches!(node, Node::Heading(_)))
            .map(|node| {
                let text = node.to_string();
                let slug = slug(&text);
                let count = seen.entry(slug.clone()).or_default();
                let anchor = match *count {
                    0 => slug,
                    count => format!("{slug}-{count}"),
                };
                *count += 1;
                Heading {
                    text,
                    anchor,
                    line: line_of(node),
                }
            })
            .collect()
    }

    /// Lays the document out for a screen of this width.
    #[must_use]
    pub fn render(&self, width: u16) -> Rendered {
        let nodes = children(&self.root);
        let (rows, ranges) = placed_blocks(nodes, usize::from(width), Look::Plain, true);
        Rendered {
            links: links_in(&rows, &self.targets),
            blocks: nodes.iter().map(line_of).zip(ranges).collect(),
            rows,
        }
    }

    fn rows(&self, width: usize) -> Vec<Row> {
        blocks(children(&self.root), width, Look::Plain, true)
    }
}

/// A document laid out for the screen.
#[derive(Debug, Clone)]
pub struct Rendered {
    rows: Vec<Row>,
    /// The rows of each top-level element, by the line of the markdown it
    /// starts on.
    blocks: HashMap<usize, Range<usize>>,
    links: Vec<Link>,
}

impl Rendered {
    /// How many rows the document takes up.
    #[must_use]
    pub fn height(&self) -> usize {
        self.rows.len()
    }

    /// The links, in the order they appear.
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// The rows of the top-level element starting on this line of the markdown.
    #[must_use]
    pub fn rows_of_line(&self, line: usize) -> Option<Range<usize>> {
        self.blocks.get(&line).cloned()
    }

    /// The rows from `top` on that fit in `height`, with a link highlighted.
    #[must_use]
    pub fn lines(&self, top: usize, height: usize, focused: Option<&Link>) -> Vec<Line<'static>> {
        let focused = focused.map(|link| Look::Link(link.id));
        self.rows
            .iter()
            .skip(top)
            .take(height)
            .map(|row| {
                row.iter()
                    .map(|piece| {
                        let style = piece.look.style();
                        let style = if Some(piece.look) == focused {
                            style.add_modifier(Modifier::REVERSED)
                        } else {
                            style
                        };
                        Span::styled(piece.text.clone(), style)
                    })
                    .collect()
            })
            .collect()
    }
}

/// Where each link points, by the offset of the link in the markdown. Links
/// can use a definition written anywhere in the document.
fn link_targets(root: &Node) -> HashMap<usize, String> {
    let mut definitions = HashMap::new();
    walk(root, &mut |node| {
        if let Node::Definition(definition) = node {
            definitions
                .entry(definition.identifier.as_str())
                .or_insert(definition.url.as_str());
        }
    });
    let mut targets = HashMap::new();
    walk(root, &mut |node| {
        let target = match node {
            Node::Link(link) => Some((link_id(link.position.as_ref()), link.url.as_str())),
            Node::LinkReference(link) => definitions
                .get(link.identifier.as_str())
                .map(|url| (link_id(link.position.as_ref()), *url)),
            _ => None,
        };
        if let Some((id, url)) = target {
            targets.insert(id, url.to_owned());
        }
    });
    targets
}

fn walk<'node>(node: &'node Node, visit: &mut impl FnMut(&'node Node)) {
    visit(node);
    for child in children(node) {
        walk(child, visit);
    }
}

fn link_id(position: Option<&Position>) -> usize {
    position.map_or(0, |position| position.start.offset)
}

/// Finds the links in laid out rows. The pieces of a link always come one after
/// another, even when it wraps onto a new row.
fn links_in(rows: &[Row], targets: &HashMap<usize, String>) -> Vec<Link> {
    let mut links: Vec<Link> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        for piece in row {
            let Look::Link(id) = piece.look else {
                continue;
            };
            match links.last_mut() {
                Some(link) if link.id == id => {
                    if link.rows.end == index {
                        link.text.push(' ');
                    }
                    link.text.push_str(&piece.text);
                    link.rows.end = index + 1;
                }
                _ => {
                    if let Some(target) = targets.get(&id) {
                        links.push(Link {
                            id,
                            text: piece.text.clone(),
                            target: target.clone(),
                            rows: index..index + 1,
                        });
                    }
                }
            }
        }
    }
    links
}

fn line_of(node: &Node) -> usize {
    node.position().map_or(0, |position| position.start.line)
}

/// The anchor GitHub gives a heading with this text.
fn slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            c if c.is_alphanumeric() || c == '-' || c == '_' => Some(c),
            _ => None,
        })
        .collect()
}

/// Lays out block elements, skipping those that draw nothing.
fn blocks(nodes: &[Node], width: usize, look: Look, spaced: bool) -> Vec<Row> {
    placed_blocks(nodes, width, look, spaced).0
}

/// Lays out block elements, and says which rows each of them took.
fn placed_blocks(
    nodes: &[Node],
    width: usize,
    look: Look,
    spaced: bool,
) -> (Vec<Row>, Vec<Range<usize>>) {
    let mut rows = Vec::new();
    let mut ranges = Vec::with_capacity(nodes.len());
    for node in nodes {
        let drawn = block(node, width, look);
        if spaced && !rows.is_empty() && !drawn.is_empty() {
            rows.push(Row::new());
        }
        let start = rows.len();
        rows.extend(drawn);
        ranges.push(start..rows.len());
    }
    (rows, ranges)
}

fn block(node: &Node, width: usize, look: Look) -> Vec<Row> {
    match node {
        Node::Heading(heading) => wrap(
            inline_pieces(&heading.children, Look::Heading(heading.depth)),
            width,
        ),
        Node::Paragraph(paragraph) => wrap(inline_pieces(&paragraph.children, look), width),
        Node::Blockquote(quote) => {
            let inner = blocks(&quote.children, width.saturating_sub(2), Look::Quote, true);
            with_prefix(inner, "│ ", "│ ", Look::Quote)
        }
        Node::List(list) => list_rows(list, width, look),
        Node::Code(code) => code
            .value
            .lines()
            .map(|line| vec![Piece::new(line.replace('\t', "    "), Look::Code)])
            .collect(),
        Node::Table(table) => table_rows(table, look),
        Node::ThematicBreak(_) => vec![vec![Piece::new("─".repeat(width), look)]],
        Node::Html(html) if !is_comment(&html.value) => html
            .value
            .lines()
            .map(|line| vec![Piece::new(line, look)])
            .collect(),
        Node::FootnoteDefinition(footnote) => {
            let marker = format!("[^{}]: ", footnote.identifier);
            let inner = blocks(
                &footnote.children,
                width.saturating_sub(text_width(&marker)),
                look,
                true,
            );
            let indent = " ".repeat(text_width(&marker));
            with_prefix(inner, &marker, &indent, look)
        }
        _ => Vec::new(),
    }
}

fn is_comment(html: &str) -> bool {
    html.trim_start().starts_with("<!--")
}

fn with_prefix(rows: Vec<Row>, first: &str, rest: &str, look: Look) -> Vec<Row> {
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            let prefix = if index == 0 { first } else { rest };
            let mut prefixed = vec![Piece::new(prefix, look)];
            prefixed.extend(row);
            prefixed
        })
        .collect()
}

fn list_rows(list: &List, width: usize, look: Look) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut number = list.start.unwrap_or(1);
    for node in &list.children {
        let Node::ListItem(item) = node else {
            continue;
        };
        let marker = match (item.checked, list.ordered) {
            (Some(true), _) => "[x] ".to_owned(),
            (Some(false), _) => "[ ] ".to_owned(),
            (None, true) => format!("{number}. "),
            (None, false) => "• ".to_owned(),
        };
        let indent = " ".repeat(text_width(&marker));
        let inner_width = width.saturating_sub(text_width(&marker));
        let inner = blocks(&item.children, inner_width, look, item.spread);
        if list.spread && !rows.is_empty() {
            rows.push(Row::new());
        }
        rows.extend(with_prefix(inner, &marker, &indent, look));
        number = number.saturating_add(1);
    }
    rows
}

fn table_rows(table: &Table, look: Look) -> Vec<Row> {
    let cells: Vec<Vec<Row>> = table
        .children
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let look = if index == 0 { Look::Bold } else { look };
            children(row)
                .iter()
                .map(|cell| inline_pieces(children(cell), look))
                .collect()
        })
        .collect();
    let widths = column_widths(&cells);
    let mut rows = Vec::new();
    for (index, row) in cells.iter().enumerate() {
        rows.push(table_row(row, &widths, &table.align, look));
        if index == 0 {
            rows.push(table_rule(&widths, look));
        }
    }
    rows
}

fn children(node: &Node) -> &[Node] {
    node.children().map_or(&[], Vec::as_slice)
}

fn column_widths(rows: &[Vec<Row>]) -> Vec<usize> {
    let mut widths = Vec::new();
    for row in rows {
        for (column, cell) in row.iter().enumerate() {
            if column == widths.len() {
                widths.push(0);
            }
            if let Some(width) = widths.get_mut(column) {
                *width = row_width(cell).max(*width);
            }
        }
    }
    widths
}

fn table_row(cells: &[Row], widths: &[usize], align: &[AlignKind], look: Look) -> Row {
    let mut row = Row::new();
    for (column, width) in widths.iter().enumerate() {
        if column > 0 {
            row.push(Piece::new(" │ ", look));
        }
        let cell = cells.get(column).map_or(&[][..], Vec::as_slice);
        let gap = width.saturating_sub(row_width(cell));
        let before = match align.get(column) {
            Some(AlignKind::Right) => gap,
            Some(AlignKind::Center) => gap / 2,
            _ => 0,
        };
        row.push(Piece::new(" ".repeat(before), look));
        row.extend_from_slice(cell);
        row.push(Piece::new(" ".repeat(gap.saturating_sub(before)), look));
    }
    row
}

fn table_rule(widths: &[usize], look: Look) -> Row {
    let columns: Vec<String> = widths.iter().map(|width| "─".repeat(*width)).collect();
    vec![Piece::new(columns.join("─┼─"), look)]
}

fn inline_pieces(nodes: &[Node], look: Look) -> Vec<Piece> {
    let mut pieces = Vec::new();
    add_inline(nodes, look, &mut pieces);
    pieces
}

fn add_inline(nodes: &[Node], look: Look, pieces: &mut Vec<Piece>) {
    for node in nodes {
        match node {
            Node::Text(text) => pieces.push(Piece::new(text.value.replace('\n', " "), look)),
            Node::Strong(strong) => add_inline(&strong.children, look.with(Look::Bold), pieces),
            Node::Emphasis(emphasis) => {
                add_inline(&emphasis.children, look.with(Look::Italic), pieces);
            }
            Node::Delete(delete) => add_inline(&delete.children, look.with(Look::Struck), pieces),
            Node::InlineCode(code) => {
                pieces.push(Piece::new(code.value.as_str(), look.with(Look::Code)));
            }
            Node::Link(link) => {
                let look = Look::Link(link_id(link.position.as_ref()));
                add_inline(&link.children, look, pieces);
            }
            Node::LinkReference(link) => {
                let look = Look::Link(link_id(link.position.as_ref()));
                add_inline(&link.children, look, pieces);
            }
            Node::Image(image) => pieces.push(Piece::new(image.alt.as_str(), look)),
            Node::ImageReference(image) => pieces.push(Piece::new(image.alt.as_str(), look)),
            Node::FootnoteReference(footnote) => {
                pieces.push(Piece::new(
                    format!("[^{}]", footnote.identifier),
                    Look::Footnote,
                ));
            }
            Node::Break(_) => pieces.push(Piece::new("\n", look)),
            Node::Html(html) if is_comment(&html.value) => {}
            other => pieces.push(Piece::new(other.to_string(), look)),
        }
    }
}

fn text_width(text: &str) -> usize {
    Span::raw(text).width()
}

fn row_width(row: &[Piece]) -> usize {
    row.iter().map(|piece| text_width(&piece.text)).sum()
}

/// Breaks inline text into rows no wider than `width`, between words. A word
/// wider than the screen gets a row to itself.
fn wrap(pieces: Vec<Piece>, width: usize) -> Vec<Row> {
    let mut wrapper = Wrapper::new(width);
    for piece in pieces {
        if piece.text == "\n" {
            wrapper.end_word();
            wrapper.end_row();
            continue;
        }
        for chunk in piece.text.split_inclusive(' ') {
            wrapper.word.push(Piece::new(chunk, piece.look));
            if chunk.ends_with(' ') {
                wrapper.end_word();
            }
        }
    }
    wrapper.finish()
}

struct Wrapper {
    width: usize,
    rows: Vec<Row>,
    row: Row,
    word: Row,
}

impl Wrapper {
    fn new(width: usize) -> Self {
        Self {
            width,
            rows: Vec::new(),
            row: Row::new(),
            word: Row::new(),
        }
    }

    /// A word can span pieces, as in `**bold**text`, and is never split.
    fn end_word(&mut self) {
        let word = mem::take(&mut self.word);
        let used = row_width(&self.row);
        let needed = row_width(&word).saturating_sub(trailing_spaces(&word));
        if used > 0 && used + needed > self.width {
            self.end_row();
        }
        for piece in word {
            match self.row.last_mut() {
                Some(last) if last.look == piece.look => last.text.push_str(&piece.text),
                _ => self.row.push(piece),
            }
        }
    }

    fn end_row(&mut self) {
        let mut row = mem::take(&mut self.row);
        if let Some(last) = row.last_mut() {
            last.text.truncate(last.text.trim_end_matches(' ').len());
        }
        self.rows.push(row);
    }

    fn finish(mut self) -> Vec<Row> {
        self.end_word();
        if !self.row.is_empty() {
            self.end_row();
        }
        self.rows
    }
}

fn trailing_spaces(word: &[Piece]) -> usize {
    word.last().map_or(0, |piece| {
        piece.text.len() - piece.text.trim_end_matches(' ').len()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(markdown: &str, width: u16) -> Vec<String> {
        Document::parse(markdown)
            .unwrap()
            .render(width)
            .lines(0, usize::MAX, None)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn long_paragraphs_wrap_between_words() {
        assert_eq!(
            screen("one two three four", 9),
            ["one two", "three", "four"]
        );
    }

    #[test]
    fn a_word_is_not_split_where_its_look_changes() {
        assert_eq!(screen("aaaa **bb**cc", 7), ["aaaa", "bbcc"]);
    }

    #[test]
    fn a_word_wider_than_the_screen_keeps_its_own_row() {
        assert_eq!(screen("a bbbbbbbb c", 4), ["a", "bbbbbbbb", "c"]);
    }

    #[test]
    fn a_soft_break_is_a_space_and_a_hard_break_is_a_new_row() {
        assert_eq!(screen("a\nb  \nc", 20), ["a b", "c"]);
    }

    #[test]
    fn blocks_are_separated_by_one_blank_row() {
        assert_eq!(screen("# A\n\nb\n\n- c", 20), ["A", "", "b", "", "• c"]);
    }

    #[test]
    fn hidden_definitions_leave_no_blank_rows() {
        let markdown = "a\n\n[//]: # \"import X\"\n[//]: # \"decisionTable Y\"\n\nb";
        assert_eq!(screen(markdown, 20), ["a", "", "b"]);
    }

    #[test]
    fn wrapped_list_items_line_up_under_their_text() {
        assert_eq!(screen("- one two", 6), ["• one", "  two"]);
    }

    #[test]
    fn nested_lists_are_indented_without_blank_rows() {
        assert_eq!(screen("- a\n  - b\n- c", 20), ["• a", "  • b", "• c"]);
    }

    #[test]
    fn numbered_lists_count_from_their_start() {
        assert_eq!(screen("3. a\n4. b", 20), ["3. a", "4. b"]);
    }

    #[test]
    fn task_items_show_whether_they_are_done() {
        assert_eq!(screen("- [x] a\n- [ ] b", 20), ["[x] a", "[ ] b"]);
    }

    #[test]
    fn every_row_of_a_quote_is_marked() {
        assert_eq!(screen("> a\n>\n> b", 20), ["│ a", "│ ", "│ b"]);
    }

    #[test]
    fn code_keeps_its_spacing_and_blank_rows() {
        assert_eq!(screen("```\na  b\n\n  c\n```", 20), ["a  b", "", "  c"]);
    }

    #[test]
    fn table_columns_line_up_and_follow_their_alignment() {
        let markdown = "| a | b | c |\n| - | -: | :-: |\n| long | 1 | x |\n| s | 22 | yyy |";
        assert_eq!(
            screen(markdown, 40),
            [
                "a    │  b │  c ",
                "─────┼────┼────",
                "long │  1 │  x ",
                "s    │ 22 │ yyy",
            ]
        );
    }

    #[test]
    fn a_link_stays_a_link_when_it_is_bold() {
        let preview = preview("[**Docs**](docs.md)").unwrap().unwrap();
        assert_eq!(preview.style, "link");
    }

    #[test]
    fn html_comments_draw_nothing() {
        assert_eq!(preview("<!-- note -->").unwrap(), None);
    }

    fn links(markdown: &str, width: u16) -> Vec<(String, String, Range<usize>)> {
        Document::parse(markdown)
            .unwrap()
            .render(width)
            .links()
            .iter()
            .map(|link| (link.text.clone(), link.target.clone(), link.rows.clone()))
            .collect()
    }

    #[test]
    fn links_are_found_everywhere_they_can_be_drawn() {
        let markdown = "# [a](1.md)\n\n> [b](2.md)\n\n- [c](3.md)\n\n| [d](4.md) |\n| - |\n";
        let targets: Vec<String> = links(markdown, 20).into_iter().map(|l| l.1).collect();
        assert_eq!(targets, ["1.md", "2.md", "3.md", "4.md"]);
    }

    #[test]
    fn a_wrapped_link_is_one_link_on_several_rows() {
        assert_eq!(
            links("[one two](a.md)", 4),
            [("one two".to_owned(), "a.md".to_owned(), 0..2)]
        );
    }

    #[test]
    fn links_next_to_each_other_stay_apart() {
        let found = links("[a](1.md)[b](2.md)", 20);
        assert_eq!(
            found,
            [
                ("a".to_owned(), "1.md".to_owned(), 0..1),
                ("b".to_owned(), "2.md".to_owned(), 0..1)
            ]
        );
    }

    #[test]
    fn a_reference_link_points_at_its_definition() {
        let found = links("[a][one]\n\n[one]: page.md\n", 20);
        assert_eq!(found, [("a".to_owned(), "page.md".to_owned(), 0..1)]);
    }

    #[test]
    fn a_footnote_is_not_a_link_to_follow() {
        assert_eq!(links("a[^1]\n\n[^1]: b\n", 20), []);
    }

    #[test]
    fn only_the_focused_link_is_highlighted() {
        let rendered = Document::parse("[a](1.md) [b](2.md)").unwrap().render(20);
        let lines = rendered.lines(0, 1, rendered.links().get(1));
        let reversed: Vec<String> = lines
            .iter()
            .flat_map(|line| &line.spans)
            .filter(|span| span.style.add_modifier.contains(Modifier::REVERSED))
            .map(ToString::to_string)
            .collect();
        assert_eq!(reversed, ["b"]);
    }

    #[test]
    fn anchors_follow_github_and_repeated_headings_are_numbered() {
        let document = Document::parse("# The `End`!\n\n## Twice\n\n## Twice\n").unwrap();
        let anchors: Vec<String> = document.headings().into_iter().map(|h| h.anchor).collect();
        assert_eq!(anchors, ["the-end", "twice", "twice-1"]);
    }

    #[test]
    fn a_line_of_the_markdown_can_be_found_on_screen() {
        let rendered = Document::parse("# A\n\none two\n\n# B\n")
            .unwrap()
            .render(4);
        assert_eq!(rendered.rows_of_line(3), Some(2..4));
        assert_eq!(rendered.rows_of_line(5), Some(5..6));
    }
}
