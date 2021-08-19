use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use eyre::Result;

use crate::downloader;
use crate::item::Item;
use crate::metadata::{self, Metadata};

#[derive(Debug, Default, Copy, Clone)]
pub struct Stats {
    downloaded: usize,
    skipped: usize,
    errored: usize,
}

struct State {
    metadata: Metadata,
    stats: Stats,
    in_progress_ids: HashSet<String>,
}

#[derive(Debug, Copy, Clone)]
enum ShouldDownload {
    Download,
    Skip,
}

impl State {
    fn create(root_dir: impl Into<PathBuf>) -> Result<Self> {
        let metadata = Metadata::create(root_dir)?;
        Ok(State {
            metadata,
            stats: Default::default(),
            in_progress_ids: Default::default(),
        })
    }

    pub fn try_start(&mut self, item: &Item) -> Result<ShouldDownload> {
        let id = item.id();
        if self.in_progress_ids.contains(id.as_str()) || self.metadata.exists(id.as_str())? {
            self.stats.skipped += 1;
            Ok(ShouldDownload::Skip)
        } else {
            self.in_progress_ids.insert(id);
            Ok(ShouldDownload::Download)
        }
    }

    pub fn done(&mut self, item: &Item, success: bool) -> Result<()> {
        let id = item.id();
        self.in_progress_ids.remove(id.as_str());
        if success {
            let fs_path = item.fs_path();
            let metadata_item = metadata::Media::new(
                id.as_str(),
                &fs_path,
                item.creation_time(),
                chrono::offset::Utc::now(),
            );
            self.metadata.insert(&metadata_item)?;
            self.stats.downloaded += 1;
        } else {
            self.stats.errored += 1;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Archiver {
    state: Arc<Mutex<State>>,
    root_dir: PathBuf,
    client: reqwest::Client,
}

impl Archiver {
    pub fn create(root_dir: impl Into<PathBuf>) -> Result<Self> {
        let root_dir = root_dir.into();
        let state = State::create(&root_dir)?;

        Ok(Archiver {
            state: Arc::new(Mutex::new(state)),
            root_dir,
            client: reqwest::Client::new(),
        })
    }

    pub async fn download_one(&self, item: &Item) -> Result<()> {
        let id = item.id();
        let should_download = self.state.lock().unwrap().try_start(item)?;
        match should_download {
            ShouldDownload::Skip => {
                log::debug!("Skipping {}", id);
                Ok(())
            }
            ShouldDownload::Download => {
                let fs_path = item.fs_path();
                let full_path = self.root_dir.clone().join(&fs_path);
                let download_url = item.download_url();

                let result =
                    downloader::download(&self.client, download_url.as_str(), full_path).await;
                (match &result {
                    Ok(()) => {
                        log::info!("Downloaded {} to {}", id, fs_path.to_string_lossy());
                    }
                    Err(error) => {
                        log::error!("Failed to download {}: {}", id, error);
                    }
                });
                self.state.lock().unwrap().done(item, result.is_ok())?;
                Ok(())
            }
        }
    }

    pub fn stats(&self) -> Stats {
        self.state.lock().unwrap().stats
    }
}
