// src/cache/gpu_cache.rs
use crate::renderer::GpuImage;
use crate::types::MipLevel;
use lru::LruCache;
use std::collections::HashMap;
use std::sync::Arc;

/// Default GPU cache: VRAM 의 25% 사용
const DEFAULT_VRAM_MB: usize = 2048;
const VRAM_USAGE_RATIO: f32 = 0.25;

/// Key for GPU cache: (doc_id, page_name_hash, mip_level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuCacheKey {
    pub doc_id: u64,
    pub page_hash: u64,
    pub mip: MipLevel,
}

/// Evicted entry from GPU cache - used to sync with TextureIndex
#[derive(Debug, Clone)]
pub struct EvictedGpuEntry {
    pub doc_id: u64,
    pub page_hash: u64,
    pub mip: MipLevel,
}

/// GPU-side texture cache with memory-based eviction.
/// Stores wgpu::Texture and related resources.
/// Key: GpuCacheKey - includes doc_id for document-aware eviction
/// Value: Arc<GpuImage>
///
/// Memory Management:
/// - Maximum memory: VRAM_MB * VRAM_USAGE_RATIO (default: 2048 * 0.5 = 1024 MB)
/// - Eviction: LRU (Least Recently Used) when memory limit exceeded
/// - Internal tracking uses bytes for precision, MB only for display
pub struct GpuTextureCache {
    cache: LruCache<GpuCacheKey, Arc<GpuImage>>,
    max_memory_bytes: u64,
    current_memory_bytes: u64,
    /// Secondary index for document-aware eviction: doc_id -> set of cache keys
    doc_index: HashMap<u64, Vec<GpuCacheKey>>,
    /// Protection priorities: page_hash -> priority (0 = highest)
    /// Note: page_hash here is the specific hash of (doc, page, mip)
    protected_entries: HashMap<u64, usize>,
}

impl GpuTextureCache {
    /// Create a new GPU texture cache with VRAM-based memory limit.
    ///
    /// # Arguments
    /// * `vram_available_mb` - Available VRAM in megabytes (query from adapter)
    pub fn new_from_vram(vram_available_mb: usize) -> Self {
        let max_memory_bytes =
            (((vram_available_mb as u64) * 1024 * 1024) as f32 * VRAM_USAGE_RATIO) as u64;
        let max_memory_bytes = max_memory_bytes.max(64 * 1024 * 1024); // Minimum 64 MB

        Self {
            cache: LruCache::unbounded(),
            max_memory_bytes,
            current_memory_bytes: 0,
            doc_index: HashMap::new(),
            protected_entries: HashMap::new(),
        }
    }

    /// Update the set of protected entries with their priorities.
    pub fn set_protection(&mut self, protections: HashMap<u64, usize>) {
        self.protected_entries = protections;
    }

    /// Create a new GPU texture cache with a fixed memory limit.
    ///
    /// # Arguments
    /// * `max_memory_mb` - Maximum memory usage in megabytes
    #[allow(dead_code)]
    pub fn new_with_memory_limit(max_memory_mb: usize) -> Self {
        Self {
            cache: LruCache::unbounded(),
            max_memory_bytes: (max_memory_mb as u64) * 1024 * 1024,
            current_memory_bytes: 0,
            doc_index: HashMap::new(),
            protected_entries: HashMap::new(),
        }
    }

    /// Create a new GPU texture cache with default settings.
    pub fn new() -> Self {
        Self::new_from_vram(DEFAULT_VRAM_MB)
    }

    /// Calculate texture memory usage in bytes (RGBA8 format).
    /// Reflects hardware overhead: 10% at 0.25MP (512x512) and 100% at 16MP (4096x4096).
    fn calculate_memory_bytes(width: u32, height: u32) -> u64 {
        let pixels = (width as u64) * (height as u64);
        let raw_bytes = pixels * 4;

        let p_min = 512.0 * 512.0; // 0.25 Megapixels
        let p_max = 4096.0 * 4096.0; // 16 Megapixels
        let p = pixels as f32;

        // Normalized ratio (0.0 at 512x512, 1.0 at 4096x4096)
        let t = ((p - p_min) / (p_max - p_min)).max(0.0);

        // Non-linear overhead: Starts at 10%, rises quadratically to 100% (2x total size).
        let overhead_ratio = 0.10 + 0.90 * t * t;

        (raw_bytes as f32 * (1.0 + overhead_ratio)) as u64
    }

    /// Find the best candidate for eviction.
    fn find_eviction_candidate(&self) -> Option<GpuCacheKey> {
        if self.cache.is_empty() {
            return None;
        }

        // Step 1: Find an unprotected item from LRU end
        for (key, _) in self.cache.iter().rev() {
            if !self.protected_entries.contains_key(&key.page_hash) {
                return Some(*key);
            }
        }

        // Step 2: All are protected. Find one with highest priority value.
        let mut worst_priority = 0;
        let mut candidate = None;

        for (key, _) in self.cache.iter() {
            if let Some(&p) = self.protected_entries.get(&key.page_hash) {
                if p >= worst_priority {
                    worst_priority = p;
                    candidate = Some(*key);
                }
            } else {
                return Some(*key);
            }
        }

        candidate
    }

    /// Evict LRU entries until we have enough space for a new texture.
    /// Returns a list of evicted entries so caller can sync with TextureIndex.
    fn evict_until_space(&mut self, required_bytes: u64) -> Vec<EvictedGpuEntry> {
        let mut evicted_entries = Vec::new();

        while self.current_memory_bytes + required_bytes > self.max_memory_bytes {
            if let Some(key) = self.find_eviction_candidate() {
                if let Some(evicted) = self.cache.pop(&key) {
                    let evicted_size = Self::calculate_memory_bytes(evicted.width, evicted.height);
                    self.current_memory_bytes =
                        self.current_memory_bytes.saturating_sub(evicted_size);

                    evicted_entries.push(EvictedGpuEntry {
                        doc_id: key.doc_id,
                        page_hash: key.page_hash,
                        mip: key.mip,
                    });

                    if let Some(keys) = self.doc_index.get_mut(&key.doc_id) {
                        keys.retain(|k| *k != key);
                        if keys.is_empty() {
                            self.doc_index.remove(&key.doc_id);
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        evicted_entries
    }

    /// Get a texture from the cache and update LRU order.
    #[allow(dead_code)]
    pub fn get(&mut self, doc_id: u64, page_hash: u64, mip: MipLevel) -> Option<Arc<GpuImage>> {
        let key = GpuCacheKey {
            doc_id,
            page_hash,
            mip,
        };
        self.cache.get(&key).cloned()
    }

    /// Insert a texture into the cache.
    /// Automatically evicts LRU entries if memory limit exceeded.
    /// Returns a list of evicted entries so caller can sync with TextureIndex.
    pub fn insert(
        &mut self,
        doc_id: u64,
        page_hash: u64,
        mip: MipLevel,
        image: Arc<GpuImage>,
    ) -> Vec<EvictedGpuEntry> {
        let texture_size_bytes = Self::calculate_memory_bytes(image.width, image.height);

        let key = GpuCacheKey {
            doc_id,
            page_hash,
            mip,
        };

        let mut evicted_entries = Vec::new();

        // Remove existing entry with same key if present
        if let Some(evicted) = self.cache.pop(&key) {
            let evicted_size = Self::calculate_memory_bytes(evicted.width, evicted.height);
            self.current_memory_bytes = self.current_memory_bytes.saturating_sub(evicted_size);
            evicted_entries.push(EvictedGpuEntry {
                doc_id,
                page_hash,
                mip,
            });
        }

        // Evict if necessary and collect evicted entries
        evicted_entries.extend(self.evict_until_space(texture_size_bytes));

        // Insert new texture and update doc_index
        self.cache.put(key, image);
        self.current_memory_bytes += texture_size_bytes;
        self.doc_index.entry(doc_id).or_default().push(key);

        // Debug validation: ensure memory tracking is accurate
        debug_assert!(
            self.current_memory_bytes <= self.max_memory_bytes,
            "GPU cache overflow detected: current={}MB, max={}MB, entry_size={}MB",
            self.current_memory_bytes / 1024 / 1024,
            self.max_memory_bytes / 1024 / 1024,
            texture_size_bytes / 1024 / 1024
        );

        // Log high memory pressure
        if self.current_memory_bytes > self.max_memory_bytes * 90 / 100 {
            tracing::warn!(
                "[GPU Cache] Critical memory pressure: {}MB / {}MB ({}%)",
                self.current_memory_bytes / 1024 / 1024,
                self.max_memory_bytes / 1024 / 1024,
                self.current_memory_bytes * 100 / self.max_memory_bytes.max(1)
            );
        }

        evicted_entries
    }

    /// Remove a specific key from the cache.
    #[allow(dead_code)]
    pub fn remove(&mut self, doc_id: u64, page_hash: u64, mip: MipLevel) -> Option<Arc<GpuImage>> {
        let key = GpuCacheKey {
            doc_id,
            page_hash,
            mip,
        };
        if let Some(evicted) = self.cache.pop(&key) {
            let evicted_size = Self::calculate_memory_bytes(evicted.width, evicted.height);
            self.current_memory_bytes = self.current_memory_bytes.saturating_sub(evicted_size);
            // Remove from doc_index
            if let Some(keys) = self.doc_index.get_mut(&doc_id) {
                keys.retain(|k| *k != key);
                if keys.is_empty() {
                    self.doc_index.remove(&doc_id);
                }
            }
            Some(evicted)
        } else {
            None
        }
    }

    /// Clear all entries from the cache.
    /// Returns all evicted entries so caller can sync with TextureIndex.
    #[allow(dead_code)]
    pub fn clear(&mut self) -> Vec<EvictedGpuEntry> {
        let mut evicted_entries = Vec::with_capacity(self.cache.len());

        for (key, _) in self.cache.iter() {
            evicted_entries.push(EvictedGpuEntry {
                doc_id: key.doc_id,
                page_hash: key.page_hash,
                mip: key.mip,
            });
        }

        self.cache.clear();
        self.current_memory_bytes = 0;
        self.doc_index.clear();
        evicted_entries
    }

    /// Clear all entries for a specific document.
    #[allow(dead_code)]
    pub fn clear_doc(&mut self, doc_id: u64) -> Vec<EvictedGpuEntry> {
        let mut evicted_entries = Vec::new();

        if let Some(keys) = self.doc_index.remove(&doc_id) {
            for key in keys {
                if let Some(evicted) = self.cache.pop(&key) {
                    let evicted_size = Self::calculate_memory_bytes(evicted.width, evicted.height);
                    self.current_memory_bytes =
                        self.current_memory_bytes.saturating_sub(evicted_size);
                    evicted_entries.push(EvictedGpuEntry {
                        doc_id: key.doc_id,
                        page_hash: key.page_hash,
                        mip: key.mip,
                    });
                }
            }
        }

        evicted_entries
    }

    /// Get the current number of entries in the cache.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get current memory usage in MB.
    #[allow(dead_code)]
    pub fn memory_usage_mb(&self) -> usize {
        (self.current_memory_bytes / 1024 / 1024) as usize
    }

    /// Get maximum memory limit in MB.
    pub fn max_memory_mb(&self) -> usize {
        (self.max_memory_bytes / 1024 / 1024) as usize
    }

    pub fn set_max_memory_mb(&mut self, max_memory_mb: usize) {
        self.max_memory_bytes = (max_memory_mb as u64) * 1024 * 1024;
        self.evict_until_space(0);
    }
}

impl Default for GpuTextureCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_cache_priority_protection() {
        // 3 MB limit (allows two 512x512 images with ~1.1MB each to fit)
        let mut cache = GpuTextureCache::new_with_memory_limit(3);

        let img1 = Arc::new(GpuImage {
            tiles: Vec::new(),
            width: 512,
            height: 512,
            mip: MipLevel::Full,
        }); // 1MB raw pixels
        let img2 = Arc::new(GpuImage {
            tiles: Vec::new(),
            width: 512,
            height: 512,
            mip: MipLevel::Full,
        });

        // Key 1: doc 1, hash 101, Full
        cache.insert(1, 101, MipLevel::Full, img1);
        // Key 2: doc 1, hash 102, Full
        cache.insert(1, 102, MipLevel::Full, img2);

        // Protect Key 1 (oldest)
        let mut protections = HashMap::new();
        protections.insert(101, 0);
        cache.set_protection(protections);

        // Insert Key 3. Must evict one.
        let img3 = Arc::new(GpuImage {
            tiles: Vec::new(),
            width: 512,
            height: 512,
            mip: MipLevel::Full,
        });
        cache.insert(1, 103, MipLevel::Full, img3);

        // 101 should be KEPT because it is protected. 102 should be EVICTED.
        assert!(cache.get(1, 101, MipLevel::Full).is_some());
        assert!(cache.get(1, 102, MipLevel::Full).is_none());
        assert!(cache.get(1, 103, MipLevel::Full).is_some());
    }
}
