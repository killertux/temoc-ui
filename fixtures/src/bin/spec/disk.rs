//! Changing the files of a workspace behind temoc-ui's back, as described in
//! `spec/writing-specs.md`.

use crate::session::samples;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The hidden folder of a workspace holding what the steps write.
const VERSIONS: &str = ".versions";
/// How long temoc-ui has to notice a change. The spec asks for a second.
const NOTICE_TIMEOUT: Duration = Duration::from_secs(1);
/// How long nothing more has to happen before a change is over. Saving can
/// take more than one step, and each may be noticed on its own.
const STILL: Duration = Duration::from_millis(150);
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// A change to a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Writes a version over the file, in place.
    Write {
        file: String,
        version: String,
    },
    /// Writes a version next to the file, then renames it over the file.
    Replace {
        file: String,
        version: String,
    },
    Delete(String),
    Rename {
        from: String,
        to: String,
    },
}

impl Change {
    /// Makes the change in a workspace.
    pub fn apply(&self, workspace: &Path) -> Result<(), String> {
        let path = |file: &str| workspace.join(file);
        let version = |name: &str| {
            let source = workspace.join(VERSIONS).join(name);
            fs::read(&source).map_err(|error| format!("cannot read version {name}: {error}"))
        };
        let done = match self {
            Self::Write {
                file,
                version: name,
            } => {
                let text = version(name)?;
                let target = path(file);
                if let Some(folder) = target.parent() {
                    fs::create_dir_all(folder).map_err(|error| error.to_string())?;
                }
                fs::write(target, text)
            }
            Self::Replace {
                file,
                version: name,
            } => {
                let text = version(name)?;
                let saving = path(&format!("{file}.saving"));
                fs::write(&saving, text).and_then(|()| fs::rename(&saving, path(file)))
            }
            Self::Delete(file) => fs::remove_file(path(file)),
            Self::Rename { from, to } => fs::rename(path(from), path(to)),
        };
        done.map_err(|error| format!("cannot change the workspace: {error}"))
    }
}

/// Waits for temoc-ui to notice a change and for it to be over. `look` lets
/// temoc-ui look at the disk and says how many changes it has seen.
pub fn wait_for_change(
    before: u64,
    mut look: impl FnMut() -> Result<u64, String>,
) -> Result<(), String> {
    let deadline = Instant::now() + NOTICE_TIMEOUT;
    let mut seen = before;
    let mut last = None;
    loop {
        let now = look()?;
        if now != seen {
            seen = now;
            last = Some(Instant::now());
        }
        match last {
            Some(last) if last.elapsed() >= STILL => return Ok(()),
            None if Instant::now() > deadline => {
                return Err("temoc-ui did not notice the change".into());
            }
            _ => sleep(POLL_INTERVAL),
        }
    }
}

/// A copy of a sample workspace, so that changing it leaves the sample alone.
/// It is deleted when dropped.
#[derive(Debug)]
pub struct Sandbox {
    folder: PathBuf,
}

impl Sandbox {
    /// Copies a workspace from `samples/`.
    pub fn of(workspace: &str) -> Result<Self, String> {
        static COUNT: AtomicU64 = AtomicU64::new(0);
        let count = COUNT.fetch_add(1, Ordering::Relaxed);
        let name = format!("temoc-ui-spec-{}-{count}", process::id());
        let folder = env::temp_dir().join(name);
        let sandbox = Self { folder };
        copy_folder(&samples().join(workspace), &sandbox.folder)
            .map_err(|error| format!("cannot copy the {workspace} workspace: {error}"))?;
        Ok(sandbox)
    }

    /// The copy.
    pub fn folder(&self) -> &Path {
        &self.folder
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // What is left behind in the temporary folder does no harm.
        match fs::remove_dir_all(&self.folder) {
            Ok(()) | Err(_) => {}
        }
    }
}

fn copy_folder(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_folder(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sandbox_is_a_copy_that_goes_away() {
        let folder = {
            let sandbox = Sandbox::of("changing").unwrap();
            let folder = sandbox.folder().to_path_buf();
            assert!(folder.join(".versions/new.md").is_file());
            Change::Delete("README.md".into()).apply(&folder).unwrap();
            folder
        };
        assert!(!folder.exists());
        assert!(samples().join("changing/README.md").is_file());
    }

    #[test]
    fn writing_makes_the_folders_it_needs() {
        let sandbox = Sandbox::of("changing").unwrap();
        let write = Change::Write {
            file: "notes/new.md".into(),
            version: "new.md".into(),
        };
        write.apply(sandbox.folder()).unwrap();
        let written = fs::read_to_string(sandbox.folder().join("notes/new.md")).unwrap();
        assert!(written.starts_with("# New"));
    }

    #[test]
    fn replacing_leaves_nothing_behind() {
        let sandbox = Sandbox::of("changing").unwrap();
        let replace = Change::Replace {
            file: "README.md".into(),
            version: "new.md".into(),
        };
        replace.apply(sandbox.folder()).unwrap();
        assert!(!sandbox.folder().join("README.md.saving").exists());
    }

    #[test]
    fn a_change_that_is_never_noticed_gives_up() {
        assert_eq!(
            wait_for_change(3, || Ok(3)),
            Err("temoc-ui did not notice the change".into())
        );
    }
}
