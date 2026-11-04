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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(name)
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
