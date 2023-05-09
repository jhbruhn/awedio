//! Backends handle the device and OS specific logic for outputting audio.
//! See comments on [Renderer][crate::manager::Renderer] for how to implement
//! a new backend which can be done in the crate or in its own crate.

#[cfg(feature = "cpal")]
mod cpal_backend;

#[cfg(feature = "cpal")]
pub use cpal_backend::*;
