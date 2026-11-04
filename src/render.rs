//! Turning markdown into styled lines for the terminal.

use crate::run::TableResult;
use crate::temoc::{self, Failure, RunResult, TestFile};
use crate::theme::Theme;
use markdown::ParseOptions;
use markdown::mdast::{AlignKind, List, Node, Table};
use markdown::unist::Position;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;
use std::mem;
use std::ops::Range;

/// Wide enough that a preview is never wrapped.
const PREVIEW_WIDTH: usize = 200;
/// How far past the end of a row its mark sits.
const MARK_GAP: usize = 2;
/// How wide a mark is.
const MARK_WIDTH: usize = 1;

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
    /// What explains a result that is not a pass.
    Note,
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
            Self::Note => "note".into(),
        }
    }

    fn style(self, theme: Theme) -> Style {
        let style = Style::new();
        match self {
            Self::Plain => style,
            Self::Heading(1) => style
                .fg(theme.heading)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            Self::Heading(_) => style.fg(theme.heading).add_modifier(Modifier::BOLD),
            Self::Bold => style.add_modifier(Modifier::BOLD),
            Self::Italic => style.add_modifier(Modifier::ITALIC),
            Self::Struck => style.add_modifier(Modifier::CROSSED_OUT),
            Self::Code => style.fg(theme.code),
            Self::Link(_) | Self::Footnote => {
                style.fg(theme.link).add_modifier(Modifier::UNDERLINED)
            }
            Self::Quote | Self::Marker => style.fg(theme.quote),
            Self::Verdict(result) => {
                let style = theme.result(result).map_or(style, |color| style.fg(color));
                match result {
                    RunResult::Failed | RunResult::Error => style.add_modifier(Modifier::BOLD),
                    _ => style,
                }
            }
            Self::Note => style.fg(theme.note),
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
    /// Whether this is part of the selection.
    selected: bool,
}

impl Piece {
    fn new(text: impl Into<String>, look: Look) -> Self {
        Self {
            text: text.into(),
            look,
            block_width: None,
            selected: false,
        }
    }

    /// The same look with other text.
    fn holding(&self, text: String) -> Self {
        Self {
            text,
            ..self.clone()
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

/// What stands out on a page.
#[derive(Debug, Clone, Copy, Default)]
pub struct Marks<'page> {
    /// The focused link.
    pub link: Option<&'page Link>,
    /// The row holding the header of the focused table.
    pub header: Option<usize>,
    /// The focused row of a table.
    pub row: Option<usize>,
    /// The selected text.
    pub selection: Option<(Spot, Spot)>,
}

/// A character of the laid out text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Spot {
    /// The row, counted from the top of the document.
    pub row: usize,
    /// The character, counted from the start of the row.
    pub index: usize,
}

/// A way to move through the laid out text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// One character back.
    Left,
    /// One character on.
    Right,
    /// One row up.
    Up,
    /// One row down.
    Down,
    /// To the next word.
    NextWord,
    /// To the previous word.
    PreviousWord,
    /// To the start of the row.
    RowStart,
    /// To the end of the row.
    RowEnd,
    /// To the start of the document.
    Top,
    /// To the end of the document.
    Bottom,
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

    /// Where the links of the document point, as written in the markdown.
    pub fn link_urls(&self) -> impl Iterator<Item = &str> {
        self.targets.values().map(|target| target.url.as_str())
    }

    /// Lays the document out for a screen of this width. Each table with a
    /// result, given by the line it starts on, shows that result in its rows
    /// and below it.
    #[must_use]
    pub fn render(&self, width: u16, results: &HashMap<usize, TableResult<'_>>) -> Rendered {
        let nodes = children(&self.root);
        let width = usize::from(width);
        let (mut rows, mut ranges) = placed_blocks(nodes, width, Look::Plain, true);
        let mut table_rows = TableRows::default();
        let mut added = 0;
        for (node, range) in nodes.iter().zip(&mut ranges) {
            *range = range.start + added..range.end + added;
            if !matches!(node, Node::Table(_)) {
                continue;
            }
            let result = results.get(&line_of(node));
            let mut end = range.end;
            end += table_rows.place(&mut rows, node, range.start, result, width);
            if let Some(result) = result {
                let lines = result_rows(result, width);
                let below = end;
                end += lines.len();
                rows.splice(below..below, lines);
            }
            added += end - range.end;
            range.end = end;
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
            table_rows,
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
    table_rows: TableRows,
    links: Vec<Link>,
}

/// Where the rows of the tables ended up, leaving out their headers.
#[derive(Debug, Clone, Default)]
struct TableRows {
    /// The row on screen of each row of a table, by its line of the markdown.
    by_line: HashMap<usize, usize>,
    /// The result of each row on screen that has one.
    results: HashMap<usize, RunResult>,
}

impl TableRows {
    /// Notes where the rows of a table drawn from row `start` are, and adds
    /// what they have to show for a run. Says how many rows that added.
    fn place(
        &mut self,
        rows: &mut Vec<Row>,
        table: &Node,
        start: usize,
        result: Option<&TableResult<'_>>,
        width: usize,
    ) -> usize {
        let cells = children(table);
        // Every row of a table takes one row on screen, and a rule follows the
        // first.
        let drawn = start..=start + cells.len();
        let results = result.map(|result| result.rows);
        let table_width = results
            .filter(|results| !results.is_empty())
            .and_then(|_| make_room_for_marks(rows.get_mut(drawn)?));
        let mut added = 0;
        for (index, row) in cells.iter().enumerate().skip(1) {
            let at = start + index + 1 + added;
            let line = line_of(row);
            self.by_line.insert(line, at);
            let Some(row) = results.and_then(|results| results.get(&line)) else {
                continue;
            };
            self.results.insert(at, row.result);
            if let (Some(pieces), Some(table_width)) = (rows.get_mut(at), table_width) {
                add_mark(pieces, row.result, table_width);
            }
            let notes = note_rows(&row.failures, width, Failure::note);
            added += notes.len();
            let below = at + 1;
            rows.splice(below..below, notes);
        }
        added
    }
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

    /// The row on screen of the row of a table on this line of the markdown.
    #[must_use]
    pub fn row_of_table_line(&self, line: usize) -> Option<usize> {
        self.table_rows.by_line.get(&line).copied()
    }

    /// The line of the markdown of the row of a table at this row on screen.
    #[must_use]
    pub fn table_line_at(&self, row: usize) -> Option<usize> {
        let mut rows = self.table_rows.by_line.iter();
        rows.find_map(|(line, at)| (*at == row).then_some(*line))
    }

    /// The position among [`Self::links`] of the link at a column of a row, as
    /// shown when scrolled sideways by `left`.
    #[must_use]
    pub fn link_at(&self, row: usize, column: usize, left: usize) -> Option<usize> {
        let mut end = 0;
        let shown = cut_to_screen(self.rows.get(row)?, left, self.width);
        let piece = shown.iter().find(|piece| {
            end += text_width(&piece.text);
            column < end
        })?;
        let Look::Link(id) = piece.look else {
            return None;
        };
        self.links.iter().position(|link| link.id == id)
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
        link.sideways
            .as_ref()
            .map_or(left, |place| self.left_showing_place(place, left))
    }

    /// How far to scroll sideways to show a character, moving as little as
    /// possible from `left`.
    #[must_use]
    pub fn left_showing_spot(&self, spot: Spot, left: usize) -> usize {
        let Some(row) = self.rows.get(spot.row) else {
            return left;
        };
        let Some(end) = row_end(row) else {
            return left;
        };
        let chars = self.chars(spot.row);
        let before: usize = chars
            .iter()
            .take(spot.index)
            .map(|c| character_width(*c))
            .sum();
        let width = chars.get(spot.index).map_or(1, |c| character_width(*c));
        let place = SidewaysPlace {
            link: before..before + width,
            start: fixed_width(row),
            end,
        };
        self.left_showing_place(&place, left)
    }

    fn left_showing_place(&self, place: &SidewaysPlace, left: usize) -> usize {
        let furthest = place.end.saturating_sub(self.width);
        let shift = left.min(furthest);
        // The markers at each edge take up a column.
        let most = place.link.start.saturating_sub(place.start + 1);
        let least = (place.link.end + 1).saturating_sub(self.width);
        let shown = shift.max(least.min(furthest)).min(most);
        if shown == shift { left } else { shown }
    }

    /// The characters of a row as they read, without the padding at its end.
    fn chars(&self, row: usize) -> Vec<char> {
        let pieces = self.rows.get(row).map_or(&[][..], Vec::as_slice);
        let text: String = pieces.iter().map(|piece| piece.text.as_str()).collect();
        text.trim_end().chars().collect()
    }

    fn last_index(&self, row: usize) -> usize {
        self.chars(row).len().saturating_sub(1)
    }

    fn last_row(&self) -> usize {
        self.rows.len().saturating_sub(1)
    }

    /// The first character at or below a row of the screen, for a cursor to
    /// start on.
    #[must_use]
    pub fn first_spot(&self, top: usize, height: usize) -> Spot {
        let row = (top..top.saturating_add(height).min(self.rows.len()))
            .find(|row| !self.chars(*row).is_empty());
        Spot {
            row: row.unwrap_or(top),
            index: 0,
        }
    }

    /// The character at a spot of the screen when scrolled sideways by `left`.
    /// A spot past the end of a row gives its last character.
    #[must_use]
    pub fn spot_at(&self, row: usize, column: usize, left: usize) -> Spot {
        let row = row.min(self.last_row());
        let Some(pieces) = self.rows.get(row) else {
            return Spot::default();
        };
        let furthest = row_end(pieces).unwrap_or(0).saturating_sub(self.width);
        let column = if column >= fixed_width(pieces) {
            column + left.min(furthest)
        } else {
            column
        };
        let mut used = 0;
        let found = self.chars(row).iter().position(|character| {
            used += character_width(*character);
            column < used
        });
        Spot {
            row,
            index: found.unwrap_or_else(|| self.last_index(row)),
        }
    }

    /// Where a motion leads from a spot, staying inside the document.
    #[must_use]
    pub fn moved(&self, spot: Spot, motion: Motion) -> Spot {
        let on_row = |row: usize| Spot {
            row,
            index: spot.index.min(self.last_index(row)),
        };
        match motion {
            Motion::Left => Spot {
                index: spot.index.saturating_sub(1),
                ..spot
            },
            Motion::Right => Spot {
                index: (spot.index + 1).min(self.last_index(spot.row)),
                ..spot
            },
            Motion::Up => on_row(spot.row.saturating_sub(1)),
            Motion::Down => on_row((spot.row + 1).min(self.last_row())),
            Motion::NextWord => self.next_word(spot).unwrap_or(spot),
            Motion::PreviousWord => self.previous_word(spot).unwrap_or(spot),
            Motion::RowStart => Spot { index: 0, ..spot },
            Motion::RowEnd => Spot {
                index: self.last_index(spot.row),
                ..spot
            },
            Motion::Top => Spot::default(),
            Motion::Bottom => Spot {
                row: self.last_row(),
                index: self.last_index(self.last_row()),
            },
        }
    }

    fn next_word(&self, spot: Spot) -> Option<Spot> {
        let chars = self.chars(spot.row);
        let here = (spot.index + 1..chars.len()).find(|index| starts_word(&chars, *index));
        here.map(|index| Spot { index, ..spot }).or_else(|| {
            (spot.row + 1..self.rows.len()).find_map(|row| {
                let chars = self.chars(row);
                let index = (0..chars.len()).find(|index| starts_word(&chars, *index))?;
                Some(Spot { row, index })
            })
        })
    }

    fn previous_word(&self, spot: Spot) -> Option<Spot> {
        let chars = self.chars(spot.row);
        let here = (0..spot.index)
            .rev()
            .find(|index| starts_word(&chars, *index));
        here.map(|index| Spot { index, ..spot }).or_else(|| {
            (0..spot.row).rev().find_map(|row| {
                let chars = self.chars(row);
                let index = (0..chars.len())
                    .rev()
                    .find(|index| starts_word(&chars, *index))?;
                Some(Spot { row, index })
            })
        })
    }

    /// The text from one spot to another, including both, with rows separated
    /// by line breaks.
    #[must_use]
    pub fn text_between(&self, from: Spot, to: Spot) -> String {
        let rows: Vec<String> = (from.row..=to.row.min(self.last_row()))
            .map(|row| {
                let range = row_range(row, from, to);
                let chars = self.chars(row);
                let end = range.end.min(chars.len());
                let start = range.start.min(end);
                chars.get(start..end).unwrap_or_default().iter().collect()
            })
            .collect();
        rows.join("\n")
    }

    /// The rows from `top` on that fit in `height`, scrolled sideways by `left`.
    #[must_use]
    pub fn lines(
        &self,
        theme: Theme,
        top: usize,
        height: usize,
        left: usize,
        marks: Marks<'_>,
    ) -> Vec<Line<'static>> {
        let Marks {
            link,
            header,
            row: focused_row,
            selection,
        } = marks;
        let focused = link.map(|link| Look::Link(link.id));
        self.rows
            .iter()
            .enumerate()
            .skip(top)
            .take(height)
            .map(|(index, row)| {
                let color = self.table_rows.results.get(&index);
                let color = color.and_then(|result| theme.result(*result));
                let row_style = color.map_or_else(Style::new, |color| Style::new().fg(color));
                let row_style = if Some(index) == header || Some(index) == focused_row {
                    row_style.add_modifier(Modifier::REVERSED)
                } else {
                    row_style
                };
                let selected = selection
                    .filter(|(from, to)| (from.row..=to.row).contains(&index))
                    .map(|(from, to)| row_range(index, from, to));
                let marked = selected.map(|range| with_selection(row, &range));
                cut_to_screen(marked.as_deref().unwrap_or(row), left, self.width)
                    .into_iter()
                    .map(|piece| {
                        let style = piece.look.style(theme).patch(row_style);
                        let style = if Some(piece.look) == focused || piece.selected {
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

fn starts_word(chars: &[char], index: usize) -> bool {
    let is_word = |index: usize| chars.get(index).is_some_and(|c| !c.is_whitespace());
    is_word(index) && (index == 0 || !is_word(index - 1))
}

/// The characters of a row that lie between two spots, including both.
fn row_range(row: usize, from: Spot, to: Spot) -> Range<usize> {
    let start = if row == from.row { from.index } else { 0 };
    let end = if row == to.row {
        to.index.saturating_add(1)
    } else {
        usize::MAX
    };
    start..end
}

/// Marks the characters of a row as selected. An empty row gets a space, so
/// that a cursor on it can be seen.
fn with_selection(row: &[Piece], range: &Range<usize>) -> Row {
    let mut marked = Row::new();
    let mut index = 0;
    for piece in row {
        let (mut before, mut inside, mut after) = (String::new(), String::new(), String::new());
        for character in piece.text.chars() {
            if index < range.start {
                before.push(character);
            } else if index < range.end {
                inside.push(character);
            } else {
                after.push(character);
            }
            index += 1;
        }
        marked.push(piece.holding(before));
        marked.push(Piece {
            selected: true,
            ..piece.holding(inside)
        });
        marked.push(piece.holding(after));
    }
    if index == 0 {
        marked.push(Piece {
            selected: true,
            ..Piece::new(" ", Look::Plain)
        });
    }
    marked
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
        before.push(piece.holding(start));
        after.push(piece.holding(rest));
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

/// Widens the rows of a table, so that a mark can be scrolled into view. Gives
/// the width of the table without the room for marks.
fn make_room_for_marks(rows: &mut [Row]) -> Option<usize> {
    let width = rows.iter().flatten().find_map(|piece| piece.block_width)?;
    for piece in rows.iter_mut().flatten() {
        piece.block_width = Some(width + MARK_GAP + MARK_WIDTH);
    }
    Some(width)
}

/// Adds a row's mark, lined up with those of the rest of its table.
fn add_mark(row: &mut Row, result: RunResult, table_width: usize) {
    let gap = (table_width + MARK_GAP).saturating_sub(row_width(row));
    let mark = Piece {
        block_width: row.first().and_then(|piece| piece.block_width),
        ..Piece::new(format!("{}{}", " ".repeat(gap), result.mark()), Look::Plain)
    };
    row.push(mark);
}

/// Explains what did not pass, each in as many rows as it takes.
fn note_rows(failures: &[Failure], width: usize, text: impl Fn(&Failure) -> String) -> Vec<Row> {
    let mut rows = Vec::new();
    for failure in failures {
        let look = match failure.result {
            RunResult::Snoozed => Look::Quote,
            _ => Look::Note,
        };
        let wrapped = wrap(
            vec![Piece::new(text(failure), look)],
            width.saturating_sub(2),
        );
        rows.extend(with_prefix(wrapped, "  ", "  ", look));
    }
    rows
}

/// What a run says below a table: its result, then what failed outside a row.
fn result_rows(table: &TableResult<'_>, width: usize) -> Vec<Row> {
    let verdict = format!("{} {}", table.result.mark(), table.result);
    let mut rows = vec![
        Row::new(),
        vec![Piece::new(verdict, Look::Verdict(table.result))],
    ];
    rows.extend(note_rows(table.failures, width, ToString::to_string));
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
    use crate::run::RowResult;
    use ratatui::style::Color;
    use std::collections::BTreeMap;

    fn selected(selection: (Spot, Spot)) -> Marks<'static> {
        Marks {
            selection: Some(selection),
            ..Marks::default()
        }
    }

    fn screen(markdown: &str, width: u16) -> Vec<String> {
        Document::parse(markdown)
            .unwrap()
            .render(width, &HashMap::new())
            .lines(Theme::DARK, 0, usize::MAX, 0, Marks::default())
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
        let lines = rendered.lines(Theme::DARK, 0, 2, 0, Marks::default());
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
        let marks = Marks {
            link: rendered.links().get(1),
            ..Marks::default()
        };
        let lines = rendered.lines(Theme::DARK, 0, 1, 0, marks);
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
            .lines(Theme::DARK, 0, usize::MAX, left, Marks::default())
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
        let lines = rendered.lines(Theme::DARK, 2, 1, 2, Marks::default());
        let colors: Vec<Option<Color>> = lines
            .iter()
            .flat_map(|line| &line.spans)
            .filter(|span| span.content.contains('b'))
            .map(|span| span.style.fg)
            .collect();
        assert_eq!(colors, [Some(Theme::DARK.code)]);
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

    #[test]
    fn a_link_is_found_by_where_it_is_on_screen() {
        let document = Document::parse("see [one](1.md) and [two](2.md)").unwrap();
        let rendered = document.render(40, &HashMap::new());
        assert_eq!(rendered.link_at(0, 0, 0), None);
        assert_eq!(rendered.link_at(0, 4, 0), Some(0));
        assert_eq!(rendered.link_at(0, 6, 0), Some(0));
        assert_eq!(rendered.link_at(0, 7, 0), None);
        assert_eq!(rendered.link_at(0, 12, 0), Some(1));
        assert_eq!(rendered.link_at(0, 39, 0), None);
        assert_eq!(rendered.link_at(1, 0, 0), None);
    }

    #[test]
    fn a_wrapped_link_is_found_on_all_of_its_rows() {
        let document = Document::parse("[one two](1.md)").unwrap();
        let rendered = document.render(4, &HashMap::new());
        assert_eq!(rendered.link_at(0, 0, 0), Some(0));
        assert_eq!(rendered.link_at(1, 0, 0), Some(0));
    }

    #[test]
    fn a_link_in_a_scrolled_table_is_found_where_it_is_shown() {
        let filler = "x".repeat(30);
        let markdown = format!("| {filler} | [far](2.md) |\n| - | - |\n");
        let document = Document::parse(&markdown).unwrap();
        let rendered = document.render(20, &HashMap::new());
        assert_eq!(rendered.link_at(0, 15, 0), None);
        let line = rendered.lines(Theme::DARK, 0, 1, 16, Marks::default());
        let text = line.first().map(ToString::to_string).unwrap_or_default();
        let column = text.chars().position(|c| c == 'f').unwrap();
        assert_eq!(rendered.link_at(0, column, 16), Some(0));
    }

    #[test]
    fn a_marker_at_the_edge_is_not_part_of_a_link() {
        let filler = "x".repeat(30);
        let markdown = format!("| [near](1.md) | {filler} |\n| - | - |\n");
        let document = Document::parse(&markdown).unwrap();
        let rendered = document.render(20, &HashMap::new());
        assert_eq!(rendered.link_at(0, 0, 2), None);
    }

    fn rendered(markdown: &str, width: u16) -> Rendered {
        let document = Document::parse(markdown).unwrap();
        document.render(width, &HashMap::new())
    }

    fn spot(row: usize, index: usize) -> Spot {
        Spot { row, index }
    }

    const WORDS: &str = "one two\n\nthree  four";

    #[test]
    fn words_are_found_across_rows() {
        let rendered = rendered(WORDS, 40);
        let next = |at| rendered.moved(at, Motion::NextWord);
        assert_eq!(next(spot(0, 0)), spot(0, 4));
        assert_eq!(next(spot(0, 1)), spot(0, 4));
        assert_eq!(next(spot(0, 4)), spot(2, 0));
        assert_eq!(next(spot(2, 0)), spot(2, 7));
    }

    #[test]
    fn the_last_word_has_no_next_one() {
        let rendered = rendered(WORDS, 40);
        assert_eq!(rendered.moved(spot(2, 7), Motion::NextWord), spot(2, 7));
    }

    #[test]
    fn previous_words_are_found_across_rows() {
        let rendered = rendered(WORDS, 40);
        let previous = |at| rendered.moved(at, Motion::PreviousWord);
        assert_eq!(previous(spot(2, 9)), spot(2, 7));
        assert_eq!(previous(spot(2, 7)), spot(2, 0));
        assert_eq!(previous(spot(2, 0)), spot(0, 4));
        assert_eq!(previous(spot(0, 0)), spot(0, 0));
    }

    #[test]
    fn moving_stays_inside_rows_and_the_document() {
        let rendered = rendered("ab\n\nabcd", 40);
        assert_eq!(rendered.moved(spot(0, 0), Motion::Left), spot(0, 0));
        assert_eq!(rendered.moved(spot(0, 1), Motion::Right), spot(0, 1));
        assert_eq!(rendered.moved(spot(0, 0), Motion::Up), spot(0, 0));
        assert_eq!(rendered.moved(spot(2, 3), Motion::Down), spot(2, 3));
        assert_eq!(rendered.moved(spot(2, 3), Motion::Up), spot(1, 0));
        assert_eq!(rendered.moved(spot(0, 0), Motion::Bottom), spot(2, 3));
        assert_eq!(rendered.moved(spot(2, 3), Motion::Top), spot(0, 0));
    }

    #[test]
    fn an_empty_document_has_nowhere_to_go() {
        let rendered = rendered("", 40);
        assert_eq!(rendered.first_spot(0, 10), spot(0, 0));
        assert_eq!(rendered.moved(spot(0, 0), Motion::Down), spot(0, 0));
        assert_eq!(rendered.moved(spot(0, 0), Motion::NextWord), spot(0, 0));
        assert_eq!(rendered.spot_at(3, 3, 0), spot(0, 0));
        assert_eq!(rendered.text_between(spot(0, 0), spot(0, 0)), "");
    }

    #[test]
    fn the_cursor_starts_on_the_first_row_with_text() {
        let rendered = rendered("a\n\nb", 40);
        assert_eq!(rendered.first_spot(1, 5), spot(2, 0));
    }

    #[test]
    fn a_screen_of_blank_rows_keeps_the_cursor_at_the_top() {
        let rendered = rendered("a\n\nb", 40);
        assert_eq!(rendered.first_spot(1, 1), spot(1, 0));
    }

    #[test]
    fn text_includes_both_ends_and_stops_at_each_row_end() {
        let rendered = rendered("abc\n\ndef", 40);
        assert_eq!(rendered.text_between(spot(0, 1), spot(0, 1)), "b");
        assert_eq!(rendered.text_between(spot(0, 1), spot(2, 0)), "bc\n\nd");
        assert_eq!(rendered.text_between(spot(1, 0), spot(1, 0)), "");
    }

    #[test]
    fn padding_at_the_end_of_a_table_row_is_not_copied() {
        let rendered = rendered("| aaaa |\n| - |\n| b |\n", 40);
        assert_eq!(rendered.moved(spot(2, 0), Motion::RowEnd), spot(2, 0));
        assert_eq!(rendered.text_between(spot(2, 0), spot(2, 3)), "b");
    }

    #[test]
    fn a_wrapped_paragraph_is_copied_row_by_row() {
        let rendered = rendered("one two", 4);
        assert_eq!(rendered.text_between(spot(0, 0), spot(1, 2)), "one\ntwo");
    }

    #[test]
    fn a_spot_on_screen_counts_columns_and_not_characters() {
        let rendered = rendered("日本 a", 40);
        assert_eq!(rendered.spot_at(0, 0, 0), spot(0, 0));
        assert_eq!(rendered.spot_at(0, 1, 0), spot(0, 0));
        assert_eq!(rendered.spot_at(0, 2, 0), spot(0, 1));
        assert_eq!(rendered.spot_at(0, 5, 0), spot(0, 3));
    }

    #[test]
    fn a_spot_past_the_row_or_page_is_the_last_one_there() {
        let rendered = rendered("ab\n\nabcd", 40);
        assert_eq!(rendered.spot_at(0, 30, 0), spot(0, 1));
        assert_eq!(rendered.spot_at(9, 30, 0), spot(2, 3));
    }

    #[test]
    fn a_spot_in_a_scrolled_block_is_found_where_it_is_shown() {
        let rendered = rendered("```\n0123456789012345\n```", 10);
        assert_eq!(rendered.spot_at(0, 3, 6), spot(0, 9));
        assert_eq!(rendered.spot_at(0, 3, 0), spot(0, 3));
    }

    #[test]
    fn a_scrolled_quote_keeps_its_bar_out_of_the_shift() {
        let rendered = rendered("> ```\n> 0123456789012345\n> ```", 10);
        assert_eq!(rendered.spot_at(0, 0, 4), spot(0, 0));
        assert_eq!(rendered.spot_at(0, 3, 4), spot(0, 7));
    }

    #[test]
    fn the_view_moves_sideways_to_show_a_spot_in_a_block() {
        let rendered = rendered("```\n0123456789012345\n```\n\nplain", 10);
        assert_eq!(rendered.left_showing_spot(spot(0, 15), 0), 6);
        assert_eq!(rendered.left_showing_spot(spot(0, 0), 6), 0);
        assert_eq!(rendered.left_showing_spot(spot(0, 4), 2), 2);
        assert_eq!(rendered.left_showing_spot(spot(2, 0), 2), 2);
    }

    fn reversed_text(rendered: &Rendered, selection: (Spot, Spot)) -> Vec<String> {
        let lines = rendered.lines(Theme::DARK, 0, 10, 0, selected(selection));
        lines
            .iter()
            .map(|line| {
                let spans = line.spans.iter();
                spans
                    .filter(|span| span.style.add_modifier.contains(Modifier::REVERSED))
                    .map(|span| span.content.as_ref())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn the_selection_is_highlighted_across_looks_and_rows() {
        let rendered = rendered("ab **cd** ef\n\nghi", 40);
        assert_eq!(
            reversed_text(&rendered, (spot(0, 1), spot(2, 0))),
            ["b cd ef", " ", "g"]
        );
    }

    #[test]
    fn a_selection_keeps_the_look_of_what_it_covers() {
        let rendered = rendered("ab **cd**", 40);
        let lines = rendered.lines(Theme::DARK, 0, 1, 0, selected((spot(0, 3), spot(0, 3))));
        let spans = lines.first().map_or(&[][..], |line| line.spans.as_slice());
        let span = spans.iter().find(|span| span.content == "c").unwrap();
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn a_selection_in_a_scrolled_block_moves_with_it() {
        let rendered = rendered("```\n0123456789012345\n```", 10);
        let selection = selected((spot(0, 8), spot(0, 9)));
        let lines = rendered.lines(Theme::DARK, 0, 1, 6, selection);
        let spans = lines.first().map_or(&[][..], |line| line.spans.as_slice());
        let selected: String = spans
            .iter()
            .filter(|span| span.style.add_modifier.contains(Modifier::REVERSED))
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(selected, "89");
    }

    const TABLE_THEN_MORE: &str = "| a |\n| - |\n| 1 |\n| 2 |\n\n[next](a.md)\n\n## After\n";

    fn failure(line: usize, snoozed: bool) -> Failure {
        Failure {
            line,
            message: "expected a long value, got another".into(),
            snoozed_until: snoozed.then(|| "2099-12-31".into()),
            result: if snoozed {
                RunResult::Snoozed
            } else {
                RunResult::Failed
            },
        }
    }

    fn row(result: RunResult, failures: Vec<Failure>) -> RowResult {
        RowResult { result, failures }
    }

    /// Renders a document whose table on line 1 has these results.
    fn with_results(
        markdown: &str,
        width: u16,
        result: RunResult,
        rows: &BTreeMap<usize, RowResult>,
        failures: &[Failure],
    ) -> Rendered {
        let document = Document::parse(markdown).unwrap();
        let table = TableResult {
            result,
            rows,
            failures,
        };
        document.render(width, &HashMap::from([(1, table)]))
    }

    fn with_rows(markdown: &str, width: u16, rows: &BTreeMap<usize, RowResult>) -> Rendered {
        with_results(markdown, width, RunResult::Failed, rows, &[])
    }

    fn shown(rendered: &Rendered) -> Vec<String> {
        let lines = rendered.lines(Theme::DARK, 0, usize::MAX, 0, Marks::default());
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_result_goes_below_its_table_and_moves_the_rest_down() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let before = document.render(60, &HashMap::new());
        let failures = [failure(1, false)];
        let after = with_results(
            TABLE_THEN_MORE,
            60,
            RunResult::Failed,
            &BTreeMap::new(),
            &failures,
        );
        assert_eq!(
            shown(&after),
            [
                "a",
                "─",
                "1",
                "2",
                "",
                "✗ failed",
                "  line 1: expected a long value, got another",
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
    fn every_result_has_its_mark_below_the_table() {
        let results = [
            (RunResult::Passed, "✓ passed"),
            (RunResult::Failed, "✗ failed"),
            (RunResult::Error, "! error"),
            (RunResult::Snoozed, "~ snoozed"),
            (RunResult::Skipped, "- skipped"),
            (RunResult::Partial, "· partial"),
            (RunResult::Running, "⋯ running"),
        ];
        for (result, line) in results {
            let rendered = with_results(TABLE_THEN_MORE, 40, result, &BTreeMap::new(), &[]);
            assert_eq!(shown(&rendered).get(5).map(String::as_str), Some(line));
        }
    }

    #[test]
    fn a_row_gets_a_mark_two_columns_after_the_table_and_a_note_below() {
        let rows = BTreeMap::from([
            (3, row(RunResult::Passed, Vec::new())),
            (4, row(RunResult::Failed, vec![failure(4, false)])),
        ]);
        let rendered = with_rows(TABLE_THEN_MORE, 60, &rows);
        assert_eq!(
            shown(&rendered).get(..5),
            Some(
                &[
                    "a".to_owned(),
                    "─".to_owned(),
                    "1  ✓".to_owned(),
                    "2  ✗".to_owned(),
                    "  expected a long value, got another".to_owned(),
                ][..]
            )
        );
    }

    #[test]
    fn marks_line_up_when_a_cell_is_narrower_than_its_column() {
        let markdown = "| wide |\n| ---- |\n| 1    |\n| 22   |\n";
        let rows = BTreeMap::from([
            (3, row(RunResult::Passed, Vec::new())),
            (4, row(RunResult::Passed, Vec::new())),
        ]);
        let rendered = with_rows(markdown, 40, &rows);
        let lines = shown(&rendered);
        assert_eq!(lines.get(2).map(String::as_str), Some("1     ✓"));
        assert_eq!(lines.get(3).map(String::as_str), Some("22    ✓"));
    }

    #[test]
    fn a_row_with_several_problems_has_a_note_for_each() {
        let failures = vec![failure(3, false), failure(3, true)];
        let rows = BTreeMap::from([(3, row(RunResult::Failed, failures))]);
        let rendered = with_rows(TABLE_THEN_MORE, 80, &rows);
        assert_eq!(
            shown(&rendered).get(3..5),
            Some(
                &[
                    "  expected a long value, got another".to_owned(),
                    "  expected a long value, got another (snoozed until 2099-12-31)".to_owned(),
                ][..]
            )
        );
    }

    #[test]
    fn a_long_note_wraps_under_its_row() {
        let rows = BTreeMap::from([(3, row(RunResult::Failed, vec![failure(3, false)]))]);
        let rendered = with_rows("| a |\n| - |\n| 1 |\n", 20, &rows);
        assert_eq!(
            shown(&rendered).get(3..6),
            Some(
                &[
                    "  expected a long".to_owned(),
                    "  value, got another".to_owned(),
                    String::new(),
                ][..]
            )
        );
    }

    #[test]
    fn rows_are_found_past_the_notes_above_them() {
        let rows = BTreeMap::from([(3, row(RunResult::Failed, vec![failure(3, false)]))]);
        let rendered = with_rows(TABLE_THEN_MORE, 60, &rows);
        assert_eq!(rendered.row_of_table_line(3), Some(2));
        assert_eq!(rendered.row_of_table_line(4), Some(4));
        assert_eq!(rendered.table_line_at(4), Some(4));
        assert_eq!(rendered.table_line_at(3), None);
        assert_eq!(rendered.table_line_at(0), None);
    }

    #[test]
    fn the_header_is_not_a_row() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let rendered = document.render(40, &HashMap::new());
        assert_eq!(rendered.row_of_table_line(1), None);
        assert_eq!(rendered.row_of_table_line(3), Some(2));
    }

    fn colors_of(rendered: &Rendered, row: usize) -> Vec<Option<Color>> {
        let lines = rendered.lines(Theme::DARK, row, 1, 0, Marks::default());
        let spans = lines.first().map_or(&[][..], |line| line.spans.as_slice());
        spans.iter().map(|span| span.style.fg).collect()
    }

    fn row_color(rendered: &Rendered, row: usize) -> Option<Color> {
        colors_of(rendered, row).first().copied().flatten()
    }

    #[test]
    fn a_result_colours_its_whole_row_and_not_the_header() {
        let rows = BTreeMap::from([(4, row(RunResult::Passed, Vec::new()))]);
        let rendered = with_rows(TABLE_THEN_MORE, 40, &rows);
        assert_eq!(row_color(&rendered, 0), None);
        assert_eq!(row_color(&rendered, 2), None);
        let colors = colors_of(&rendered, 3);
        assert_ne!(colors.len(), 0);
        assert!(colors.iter().all(|fg| *fg == Some(Theme::DARK.passed)));
    }

    #[test]
    fn each_result_has_its_row_colour() {
        let colors = [
            (RunResult::Passed, Some(Theme::DARK.passed)),
            (RunResult::Failed, Some(Theme::DARK.failed)),
            (RunResult::Error, Some(Theme::DARK.failed)),
            (RunResult::Snoozed, Some(Theme::DARK.snoozed)),
            (RunResult::Skipped, None),
            (RunResult::Running, None),
        ];
        for (result, color) in colors {
            let rows = BTreeMap::from([(3, row(result, Vec::new()))]);
            let rendered = with_rows(TABLE_THEN_MORE, 40, &rows);
            assert_eq!(row_color(&rendered, 2), color, "{result}");
        }
    }

    fn note_color(failure: Failure) -> Option<Color> {
        let rows = BTreeMap::from([(3, row(failure.result, vec![failure]))]);
        let rendered = with_rows(TABLE_THEN_MORE, 80, &rows);
        colors_of(&rendered, 3).last().copied().flatten()
    }

    #[test]
    fn a_note_is_quieter_than_its_row() {
        assert_eq!(note_color(failure(3, false)), Some(Theme::DARK.note));
    }

    #[test]
    fn a_snoozed_note_uses_the_quote_colour() {
        assert_eq!(note_color(failure(3, true)), Some(Theme::DARK.quote));
    }

    #[test]
    fn a_failure_outside_a_row_is_as_quiet_as_a_note() {
        let failures = [failure(1, false)];
        let rendered = with_results(
            TABLE_THEN_MORE,
            80,
            RunResult::Error,
            &BTreeMap::new(),
            &failures,
        );
        let colors = colors_of(&rendered, 6);
        assert_eq!(colors.last().copied().flatten(), Some(Theme::DARK.note));
    }

    #[test]
    fn a_wide_table_can_scroll_to_its_marks() {
        let markdown = "| aaaaaaaaaaaa |\n| ------------ |\n| 1            |\n";
        let rows = BTreeMap::from([(3, row(RunResult::Passed, Vec::new()))]);
        let rendered = with_rows(markdown, 10, &rows);
        assert_eq!(rendered.last_left(), 5);
        let lines = rendered.lines(Theme::DARK, 2, 1, 5, Marks::default());
        let text = lines.first().map(ToString::to_string).unwrap_or_default();
        assert!(text.ends_with('✓'), "{text}");
    }

    #[test]
    fn a_wide_table_keeps_its_colour_with_the_mark_out_of_sight() {
        let markdown = "| aaaaaaaaaaaa |\n| ------------ |\n| 1            |\n";
        let rows = BTreeMap::from([(3, row(RunResult::Failed, Vec::new()))]);
        let rendered = with_rows(markdown, 10, &rows);
        let lines = rendered.lines(Theme::DARK, 2, 1, 0, Marks::default());
        let text = lines.first().map(ToString::to_string).unwrap_or_default();
        assert!(!text.contains('✗'), "{text}");
        assert_eq!(row_color(&rendered, 2), Some(Theme::DARK.failed));
    }

    #[test]
    fn a_table_without_row_results_keeps_its_width() {
        let rendered = with_rows("| aaaaaaaaaaaa |\n| - |\n| 1 |\n", 10, &BTreeMap::new());
        assert_eq!(rendered.last_left(), 2);
    }

    #[test]
    fn the_header_and_the_focused_row_are_both_reversed() {
        let document = Document::parse(TABLE_THEN_MORE).unwrap();
        let rendered = document.render(40, &HashMap::new());
        let marks = Marks {
            header: Some(0),
            row: Some(3),
            ..Marks::default()
        };
        let lines = rendered.lines(Theme::DARK, 0, 4, 0, marks);
        let reversed: Vec<bool> = lines
            .iter()
            .map(|line| {
                let mut spans = line.spans.iter();
                spans.all(|span| span.style.add_modifier.contains(Modifier::REVERSED))
            })
            .collect();
        assert_eq!(reversed, [true, false, false, true]);
    }

    #[test]
    fn the_focused_row_is_reversed_whole() {
        let document = Document::parse("| a | b |\n| - | - |\n| 1 | 2 |\n").unwrap();
        let rendered = document.render(40, &HashMap::new());
        let marks = Marks {
            header: Some(0),
            ..Marks::default()
        };
        let lines = rendered.lines(Theme::DARK, 0, 1, 0, marks);
        let spans = lines.first().map_or(&[][..], |line| line.spans.as_slice());
        assert_ne!(spans.len(), 0);
        for span in spans {
            assert!(span.style.add_modifier.contains(Modifier::REVERSED));
        }
    }
}
