use crossbeam_channel::Receiver;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

use crate::error::Result;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<std::result::Result<notify::Event, notify::Error>>,
    vault_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

impl FileWatcher {
    pub fn new(vault_path: &Path) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut watcher = notify::recommended_watcher(tx).map_err(|e| {
            crate::error::ZtlgrError::Parse(format!("Failed to create watcher: {}", e))
        })?;

        watcher
            .watch(vault_path, RecursiveMode::Recursive)
            .map_err(|e| crate::error::ZtlgrError::Parse(format!("Failed to watch: {}", e)))?;

        Ok(Self {
            watcher,
            receiver: rx,
            vault_path: vault_path.to_path_buf(),
        })
    }

    pub fn next_event(&self) -> Option<FileEvent> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => {
                if let Some(path) = event.paths.first() {
                    if self.is_note_file(path) {
                        match event.kind {
                            notify::EventKind::Create(_) => {
                                Some(FileEvent::Created(path.to_path_buf()))
                            }
                            notify::EventKind::Modify(_) => {
                                Some(FileEvent::Modified(path.to_path_buf()))
                            }
                            notify::EventKind::Remove(_) => {
                                Some(FileEvent::Deleted(path.to_path_buf()))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Ok(Err(_)) | Err(_) => None,
        }
    }

    fn is_note_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "md" || ext == "org")
            .unwrap_or(false)
    }

    pub fn stop(&mut self) -> Result<()> {
        self.watcher
            .unwatch(&self.vault_path)
            .map_err(|e| crate::error::ZtlgrError::Parse(format!("Failed to unwatch: {}", e)))?;
        Ok(())
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
