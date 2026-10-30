use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub visited_at: u64, // Unix timestamp in seconds
}

#[derive(Clone)]
pub struct HistoryStore {
    file_path: PathBuf,
    entries: Arc<Mutex<Vec<HistoryEntry>>>,
}

impl HistoryStore {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let storage_dir = app_data_dir.join("titan_browser");
        let _ = create_dir_all(&storage_dir);
        let file_path = storage_dir.join("history.json");

        let entries = if file_path.exists() {
            File::open(&file_path)
                .ok()
                .and_then(|f| serde_json::from_reader(BufReader::new(f)).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            file_path,
            entries: Arc::new(Mutex::new(entries)),
        }
    }

    pub fn record_visit(&self, title: &str, url: &str) {
        // Ignore internal or empty schemes
        if url.is_empty() || url == "about:blank" || url.starts_with("browser://") {
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = HistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            url: url.to_string(),
            visited_at: now,
        };

        let mut lock = self.entries.lock();
        lock.push(entry);

        // Keep maximum 5,000 recent history entries locally
        if lock.len() > 5000 {
            let trim_count = lock.len() - 5000;
            lock.drain(0..trim_count);
        }

        // Persist to disk asynchronously or on change
        if let Ok(file) = File::create(&self.file_path) {
            let _ = serde_json::to_writer_pretty(BufWriter::new(file), &*lock);
        }
    }

    pub fn get_recent_entries(&self, limit: usize) -> Vec<HistoryEntry> {
        let lock = self.entries.lock();
        lock.iter().rev().take(limit).cloned().collect()
    }
}

