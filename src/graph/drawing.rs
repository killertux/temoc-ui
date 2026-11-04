//! Working out where the graph's files and lines go.
//!
//! Files sit on even rows, which leaves the odd rows free for lines that have to
//! pass a column by. Between two columns is a gutter, whose upright lines each
//! run along a track of their own unless they cannot meet.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Range;

/// A place in the drawing: how far across and how far down it is.
pub type Point = (usize, usize);

/// How many rows of the drawing one file takes up.
pub const ROW_STEP: usize = 2;
/// The cells a gutter keeps free on each side of its tracks.
const MARGIN: usize = 2;

const UP: u8 = 1;
const DOWN: u8 = 2;
const LEFT: u8 = 4;
const RIGHT: u8 = 8;

/// What is drawn in a cell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ink {
    /// The sides of the cell that lines reach, as a set of bits.
    sides: u8,
    head: Option<char>,
}

impl Ink {
    /// The character to draw.
    #[must_use]
    pub fn symbol(self) -> char {
        self.head.unwrap_or(match self.sides {
            UP | DOWN | 3 => '│',
            5 => '┘',
            6 => '┐',
            7 => '┤',
            9 => '└',
            10 => '┌',
            11 => '├',
            13 => '┴',
            14 => '┬',
            15 => '┼',
            0 => ' ',
            _ => '─',
        })
    }

    fn add(&mut self, other: Self) {
        self.sides |= other.sides;
        self.head = self.head.or(other.head);
    }
}

fn paint(ink: &mut HashMap<Point, Ink>, point: Point, drawn: Ink) {
    ink.entry(point).or_default().add(drawn);
}

/// Draws a straight line.
fn stroke(ink: &mut HashMap<Point, Ink>, from: Point, to: Point) {
    let (mut x, mut y) = from;
    while (x, y) != to {
        let (next, leaving, arriving) = if x < to.0 {
            ((x + 1, y), RIGHT, LEFT)
        } else if x > to.0 {
            ((x - 1, y), LEFT, RIGHT)
        } else if y < to.1 {
            ((x, y + 1), DOWN, UP)
        } else {
            ((x, y - 1), UP, DOWN)
        };
        let side = |sides| Ink { sides, head: None };
        paint(ink, (x, y), side(leaving));
        paint(ink, next, side(arriving));
        (x, y) = next;
    }
}

/// Where a file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Place {
    column: usize,
    index: usize,
    /// How wide its name is.
    width: usize,
}

impl Place {
    fn y(self) -> usize {
        self.index * ROW_STEP
    }
}

/// The file an upright line in a gutter belongs to. Lines of the same file can
/// share a track, because they all lead to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Owner {
    /// The file on the left, or the upper one when they share a column.
    Near(usize),
    /// The file on the right, for a line that came from further away.
    Far(usize),
}

type Track = (usize, Owner);

/// The rows free for lines that pass columns by, as they get used.
#[derive(Debug, Default)]
struct Channels {
    /// The gutters each row is used over, and by which file.
    taken: HashMap<usize, Vec<(Range<usize>, usize)>>,
}

impl Channels {
    /// Finds the closest free row below `y`. Lines of one file can share a row.
    fn claim(&mut self, y: usize, gutters: Range<usize>, owner: usize) -> usize {
        let is_free = |row: &usize| {
            self.taken.get(row).is_none_or(|used| {
                used.iter().all(|(span, user)| {
                    *user == owner || span.end <= gutters.start || gutters.end <= span.start
                })
            })
        };
        let first = y + 1;
        let row = (first..).step_by(ROW_STEP).find(is_free).unwrap_or(first);
        self.taken.entry(row).or_default().push((gutters, owner));
        row
    }
}

/// A line between two files, on its way to being drawn.
#[derive(Debug, Clone, Copy)]
struct Route {
    near: usize,
    far: usize,
    /// Which ends get an arrowhead.
    into_near: bool,
    into_far: bool,
    near_place: Place,
    far_place: Place,
    /// The row it passes columns by on, if there are columns in the way.
    channel: Option<usize>,
}

impl Route {
    fn last_gutter(&self) -> usize {
        self.far_place.column.saturating_sub(1)
    }

    /// Stretches the tracks of this line far enough to reach its rows.
    fn reach(&self, spans: &mut BTreeMap<Track, (usize, usize)>) {
        let mut stretch = |track, from: usize, to: usize| {
            let span = spans.entry(track).or_insert((from, from));
            *span = (span.0.min(from).min(to), span.1.max(from).max(to));
        };
        let (near, far) = (self.near_place, self.far_place);
        let first = (near.column, Owner::Near(self.near));
        match self.channel {
            None => stretch(first, near.y(), far.y()),
            Some(channel) => {
                stretch(first, near.y(), channel);
                let last = (self.last_gutter(), Owner::Far(self.far));
                stretch(last, channel, far.y());
            }
        }
    }

    fn drawn(&self, lay_out: &Layout) -> Option<Line> {
        let (near, far) = (self.near_place, self.far_place);
        let start = (lay_out.name_end(near)?, near.y());
        let out = lay_out.track(near.column, Owner::Near(self.near))?;
        let same_column = near.column == far.column;
        let end_x = if same_column {
            lay_out.name_end(far)?
        } else {
            lay_out.starts.get(far.column)?.saturating_sub(1)
        };
        let end = (end_x, far.y());
        let mut points = vec![start, (out, near.y())];
        let across = if let Some(channel) = self.channel {
            let last = lay_out.track(self.last_gutter(), Owner::Far(self.far))?;
            points.extend([(out, channel), (last, channel)]);
            last
        } else {
            out
        };
        if !same_column && lay_out.is_crowded(self.last_gutter(), far.y(), across) {
            let (past, below) = (end_x.saturating_sub(1), far.y() + 1);
            points.extend([(across, below), (past, below), (past, far.y())]);
        } else {
            points.push((across, far.y()));
        }
        points.push(end);
        let mut ink = HashMap::new();
        for segment in points.windows(2) {
            if let [from, to] = segment {
                stroke(&mut ink, *from, *to);
            }
        }
        let head = |head| Ink {
            sides: 0,
            head: Some(head),
        };
        if self.into_near {
            paint(&mut ink, start, head('◀'));
        }
        if self.into_far {
            paint(&mut ink, end, head(if same_column { '◀' } else { '▶' }));
        }
        Some(Line {
            ends: (self.near, self.far),
            ink,
        })
    }
}

/// Where the columns and tracks ended up.
#[derive(Debug)]
struct Layout {
    /// Where each column starts.
    starts: Vec<usize>,
    /// Where the first track of each gutter is.
    gutters: Vec<usize>,
    tracks: HashMap<Track, usize>,
    /// How far into a gutter lines coming out of the column before it reach,
    /// by the gutter and the row.
    exits: HashMap<(usize, usize), usize>,
}

impl Layout {
    /// The cell just after a file's name.
    fn name_end(&self, place: Place) -> Option<usize> {
        Some(self.starts.get(place.column)? + place.width)
    }

    fn track(&self, gutter: usize, owner: Owner) -> Option<usize> {
        Some(self.gutters.get(gutter)? + self.tracks.get(&(gutter, owner))?)
    }

    fn note_exits(&mut self, routes: &[Route]) {
        for route in routes {
            let (near, far) = (route.near_place, route.far_place);
            let Some(track) = self.track(near.column, Owner::Near(route.near)) else {
                continue;
            };
            let mut note = |row| {
                let exit = self.exits.entry((near.column, row)).or_default();
                *exit = track.max(*exit);
            };
            note(near.y());
            if near.column == far.column {
                note(far.y());
            }
        }
    }

    /// Whether a line from `x` to the file on this row would run along a line
    /// coming out of the file across the gutter, and look connected to it.
    fn is_crowded(&self, gutter: usize, row: usize, x: usize) -> bool {
        self.exits.get(&(gutter, row)).is_some_and(|exit| *exit > x)
    }
}

/// A line of the drawing.
#[derive(Debug, Clone)]
struct Line {
    ends: (usize, usize),
    ink: HashMap<Point, Ink>,
}

/// The files and lines of the graph, as a drawing bigger than the screen.
#[derive(Debug, Clone, Default)]
pub struct Drawing {
    starts: Vec<usize>,
    places: HashMap<usize, Place>,
    lines: Vec<Line>,
    ink: HashMap<Point, Ink>,
    width: usize,
    height: usize,
}

impl Drawing {
    /// Lays out `columns` of files, given how wide each file's name is and
    /// the links between them as `(from, to)`.
    #[must_use]
    pub fn lay_out(
        columns: &[Vec<usize>],
        widths: &[usize],
        links: &BTreeSet<(usize, usize)>,
    ) -> Self {
        let places = places_of(columns, widths);
        let routes = routes_of(links, &places);
        let mut spans = BTreeMap::new();
        for route in &routes {
            route.reach(&mut spans);
        }
        let (tracks, counts) = assign_tracks(&spans);
        let mut lay_out = measure(columns, &places, tracks, &counts);
        lay_out.note_exits(&routes);
        let drawn: Vec<Line> = routes
            .iter()
            .filter_map(|route| route.drawn(&lay_out))
            .collect();
        let mut ink = HashMap::new();
        for (point, part) in drawn.iter().flat_map(|line| &line.ink) {
            paint(&mut ink, *point, *part);
        }
        let mut drawing = Self {
            starts: lay_out.starts,
            places,
            lines: drawn,
            ink,
            width: 0,
            height: 0,
        };
        drawing.measure_size();
        drawing
    }

    fn measure_size(&mut self) {
        let names = self.places.values().map(|place| {
            let start = self.starts.get(place.column).copied().unwrap_or(0);
            (start + place.width, place.y() + 1)
        });
        let lines = self.ink.keys().map(|(x, y)| (x + 1, y + 1));
        let (width, height) = names
            .chain(lines)
            .fold((0, 0), |size, end| (size.0.max(end.0), size.1.max(end.1)));
        (self.width, self.height) = (width, height);
    }

    /// How wide the drawing is.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// How tall the drawing is.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Where a column starts.
    #[must_use]
    pub fn column_start(&self, column: usize) -> usize {
        self.starts.get(column).copied().unwrap_or(self.width)
    }

    /// What is drawn at a point.
    #[must_use]
    pub fn ink(&self, point: Point) -> Option<Ink> {
        self.ink.get(&point).copied()
    }

    /// Every point the lines touching a file go through.
    #[must_use]
    pub fn touching(&self, file: usize) -> HashSet<Point> {
        let lines = self
            .lines
            .iter()
            .filter(|line| line.ends.0 == file || line.ends.1 == file);
        lines.flat_map(|line| line.ink.keys().copied()).collect()
    }

    /// The column and position in it of the file whose name is at a point.
    #[must_use]
    pub fn file_at(&self, (x, y): Point) -> Option<(usize, usize)> {
        let place = self.places.values().find(|place| {
            let start = self.column_start(place.column);
            place.y() == y && (start..start + place.width).contains(&x)
        })?;
        Some((place.column, place.index))
    }
}

fn places_of(columns: &[Vec<usize>], widths: &[usize]) -> HashMap<usize, Place> {
    let mut places = HashMap::new();
    for (column, files) in columns.iter().enumerate() {
        for (index, file) in files.iter().enumerate() {
            let width = widths.get(*file).copied().unwrap_or(0);
            let place = Place {
                column,
                index,
                width,
            };
            places.insert(*file, place);
        }
    }
    places
}

/// Works out the line of each pair of linked files, which share just one.
fn routes_of(links: &BTreeSet<(usize, usize)>, places: &HashMap<usize, Place>) -> Vec<Route> {
    let mut pairs: BTreeMap<(usize, usize), (bool, bool)> = BTreeMap::new();
    for (from, to) in links {
        let (Some(start), Some(end)) = (places.get(from), places.get(to)) else {
            continue;
        };
        if (start.column, start.index) < (end.column, end.index) {
            pairs.entry((*from, *to)).or_default().1 = true;
        } else {
            pairs.entry((*to, *from)).or_default().0 = true;
        }
    }
    let mut channels = Channels::default();
    pairs
        .into_iter()
        .filter_map(|((near, far), (into_near, into_far))| {
            let (near_place, far_place) = (*places.get(&near)?, *places.get(&far)?);
            let channel = (far_place.column > near_place.column + 1).then(|| {
                let gutters = near_place.column..far_place.column;
                channels.claim(near_place.y(), gutters, near)
            });
            Some(Route {
                near,
                far,
                into_near,
                into_far,
                near_place,
                far_place,
                channel,
            })
        })
        .collect()
}

/// Gives each upright line a track, using as few as it can. Says which track
/// each one got, and how many each gutter needs.
fn assign_tracks(
    spans: &BTreeMap<Track, (usize, usize)>,
) -> (HashMap<Track, usize>, HashMap<usize, usize>) {
    let mut ordered: Vec<(&Track, &(usize, usize))> = spans.iter().collect();
    ordered.sort_by_key(|(track, span)| (track.0, **span));
    let mut ends: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut tracks = HashMap::new();
    for (track, (start, end)) in ordered {
        let gutter = ends.entry(track.0).or_default();
        // A line can take the row below its end, so that row stays free.
        let free = gutter.iter().position(|last| last + 1 < *start);
        let position = free.unwrap_or(gutter.len());
        match gutter.get_mut(position) {
            Some(last) => *last = *end,
            None => gutter.push(*end),
        }
        tracks.insert(*track, position);
    }
    let counts = ends
        .into_iter()
        .map(|(gutter, tracks)| (gutter, tracks.len()))
        .collect();
    (tracks, counts)
}

fn measure(
    columns: &[Vec<usize>],
    places: &HashMap<usize, Place>,
    tracks: HashMap<Track, usize>,
    counts: &HashMap<usize, usize>,
) -> Layout {
    let mut starts = Vec::with_capacity(columns.len());
    let mut gutters = Vec::with_capacity(columns.len());
    let mut x = 0;
    for (column, files) in columns.iter().enumerate() {
        starts.push(x);
        let found = files.iter().filter_map(|file| places.get(file));
        x += found.map(|place| place.width).max().unwrap_or(0);
        gutters.push(x + MARGIN);
        let count = counts.get(&column).copied().unwrap_or(0).max(1);
        x += count + MARGIN * 2;
    }
    Layout {
        starts,
        gutters,
        tracks,
        exits: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lays out files whose names are all four cells wide.
    fn drawing(columns: &[&[usize]], links: &[(usize, usize)]) -> Drawing {
        let columns: Vec<Vec<usize>> = columns.iter().map(|files| files.to_vec()).collect();
        let count = columns.iter().flatten().max().map_or(0, |last| last + 1);
        Drawing::lay_out(&columns, &vec![4; count], &links.iter().copied().collect())
    }

    fn symbols(drawing: &Drawing) -> String {
        let rows = (0..drawing.height()).map(|y| {
            let row: String = (0..drawing.width())
                .map(|x| drawing.ink((x, y)).map_or(' ', Ink::symbol))
                .collect();
            row.trim_end().to_owned()
        });
        rows.collect::<Vec<_>>().join("\n")
    }

    fn count(drawing: &Drawing, symbol: char) -> usize {
        symbols(drawing).chars().filter(|c| *c == symbol).count()
    }

    #[test]
    fn a_link_ends_in_an_arrowhead_at_its_target() {
        assert_eq!(symbols(&drawing(&[&[0], &[1]], &[(0, 1)])), "    ────▶");
    }

    #[test]
    fn a_link_to_the_left_points_left() {
        assert_eq!(symbols(&drawing(&[&[0], &[1]], &[(1, 0)])), "    ◀────");
    }

    #[test]
    fn two_files_linking_each_other_share_a_line() {
        assert_eq!(
            symbols(&drawing(&[&[0], &[1]], &[(0, 1), (1, 0)])),
            "    ◀───▶"
        );
    }

    #[test]
    fn a_file_shares_its_track_between_its_links() {
        assert_eq!(
            symbols(&drawing(&[&[0], &[1, 2]], &[(0, 1), (0, 2)])),
            "    ──┬─▶\n      │\n      └─▶"
        );
    }

    #[test]
    fn a_link_within_a_column_goes_around_through_the_gutter() {
        assert_eq!(
            symbols(&drawing(&[&[0, 1]], &[(0, 1)])),
            "    ──┐\n      │\n    ◀─┘"
        );
    }

    #[test]
    fn overlapping_lines_of_different_files_get_their_own_tracks() {
        let drawn = drawing(&[&[0, 1, 2], &[3, 4, 5]], &[(0, 5), (1, 4)]);
        assert_eq!(drawn.column_start(1), 4 + 6);
    }

    #[test]
    fn a_line_into_a_file_steps_around_a_line_out_of_the_file_across_from_it() {
        assert_eq!(
            symbols(&drawing(&[&[0, 1], &[2, 3]], &[(0, 3), (1, 2)])),
            "    ──┐┌─▶\n      ││\n    ──┼┘┌▶\n      └─┘"
        );
    }

    #[test]
    fn a_line_straight_across_stays_straight() {
        let drawn = drawing(&[&[0, 1], &[2, 3]], &[(0, 2), (1, 3)]);
        assert_eq!(symbols(&drawn), "    ────▶\n\n    ────▶");
    }

    #[test]
    fn lines_that_cannot_meet_share_a_track() {
        let lines = drawing(&[&[0, 1, 2, 3], &[4, 5, 6, 7]], &[(0, 4), (3, 7)]);
        assert_eq!(lines.column_start(1), 4 + 5);
    }

    #[test]
    fn a_line_past_a_column_stays_off_the_rows_of_its_files() {
        let drawn = drawing(&[&[0], &[1, 2], &[3]], &[(0, 3)]);
        for x in 9..13 {
            for y in [0, 2] {
                assert_eq!(drawn.ink((x, y)), None, "at {x}, {y}");
            }
        }
        assert_eq!(count(&drawn, '▶'), 1);
    }

    #[test]
    fn lines_of_different_files_do_not_share_a_row_past_a_column() {
        let drawn = drawing(&[&[0, 1], &[2], &[3, 4]], &[(0, 3), (1, 4)]);
        assert_eq!(drawn.ink((10, 1)).map(Ink::symbol), Some('─'));
        assert_eq!(drawn.ink((10, 3)).map(Ink::symbol), Some('─'));
    }

    #[test]
    fn lines_of_one_file_share_a_row_past_a_column() {
        let drawn = drawing(&[&[0], &[1], &[2, 3]], &[(0, 2), (0, 3)]);
        assert_eq!(drawn.ink((10, 1)).map(Ink::symbol), Some('─'));
        assert_eq!(drawn.ink((10, 3)), None);
    }

    #[test]
    fn only_the_lines_of_a_file_touch_it() {
        let drawn = drawing(&[&[0], &[1, 2]], &[(0, 1), (0, 2)]);
        assert_eq!(drawn.touching(2).len(), 7);
        assert_eq!(drawn.touching(1).len(), 5);
        assert_eq!(drawn.touching(0).len(), drawn.ink.len());
    }

    #[test]
    fn names_are_found_by_where_they_are_drawn() {
        let drawn = drawing(&[&[0], &[1, 2]], &[]);
        assert_eq!(drawn.file_at((0, 0)), Some((0, 0)));
        assert_eq!(drawn.file_at((3, 0)), Some((0, 0)));
        assert_eq!(drawn.file_at((4, 0)), None);
        assert_eq!(drawn.file_at((9, 2)), Some((1, 1)));
        assert_eq!(drawn.file_at((9, 1)), None);
    }

    #[test]
    fn nothing_to_draw_has_no_size() {
        let drawn = drawing(&[], &[]);
        assert_eq!((drawn.width(), drawn.height()), (0, 0));
        assert_eq!(drawn.column_start(3), 0);
    }
}
