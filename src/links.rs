//! How the markdown files of a workspace link to each other.

use crate::document::Document;
use crate::workspace;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fs;
use std::path::Path;

/// The file a workspace starts from, if it has one.
const START_PAGE: &str = "README.md";

/// What the files of a workspace link to, by their position among the files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Links {
    /// Links between files, as `(from, to)`.
    pub(crate) between: BTreeSet<(usize, usize)>,
    /// The links of each file to files that do not exist.
    pub(crate) missing: HashMap<usize, BTreeSet<String>>,
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

/// Every markdown file of a workspace, and how they link. A file is known by
/// its position among the files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkGraph {
    files: Vec<String>,
    links: Links,
    start: Option<usize>,
    distances: Vec<Option<usize>>,
    columns: Vec<Vec<usize>>,
}

impl LinkGraph {
    /// Reads the links of a workspace.
    #[must_use]
    pub fn read(workspace: &Path) -> Self {
        let files = workspace::markdown_files(workspace);
        let links = Links::read(workspace, &files);
        Self::over(files, links)
    }

    pub(crate) fn over(files: Vec<String>, links: Links) -> Self {
        let start = files
            .iter()
            .position(|file| file == START_PAGE)
            .or((!files.is_empty()).then_some(0));
        let distances = distances_from(start, files.len(), &links.between);
        let columns = columns_by(&distances);
        Self {
            files,
            links,
            start,
            distances,
            columns,
        }
    }

    /// The name of each file.
    #[must_use]
    pub fn files(&self) -> &[String] {
        &self.files
    }

    /// The name of a file, relative to the workspace.
    #[must_use]
    pub fn name(&self, file: usize) -> Option<&String> {
        self.files.get(file)
    }

    /// Finds a file by its name.
    #[must_use]
    pub fn position(&self, name: &str) -> Option<usize> {
        self.files.iter().position(|file| file == name)
    }

    /// The file that distances are counted from.
    #[must_use]
    pub fn start(&self) -> Option<usize> {
        self.start
    }

    /// The files in each column, from left to right.
    #[must_use]
    pub fn columns(&self) -> &[Vec<usize>] {
        &self.columns
    }

    /// The names of the files in each column, from left to right.
    #[must_use]
    pub fn column_names(&self) -> Vec<Vec<String>> {
        let names = |files: &Vec<usize>| {
            let found = files.iter().filter_map(|file| self.name(*file));
            found.cloned().collect()
        };
        self.columns.iter().map(names).collect()
    }

    /// Links between files, as `(from, to)`.
    #[must_use]
    pub fn between(&self) -> &BTreeSet<(usize, usize)> {
        &self.links.between
    }

    /// The names of the files at each end of every link, as `(from, to)`.
    #[must_use]
    pub fn link_names(&self) -> Vec<(String, String)> {
        let links = self.links.between.iter();
        links
            .filter_map(|(from, to)| Some((self.name(*from)?.clone(), self.name(*to)?.clone())))
            .collect()
    }

    /// Links from the start page to a file, if it can be reached.
    #[must_use]
    pub fn distance(&self, file: usize) -> Option<usize> {
        self.distances.get(file).copied().flatten()
    }

    /// The links out of a file, including those to files that do not exist.
    #[must_use]
    pub fn links_out(&self, file: usize) -> Vec<String> {
        let links = self.links.between.range((file, 0)..=(file, usize::MAX));
        let found = links.filter_map(|(_, to)| self.name(*to).cloned());
        let missing = self.links.missing.get(&file).into_iter().flatten();
        sorted(
            found
                .chain(missing.map(|path| format!("{path} (missing)")))
                .collect(),
        )
    }

    /// The links into a file.
    #[must_use]
    pub fn links_in(&self, file: usize) -> Vec<String> {
        let links = self.links.between.iter().filter(|(_, to)| *to == file);
        sorted(
            links
                .filter_map(|(from, _)| self.name(*from).cloned())
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_in_a_circle_end() {
        let links = Links {
            between: [(0, 1), (1, 0), (0, 0)].into_iter().collect(),
            missing: HashMap::new(),
        };
        let found = LinkGraph::over(vec!["a.md".into(), "b.md".into()], links);
        assert_eq!(found.distances, [Some(0), Some(1)]);
    }
}
