use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

pub fn start_watching(
    path: PathBuf,
    sender: mpsc::Sender<String>,
) -> Result<RecommendedWatcher, Box<dyn std::error::Error>> {
    let watch_path = path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() {
                    // Small delay to let the write complete
                    std::thread::sleep(Duration::from_millis(50));
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let _ = sender.send(content);
                    }
                }
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    // Watch the parent directory (works better with editors that write temp files)
    let watch_dir = watch_path
        .parent()
        .unwrap_or(&watch_path)
        .to_path_buf();
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}
