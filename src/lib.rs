//! `ztr_lib` 是一个可配置的压缩工具库，支持多种压缩格式（zip, tar.gz, 7z），
//! 并能根据配置忽略指定文件和目录。
//!
//! 主要功能包括：
//! - 从配置文件加载压缩配置。
//! - 根据 Gitignore 风格的规则过滤文件。
//! - 支持多种压缩格式进行文件压缩。
//!
//! # 示例
//!
//! ```no_run
//! use ztr_lib::config::Config;
//! use ztr_lib::compressor;
//! use std::path::PathBuf;
//!
//! fn main() -> anyhow::Result<()> {
//!     let config_path = PathBuf::from("ztr.toml");
//!     let config = Config::load(&config_path)?;
//!     
//!     let current_dir = std::env::current_dir()?;
//!     compressor::compress_directory(&config, &current_dir)?;
//!     
//!     Ok(())
//! }
//! ```
pub mod compressor;
pub mod config;
pub mod ignore_rules;
