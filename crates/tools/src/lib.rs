//! Ria Coder Tool Integration Layer
//!
//! Per SPEC-030 through SPEC-037:
//! - Tool Registry (SPEC-030)
//! - File System Tools (SPEC-031)
//! - Git Tools (SPEC-032)
//! - Build Tools (SPEC-033)
//! - Test Tools (SPEC-034)

pub mod build;
pub mod filesystem;
pub mod git;
pub mod process;
pub mod registry;
pub mod test;
pub mod watcher;

pub use build::BuildTools;
pub use filesystem::FileSystemTools;
pub use git::GitTools;
pub use process::ProcessTools;
pub use registry::ToolRegistry;
pub use test::TestTools;
pub use watcher::FileWatcher;
