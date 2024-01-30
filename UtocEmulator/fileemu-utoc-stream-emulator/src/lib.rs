//! # Some Cool Reloaded Library
//! Here's the crate documentation.

pub mod asset_collector; // Building tree of directories/files
pub mod exports; // FFI (called from C#)
pub mod io_package; // Handling IO Store packages
pub mod io_toc; // Types for IO Store Table of Contents
pub mod pak_package; // Handling cooked packages (WIP)
pub mod toc_factory; // Build IO Store TOC
pub mod platform; // Platform agnostic abstractions
pub mod string; // Unreal serialized string types