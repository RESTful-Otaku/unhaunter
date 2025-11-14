//! ## Optimized Noise System
//!
//! This module provides an optimized Perlin noise system with reduced memory usage
//! and lazy loading capabilities.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

/// Optimized Perlin noise resource with reduced memory usage
#[derive(Resource, Debug)]
pub struct OptimizedPerlinNoise {
    /// The underlying Perlin noise generator
    perlin: Perlin,
    /// Cache for frequently accessed noise values
    cache: HashMap<(i32, i32), f32>,
    /// Maximum cache size to prevent memory bloat
    max_cache_size: usize,
    /// Resolution multiplier for coordinate mapping
    resolution: f32,
    /// Cache hit counter for statistics
    cache_hits: u64,
    /// Cache miss counter for statistics
    cache_misses: u64,
}

impl OptimizedPerlinNoise {
    /// Create a new OptimizedPerlinNoise with minimal memory footprint
    pub fn new(seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        
        info!("Optimized Perlin noise system initialized with seed: {}", seed);
        
        Self {
            perlin,
            cache: HashMap::new(),
            max_cache_size: 10000, // Much smaller than the 16M entries in the original
            resolution: 0.01,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Get noise value with caching and lazy computation
    pub fn get(&mut self, x: f32, y: f32) -> f32 {
        // Quantize coordinates to cache grid
        let cache_x = (x / self.resolution).round() as i32;
        let cache_y = (y / self.resolution).round() as i32;
        
        // Check cache first
        if let Some(&value) = self.cache.get(&(cache_x, cache_y)) {
            self.cache_hits += 1;
            return value;
        }
        
        // Cache miss - compute the value
        self.cache_misses += 1;
        let value = self.perlin.get([x as f64, y as f64]) as f32;
        
        // Add to cache if we haven't exceeded the limit
        if self.cache.len() < self.max_cache_size {
            self.cache.insert((cache_x, cache_y), value);
        } else {
            // Cache is full, remove oldest entries (simple cleanup)
            self.cleanup_cache();
            self.cache.insert((cache_x, cache_y), value);
        }
        
        value
    }
    
    /// Get noise value without caching (for one-off calculations)
    pub fn get_direct(&self, x: f32, y: f32) -> f32 {
        self.perlin.get([x as f64, y as f64]) as f32
    }
    
    /// Get noise value with multiple octaves for more complex patterns
    pub fn get_octaves(&mut self, x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;
        
        for _ in 0..octaves {
            value += self.get(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        
        value / max_value
    }
    
    /// Clean up cache by removing some entries
    fn cleanup_cache(&mut self) {
        // Remove 25% of the cache entries
        let target_size = self.max_cache_size * 3 / 4;
        let keys_to_remove: Vec<_> = self.cache.keys().take(self.cache.len() - target_size).cloned().collect();
        
        for key in keys_to_remove {
            self.cache.remove(&key);
        }
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> NoiseCacheStats {
        let total_requests = self.cache_hits + self.cache_misses;
        let hit_rate = if total_requests > 0 {
            (self.cache_hits as f32 / total_requests as f32) * 100.0
        } else {
            0.0
        };
        
        NoiseCacheStats {
            cache_size: self.cache.len(),
            max_cache_size: self.max_cache_size,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            hit_rate,
        }
    }
    
    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
}

/// Statistics about noise cache performance
#[derive(Debug, Clone)]
pub struct NoiseCacheStats {
    pub cache_size: usize,
    pub max_cache_size: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f32,
}

impl Default for OptimizedPerlinNoise {
    fn default() -> Self {
        Self::new(42) // Default seed
    }
}

/// System that periodically reports noise cache statistics
pub fn report_noise_cache_stats(
    noise: ResMut<OptimizedPerlinNoise>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Report every 30 seconds
    if *timer >= 30.0 {
        *timer = 0.0;
        
        let stats = (*noise).get_cache_stats();
        if stats.cache_hits + stats.cache_misses > 0 {
            info!(
                "Noise cache stats: {}/{} entries, {:.1}% hit rate ({} hits, {} misses)",
                stats.cache_size,
                stats.max_cache_size,
                stats.hit_rate,
                stats.cache_hits,
                stats.cache_misses
            );
        }
    }
}

/// System that cleans up noise cache periodically
pub fn cleanup_noise_cache(
    mut noise: ResMut<OptimizedPerlinNoise>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Clean up every 60 seconds
    if *timer >= 60.0 {
        *timer = 0.0;
        
        let stats_before = noise.get_cache_stats();
        noise.clear_cache();
        info!("Cleared noise cache (was {} entries)", stats_before.cache_size);
    }
}

/// Helper functions for common noise patterns
pub mod noise_patterns {
    use super::*;
    
    /// Generate terrain-like noise
    pub fn terrain_noise(noise: &mut OptimizedPerlinNoise, x: f32, y: f32) -> f32 {
        noise.get_octaves(x, y, 4, 0.5, 2.0)
    }
    
    /// Generate cloud-like noise
    pub fn cloud_noise(noise: &mut OptimizedPerlinNoise, x: f32, y: f32) -> f32 {
        noise.get_octaves(x, y, 6, 0.6, 2.0)
    }
    
    /// Generate marble-like noise
    pub fn marble_noise(noise: &mut OptimizedPerlinNoise, x: f32, y: f32) -> f32 {
        let base = noise.get(x * 0.1, y * 0.1);
        (base * 10.0).sin() * 0.5 + 0.5
    }
    
    /// Generate wood-like noise
    pub fn wood_noise(noise: &mut OptimizedPerlinNoise, x: f32, y: f32) -> f32 {
        let rings = (x * x + y * y).sqrt() * 10.0;
        let noise_value = noise.get(rings, y * 0.1);
        (rings + noise_value * 0.5).sin() * 0.5 + 0.5
    }
}
