//! Synapsis Recycle Module
//!
//! Provides intelligent message recycling with smart categorization.
//!
//! # Components
//!
//! - [`categorizer`] - Smart message categorization
//! - [`bin`] - Recycle bin storage and search

pub mod bin;
pub mod categorizer;

pub use bin::*;
pub use categorizer::*;
