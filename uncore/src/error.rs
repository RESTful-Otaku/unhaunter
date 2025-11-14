//! ## Error Handling Module
//!
//! This module provides unified error handling for the Unhaunter game.
//! It defines a centralized error type that can represent various kinds
//! of errors that may occur during game execution.

use bevy::prelude::*;
use thiserror::Error;

/// Unified error type for the Unhaunter game
#[derive(Error, Debug)]
pub enum UnhaunterError {
    #[error("Profile persistence failed: {0}")]
    ProfilePersistence(String),
    
    #[error("Asset loading failed: {0}")]
    AssetLoading(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Game state error: {0}")]
    GameState(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for UnhaunterError {
    fn from(err: std::io::Error) -> Self {
        UnhaunterError::FileSystem(err.to_string())
    }
}

impl From<ron::Error> for UnhaunterError {
    fn from(err: ron::Error) -> Self {
        UnhaunterError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for UnhaunterError {
    fn from(err: serde_json::Error) -> Self {
        UnhaunterError::Serialization(err.to_string())
    }
}

/// Result type alias for Unhaunter operations
pub type UnhaunterResult<T> = Result<T, UnhaunterError>;

/// Extension trait for better error handling
pub trait UnhaunterResultExt<T> {
    /// Log error and continue with default value instead of panicking
    fn log_and_default(self, default: T, context: &str) -> T;
    
    /// Log error and continue with None instead of panicking
    fn log_and_none(self, context: &str) -> Option<T>;
}

impl<T> UnhaunterResultExt<T> for UnhaunterResult<T> {
    fn log_and_default(self, default: T, context: &str) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                error!("{}: {}", context, e);
                default
            }
        }
    }
    
    fn log_and_none(self, context: &str) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                error!("{}: {}", context, e);
                None
            }
        }
    }
}

/// Resource for tracking error state
#[derive(Resource, Default)]
pub struct ErrorTracker {
    pub errors: Vec<UnhaunterError>,
    pub warnings: Vec<String>,
}

impl ErrorTracker {
    pub fn add_error(&mut self, error: UnhaunterError) {
        error!("Game error: {}", error);
        self.errors.push(error);
        
        // Limit error history to prevent memory issues
        if self.errors.len() > 100 {
            self.errors.drain(0..50);
        }
    }
    
    pub fn add_warning(&mut self, warning: String) {
        warn!("Game warning: {}", warning);
        self.warnings.push(warning);
        
        // Limit warning history
        if self.warnings.len() > 50 {
            self.warnings.drain(0..25);
        }
    }
    
    pub fn clear_errors(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}
