//! Where the files of the graph go. Files that link to each other pull
//! together, every file pushes the others away, and all of them drift towards
//! the middle, so that a file with no links floats free without flying off.
//!
//! The moves get smaller with each step until the files stop. Nothing here
//! depends on time or chance, so that a workspace always looks the same.

use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

/// How far apart linked files like to be.
const SPACING: f32 = 110.0;
/// How far a file can move in the first step.
const HEAT: f32 = 40.0;
/// How much of the heat each step keeps.
const COOLING: f32 = 0.985;
/// The heat at which the files have stopped moving.
const COLD: f32 = 0.2;
/// How hard the middle pulls.
const GRAVITY: f32 = 0.04;
/// The turn between one starting place and the next, which spreads them evenly.
const GOLDEN_ANGLE: f32 = 2.399_963;

/// The places of the files, as they settle.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Layout {
    places: Vec<Pos2>,
    links: Vec<(usize, usize)>,
    heat: f32,
}

impl Layout {
    /// Puts this many files in their starting places.
    pub fn new(files: usize, links: impl Iterator<Item = (usize, usize)>) -> Self {
        let mut turn = 0.0_f32;
        let mut count = 1.0_f32;
        let places = (0..files).map(|_| {
            let place = pos2(turn.cos(), turn.sin()) * (SPACING * count.sqrt());
            turn += GOLDEN_ANGLE;
            count += 1.0;
            place
        });
        Self {
            places: places.collect(),
            links: links.collect(),
            heat: HEAT,
        }
    }

    /// Where a file is.
    pub fn place(&self, file: usize) -> Option<Pos2> {
        self.places.get(file).copied()
    }

    /// The smallest rectangle around every file.
    pub fn bounds(&self) -> Rect {
        let mut bounds = Rect::NOTHING;
        for place in &self.places {
            bounds.extend_with(*place);
        }
        bounds
    }

    pub fn is_settled(&self) -> bool {
        self.heat < COLD
    }

    /// Moves every file a little, and cools down.
    pub fn step(&mut self) {
        if self.is_settled() {
            return;
        }
        let forces = self.forces();
        for (place, force) in self.places.iter_mut().zip(forces) {
            let strength = force.length();
            if strength > 0.0 {
                *place += force * (strength.min(self.heat) / strength);
            }
        }
        self.heat *= COOLING;
    }

    /// What pushes and pulls on each file.
    fn forces(&self) -> Vec<Vec2> {
        let mut forces: Vec<Vec2> = self
            .places
            .iter()
            .map(|place| -place.to_vec2() * GRAVITY)
            .collect();
        let mut add = |file: usize, force: Vec2| {
            if let Some(total) = forces.get_mut(file) {
                *total += force;
            }
        };
        for (file, place) in self.places.iter().enumerate() {
            for (other, there) in self.places.iter().enumerate().skip(file + 1) {
                let apart = *place - *there;
                let distance = apart.length().max(1.0);
                let push = apart / distance * (SPACING * SPACING / distance);
                add(file, push);
                add(other, -push);
            }
        }
        for (from, to) in &self.links {
            if let (Some(place), Some(there)) = (self.place(*from), self.place(*to)) {
                let apart = there - place;
                let pull = apart * (apart.length() / SPACING);
                add(*from, pull);
                add(*to, -pull);
            }
        }
        forces
    }
}

/// Which way to go from a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::gui) enum Heading {
    Left,
    Right,
    Up,
    Down,
}

impl Heading {
    fn way(self) -> Vec2 {
        match self {
            Self::Left => vec2(-1.0, 0.0),
            Self::Right => vec2(1.0, 0.0),
            Self::Up => vec2(0.0, -1.0),
            Self::Down => vec2(0.0, 1.0),
        }
    }
}

/// The nearest of `places` in this direction from `from`, which has to be more
/// that way than off to the side of it.
pub(super) fn nearest(places: &[Pos2], from: usize, heading: Heading) -> Option<usize> {
    let start = *places.get(from)?;
    let way = heading.way();
    let found = places.iter().enumerate().filter_map(|(file, place)| {
        let apart = *place - start;
        let along = apart.dot(way);
        let across = apart.dot(way.rot90()).abs();
        (file != from && along > 0.0 && along >= across).then_some((file, apart.length()))
    });
    let closest = found.min_by(|one, other| one.1.total_cmp(&other.1));
    closest.map(|(file, _)| file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settled(files: usize, links: &[(usize, usize)]) -> Layout {
        let mut layout = Layout::new(files, links.iter().copied());
        for _ in 0..1000 {
            layout.step();
        }
        layout
    }

    fn apart(layout: &Layout, one: usize, other: usize) -> f32 {
        let (one, other) = (layout.place(one), layout.place(other));
        one.zip(other)
            .map_or(f32::NAN, |(one, other)| (one - other).length())
    }

    #[test]
    fn the_files_stop_moving() {
        assert!(settled(6, &[(0, 1), (1, 2)]).is_settled());
    }

    #[test]
    fn it_comes_out_the_same_every_time() {
        let links = [(0, 1), (1, 2), (2, 0), (3, 4)];
        assert_eq!(settled(7, &links), settled(7, &links));
    }

    #[test]
    fn linked_files_end_up_closer_than_unlinked_ones() {
        let layout = settled(4, &[(0, 1)]);
        assert!(apart(&layout, 0, 1) < apart(&layout, 0, 2));
        assert!(apart(&layout, 0, 1) < apart(&layout, 1, 3));
    }

    #[test]
    fn no_two_files_end_up_on_top_of_each_other() {
        let links: Vec<(usize, usize)> = (1..30).map(|file| (0, file)).collect();
        let layout = settled(30, &links);
        for file in 0..30 {
            for other in file + 1..30 {
                assert!(apart(&layout, file, other) > 20.0, "{file} and {other}");
            }
        }
    }

    #[test]
    fn a_file_with_no_links_stays_near() {
        let layout = settled(3, &[(0, 1)]);
        assert!(apart(&layout, 0, 2) < 1000.0);
    }

    #[test]
    fn the_nearest_file_is_the_closest_one_that_way() {
        let places = [
            pos2(0.0, 0.0),
            pos2(100.0, 10.0),
            pos2(50.0, -5.0),
            pos2(10.0, 90.0),
        ];
        assert_eq!(nearest(&places, 0, Heading::Right), Some(2));
        assert_eq!(nearest(&places, 0, Heading::Down), Some(3));
        assert_eq!(nearest(&places, 1, Heading::Left), Some(2));
    }

    #[test]
    fn nothing_that_way_leaves_nowhere_to_go() {
        let places = [pos2(0.0, 0.0), pos2(100.0, 0.0)];
        assert_eq!(nearest(&places, 0, Heading::Left), None);
        assert_eq!(nearest(&places, 0, Heading::Up), None);
    }
}
