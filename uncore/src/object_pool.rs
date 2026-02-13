//! ## Object Pool Module
//!
//! This module provides object pooling functionality to improve performance
//! by reusing entities instead of frequently creating and destroying them.
//! This is particularly useful for particles, temporary effects, and other
//! short-lived entities.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

/// Resource for managing object pools
#[derive(Resource, Default)]
pub struct ObjectPoolManager {
    /// Pools for different entity types
    pools: HashMap<String, EntityPool>,
}

/// A pool of entities that can be reused
#[derive(Debug)]
struct EntityPool {
    /// Available entities ready to be reused
    available: VecDeque<Entity>,
    /// Currently active entities
    active: std::collections::HashSet<Entity>,
    /// Maximum pool size to prevent memory bloat
    max_size: usize,
}

impl EntityPool {
    fn new(max_size: usize) -> Self {
        Self {
            available: VecDeque::new(),
            active: std::collections::HashSet::new(),
            max_size,
        }
    }
}

impl ObjectPoolManager {
    /// Get an entity from the pool, or create a new one if none available
    pub fn get_entity(&mut self, pool_name: &str, commands: &mut Commands, spawn_fn: impl FnOnce(&mut Commands) -> Entity) -> Entity {
        let pool = self.pools.entry(pool_name.to_string()).or_insert_with(|| EntityPool::new(100));
        
        if let Some(entity) = pool.available.pop_front() {
            // Reuse existing entity
            pool.active.insert(entity);
            entity
        } else {
            // Create new entity
            let entity = spawn_fn(commands);
            pool.active.insert(entity);
            entity
        }
    }
    
    /// Return an entity to the pool for reuse
    pub fn return_entity(&mut self, pool_name: &str, entity: Entity, commands: &mut Commands) {
        if let Some(pool) = self.pools.get_mut(pool_name) {
            if pool.active.remove(&entity) {
                // Only add back to pool if we haven't exceeded max size
                if pool.available.len() < pool.max_size {
                    pool.available.push_back(entity);
                } else {
                    // Pool is full, despawn the entity
                    commands.entity(entity).despawn();
                }
            }
        }
    }
    
    /// Clear all pools (useful for cleanup)
    pub fn clear_all(&mut self, commands: &mut Commands) {
        for pool in self.pools.values_mut() {
            // Despawn all entities in the pool
            for entity in pool.available.drain(..) {
                commands.entity(entity).despawn();
            }
            pool.active.clear();
        }
    }
    
    /// Get statistics about pool usage
    pub fn get_stats(&self) -> HashMap<String, PoolStats> {
        self.pools.iter().map(|(name, pool)| {
            (name.clone(), PoolStats {
                available: pool.available.len(),
                active: pool.active.len(),
                total: pool.available.len() + pool.active.len(),
            })
        }).collect()
    }
}

/// Statistics about a pool's usage
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available: usize,
    pub active: usize,
    pub total: usize,
}

/// Component to mark entities that should be pooled
#[derive(Component, Debug, Clone)]
pub struct PooledEntity {
    pub pool_name: String,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl PooledEntity {
    pub fn new(pool_name: String, max_lifetime: f32) -> Self {
        Self {
            pool_name,
            lifetime: 0.0,
            max_lifetime,
        }
    }
    
    /// Check if the entity has exceeded its lifetime
    pub fn is_expired(&self) -> bool {
        self.lifetime >= self.max_lifetime
    }
    
    /// Update the entity's lifetime
    pub fn update_lifetime(&mut self, delta_time: f32) {
        self.lifetime += delta_time;
    }
}

/// System that manages pooled entities lifetime and returns expired ones to the pool
pub fn manage_pooled_entities(
    mut pool_manager: ResMut<ObjectPoolManager>,
    mut commands: Commands,
    mut pooled_entities: Query<(Entity, &mut PooledEntity)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    
    for (entity, mut pooled) in pooled_entities.iter_mut() {
        pooled.update_lifetime(delta_time);
        
        if pooled.is_expired() {
            // Return entity to pool
            pool_manager.return_entity(&pooled.pool_name, entity, &mut commands);
        }
    }
}

/// System that periodically cleans up empty pools to prevent memory leaks
pub fn cleanup_empty_pools(
    mut pool_manager: ResMut<ObjectPoolManager>,
    mut cleanup_timer: Local<f32>,
    time: Res<Time>,
) {
    *cleanup_timer += time.delta_secs();
    
    // Clean up every 30 seconds
    if *cleanup_timer >= 30.0 {
        *cleanup_timer = 0.0;
        
        // Remove pools that have been empty for too long
        pool_manager.pools.retain(|_name, pool| {
            pool.available.len() + pool.active.len() > 0
        });
    }
}

/// Helper function to spawn a pooled particle effect
pub fn spawn_pooled_particle(
    pool_manager: &mut ResMut<ObjectPoolManager>,
    commands: &mut Commands,
    position: Vec3,
    effect_type: ParticleEffectType,
) -> Entity {
    pool_manager.get_entity("particles", commands, |cmds| {
        cmds.spawn((
            PooledEntity::new("particles".to_string(), 2.0), // 2 second lifetime
            Transform::from_translation(position),
            effect_type.clone(),
        )).id()
    })
}

/// Types of particle effects that can be pooled
#[derive(Component, Debug, Clone)]
pub enum ParticleEffectType {
    Smoke,
    Spark,
    Dust,
    Explosion,
    Ectoplasm,
    FreezingEffect,
    EMFEffect,
}

/// System that processes different particle effect types
pub fn process_particle_effects(
    mut particles: Query<(&mut Transform, &ParticleEffectType, &PooledEntity)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    
    for (mut transform, effect_type, pooled) in particles.iter_mut() {
        match effect_type {
            ParticleEffectType::Smoke => {
                // Smoke particles float upward
                transform.translation.y += delta_time * 0.5;
                transform.translation.z += delta_time * 0.1;
            }
            ParticleEffectType::Spark => {
                // Spark particles fall down
                transform.translation.y -= delta_time * 2.0;
            }
            ParticleEffectType::Dust => {
                // Dust particles have random movement
                transform.translation.x += (delta_time * 0.3) * (pooled.lifetime * 0.5).sin();
                transform.translation.y += (delta_time * 0.2) * (pooled.lifetime * 0.7).cos();
            }
            ParticleEffectType::Explosion => {
                // Explosion particles spread outward
                let spread = pooled.lifetime * 2.0;
                transform.translation.x += delta_time * spread * 0.5;
                transform.translation.y += delta_time * spread * 0.3;
            }
            ParticleEffectType::Ectoplasm => {
                // Ectoplasm particles have ghostly movement
                transform.translation.y += delta_time * 0.3 * (pooled.lifetime * 1.2).sin();
                transform.translation.x += delta_time * 0.2 * (pooled.lifetime * 0.8).cos();
            }
            ParticleEffectType::FreezingEffect => {
                // Freezing effect particles are slow and crystalline
                transform.translation.y += delta_time * 0.1;
            }
            ParticleEffectType::EMFEffect => {
                // EMF particles have electrical movement
                transform.translation.y += delta_time * 0.4 * (pooled.lifetime * 3.0).sin();
                transform.translation.x += delta_time * 0.3 * (pooled.lifetime * 2.5).cos();
            }
        }
    }
}
