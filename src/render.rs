//! Turning markdown into styled lines for the terminal.

use markdown::ParseOptions;
use markdown::mdast::{AlignKind, List, Node, Table};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::mem;

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
    Link,
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
            Self::Link => "link".into(),
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
            Self::Link => style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
            Self::Quote => style.fg(Color::Gray),
        }
    }

    /// The look of `emphasis` inside this look. Links and headings stay as they
    /// are, so that they can still be recognized.
    fn with(self, emphasis: Self) -> Self {
        match self {
            Self::Link | Self::Heading(_) => self,
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

/// A parsed markdown document.
#[derive(Debug, Clone)]
pub struct Document {
    root: Node,
}

impl Document {
    /// Parses markdown the same way temoc does.
    ///
    /// # Errors
    ///
    /// Fails when the markdown cannot be parsed.
    pub fn parse(markdown: &str) -> Result<Self, String> {
        markdown::to_mdast(markdown, &ParseOptions::gfm())
            .map(|root| Self { root })
            .map_err(|message| message.to_string())
    }

    /// Lays the document out for a screen of this width.
    #[must_use]
    pub fn lines(&self, width: u16) -> Vec<Line<'static>> {
        self.rows(usize::from(width))
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|piece| Span::styled(piece.text, piece.look.style()))
                    .collect()
            })
            .collect()
    }

    fn rows(&self, width: usize) -> Vec<Row> {
        let nodes = self.root.children().map_or(&[][..], Vec::as_slice);
        blocks(nodes, width, Look::Plain, true)
    }
}

/// Lays out block elements, skipping those that draw nothing.
fn blocks(nodes: &[Node], width: usize, look: Look, spaced: bool) -> Vec<Row> {
    let mut rows = Vec::new();
    for node in nodes {
        let drawn = block(node, width, look);
        if spaced && !rows.is_empty() && !drawn.is_empty() {
            rows.push(Row::new());
        }
        rows.extend(drawn);
    }
    rows
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
            Node::Link(link) => add_inline(&link.children, Look::Link, pieces),
            Node::LinkReference(link) => add_inline(&link.children, Look::Link, pieces),
            Node::Image(image) => pieces.push(Piece::new(image.alt.as_str(), look)),
            Node::ImageReference(image) => pieces.push(Piece::new(image.alt.as_str(), look)),
            Node::FootnoteReference(footnote) => {
                pieces.push(Piece::new(
                    format!("[^{}]", footnote.identifier),
                    Look::Link,
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
            .lines(width)
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
}
