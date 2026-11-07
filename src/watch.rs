//! Noticing files of the workspace that change on disk.

use crate::workspace;
use notify::event::{AccessKind, AccessMode};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// How long the workspace has to be still before a change counts. Saving a file
/// can take a few steps, and a page is better shown once they are all done.
const QUIET: Duration = Duration::from_millis(50);

/// What changed in the workspace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    /// The markdown files that changed, relative to the workspace.
    files: BTreeSet<String>,
    /// Whether anything else changed, such as a folder, which could mean any
    /// file did.
    anything: bool,
}

impl Changes {
    /// Whether nothing changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && !self.anything
    }

    /// Whether this file may have changed.
    #[must_use]
    pub fn covers(&self, file: &str) -> bool {
        self.anything || self.files.contains(file)
    }

    fn add(&mut self, other: Self) {
        self.files.extend(other.files);
        self.anything |= other.anything;
    }

    fn anything() -> Self {
        Self {
            files: BTreeSet::new(),
            anything: true,
        }
    }
}

/// What to call when a change arrives, so that an interface that only draws
/// when asked to can look at it.
type Waker = Box<dyn Fn() + Send + Sync>;

/// Watches a workspace and everything inside it.
pub struct Watcher {
    /// What was noticed, as it arrives.
    noticed: Receiver<Changes>,
    /// What has been noticed but not yet reported.
    pending: Changes,
    /// When something was last noticed.
    last: Option<Instant>,
    waker: Arc<OnceLock<Waker>>,
    /// What the system watches with. Watching stops once this is dropped.
    _system: RecommendedWatcher,
}

impl std::fmt::Debug for Watcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Watcher")
            .field("pending", &self.pending)
            .field("last", &self.last)
            .finish_non_exhaustive()
    }
}

impl Watcher {
    /// Starts watching a workspace.
    ///
    /// # Errors
    ///
    /// Fails with a message for the user when the system will not watch it.
    pub fn start(workspace: &Path) -> Result<Self, String> {
        // The system names what changed by its real path.
        let root = workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.to_path_buf());
        let (sender, noticed) = mpsc::channel();
        let waker: Arc<OnceLock<Waker>> = Arc::new(OnceLock::new());
        let wake = Arc::clone(&waker);
        let inside = root.clone();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let changes = match event {
                    Ok(event) if !matters(event.kind) => return,
                    Ok(event) if event.need_rescan() => Changes::anything(),
                    Ok(event) => changes_in(&inside, &event.paths),
                    // Whatever went wrong, any file could have changed.
                    Err(_) => Changes::anything(),
                };
                if changes.is_empty() || sender.send(changes).is_err() {
                    return;
                }
                if let Some(wake) = wake.get() {
                    wake();
                }
            })
            .map_err(|error| cannot_watch(&error))?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|error| cannot_watch(&error))?;
        Ok(Self {
            noticed,
            pending: Changes::default(),
            last: None,
            waker,
            _system: watcher,
        })
    }

    /// Calls `wake` whenever something changes, from another thread. Only the
    /// first one given is kept.
    pub fn wake_with(&self, wake: impl Fn() + Send + Sync + 'static) {
        // The interface only ever gives one.
        match self.waker.set(Box::new(wake)) {
            Ok(()) | Err(_) => {}
        }
    }

    /// What changed, once the workspace has been still for a moment.
    pub fn changes(&mut self) -> Option<Changes> {
        // Once the system stops watching, there is nothing more to hear.
        while let Ok(changes) = self.noticed.try_recv() {
            self.pending.add(changes);
            self.last = Some(Instant::now());
        }
        let still = self.last.is_some_and(|last| last.elapsed() >= QUIET);
        if self.pending.is_empty() || !still {
            return None;
        }
        self.last = None;
        Some(std::mem::take(&mut self.pending))
    }

    /// Whether something changed that has not been reported yet.
    #[must_use]
    pub fn is_settling(&self) -> bool {
        !self.pending.is_empty() || self.last.is_some()
    }
}

fn cannot_watch(error: &notify::Error) -> String {
    format!("Cannot watch the files for changes: {error}")
}

/// Whether an event could mean that something changed. Reading a file is not
/// a change, and temoc-ui reads every file that changes.
fn matters(kind: EventKind) -> bool {
    match kind {
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => true,
        EventKind::Access(_) => false,
        _ => true,
    }
}

/// What changed, given the paths the system named.
fn changes_in(workspace: &Path, paths: &[PathBuf]) -> Changes {
    let mut changes = Changes::default();
    for path in paths {
        let Ok(inside) = path.strip_prefix(workspace) else {
            continue;
        };
        if workspace::is_hidden(inside) {
            continue;
        }
        if inside.as_os_str().is_empty() || !workspace::is_markdown(inside) {
            changes.anything = true;
        } else {
            changes.files.insert(workspace::relative_name(inside));
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::{env, fs, process};

    /// Waits for the watcher to report something.
    fn next_changes(watcher: &mut Watcher) -> Changes {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(changes) = watcher.changes() {
                return changes;
            }
            assert!(Instant::now() < deadline, "nothing was noticed");
            sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn a_file_written_in_the_workspace_is_noticed() {
        let folder = env::temp_dir().join(format!("temoc-ui-watch-{}", process::id()));
        fs::create_dir_all(&folder).unwrap();
        let mut watcher = Watcher::start(&folder).unwrap();
        assert!(!watcher.is_settling());
        fs::write(folder.join("a.md"), "# A").unwrap();
        assert!(next_changes(&mut watcher).covers("a.md"));
        fs::remove_file(folder.join("a.md")).unwrap();
        assert!(next_changes(&mut watcher).covers("a.md"));
        drop(watcher);
        fs::remove_dir_all(&folder).unwrap();
    }

    fn changes(paths: &[&str]) -> Changes {
        let root = Path::new("/w");
        let paths: Vec<PathBuf> = paths.iter().map(|path| root.join(path)).collect();
        changes_in(root, &paths)
    }

    #[test]
    fn a_markdown_file_is_named() {
        let changed = changes(&["guide/a.md"]);
        assert!(changed.covers("guide/a.md"));
        assert!(!changed.covers("b.md"));
    }

    #[test]
    fn anything_else_could_mean_any_file() {
        assert!(changes(&["guide"]).covers("guide/a.md"));
        assert!(changes(&["a.md~"]).covers("a.md"));
    }

    #[test]
    fn hidden_and_skipped_folders_do_not_count() {
        assert!(changes(&[".git/index", ".versions/a.md", "target/x", ".a.md.swp"]).is_empty());
    }

    #[test]
    fn paths_outside_do_not_count() {
        let outside = changes_in(Path::new("/w"), &[PathBuf::from("/elsewhere/a.md")]);
        assert!(outside.is_empty());
    }

    #[test]
    fn reading_is_not_changing() {
        assert!(!matters(EventKind::Access(AccessKind::Read)));
        let written = EventKind::Access(AccessKind::Close(AccessMode::Write));
        assert!(matters(written));
    }
}
