//! Deciding what opens first, and which files a workspace holds.

use std::fs;
use std::path::{Path, PathBuf};

/// Folders that never hold pages of interest, besides hidden ones.
const SKIPPED_FOLDERS: [&str; 2] = ["target", "node_modules"];

/// Where temoc-ui starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opening {
    /// The folder whose files can be found.
    pub workspace: PathBuf,
    /// The file to open, relative to the workspace.
    pub file: Option<String>,
}

/// Works out the workspace and first file from the optional path given.
///
/// # Errors
///
/// Returns the message to show when the path cannot be opened.
pub fn locate(current_dir: &Path, path: Option<&str>) -> Result<Opening, String> {
    let Some(path) = path else {
        return Ok(open_folder(current_dir));
    };
    let target = current_dir
        .join(path)
        .canonicalize()
        .map_err(|_| format!("Path not found: {path}"))?;
    if target.is_dir() {
        return Ok(open_folder(&target));
    }
    if !is_markdown(&target) {
        return Err(format!("Not a markdown file: {path}"));
    }
    let current = current_dir
        .canonicalize()
        .unwrap_or_else(|_| current_dir.to_path_buf());
    if let Ok(inside) = target.strip_prefix(&current) {
        return Ok(Opening {
            file: Some(relative_name(inside)),
            workspace: current,
        });
    }
    let (Some(folder), Some(name)) = (target.parent(), target.file_name()) else {
        return Err(format!("Path not found: {path}"));
    };
    Ok(Opening {
        workspace: folder.to_path_buf(),
        file: Some(relative_name(Path::new(name))),
    })
}

fn open_folder(folder: &Path) -> Opening {
    let readme = "README.md";
    Opening {
        workspace: folder.to_path_buf(),
        file: folder.join(readme).is_file().then(|| readme.to_owned()),
    }
}

/// Lists the markdown files in a workspace, alphabetically and ignoring case.
#[must_use]
pub fn markdown_files(workspace: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_markdown(workspace, workspace, &mut files);
    files.sort_by_key(|file| file.to_lowercase());
    files
}

/// A folder that cannot be read simply has nothing to list.
fn collect_markdown(workspace: &Path, folder: &Path, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            if !is_skipped_folder(&path) {
                collect_markdown(workspace, &path, files);
            }
        } else if is_markdown(&path)
            && let Ok(inside) = path.strip_prefix(workspace)
        {
            files.push(relative_name(inside));
        }
    }
}

fn is_skipped_folder(folder: &Path) -> bool {
    folder
        .file_name()
        .map(|name| name.to_string_lossy())
        .is_some_and(|name| name.starts_with('.') || SKIPPED_FOLDERS.contains(&name.as_ref()))
}

fn is_markdown(file: &Path) -> bool {
    file.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn relative_name(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// A spot in a file of the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The file, relative to the workspace.
    pub file: String,
    /// The heading to scroll to.
    pub anchor: Option<String>,
}

/// Works out where a link in `document` leads.
///
/// # Errors
///
/// Returns the message to show when the link cannot be opened.
pub fn follow_link(workspace: &Path, document: &str, target: &str) -> Result<Place, String> {
    if is_external(target) {
        return Err(format!("External link: {target}"));
    }
    let (written, anchor) = target
        .split_once('#')
        .map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let file = if written.is_empty() {
        document.to_owned()
    } else {
        resolve(document, written)
    };
    let path = workspace.join(&file);
    if !path.is_file() {
        return Err(format!("Link target not found: {written}"));
    }
    if !is_markdown(&path) {
        return Err(format!("Not a markdown file: {written}"));
    }
    Ok(Place {
        file,
        anchor: anchor.map(String::from),
    })
}

/// Whether a link target starts with a URL scheme, such as `https:`.
fn is_external(target: &str) -> bool {
    target.split_once(':').is_some_and(|(scheme, _)| {
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
    })
}

/// Resolves a relative path from a document. A path can lead out of the
/// workspace, in which case it keeps the `..` that does so.
fn resolve(document: &str, path: &str) -> String {
    let folder = document.rsplit_once('/').map_or("", |(folder, _)| folder);
    let mut parts: Vec<&str> = Vec::new();
    for part in folder.split('/').chain(path.split('/')) {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|last| *last != "..") => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(name)
    }

    fn follow(document: &str, target: &str) -> Result<Place, String> {
        follow_link(&sample("docs"), document, target)
    }

    fn place(file: &str, anchor: Option<&str>) -> Place {
        Place {
            file: file.into(),
            anchor: anchor.map(String::from),
        }
    }

    #[test]
    fn links_are_resolved_from_the_document() {
        assert_eq!(
            follow("guide/long-page.md", "getting-started.md"),
            Ok(place("guide/getting-started.md", None))
        );
        assert_eq!(
            follow("guide/long-page.md", "../reference.md"),
            Ok(place("reference.md", None))
        );
        assert_eq!(
            follow("README.md", "./guide/../reference.md"),
            Ok(place("reference.md", None))
        );
    }

    #[test]
    fn an_anchor_alone_stays_in_the_document() {
        assert_eq!(
            follow("guide/long-page.md", "#the-end"),
            Ok(place("guide/long-page.md", Some("the-end")))
        );
    }

    #[test]
    fn a_link_can_lead_out_of_the_workspace() {
        let found = follow_link(&sample("docs").join("guide"), "a.md", "../README.md");
        assert_eq!(found, Ok(place("../README.md", None)));
        assert_eq!(resolve("../a.md", "../b.md"), "../../b.md");
    }

    #[test]
    fn links_that_cannot_be_opened_say_why() {
        for (target, message) in [
            ("missing.md#top", "Link target not found: missing.md"),
            ("guide", "Link target not found: guide"),
            ("data.txt", "Not a markdown file: data.txt"),
            ("mailto:a@b.c", "External link: mailto:a@b.c"),
        ] {
            assert_eq!(follow("README.md", target), Err(message.to_owned()));
        }
    }

    #[test]
    fn skipped_folders_and_other_files_are_not_listed() {
        assert_eq!(
            markdown_files(&sample("docs")),
            [
                "guide/getting-started.md",
                "guide/long-page.md",
                "notes.MD",
                "README.md",
                "reference.md"
            ]
        );
    }

    #[test]
    fn a_missing_workspace_lists_nothing() {
        assert_eq!(markdown_files(Path::new("/no/such/folder")), [""; 0]);
    }

    #[test]
    fn a_file_outside_the_current_folder_moves_the_workspace() {
        let opening = locate(&sample("docs"), Some("../calculator/passing.md")).unwrap();
        assert_eq!(opening.file.as_deref(), Some("passing.md"));
        assert!(opening.workspace.ends_with("samples/calculator"));
    }

    #[test]
    fn a_path_is_reported_as_it_was_written() {
        let refusal = locate(&sample("docs"), Some("./nope.md")).unwrap_err();
        assert_eq!(refusal, "Path not found: ./nope.md");
    }
}
