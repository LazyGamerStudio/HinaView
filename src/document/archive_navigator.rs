use crate::settings::model::ArchiveSortingMode;
use crate::util::formats::{is_supported_archive_file, is_supported_image_path, normalize_path};
use crate::util::sorting::natural_cmp_ci;
use std::path::{Path, PathBuf};
use tracing::warn;

pub struct NavigationResult {
    pub path: PathBuf,
    pub looped: bool,
    pub skipped_folders: Vec<String>,
}

pub struct ArchiveNavigator {
    // (Normalized Dir Path String, List of (Normalized Path, Original PathBuf), Sort Mode)
    cached_entries: Option<(String, Vec<(String, PathBuf)>, ArchiveSortingMode)>,
}

impl ArchiveNavigator {
    pub fn new() -> Self {
        Self {
            cached_entries: None,
        }
    }

    pub fn invalidate_cache(&mut self) {
        self.cached_entries = None;
    }

    pub fn find_neighbor(
        &mut self,
        current: &Path,
        step: i32,
        sort_mode: ArchiveSortingMode,
    ) -> Option<NavigationResult> {
        let current_norm = normalize_path(current);
        let current_dir = current.parent()?.to_path_buf();
        let current_dir_norm = normalize_path(&current_dir);

        self.ensure_cache_for_dir(&current_dir, &current_dir_norm, sort_mode);

        let (_, entries, _) = self.cached_entries.as_ref()?;
        if entries.is_empty() {
            return None;
        }

        // Find current position using normalized string comparison
        let current_idx = entries.iter().position(|(norm, _)| norm == &current_norm)?;
        let len = entries.len() as i32;

        let mut skipped_folders = Vec::new();
        let mut looped = false;
        let mut visited_count = 0;
        let mut check_idx = current_idx as i32;

        while visited_count < len {
            check_idx = (check_idx + step + len) % len;
            visited_count += 1;

            if step > 0 && check_idx < current_idx as i32 {
                looped = true;
            } else if step < 0 && check_idx > current_idx as i32 {
                looped = true;
            }

            let (_, original_path) = &entries[check_idx as usize];
            if self.is_valid_target(original_path) {
                return Some(NavigationResult {
                    path: original_path.clone(),
                    looped,
                    skipped_folders,
                });
            } else {
                if let Some(name) = original_path.file_name().and_then(|n| n.to_str()) {
                    skipped_folders.push(name.to_string());
                }
            }
        }

        None
    }

    fn is_valid_target(&self, path: &Path) -> bool {
        if is_supported_archive_file(path) {
            return true;
        }

        if path.is_dir() {
            if let Ok(read_dir) = std::fs::read_dir(path) {
                for entry in read_dir.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_file() && is_supported_image_path(&p) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn ensure_cache_for_dir(&mut self, dir: &Path, dir_norm: &str, sort_mode: ArchiveSortingMode) {
        if let Some((cached_dir_norm, _, cached_mode)) = &self.cached_entries
            && cached_dir_norm == dir_norm
            && *cached_mode == sort_mode
        {
            return;
        }

        let mut raw_entries: Vec<PathBuf> = match std::fs::read_dir(dir) {
            Ok(read_dir) => read_dir
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir() || is_supported_archive_file(path))
                .collect(),
            Err(e) => {
                warn!(
                    "[ArchiveNavigator] Failed to read directory {:?}: {}",
                    dir, e
                );
                Vec::new()
            }
        };

        match sort_mode {
            ArchiveSortingMode::Mixed => {
                raw_entries.sort_by(|a, b| natural_cmp_ci(a, b));
            }
            ArchiveSortingMode::FoldersFirst => {
                raw_entries.sort_by(|a, b| {
                    let a_is_dir = a.is_dir();
                    let b_is_dir = b.is_dir();
                    if a_is_dir != b_is_dir {
                        b_is_dir.cmp(&a_is_dir)
                    } else {
                        natural_cmp_ci(a, b)
                    }
                });
            }
        }

        // Store both normalized string for comparison and original PathBuf for opening
        let processed_entries: Vec<(String, PathBuf)> = raw_entries
            .into_iter()
            .map(|p| (normalize_path(&p), p))
            .collect();

        self.cached_entries = Some((dir_norm.to_string(), processed_entries, sort_mode));
    }
}
