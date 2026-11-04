//! The graph view: every markdown file of the workspace and how they link.

mod drawing;

use crate::render::Document;
use crate::workspace;
use drawing::{Drawing, ROW_STEP};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

/// The file a workspace starts from, if it has one.
const START_PAGE: &str = "README.md";
/// How tall the panel below the graph is, border included.
const PANEL_HEIGHT: u16 = 8;
/// What the panel lists when there is nothing to list.
const NOTHING: &str = "none";

/// What the graph view currently shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphState {
    /// The files in each column, from left to right.
    pub columns: Vec<Vec<String>>,
    /// Every link in the graph, as `(from, to)`.
    pub links: Vec<(String, String)>,
    /// The selected file.
    pub selected: Option<String>,
    /// Links from the start page to the selected file, if it can be reached.
    pub distance: Option<usize>,
    /// The panel's list of links out of the selected file.
    pub links_out: Vec<String>,
    /// The panel's list of links into the selected file.
    pub links_in: Vec<String>,
    /// The part of the screen holding the graph, without the panel.
    pub area: Rect,
}

/// What the files of a workspace link to, by their position among the files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Links {
    /// Links between files, as `(from, to)`.
    between: BTreeSet<(usize, usize)>,
    /// The links of each file to files that do not exist.
    missing: HashMap<usize, BTreeSet<String>>,
}

impl Links {
    fn read(workspace: &Path, files: &[String]) -> Self {
        let positions: HashMap<&str, usize> = files
            .iter()
            .enumerate()
            .map(|(position, file)| (file.as_str(), position))
            .collect();
        let mut links = Self::default();
        for (from, file) in files.iter().enumerate() {
            for path in linked_paths(workspace, file) {
                if path == *file || is_outside(&path) {
                    continue;
                }
                if let Some(to) = positions.get(path.as_str()) {
                    links.between.insert((from, *to));
                } else if !workspace.join(&path).is_file() {
                    links.missing.entry(from).or_default().insert(path);
                }
            }
        }
        links
    }
}

/// Where the links of a file lead. A file that cannot be read or parsed still
/// has its place in the graph, and opening it says what is wrong.
fn linked_paths(workspace: &Path, file: &str) -> Vec<String> {
    let parsed = fs::read_to_string(workspace.join(file))
        .map_err(|error| error.to_string())
        .and_then(|text| Document::parse(&text));
    match parsed {
        Ok(document) => document
            .link_urls()
            .filter_map(|url| workspace::link_path(file, url))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Whether a path from a link leads out of the workspace, or to the workspace
/// folder itself.
fn is_outside(path: &str) -> bool {
    path.is_empty() || path == ".." || path.starts_with("../")
}

/// How many links away from the start each file is, if it can be reached.
fn distances_from(
    start: Option<usize>,
    count: usize,
    links: &BTreeSet<(usize, usize)>,
) -> Vec<Option<usize>> {
    let mut distances = vec![None; count];
    let mut waiting = VecDeque::new();
    if let Some(start) = start {
        waiting.push_back((start, 0));
    }
    while let Some((file, distance)) = waiting.pop_front() {
        match distances.get_mut(file) {
            Some(found @ None) => *found = Some(distance),
            _ => continue,
        }
        for (_, to) in links.range((file, 0)..=(file, usize::MAX)) {
            waiting.push_back((*to, distance + 1));
        }
    }
    distances
}

/// Puts each file in the column of its distance, and the rest in a last one.
fn columns_by(distances: &[Option<usize>]) -> Vec<Vec<usize>> {
    let deepest = distances.iter().flatten().max();
    let mut columns = vec![Vec::new(); deepest.map_or(0, |deepest| deepest + 1)];
    let mut unreachable = Vec::new();
    for (file, distance) in distances.iter().enumerate() {
        let column = distance.and_then(|distance| columns.get_mut(distance));
        column.unwrap_or(&mut unreachable).push(file);
    }
    if !unreachable.is_empty() {
        columns.push(unreachable);
    }
    columns
}

/// Sorts the way files are sorted everywhere else.
fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort_by_key(|name| name.to_lowercase());
    names
}

/// A list for the panel, cut down to fit in `room` rows.
fn fitted(items: &[String], room: usize) -> Vec<String> {
    if items.is_empty() {
        return vec![NOTHING.into()];
    }
    if items.len() <= room {
        return items.to_vec();
    }
    let shown = room.saturating_sub(1);
    let rest = items.len().saturating_sub(shown);
    let mut fitted: Vec<String> = items.iter().take(shown).cloned().collect();
    fitted.push(format!("and {rest} more"));
    fitted
}

/// The graph view.
#[derive(Debug, Clone)]
pub struct Graph {
    files: Vec<String>,
    links: Links,
    distances: Vec<Option<usize>>,
    columns: Vec<Vec<usize>>,
    drawing: Drawing,
    /// The column of the selected file, and its position there.
    selected: Option<(usize, usize)>,
    /// The first row of the drawing on screen.
    top: usize,
    /// The first column of files on screen.
    left: usize,
    /// Whether the view follows the selection. The wheel lets the selection
    /// go off screen, until it next moves.
    follow: bool,
    /// Where the graph was last drawn.
    area: Rect,
}

impl Graph {
    /// Reads the links of a workspace, with `document` selected.
    #[must_use]
    pub fn open(workspace: &Path, document: Option<&str>) -> Self {
        let files = workspace::markdown_files(workspace);
        let links = Links::read(workspace, &files);
        Self::over(files, links, document)
    }

    fn over(files: Vec<String>, links: Links, document: Option<&str>) -> Self {
        let position = |wanted: &str| files.iter().position(|file| file == wanted);
        let start = position(START_PAGE).or((!files.is_empty()).then_some(0));
        let distances = distances_from(start, files.len(), &links.between);
        let columns = columns_by(&distances);
        let widths: Vec<usize> = files
            .iter()
            .map(|file| Line::from(file.as_str()).width())
            .collect();
        let drawing = Drawing::lay_out(&columns, &widths, &links.between);
        let chosen = document.and_then(position).or(start);
        let selected = chosen.and_then(|chosen| spot_of(&columns, chosen));
        Self {
            files,
            links,
            distances,
            columns,
            drawing,
            selected,
            top: 0,
            left: 0,
            follow: true,
            area: Rect::default(),
        }
    }

    fn name(&self, file: usize) -> Option<&String> {
        self.files.get(file)
    }

    fn selected_position(&self) -> Option<usize> {
        let (column, index) = self.selected?;
        self.columns.get(column)?.get(index).copied()
    }

    /// The selected file, relative to the workspace.
    #[must_use]
    pub fn chosen_file(&self) -> Option<&str> {
        self.name(self.selected_position()?).map(String::as_str)
    }

    fn links_out(&self, file: usize) -> Vec<String> {
        let links = self.links.between.range((file, 0)..=(file, usize::MAX));
        let found = links.filter_map(|(_, to)| self.name(*to).cloned());
        let missing = self.links.missing.get(&file).into_iter().flatten();
        sorted(
            found
                .chain(missing.map(|path| format!("{path} (missing)")))
                .collect(),
        )
    }

    fn links_in(&self, file: usize) -> Vec<String> {
        let links = self.links.between.iter().filter(|(_, to)| *to == file);
        sorted(
            links
                .filter_map(|(from, _)| self.name(*from).cloned())
                .collect(),
        )
    }

    /// A snapshot of the graph.
    #[must_use]
    pub fn state(&self) -> GraphState {
        let names = |files: &Vec<usize>| {
            let found = files.iter().filter_map(|file| self.name(*file));
            found.cloned().collect()
        };
        let file = self.selected_position();
        GraphState {
            columns: self.columns.iter().map(names).collect(),
            links: self
                .links
                .between
                .iter()
                .filter_map(|(from, to)| Some((self.name(*from)?.clone(), self.name(*to)?.clone())))
                .collect(),
            selected: self.chosen_file().map(String::from),
            distance: file.and_then(|file| self.distances.get(file).copied().flatten()),
            links_out: file.map_or_else(Vec::new, |file| self.links_out(file)),
            links_in: file.map_or_else(Vec::new, |file| self.links_in(file)),
            area: self.area,
        }
    }

    fn select(&mut self, column: usize, index: usize) {
        self.selected = Some((column, index));
        self.follow = true;
    }

    fn column_length(&self, column: usize) -> usize {
        self.columns.get(column).map_or(0, Vec::len)
    }

    /// Selects the next file of the column, stopping at the last one.
    pub fn choose_next(&mut self) {
        if let Some((column, index)) = self.selected {
            let last = self.column_length(column).saturating_sub(1);
            self.select(column, index.saturating_add(1).min(last));
        }
    }

    /// Selects the previous file of the column, stopping at the first one.
    pub fn choose_previous(&mut self) {
        if let Some((column, index)) = self.selected {
            self.select(column, index.saturating_sub(1));
        }
    }

    /// Selects the first file of the column.
    pub fn choose_first(&mut self) {
        if let Some((column, _)) = self.selected {
            self.select(column, 0);
        }
    }

    /// Selects the last file of the column.
    pub fn choose_last(&mut self) {
        if let Some((column, _)) = self.selected {
            self.select(column, self.column_length(column).saturating_sub(1));
        }
    }

    /// Selects the nearest file of the column to the right, if there is one.
    pub fn choose_right(&mut self) {
        if let Some((column, index)) = self.selected {
            self.land_on(column.saturating_add(1), index);
        }
    }

    /// Selects the nearest file of the column to the left, if there is one.
    pub fn choose_left(&mut self) {
        if let Some((column, index)) = self.selected
            && let Some(column) = column.checked_sub(1)
        {
            self.land_on(column, index);
        }
    }

    /// Files sit at even steps, so the nearest one is at the same position, or
    /// else at the end of a shorter column.
    fn land_on(&mut self, column: usize, index: usize) {
        if let Some(last) = self.column_length(column).checked_sub(1) {
            self.select(column, index.min(last));
        }
    }

    /// Reacts to a click on the screen: selects the file there, or says that
    /// it is selected already and should open.
    pub fn click(&mut self, spot: Position) -> bool {
        if !self.area.contains(spot) {
            return false;
        }
        let x = usize::from(spot.x.saturating_sub(self.area.x));
        let y = usize::from(spot.y.saturating_sub(self.area.y));
        let left = self.drawing.column_start(self.left);
        let Some(found) = self.drawing.file_at((left + x, self.top + y)) else {
            return false;
        };
        let open = self.selected == Some(found);
        self.select(found.0, found.1);
        open
    }

    fn last_top(&self) -> usize {
        let room = usize::from(self.area.height);
        self.drawing.height().saturating_sub(room)
    }

    /// The first column that shows everything to its right.
    fn last_left(&self) -> usize {
        let last = self.columns.len().saturating_sub(1);
        let room = usize::from(self.area.width);
        let fits = |column: &usize| {
            let start = self.drawing.column_start(*column);
            self.drawing.width().saturating_sub(start) <= room
        };
        (0..last).find(fits).unwrap_or(last)
    }

    /// Scrolls down, leaving the selection where it is.
    pub fn scroll_down(&mut self, rows: usize) {
        self.top = self.top.saturating_add(rows).min(self.last_top());
        self.follow = false;
    }

    /// Scrolls up, leaving the selection where it is.
    pub fn scroll_up(&mut self, rows: usize) {
        self.top = self.top.saturating_sub(rows);
        self.follow = false;
    }

    /// Scrolls one column to the right, leaving the selection where it is.
    pub fn scroll_right(&mut self) {
        self.left = self.left.saturating_add(1).min(self.last_left());
        self.follow = false;
    }

    /// Scrolls one column to the left, leaving the selection where it is.
    pub fn scroll_left(&mut self) {
        self.left = self.left.saturating_sub(1);
        self.follow = false;
    }

    /// Brings the selected file on screen.
    fn reveal(&mut self) {
        let Some((column, index)) = self.selected else {
            return;
        };
        let y = index * ROW_STEP;
        let room = usize::from(self.area.height).max(1);
        self.top = self.top.min(y).max((y + 1).saturating_sub(room));
        let width = self.selected_position().and_then(|file| self.name(file));
        let width = width.map_or(0, |name| Line::from(name.as_str()).width());
        let end = self.drawing.column_start(column) + width;
        let room = usize::from(self.area.width);
        self.left = self.left.min(column);
        while self.left < column && end > self.drawing.column_start(self.left) + room {
            self.left += 1;
        }
    }

    /// Draws the graph and the panel below it, and adapts to their size.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let panel_height = PANEL_HEIGHT.min(area.height / 2);
        let [graph, panel] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(panel_height)]).areas(area);
        self.area = graph;
        if self.follow {
            self.reveal();
        }
        self.top = self.top.min(self.last_top());
        self.left = self.left.min(self.last_left());
        self.draw_lines(frame.buffer_mut());
        self.draw_names(frame.buffer_mut());
        self.draw_panel(frame, panel);
    }

    /// Draws the lines, with those of the selected file highlighted.
    fn draw_lines(&self, buffer: &mut Buffer) {
        let lit = self
            .selected_position()
            .map_or_else(HashSet::new, |file| self.drawing.touching(file));
        let left = self.drawing.column_start(self.left);
        let area = self.area;
        for y in 0..area.height {
            for x in 0..area.width {
                let point = (left + usize::from(x), self.top + usize::from(y));
                if let Some(ink) = self.drawing.ink(point)
                    && let Some(cell) = buffer.cell_mut((area.x + x, area.y + y))
                {
                    cell.set_char(ink.symbol());
                    if lit.contains(&point) {
                        cell.set_style(lit_style());
                    }
                }
            }
        }
    }

    fn draw_names(&self, buffer: &mut Buffer) {
        let left = self.drawing.column_start(self.left);
        let area = self.area;
        for (column, files) in self.columns.iter().enumerate().skip(self.left) {
            let x = self.drawing.column_start(column).saturating_sub(left);
            let Some(x) = u16::try_from(x).ok().filter(|x| *x < area.width) else {
                break;
            };
            for (index, file) in files.iter().enumerate() {
                let y = (index * ROW_STEP).checked_sub(self.top);
                let y = y.and_then(|y| u16::try_from(y).ok());
                let (Some(y), Some(name)) = (y.filter(|y| *y < area.height), self.name(*file))
                else {
                    continue;
                };
                let style = if self.selected == Some((column, index)) {
                    Style::new().add_modifier(Modifier::REVERSED)
                } else {
                    Style::new()
                };
                let room = usize::from(area.width - x);
                buffer.set_stringn(area.x + x, area.y + y, name, room, style);
            }
        }
    }

    fn draw_panel(&self, frame: &mut Frame, area: Rect) {
        let file = self.selected_position();
        let block = Block::bordered().title(self.chosen_file().unwrap_or("No files").to_owned());
        let inside = block.inner(area);
        frame.render_widget(block, area);
        let [links_out, links_in] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(inside);
        let list = |title: &str, links: Vec<String>, area: Rect| {
            // The title takes a row.
            let room = usize::from(area.height).saturating_sub(1);
            let title = Line::styled(title.to_owned(), Style::new().add_modifier(Modifier::BOLD));
            let links = fitted(&links, room).into_iter().map(Line::from);
            Paragraph::new(std::iter::once(title).chain(links).collect::<Vec<_>>())
        };
        let (out, into) = file.map_or_else(Default::default, |file| {
            (self.links_out(file), self.links_in(file))
        });
        frame.render_widget(list("Links out", out, links_out), links_out);
        frame.render_widget(list("Links in", into, links_in), links_in);
    }
}

/// How the lines of the selected file are drawn.
fn lit_style() -> Style {
    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

fn spot_of(columns: &[Vec<usize>], file: usize) -> Option<(usize, usize)> {
    columns.iter().enumerate().find_map(|(column, files)| {
        let index = files.iter().position(|found| *found == file)?;
        Some((column, index))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Cell;
    use std::{env, process};

    /// A graph of files named by their position, linked as `(from, to)`.
    fn graph(names: &[&str], links: &[(usize, usize)], document: Option<&str>) -> Graph {
        let files = names.iter().map(ToString::to_string).collect();
        let links = Links {
            between: links.iter().copied().collect(),
            missing: HashMap::new(),
        };
        Graph::over(files, links, document)
    }

    /// README.md links to `count` pages, and those to nothing.
    fn fan(count: usize) -> Graph {
        let names: Vec<String> = std::iter::once("README.md".to_owned())
            .chain((1..=count).map(|page| format!("page-{page:02}.md")))
            .collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let links: Vec<(usize, usize)> = (1..=count).map(|page| (0, page)).collect();
        graph(&names, &links, None)
    }

    /// A chain of `count` files.
    fn chain(count: usize) -> Graph {
        let names: Vec<String> = (0..count)
            .map(|page| format!("page-{page:02}.md"))
            .collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let links: Vec<(usize, usize)> = (1..count).map(|page| (page - 1, page)).collect();
        graph(&names, &links, None)
    }

    fn draw(graph: &mut Graph, width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| graph.draw(frame, frame.area()))
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn chosen(graph: &Graph) -> Option<&str> {
        graph.chosen_file()
    }

    #[test]
    fn links_in_a_circle_end() {
        let found = graph(&["a.md", "b.md"], &[(0, 1), (1, 0), (0, 0)], None);
        assert_eq!(found.distances, [Some(0), Some(1)]);
    }

    #[test]
    fn without_files_there_is_nothing_to_select_or_move() {
        let mut empty = graph(&[], &[], Some("gone.md"));
        for act in [
            Graph::choose_next,
            Graph::choose_previous,
            Graph::choose_first,
            Graph::choose_last,
            Graph::choose_left,
            Graph::choose_right,
            Graph::scroll_right,
        ] {
            act(&mut empty);
        }
        draw(&mut empty, 40, 12);
        assert_eq!(empty.state().columns, Vec::<Vec<String>>::new());
        assert_eq!(chosen(&empty), None);
    }

    #[test]
    fn a_document_outside_the_graph_selects_the_start_page() {
        let found = graph(&["a.md", "README.md"], &[], Some(".hidden/a.md"));
        assert_eq!(chosen(&found), Some("README.md"));
    }

    #[test]
    fn moving_to_a_shorter_column_lands_on_its_last_file() {
        let names = ["README.md", "a.md", "b.md", "c.md", "z.md"];
        let mut found = graph(&names, &[(0, 1), (0, 2), (0, 3), (1, 4)], None);
        found.choose_right();
        found.choose_last();
        assert_eq!(chosen(&found), Some("c.md"));
        found.choose_right();
        assert_eq!(chosen(&found), Some("z.md"));
    }

    #[test]
    fn a_short_list_fits_and_a_long_one_says_what_is_left_out() {
        let items: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        assert_eq!(fitted(&[], 3), [NOTHING]);
        assert_eq!(fitted(&items, 3), items);
        assert_eq!(fitted(&items, 2), ["a", "and 2 more"]);
        assert_eq!(fitted(&items, 0), ["and 3 more"]);
    }

    #[test]
    fn the_wheel_stops_at_the_bottom_and_keys_bring_the_selection_back() {
        let mut found = fan(30);
        draw(&mut found, 80, 24);
        let room = usize::from(found.area.height);
        found.scroll_down(1000);
        assert_eq!(found.top, found.drawing.height() - room);
        found.choose_next();
        draw(&mut found, 80, 24);
        assert_eq!(found.top, 0);
    }

    #[test]
    fn the_wheel_stops_when_the_last_column_is_showing() {
        let mut found = chain(12);
        draw(&mut found, 80, 24);
        for _ in 0..20 {
            found.scroll_right();
        }
        let start = found.drawing.column_start(found.left);
        assert!(found.drawing.width() - start <= 80);
        assert!(found.left < 11);
        draw(&mut found, 80, 24);
        assert_eq!(found.left, found.last_left());
    }

    #[test]
    fn a_graph_that_fits_does_not_scroll() {
        let mut found = chain(2);
        draw(&mut found, 80, 24);
        found.scroll_right();
        found.scroll_down(3);
        assert_eq!((found.left, found.top), (0, 0));
    }

    #[test]
    fn a_bigger_screen_takes_back_what_was_scrolled() {
        let mut found = fan(30);
        draw(&mut found, 80, 24);
        found.scroll_down(1000);
        draw(&mut found, 80, 200);
        assert_eq!(found.top, 0);
    }

    #[test]
    fn a_name_wider_than_the_screen_still_shows_its_start() {
        let name = "x".repeat(100);
        let mut found = graph(&["README.md", &name], &[(0, 1)], None);
        found.choose_right();
        draw(&mut found, 40, 24);
        assert_eq!(found.left, 1);
    }

    #[test]
    fn a_tiny_screen_draws_without_trouble() {
        for (width, height) in [(1, 1), (2, 3), (80, 1), (1, 24)] {
            draw(&mut fan(30), width, height);
        }
    }

    #[test]
    fn a_click_between_files_does_nothing() {
        let mut found = fan(3);
        draw(&mut found, 80, 24);
        assert!(!found.click(Position::new(40, 1)));
        assert_eq!(chosen(&found), Some("README.md"));
    }

    #[test]
    fn a_click_finds_files_in_a_scrolled_graph() {
        let mut found = fan(30);
        draw(&mut found, 80, 24);
        found.scroll_down(3);
        found.scroll_right();
        draw(&mut found, 80, 24);
        let left = found.left;
        assert_eq!(left, 0, "this graph fits sideways");
        let x = u16::try_from(found.drawing.column_start(1)).unwrap();
        assert!(!found.click(Position::new(x, 1)));
        assert_eq!(chosen(&found), Some("page-03.md"));
    }

    #[test]
    fn the_lines_of_the_selection_are_highlighted_and_the_others_are_not() {
        let names = ["README.md", "a.md", "b.md"];
        let mut found = graph(&names, &[(0, 1), (0, 2)], Some("a.md"));
        let buffer = draw(&mut found, 80, 24);
        let arrow = |y| {
            let x = found.drawing.column_start(1) - 1;
            let x = u16::try_from(x).unwrap();
            buffer.cell((x, y)).unwrap().clone()
        };
        assert_eq!(arrow(0).symbol(), "▶");
        assert_eq!(arrow(0).fg, Color::Cyan);
        assert_eq!(arrow(2).symbol(), "▶");
        assert_eq!(arrow(2).fg, Color::Reset);
    }

    #[test]
    fn a_highlighted_line_does_not_hide_what_shares_its_cells() {
        let names = ["README.md", "a.md", "b.md"];
        let mut found = graph(&names, &[(0, 1), (1, 0), (2, 0)], Some("b.md"));
        let buffer = draw(&mut found, 80, 24);
        let row: String = (0..20)
            .map(|x| buffer.cell((x, 0)).unwrap().symbol().to_owned())
            .collect();
        assert!(row.starts_with("README.md◀─┬─▶a.md"), "{row}");
    }

    #[test]
    fn the_selected_file_is_reversed() {
        let mut found = fan(2);
        found.choose_right();
        let buffer = draw(&mut found, 80, 24);
        let x = u16::try_from(found.drawing.column_start(1)).unwrap();
        let reversed = |y: u16| {
            let cell = buffer.cell((x, y)).unwrap();
            cell.modifier.contains(Modifier::REVERSED)
        };
        assert!(reversed(0));
        assert!(!reversed(2));
    }

    #[test]
    fn the_panel_cuts_a_long_list_short() {
        let mut found = fan(30);
        let buffer = draw(&mut found, 80, 24);
        let text: String = buffer.content().iter().map(Cell::symbol).collect();
        assert!(text.contains("and 26 more"), "{text}");
    }

    struct Folder(std::path::PathBuf);

    impl Folder {
        fn new(name: &str, files: &[(&str, &str)]) -> Self {
            let folder = env::temp_dir().join(format!("temoc-ui-{name}-{}", process::id()));
            for (file, text) in files {
                let path = folder.join(file);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, text).unwrap();
            }
            Self(folder)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn only_the_links_a_page_can_be_blamed_for_are_listed() {
        let readme = "[a](a.md) [again](a.md#top) [also](./sub/../a.md) \
            [gone](gone.md) [gone too](gone.md#top) \
            [text](data.txt) [secret](.hidden/b.md) [folder](sub) [here](#top) \
            [self](README.md) [up](../other.md) [web](https://example.com/a.md)";
        let folder = Folder::new(
            "graph-links",
            &[
                ("README.md", readme),
                ("a.md", "No links"),
                ("data.txt", "Not a page"),
                (".hidden/b.md", "Not listed"),
                ("sub/c.md", "Not linked"),
            ],
        );
        let state = Graph::open(&folder.0, None).state();
        assert_eq!(
            state.links_out,
            ["a.md", "gone.md (missing)", "sub (missing)"]
        );
        assert_eq!(state.links, [("README.md".to_owned(), "a.md".to_owned())]);
    }

    #[test]
    fn a_file_that_cannot_be_read_keeps_its_place() {
        let folder = Folder::new("graph-unreadable", &[("README.md", "[a](a.md)")]);
        fs::write(folder.0.join("a.md"), [0xff, 0xfe]).unwrap();
        let state = Graph::open(&folder.0, Some("a.md")).state();
        assert_eq!(state.selected.as_deref(), Some("a.md"));
        assert_eq!(state.links_out, [""; 0]);
        assert_eq!(state.links_in, ["README.md"]);
    }
}
