//! Performance Optimization for Production
//!
//! Caching, lazy evaluation, and optimizations for hot paths:
//! - Merkle tree construction
//! - Header validation
//! - State compression

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// LRU Cache for commonly accessed items
#[derive(Debug)]
pub struct LruCache<K: Clone + Eq + std::hash::Hash, V: Clone> {
    /// Cache data
    data: HashMap<K, CacheEntry<V>>,
    /// Maximum cache size
    max_size: usize,
    /// Access order (for LRU eviction)
    access_order: Vec<K>,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V: Clone> {
    value: V,
    created_at: SystemTime,
    accessed_at: SystemTime,
    hit_count: u32,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LruCache<K, V> {
    /// Create a new LRU cache
    pub fn new(max_size: usize) -> Self {
        LruCache {
            data: HashMap::new(),
            max_size,
            access_order: Vec::new(),
        }
    }

    /// Get a value from cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.get_mut(key) {
            entry.accessed_at = SystemTime::now();
            entry.hit_count += 1;
            let value = entry.value.clone();
            
            // Update access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
                self.access_order.push(key.clone());
            }
            
            Some(value)
        } else {
            None
        }
    }

    /// Insert a value into cache
    pub fn insert(&mut self, key: K, value: V) {
        // Remove oldest entry if cache is full
        if self.data.len() >= self.max_size && !self.data.contains_key(&key) {
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.data.remove(&lru_key);
                self.access_order.remove(0);
            }
        }

        let entry = CacheEntry {
            value,
            created_at: SystemTime::now(),
            accessed_at: SystemTime::now(),
            hit_count: 0,
        };

        if self.data.insert(key.clone(), entry).is_none() {
            self.access_order.push(key);
        }
    }

    /// Get cache statistics
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.data.clear();
        self.access_order.clear();
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }

        let total_hits: u32 = self.data.values().map(|e| e.hit_count).sum();
        let total_entries = self.data.len() as u32;

        if total_entries == 0 {
            0.0
        } else {
            (total_hits as f64) / (total_entries as f64)
        }
    }
}

/// TTL (Time-to-Live) Cache for temporary data
#[derive(Debug)]
pub struct TtlCache<K: Clone + Eq + std::hash::Hash, V: Clone> {
    /// Cache data
    data: HashMap<K, TtlCacheEntry<V>>,
    /// TTL duration
    ttl: Duration,
}

/// TTL cache entry
#[derive(Debug, Clone)]
struct TtlCacheEntry<V: Clone> {
    value: V,
    created_at: SystemTime,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> TtlCache<K, V> {
    /// Create a new TTL cache
    pub fn new(ttl: Duration) -> Self {
        TtlCache {
            data: HashMap::new(),
            ttl,
        }
    }

    /// Get a value if it hasn't expired
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.get(key) {
            if let Ok(elapsed) = entry.created_at.elapsed() {
                if elapsed < self.ttl {
                    return Some(entry.value.clone());
                } else {
                    // Entry expired, remove it
                    self.data.remove(key);
                }
            }
        }
        None
    }

    /// Insert a value into cache
    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(
            key,
            TtlCacheEntry {
                value,
                created_at: SystemTime::now(),
            },
        );
    }

    /// Clean expired entries
    pub fn cleanup_expired(&mut self) {
        let mut expired_keys = Vec::new();
        
        for (key, entry) in self.data.iter() {
            if let Ok(elapsed) = entry.created_at.elapsed() {
                if elapsed >= self.ttl {
                    expired_keys.push(key.clone());
                }
            }
        }

        for key in expired_keys {
            self.data.remove(&key);
        }
    }

    /// Cache size
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Lazy computed value for expensive operations
pub struct Lazy<T> {
    value: Option<T>,
    computed: bool,
}

impl<T> Lazy<T> {
    /// Create a new lazy value
    pub fn new() -> Self {
        Lazy {
            value: None,
            computed: false,
        }
    }

    /// Get or compute the value
    pub fn get_or_compute<F>(&mut self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.computed {
            self.value = Some(f());
            self.computed = true;
        }
        self.value.as_ref().unwrap()
    }

    /// Check if value has been computed
    pub fn is_computed(&self) -> bool {
        self.computed
    }

    /// Reset lazy value
    pub fn reset(&mut self) {
        self.value = None;
        self.computed = false;
    }
}

impl<T> Default for Lazy<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch processor for high-volume operations
pub struct BatchProcessor<T> {
    /// Pending items
    queue: Vec<T>,
    /// Batch size threshold
    batch_size: usize,
    /// Last flush timestamp
    last_flush: SystemTime,
    /// Flush timeout
    flush_timeout: Duration,
}

impl<T> BatchProcessor<T> {
    /// Create a new batch processor
    pub fn new(batch_size: usize, flush_timeout: Duration) -> Self {
        BatchProcessor {
            queue: Vec::new(),
            batch_size,
            last_flush: SystemTime::now(),
            flush_timeout,
        }
    }

    /// Add item to batch
    pub fn add(&mut self, item: T) -> bool {
        self.queue.push(item);
        self.should_flush()
    }

    /// Check if batch should be flushed
    pub fn should_flush(&self) -> bool {
        // Flush if batch size reached
        if self.queue.len() >= self.batch_size {
            return true;
        }

        // Flush if timeout elapsed
        if let Ok(elapsed) = self.last_flush.elapsed() {
            if elapsed >= self.flush_timeout {
                return true;
            }
        }

        false
    }

    /// Get pending items for flush
    pub fn take_batch(&mut self) -> Vec<T> {
        self.last_flush = SystemTime::now();
        std::mem::take(&mut self.queue)
    }

    /// Get queue size
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Clear queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

/// Statistics tracker for performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Number of operations
    pub operation_count: u64,
    /// Total time spent (in milliseconds)
    pub total_time_ms: u64,
    /// Minimum operation time (in milliseconds)
    pub min_time_ms: u64,
    /// Maximum operation time (in milliseconds)
    pub max_time_ms: u64,
    /// Cache hit count
    pub cache_hits: u64,
    /// Cache miss count
    pub cache_misses: u64,
}

impl PerformanceStats {
    /// Create new stats tracker
    pub fn new() -> Self {
        PerformanceStats {
            operation_count: 0,
            total_time_ms: 0,
            min_time_ms: u64::MAX,
            max_time_ms: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Record an operation
    pub fn record_operation(&mut self, duration_ms: u64) {
        self.operation_count += 1;
        self.total_time_ms += duration_ms;
        self.min_time_ms = self.min_time_ms.min(duration_ms);
        self.max_time_ms = self.max_time_ms.max(duration_ms);
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Get average operation time
    pub fn avg_time_ms(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            (self.total_time_ms as f64) / (self.operation_count as f64)
        }
    }

    /// Get cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (total as f64)
        }
    }
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_creation() {
        let cache: LruCache<String, i32> = LruCache::new(2);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_lru_cache_insert_get() {
        let mut cache: LruCache<String, i32> = LruCache::new(2);
        cache.insert("key1".to_string(), 42);
        
        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(42));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache: LruCache<String, i32> = LruCache::new(2);
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);  // Should evict key1
        
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), Some(2));
        assert_eq!(cache.get(&"key3".to_string()), Some(3));
    }

    #[test]
    fn test_lru_cache_clear() {
        let mut cache: LruCache<String, i32> = LruCache::new(2);
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_ttl_cache_creation() {
        let cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(10));
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_ttl_cache_insert_get() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(10));
        cache.insert("key1".to_string(), 42);
        
        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(42));
    }

    #[test]
    fn test_ttl_cache_expiration() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_millis(100));
        cache.insert("key1".to_string(), 42);
        
        // Value should be available immediately
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));
        
        // Value should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_lazy_value() {
        let mut lazy = Lazy::new();
        assert!(!lazy.is_computed());
        
        let value = lazy.get_or_compute(|| 42);
        assert_eq!(*value, 42);
        assert!(lazy.is_computed());
    }

    #[test]
    fn test_lazy_value_compute_once() {
        let mut lazy = Lazy::new();
        
        let value1 = lazy.get_or_compute(|| 42);
        assert_eq!(*value1, 42);
        
        let value2 = lazy.get_or_compute(|| 100);
        assert_eq!(*value2, 42);  // Should return same value
    }

    #[test]
    fn test_batch_processor_creation() {
        let processor: BatchProcessor<i32> = BatchProcessor::new(10, Duration::from_secs(5));
        assert_eq!(processor.pending_count(), 0);
    }

    #[test]
    fn test_batch_processor_flush_on_size() {
        let mut processor = BatchProcessor::new(2, Duration::from_secs(10));
        
        assert!(!processor.add(1));
        assert!(processor.add(2));  // Should trigger flush
    }

    #[test]
    fn test_batch_processor_take_batch() {
        let mut processor = BatchProcessor::new(10, Duration::from_secs(5));
        processor.add(1);
        processor.add(2);
        processor.add(3);
        
        let batch = processor.take_batch();
        assert_eq!(batch.len(), 3);
        assert_eq!(processor.pending_count(), 0);
    }

    #[test]
    fn test_performance_stats_creation() {
        let stats = PerformanceStats::new();
        assert_eq!(stats.operation_count, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_performance_stats_record_operation() {
        let mut stats = PerformanceStats::new();
        stats.record_operation(10);
        stats.record_operation(20);
        stats.record_operation(30);
        
        assert_eq!(stats.operation_count, 3);
        assert_eq!(stats.total_time_ms, 60);
        assert_eq!(stats.min_time_ms, 10);
        assert_eq!(stats.max_time_ms, 30);
    }

    #[test]
    fn test_performance_stats_avg_time() {
        let mut stats = PerformanceStats::new();
        stats.record_operation(10);
        stats.record_operation(20);
        
        assert_eq!(stats.avg_time_ms(), 15.0);
    }

    #[test]
    fn test_performance_stats_cache_hit_ratio() {
        let mut stats = PerformanceStats::new();
        stats.record_cache_hit();
        stats.record_cache_hit();
        stats.record_cache_miss();
        
        assert_eq!(stats.cache_hit_ratio(), 2.0 / 3.0);
    }

    #[test]
    fn test_lru_cache_hit_rate() {
        let mut cache: LruCache<String, i32> = LruCache::new(2);
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        
        cache.get(&"key1".to_string());
        cache.get(&"key1".to_string());
        cache.get(&"key2".to_string());
        
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_batch_processor_clear() {
        let mut processor = BatchProcessor::new(10, Duration::from_secs(5));
        processor.add(1);
        processor.add(2);
        
        processor.clear();
        assert_eq!(processor.pending_count(), 0);
    }

    #[test]
    fn test_ttl_cache_cleanup_expired() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_millis(100));
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        
        std::thread::sleep(Duration::from_millis(150));
        cache.cleanup_expired();
        
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_lazy_value_reset() {
        let mut lazy = Lazy::new();
        lazy.get_or_compute(|| 42);
        assert!(lazy.is_computed());
        
        lazy.reset();
        assert!(!lazy.is_computed());
    }
}
