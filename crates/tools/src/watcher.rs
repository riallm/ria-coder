//! File Watcher (SPEC-031)

use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Receiver;

pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    pub rx: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let watcher = notify::RecommendedWatcher::new(tx, notify::Config::default())?;

        Ok(Self { watcher, rx })
    }

    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)?;
        Ok(())
    }

    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.watcher.unwatch(path.as_ref())?;
        Ok(())
    }
}
