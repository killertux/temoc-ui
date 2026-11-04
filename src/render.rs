//! Turning markdown into styled lines for the terminal.

use crate::run::TableResult;
use crate::temoc::{self, Failure, RunResult, TestFile};
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
/// How a real failure stands out.
const FAILED: Color = Color::Red;
/// How a snoozed failure stands out.
const SNOOZED: Color = Color::Yellow;

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
    /// Shows that a row is cut off by the edge of the screen.
    Marker,
    /// The result of a test table.
    Verdict(RunResult),
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
            Self::Marker => "marker".into(),
            Self::Verdict(result) => result.to_string(),
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
            Self::Marker => style.fg(Color::Cyan),
            Self::Verdict(RunResult::Passed) => style.fg(Color::Green),
            Self::Verdict(RunResult::Snoozed) => style.fg(SNOOZED),
            Self::Verdict(_) => style.fg(FAILED).add_modifier(Modifier::BOLD),
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
    /// The width of the table or code block this is part of, if any. These
    /// are never wrapped, and scroll sideways instead.
    block_width: Option<usize>,
}

impl Piece {
    fn new(text: impl Into<String>, look: Look) -> Self {
        Self {
            text: text.into(),
            look,
            block_width: None,
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
    sideways: Option<SidewaysPlace>,
}

/// Where a link sits in a row that can scroll sideways, in columns from the
/// start of the row.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SidewaysPlace {
    link: Range<usize>,
    /// Where the part of the row that scrolls begins.
    start: usize,
    /// Where the row ends.
    end: usize,
}

/// A parsed markdown document.
#[derive(Debug, Clone)]
pub struct Document {
    root: Node,
    targets: HashMap<usize, LinkTarget>,
}

/// What a link in the markdown says, by its offset in the markdown.
#[derive(Debug, Clone)]
struct LinkTarget {
    text: String,
    url: String,
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

    /// Lays the document out for a screen of this width. Each table with a
    /// result, given by the line it starts on, shows that result below it.
    #[must_use]
    pub fn render(&self, width: u16, results: &HashMap<usize, &TableResult>) -> Rendered {
        let nodes = children(&self.root);
        let width = usize::from(width);
        let (mut rows, mut ranges) = placed_blocks(nodes, width, Look::Plain, true);
        let mut failing = HashMap::new();
        let mut added = 0;
        for (node, range) in nodes.iter().zip(&mut ranges) {
            *range = range.start + added..range.end + added;
            if let Some(result) = results.get(&line_of(node)) {
                failing.extend(failing_rows(node, range.start, &result.failures));
                let lines = result_rows(result, width);
                added += lines.len();
                let end = range.end + lines.len();
                rows.splice(range.end..range.end, lines);
                range.end = end;
            }
        }
        Rendered {
            width,
            widest: rows
                .iter()
                .filter_map(|row| row_end(row))
                .max()
                .unwrap_or(0),
            links: links_in(&rows, &self.targets),
            blocks: nodes.iter().map(line_of).zip(ranges).collect(),
            failing,
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
    width: usize,
    /// How far the widest table or code block reaches.
    widest: usize,
    rows: Vec<Row>,
    /// The rows of each top-level element, by the line of the markdown it
    /// starts on.
    blocks: HashMap<usize, Range<usize>>,
    /// The color of each row that holds a failure.
    failing: HashMap<usize, Color>,
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

    /// How far the page can scroll sideways.
    #[must_use]
    pub fn last_left(&self) -> usize {
        self.widest.saturating_sub(self.width)
    }

    /// How far to scroll sideways to show a link, moving as little as possible
    /// from `left`. A link too wide to show is shown from its start.
    #[must_use]
    pub fn left_showing(&self, link: &Link, left: usize) -> usize {
        let Some(place) = &link.sideways else {
            return left;
        };
        let furthest = place.end.saturating_sub(self.width);
        let shift = left.min(furthest);
        // The markers at each edge take up a column.
        let most = place.link.start.saturating_sub(place.start + 1);
        let least = (place.link.end + 1).saturating_sub(self.width);
        let shown = shift.max(least.min(furthest)).min(most);
        if shown == shift { left } else { shown }
    }

    /// The rows from `top` on that fit in `height`, scrolled sideways by `left`,
    /// with a link and a whole row highlighted.
    #[must_use]
    pub fn lines(
        &self,
        top: usize,
        height: usize,
        left: usize,
        focused: Option<&Link>,
        focused_row: Option<usize>,
    ) -> Vec<Line<'static>> {
        let focused = focused.map(|link| Look::Link(link.id));
        self.rows
            .iter()
            .enumerate()
            .skip(top)
            .take(height)
            .map(|(index, row)| {
                let row_style = self
                    .failing
                    .get(&index)
                    .map_or_else(Style::new, |color| Style::new().fg(*color));
                let row_style = if Some(index) == focused_row {
                    row_style.add_modifier(Modifier::REVERSED)
                } else {
                    row_style
                };
                cut_to_screen(row, left, self.width)
                    .into_iter()
                    .map(|piece| {
                        let style = piece.look.style().patch(row_style);
                        let style = if Some(piece.look) == focused {
                            style.add_modifier(Modifier::REVERSED)
                        } else {
                            style
                        };
                        Span::styled(piece.text, style)
                    })
                    .collect()
            })
            .collect()
    }
}

/// How much of a row comes before the part that scrolls sideways. A quote's
/// bar or a list's bullet stays where it is.
fn fixed_width(row: &[Piece]) -> usize {
    let fixed = row.iter().take_while(|piece| piece.block_width.is_none());
    fixed.map(|piece| text_width(&piece.text)).sum()
}

/// Where a row ends, if it scrolls sideways.
fn row_end(row: &[Piece]) -> Option<usize> {
    let block_width = row.iter().find_map(|piece| piece.block_width)?;
    Some(fixed_width(row) + block_width)
}

/// What a row shows on a screen of this width when scrolled sideways. A table
/// or code block has markers where it is cut off.
fn cut_to_screen(row: &[Piece], left: usize, width: usize) -> Row {
    let start = row.iter().position(|piece| piece.block_width.is_some());
    let (fixed, sliding) = row.split_at(start.unwrap_or(row.len()));
    let end = row_end(row).unwrap_or(0);
    let shift = left.min(end.saturating_sub(width));
    let mut shown = fixed.to_vec();
    if shift > 0 && row_width(sliding) > 0 {
        shown.push(Piece::new("‹", Look::Marker));
        shown.extend(split_at_column(sliding, shift + 1).1);
    } else {
        shown.extend_from_slice(sliding);
    }
    if !sliding.is_empty()
        && let Some(room) = width.checked_sub(1)
        && row_width(&shown) > width
    {
        shown = split_at_column(&shown, room).0;
        shown.push(Piece::new("›", Look::Marker));
    }
    shown
}

/// Splits a row at a column. A wide character on the split is lost, and a space
/// on each side takes its place so that everything else stays aligned.
fn split_at_column(row: &[Piece], column: usize) -> (Row, Row) {
    let mut used = 0;
    let mut crossed = false;
    let (mut before, mut after) = (Row::new(), Row::new());
    for piece in row {
        let (mut start, mut rest) = (String::new(), String::new());
        for character in piece.text.chars() {
            let width = character_width(character);
            if crossed || used + width > column {
                if !crossed && used < column {
                    start.push(' ');
                    rest.push(' ');
                } else {
                    rest.push(character);
                }
                crossed = true;
            } else {
                start.push(character);
                used += width;
            }
        }
        before.push(Piece::new(start, piece.look));
        after.push(Piece::new(rest, piece.look));
    }
    (before, after)
}

/// Finds what each link says. Links can use a definition written anywhere in
/// the document.
fn link_targets(root: &Node) -> HashMap<usize, LinkTarget> {
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
        let found = match node {
            Node::Link(link) => Some((&link.position, &link.children, link.url.as_str())),
            Node::LinkReference(link) => definitions
                .get(link.identifier.as_str())
                .map(|url| (&link.position, &link.children, *url)),
            _ => None,
        };
        if let Some((position, children, url)) = found {
            let target = LinkTarget {
                text: plain_text(children),
                url: url.to_owned(),
            };
            targets.insert(link_id(position.as_ref()), target);
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

/// Inline elements as they read on screen, however they end up being wrapped.
fn plain_text(nodes: &[Node]) -> String {
    let pieces = inline_pieces(nodes, Look::Plain);
    let text: String = pieces.iter().map(|piece| piece.text.as_str()).collect();
    text.replace('\n', " ")
}

fn link_id(position: Option<&Position>) -> usize {
    position.map_or(0, |position| position.start.offset)
}

/// Finds the links in laid out rows. The pieces of a link always come one after
/// another, even when it wraps onto a new row.
fn links_in(rows: &[Row], targets: &HashMap<usize, LinkTarget>) -> Vec<Link> {
    let mut links: Vec<Link> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let mut column = 0;
        for piece in row {
            let columns = column..column + text_width(&piece.text);
            column = columns.end;
            let Look::Link(id) = piece.look else {
                continue;
            };
            match links.last_mut() {
                Some(link) if link.id == id => {
                    link.rows.end = index + 1;
                    if let Some(place) = &mut link.sideways {
                        place.link.end = columns.end;
                    }
                }
                _ => {
                    if let Some(target) = targets.get(&id) {
                        links.push(Link {
                            id,
                            text: target.text.clone(),
                            target: target.url.clone(),
                            rows: index..index + 1,
                            sideways: row_end(row).map(|end| SidewaysPlace {
                                link: columns,
                                start: fixed_width(row),
                                end,
                            }),
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

/// The rows of a table, drawn from row `start`, that hold a failure. Every row
/// of a table takes one row on screen, and a rule follows the first.
fn failing_rows(table: &Node, start: usize, failures: &[Failure]) -> Vec<(usize, Color)> {
    let rows = children(table).iter().enumerate();
    rows.filter_map(|(index, row)| {
        let line = line_of(row);
        let failure = failures.iter().find(|failure| failure.line == Some(line))?;
        let color = match failure.snoozed_until {
            Some(_) => SNOOZED,
            None => FAILED,
        };
        Some((start + index + usize::from(index > 0), color))
    })
    .collect()
}

/// What a run says below a table: its result, then its failures.
fn result_rows(table: &TableResult, width: usize) -> Vec<Row> {
    let mark = match table.result {
        RunResult::Passed => "✓",
        RunResult::Snoozed => "~",
        _ => "✗",
    };
    let verdict = Piece::new(
        format!("{mark} {}", table.result),
        Look::Verdict(table.result),
    );
    let mut rows = vec![Row::new(), vec![verdict]];
    for failure in &table.failures {
        let text = Piece::new(failure.to_string(), Look::Plain);
        let wrapped = wrap(vec![text], width.saturating_sub(2));
        rows.extend(with_prefix(wrapped, "  ", "  ", Look::Plain));
    }
    rows
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
        Node::Code(code) => unwrapped(
            code.value
                .lines()
                .map(|line| vec![Piece::new(line.replace('\t', "    "), Look::Code)])
                .collect(),
        ),
        Node::Table(table) => unwrapped(table_rows(table, look)),
        Node::ThematicBreak(_) => vec![vec![Piece::new("─".repeat(width), look)]],
        Node::Html(html) if !is_comment(&html.value) => html
            .value
            .lines()
            .flat_map(|line| match wrap(vec![Piece::new(line, look)], width) {
                rows if rows.is_empty() => vec![Row::new()],
                rows => rows,
            })
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

/// Marks rows as a block that scrolls sideways as one.
fn unwrapped(mut rows: Vec<Row>) -> Vec<Row> {
    let width = rows.iter().map(|row| row_width(row)).max().unwrap_or(0);
    for piece in rows.iter_mut().flatten() {
        piece.block_width = Some(width);
    }
    rows
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

fn character_width(character: char) -> usize {
    text_width(character.encode_utf8(&mut [0; 4]))
}

fn row_width(row: &[Piece]) -> usize {
    row.iter().map(|piece| text_width(&piece.text)).sum()
}

/// Breaks inline text into rows no wider than `width`, between words. A word
/// wider than the screen starts a new row and continues on the rows below.
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

    /// A word can span pieces, as in `**bold**text`, and is only split when it
    /// is too wide for a row.
    fn end_word(&mut self) {
        let word = mem::take(&mut self.word);
        let used = row_width(&self.row);
        let needed = row_width(&word).saturating_sub(trailing_spaces(&word));
        if used > 0 && used + needed > self.width {
            self.end_row();
        }
        if needed > self.width {
            self.break_word(word);
        } else {
            for piece in word {
                self.push(piece);
            }
        }
    }

    /// Fills rows with a word, one character at a time. A row always takes at
    /// least one character, however narrow the screen is. Only trailing spaces
    /// can be left over, and they are not needed on the next row.
    fn break_word(&mut self, word: Row) {
        for piece in word {
            for character in piece.text.chars() {
                let used = row_width(&self.row);
                if used > 0 && used + character_width(character) > self.width {
                    if character == ' ' {
                        continue;
                    }
                    self.end_row();
                }
                self.push(Piece::new(character, piece.look));
            }
        }
    }

    fn push(&mut self, piece: Piece) {
        match self.row.last_mut() {
            Some(last) if last.look == piece.look => last.text.push_str(&piece.text),
            _ => self.row.push(piece),
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
            .render(width, &HashMap::new())
            .lines(0, usize::MAX, 0, None, None)
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
    fn a_word_wider_than_the_screen_continues_on_the_rows_below() {
        assert_eq!(screen("a bbbbbbbb c", 4), ["a", "bbbb", "bbbb", "c"]);
    }

    #[test]
    fn the_end_of_a_broken_word_shares_its_row_with_what_follows() {
        assert_eq!(screen("aaaaaa b", 4), ["aaaa", "aa b"]);
    }

    #[test]
    fn a_broken_word_keeps_its_looks() {
        let rendered = Document::parse("**aa**bbbb")
            .unwrap()
            .render(3, &HashMap::new());
        let lines = rendered.lines(0, 2, 0, None, None);
        let spans: Vec<Vec<String>> = lines
            .iter()
            .map(|line| line.spans.iter().map(ToString::to_string).collect())
            .collect();
        assert_eq!(spans, [vec!["aa", "b"], vec!["bbb"]]);
    }

    #[test]
    fn a_broken_word_lines_up_under_its_list_item() {
        assert_eq!(screen("- aaaaaaaa", 6), ["• aaaa", "  aaaa"]);
    }

    #[test]
    fn a_wide_character_is_never_split_by_a_broken_word() {
        assert_eq!(screen("你好世", 4), ["你好", "世"]);
        assert_eq!(screen("你好世", 3), ["你", "好", "世"]);
    }

    #[test]
    fn a_screen_too_narrow_for_a_character_still_shows_the_word() {
        assert_eq!(screen("你好", 1), ["你", "好"]);
        assert_eq!(screen("ab", 0), ["a", "b"]);
    }

    #[test]
    fn a_word_that_fills_its_rows_leaves_no_blank_row() {
        assert_eq!(screen("aaaaaaaa b", 4), ["aaaa", "aaaa", "b"]);
    }

    #[test]
    fn html_shown_as_text_wraps_and_keeps_its_blank_rows() {
        assert_eq!(
            screen("<pre>\naaaa bbbb\n\nc\n</pre>", 6),
            ["<pre>", "aaaa", "bbbb", "", "c", "</pre>"]
        );
    }

    #[test]
    fn text_is_never_marked() {
        assert_eq!(scrolled("aaaaaaaa", 4, 0), ["aaaa", "aaaa"]);
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
            .render(width, &HashMap::new())
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
    fn a_broken_link_reads_as_it_was_written() {
        assert_eq!(
            links("[aaaaaa](a.md)", 4),
            [("aaaaaa".to_owned(), "a.md".to_owned(), 0..2)]
        );
    }

    #[test]
    fn a_link_around_an_image_reads_as_its_description() {
        let found = links("[![a picture](i.png)](a.md)", 20);
        assert_eq!(found, [("a picture".to_owned(), "a.md".to_owned(), 0..1)]);
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
        let rendered = Document::parse("[a](1.md) [b](2.md)")
            .unwrap()
            .render(20, &HashMap::new());
        let lines = rendered.lines(0, 1, 0, rendered.links().get(1), None);
        let reversed: Vec<String> = lines
            .iter()
            .flat_map(|line| &line.spans)
            .filter(|span| span.style.add_modifier.contains(Modifier::REVERSED))
            .map(ToString::to_string)
            .collect();
        assert_eq!(reversed, ["b"]);
    }

    fn scrolled(markdown: &str, width: u16, left: usize) -> Vec<String> {
        Document::parse(markdown)
            .unwrap()
            .render(width, &HashMap::new())
            .lines(0, usize::MAX, left, None, None)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn a_cut_off_row_is_marked_on_each_side_with_more_to_see() {
        let code = "```\nabcdefgh\n```";
        assert_eq!(scrolled(code, 5, 0), ["abcd›"]);
        assert_eq!(scrolled(code, 5, 1), ["‹cde›"]);
        assert_eq!(scrolled(code, 5, 3), ["‹efgh"]);
    }

    #[test]
    fn a_row_never_scrolls_past_its_end() {
        assert_eq!(scrolled("```\nabcdefgh\n```", 5, 100), ["‹efgh"]);
    }

    #[test]
    fn a_code_block_moves_as_one_and_blank_rows_have_no_marker() {
        let code = "```\nabcdefgh\n\nab\n```";
        assert_eq!(scrolled(code, 5, 3), ["‹efgh", "", "‹"]);
    }

    #[test]
    fn what_comes_before_a_block_stays_in_place() {
        let code = "> ```\n> abcdefgh\n> ```";
        assert_eq!(scrolled(code, 6, 2), ["│ ‹de›"]);
    }

    #[test]
    fn a_wide_character_on_the_edge_becomes_a_space() {
        let code = "```\n你好世界\n```";
        assert_eq!(scrolled(code, 5, 0), ["你好›"]);
        assert_eq!(scrolled(code, 5, 1), ["‹好 ›"]);
        assert_eq!(scrolled(code, 5, 2), ["‹ 世›"]);
    }

    #[test]
    fn text_does_not_scroll() {
        assert_eq!(scrolled("short\n\n```\nabcdefgh\n```", 5, 2)[0], "short");
    }

    #[test]
    fn a_table_keeps_its_look_when_it_scrolls() {
        let rendered = Document::parse("| a |\n| - |\n| `bbbbbbbb` |\n")
            .unwrap()
            .render(5, &HashMap::new());
        let lines = rendered.lines(2, 1, 2, None, None);
        let colors: Vec<Option<Color>> = lines
            .iter()
            .flat_map(|line| &line.spans)
            .filter(|span| span.content.contains('b'))
            .map(|span| span.style.fg)
            .collect();
        assert_eq!(colors, [Some(Color::Yellow)]);
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
            .render(4, &HashMap::new());
        assert_eq!(rendered.rows_of_line(3), Some(2..4));
        assert_eq!(rendered.rows_of_line(5), Some(5..6));
    }

    fn failed_at(line: usize, snoozed: bool) -> TableResult {
        TableResult {
            result: if snoozed {
                RunResult::Snoozed
            } else {
                RunResult::Failed
            },
            failures: vec![Failure {
                line: Some(line),
                message: "expected a long value, got another".into(),
                snoozed_until: snoozed.then(|| "2099-12-31".into()),
            }],
        }
    }

    const TABLE_THEN_MORE: &str = "| a |\n| - |\n| 1 |\n| 2 |\n\n[next](a.md)\n\n## After\n";

    fn shown(rendered: &Rendered) -> Vec<String> {
        let lines = rendered.lines(0, usize::MAX, 0, None, None);
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_result_goes_below_its_table_and_moves_the_rest_down() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let before = document.render(60, &HashMap::new());
        let result = failed_at(4, false);
        let after = document.render(60, &HashMap::from([(1, &result)]));
        assert_eq!(
            shown(&after),
            [
                "a",
                "─",
                "1",
                "2",
                "",
                "✗ failed",
                "  line 4: expected a long value, got another",
                "",
                "next",
                "",
                "After",
            ]
        );
        assert_eq!(after.rows_of_line(1), Some(0..7));
        assert_eq!(after.rows_of_line(8), Some(10..11));
        assert_eq!(
            after.links().first().map(|link| link.rows.clone()),
            Some(8..9)
        );
        assert_eq!(before.height() + 3, after.height());
    }

    #[test]
    fn a_long_failure_wraps_under_its_table() {
        let document = Document::parse("| a |\n| - |\n| 1 |\n").unwrap();
        let result = failed_at(3, false);
        let rendered = document.render(20, &HashMap::from([(1, &result)]));
        assert_eq!(
            shown(&rendered).get(5..),
            Some(
                &[
                    "  line 3: expected a".to_owned(),
                    "  long value, got".to_owned(),
                    "  another".to_owned(),
                ][..]
            )
        );
    }

    fn row_color(rendered: &Rendered, row: usize) -> Option<Color> {
        let lines = rendered.lines(row, 1, 0, None, None);
        lines.first()?.spans.first()?.style.fg
    }

    #[test]
    fn a_failing_row_skips_the_rule_under_the_header() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let result = failed_at(4, false);
        let rendered = document.render(40, &HashMap::from([(1, &result)]));
        assert_eq!(row_color(&rendered, 2), None);
        assert_eq!(row_color(&rendered, 3), Some(FAILED));
    }

    #[test]
    fn a_failure_on_the_header_marks_the_header() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let result = failed_at(1, true);
        let rendered = document.render(40, &HashMap::from([(1, &result)]));
        assert_eq!(row_color(&rendered, 0), Some(SNOOZED));
    }

    #[test]
    fn a_failure_outside_the_rows_marks_nothing() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let result = failed_at(100, false);
        let rendered = document.render(40, &HashMap::from([(1, &result)]));
        assert!(rendered.failing.is_empty());
    }

    #[test]
    fn the_focused_row_is_reversed_whole() {
        let document = Document::parse("| a | b |\n| - | - |\n| 1 | 2 |\n").unwrap();
        let rendered = document.render(40, &HashMap::new());
        let lines = rendered.lines(0, 1, 0, None, Some(0));
        let spans = lines.first().map_or(&[][..], |line| line.spans.as_slice());
        assert_ne!(spans.len(), 0);
        for span in spans {
            assert!(span.style.add_modifier.contains(Modifier::REVERSED));
        }
    }
}
