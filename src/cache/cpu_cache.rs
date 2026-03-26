// src/cache/cpu_cache.rs
use crate::types::{DecodedImage, MipLevel};
use lru::LruCache;
use std::collections::HashMap;
use std::sync::Arc;

/// Default CPU cache memory limit: 256 MB
const DEFAULT_MEMORY_LIMIT_MB: usize = 256;

/// CPU-side decoded image cache with memory-based eviction.
/// Stores RGBA8 pixel data after decoding and resampling.
/// Key: (image_hash, MipLevel) - image_hash includes page_id and file identity
/// Value: Arc<DecodedImage>
///
/// Memory Management:
/// - Maximum memory: DEFAULT_MEMORY_LIMIT_MB (256 MB)
/// - Eviction: Priority-aware LRU when memory limit exceeded
pub struct CpuDecodeCache {
    cache: LruCache<(u64, MipLevel), Arc<DecodedImage>>,
    max_memory_bytes: u64,
    current_bytes: u64,
    /// Protection priorities: image_hash -> priority (0 = highest)
    protected_pages: HashMap<u64, usize>,
}

impl CpuDecodeCache {
    /// Create a new CPU decode cache with a fixed memory limit.
    pub fn new_with_memory_limit(max_memory_mb: usize) -> Self {
        Self {
            cache: LruCache::unbounded(),
            max_memory_bytes: (max_memory_mb as u64) * 1024 * 1024,
            current_bytes: 0,
            protected_pages: HashMap::new(),
        }
    }

    /// Create a new CPU decode cache with default memory limit (256 MB).
    pub fn new() -> Self {
        Self::new_with_memory_limit(DEFAULT_MEMORY_LIMIT_MB)
    }

    /// Update the set of protected pages with their priorities.
    pub fn set_protection(&mut self, protections: HashMap<u64, usize>) {
        self.protected_pages = protections;
    }

    /// Calculate decoded image memory usage in bytes (RGBA8 format).
    fn calculate_bytes(width: u32, height: u32) -> u64 {
        (width as u64) * (height as u64) * 4
    }

    /// Find the best candidate for eviction.
    /// 1. Prefer unprotected items (starting from LRU end).
    /// 2. If all items are protected, pick the one with the highest priority value (lowest protection).
    fn find_eviction_candidate(&self) -> Option<(u64, MipLevel)> {
        if self.cache.is_empty() {
            return None;
        }

        // Step 1: Find an unprotected item.
        // lru::iter() is MRU -> LRU. We want LRU first, so we use rev().
        for (key, _) in self.cache.iter().rev() {
            if !self.protected_pages.contains_key(&key.0) {
                return Some(*key);
            }
        }

        // Step 2: All items are protected. Find the one with highest priority value.
        let mut worst_priority = 0;
        let mut candidate = None;

        for (key, _) in self.cache.iter() {
            if let Some(&p) = self.protected_pages.get(&key.0) {
                if p >= worst_priority {
                    worst_priority = p;
                    candidate = Some(*key);
                }
            } else {
                // Theoretical fallback: if we missed an unprotected item in Step 1
                return Some(*key);
            }
        }

        candidate
    }

    /// Evict entries until we have enough space for a new image.
    fn evict_until_space(&mut self, required_bytes: u64) {
        while self.current_bytes + required_bytes > self.max_memory_bytes {
            if let Some(key) = self.find_eviction_candidate() {
                if let Some(evicted) = self.cache.pop(&key) {
                    let evicted_bytes = Self::calculate_bytes(evicted.width, evicted.height);
                    self.current_bytes = self.current_bytes.saturating_sub(evicted_bytes);
                } else {
                    break;
                }
            } else {
                break; // Cache is empty or logic failure
            }
        }
    }

    /// Get a decoded image from the cache and update LRU order.
    #[allow(dead_code)]
    pub fn get(&mut self, image_hash: u64, mip: MipLevel) -> Option<Arc<DecodedImage>> {
        self.cache.get(&(image_hash, mip)).cloned()
    }

    /// Get a decoded image from the cache without updating LRU order.
    #[allow(dead_code)]
    pub fn peek(&self, image_hash: u64, mip: MipLevel) -> Option<Arc<DecodedImage>> {
        self.cache.peek(&(image_hash, mip)).cloned()
    }

    /// Insert a decoded image into the cache.
    /// Automatically evicts LRU entries if memory limit exceeded.
    pub fn insert(&mut self, image_hash: u64, mip: MipLevel, image: Arc<DecodedImage>) {
        let image_bytes = Self::calculate_bytes(image.width, image.height);

        // Evict if necessary
        self.evict_until_space(image_bytes);

        // Insert new image
        self.cache.put((image_hash, mip), image);
        self.current_bytes += image_bytes;
    }

    /// Check if the cache contains a specific key.
    pub fn contains(&self, image_hash: u64, mip: MipLevel) -> bool {
        self.cache.contains(&(image_hash, mip))
    }

    /// Remove a specific key from the cache.
    pub fn remove(&mut self, image_hash: u64, mip: MipLevel) -> Option<Arc<DecodedImage>> {
        if let Some(evicted) = self.cache.pop(&(image_hash, mip)) {
            let evicted_bytes = Self::calculate_bytes(evicted.width, evicted.height);
            self.current_bytes = self.current_bytes.saturating_sub(evicted_bytes);
            Some(evicted)
        } else {
            None
        }
    }

    /// Clear all entries from the cache.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
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
        (self.current_bytes / 1024 / 1024) as usize
    }

    /// Get maximum memory limit in MB.
    #[allow(dead_code)]
    pub fn max_memory_mb(&self) -> usize {
        (self.max_memory_bytes / 1024 / 1024) as usize
    }

    pub fn set_max_memory_mb(&mut self, max_memory_mb: usize) {
        self.max_memory_bytes = (max_memory_mb as u64) * 1024 * 1024;
        self.evict_until_space(0);
    }
}

impl Default for CpuDecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_cache_basic() {
        let mut cache = CpuDecodeCache::new_with_memory_limit(10);

        let image = Arc::new(DecodedImage {
            width: 100,
            height: 100,
            original_width: 100,
            original_height: 100,
            pixels: vec![0u8; 100 * 100 * 4],
            icc_profile: None,
            exif: None,
        });

        cache.insert(0, MipLevel::Full, image.clone());

        assert!(cache.contains(0, MipLevel::Full));
        assert!(!cache.contains(0, MipLevel::Half));

        let retrieved = cache.get(0, MipLevel::Full);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_cpu_cache_small_image_tracking() {
        // 10 MB limit
        let mut cache = CpuDecodeCache::new_with_memory_limit(10);

        // 512x256 RGBA = 0.5 MB = 524288 bytes
        let image = Arc::new(DecodedImage {
            width: 512,
            height: 256,
            original_width: 512,
            original_height: 256,
            pixels: vec![0u8; 512 * 256 * 4],
            icc_profile: None,
            exif: None,
        });
        cache.insert(1, MipLevel::Full, image);

        // Must be tracked accurately as 524288 bytes
        assert_eq!(cache.current_bytes, 512 * 256 * 4);
    }

    #[test]
    fn test_cpu_cache_eviction_precision() {
        // 1 MB limit
        let mut cache = CpuDecodeCache::new_with_memory_limit(1);

        // Insert 0.75 MB image (fits)
        let img1 = Arc::new(DecodedImage {
            width: 768,
            height: 256,
            original_width: 768,
            original_height: 256,
            pixels: vec![0u8; 768 * 256 * 4],
            icc_profile: None,
            exif: None,
        });
        cache.insert(1, MipLevel::Full, img1);
        assert!(cache.contains(1, MipLevel::Full));

        // Insert another 0.75 MB image (total 1.5MB > 1MB limit -> evicts first)
        let img2 = Arc::new(DecodedImage {
            width: 768,
            height: 256,
            original_width: 768,
            original_height: 256,
            pixels: vec![0u8; 768 * 256 * 4],
            icc_profile: None,
            exif: None,
        });
        cache.insert(2, MipLevel::Full, img2);

        assert!(!cache.contains(1, MipLevel::Full)); // Evicted
        assert!(cache.contains(2, MipLevel::Full)); // Kept
    }

    #[test]
    fn test_cpu_cache_priority_protection() {
        // 1 MB limit (fits ~2 images of 0.75MB each if we calculate carefully, or just 1 if strictly 1MB)
        // 768x256x4 = 786432 bytes = 0.75 MB.
        let mut cache = CpuDecodeCache::new_with_memory_limit(1);

        let img1 = Arc::new(DecodedImage {
            width: 768,
            height: 256,
            original_width: 768,
            original_height: 256,
            pixels: vec![0u8; 768 * 256 * 4],
            icc_profile: None,
            exif: None,
        });
        let img2 = Arc::new(DecodedImage {
            width: 768,
            height: 256,
            original_width: 768,
            original_height: 256,
            pixels: vec![0u8; 768 * 256 * 4],
            icc_profile: None,
            exif: None,
        });

        cache.insert(1, MipLevel::Full, img1);
        cache.insert(2, MipLevel::Full, img2);
        // Cache now contains [1, 2] (1 is LRU, 2 is MRU)
        // Memory: 1.5MB > 1MB. Wait, our insert evicts until space.
        // So at this point, cache ONLY contains [2]. 1 was evicted when inserting 2.

        // Correct test:
        let mut cache = CpuDecodeCache::new_with_memory_limit(2); // 2MB limit
        cache.insert(
            1,
            MipLevel::Full,
            Arc::new(DecodedImage {
                width: 512,
                height: 512,
                original_width: 512,
                original_height: 512,
                pixels: vec![0u8; 512 * 512 * 4],
                icc_profile: None,
                exif: None,
            }),
        ); // 1MB
        cache.insert(
            2,
            MipLevel::Full,
            Arc::new(DecodedImage {
                width: 512,
                height: 512,
                original_width: 512,
                original_height: 512,
                pixels: vec![0u8; 512 * 512 * 4],
                icc_profile: None,
                exif: None,
            }),
        ); // 1MB
        // Cache: [1, 2], total 2MB. 1 is LRU.

        // Protect 1 (older one)
        let mut protections = HashMap::new();
        protections.insert(1, 0);
        cache.set_protection(protections);

        // Insert 3 (1MB). Needs to evict 1MB.
        cache.insert(
            3,
            MipLevel::Full,
            Arc::new(DecodedImage {
                width: 512,
                height: 512,
                original_width: 512,
                original_height: 512,
                pixels: vec![0u8; 512 * 512 * 4],
                icc_profile: None,
                exif: None,
            }),
        );

        // 1 should be KEPT because it's protected. 2 should be EVICTED even though it's MRU.
        assert!(cache.contains(1, MipLevel::Full));
        assert!(!cache.contains(2, MipLevel::Full));
        assert!(cache.contains(3, MipLevel::Full));
    }

    #[test]
    fn test_cpu_cache_priority_among_protected() {
        let mut cache = CpuDecodeCache::new_with_memory_limit(2);
        cache.insert(
            1,
            MipLevel::Full,
            Arc::new(DecodedImage {
                width: 512,
                height: 512,
                original_width: 512,
                original_height: 512,
                pixels: vec![0u8; 512 * 512 * 4],
                icc_profile: None,
                exif: None,
            }),
        );
        cache.insert(
            2,
            MipLevel::Full,
            Arc::new(DecodedImage {
                width: 512,
                height: 512,
                original_width: 512,
                original_height: 512,
                pixels: vec![0u8; 512 * 512 * 4],
                icc_profile: None,
                exif: None,
            }),
        );

        // Both protected, but 1 has better priority
        let mut protections = HashMap::new();
        protections.insert(1, 0); // High priority
        protections.insert(2, 10); // Low priority (within window)
        cache.set_protection(protections);

        // Insert 3. Must evict one.
        cache.insert(
            3,
            MipLevel::Full,
            Arc::new(DecodedImage {
                width: 512,
                height: 512,
                original_width: 512,
                original_height: 512,
                pixels: vec![0u8; 512 * 512 * 4],
                icc_profile: None,
                exif: None,
            }),
        );

        // 2 should be evicted because it has worse priority (10) than 1 (0).
        assert!(cache.contains(1, MipLevel::Full));
        assert!(!cache.contains(2, MipLevel::Full));
        assert!(cache.contains(3, MipLevel::Full));
    }
}
