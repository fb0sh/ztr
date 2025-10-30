use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use walkdir::WalkDir;

use ztr_lib::compressor;
use ztr_lib::config::Config;
use ztr_lib::ignore_rules::IgnoreRules;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 指定配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 初始化：创建默认配置文件 ztr.toml
    Init,
    /// 显示支持的压缩格式
    Show,
    /// 压缩指定目录
    Compress {
        /// 要压缩的目录路径，默认为当前目录
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            Config::create_default_config_file(Some(&PathBuf::from("ztr.toml")))?;
            println!("默认配置文件 ztr.toml 已创建。");
        }
        Some(Commands::Show) => {
            println!("支持的压缩格式：");
            println!("- zip: 兼容性最好，几乎所有系统都支持");
            println!("- tar.gz: Linux 常用格式，压缩率适中");
            println!("- 7z: 压缩率最高，支持多种算法，但需要系统安装 7z 命令行工具");
        }
        Some(Commands::Compress { path }) => {
            let config_path = cli.config.unwrap_or_else(|| PathBuf::from("ztr.toml"));
            let config = Config::load(&config_path)
                .with_context(|| format!("无法加载配置文件: {}", config_path.display()))?;

            let base_dir =
                path.unwrap_or_else(|| std::env::current_dir().expect("无法获取当前目录"));
            if !base_dir.is_dir() {
                anyhow::bail!("要压缩的路径不是一个目录: {}", base_dir.display());
            }

            // 收集所有文件路径
            let all_files = collect_all_files(&base_dir)?;

            // 应用忽略规则
            let ignore_rules = IgnoreRules::new(&config.get_ignore_rules(), &base_dir)?;
            let files_to_compress = ignore_rules.filter_files(all_files.into_iter())?;

            if files_to_compress.is_empty() {
                println!("没有需要压缩的文件。");
                return Ok(());
            }

            let output_archive_path =
                compressor::compress_directory(&config, &base_dir, files_to_compress)?;
            println!("压缩文件已创建: {}", output_archive_path.display());
        }
        None => {
            let config_path = cli.config.unwrap_or_else(|| PathBuf::from("ztr.toml"));
            if !config_path.exists() {
                println!("未找到配置文件 ztr.toml。您可以运行 `ztr init` 创建一个默认配置文件。");
                return Ok(());
            }
            let config = Config::load(&config_path)
                .with_context(|| format!("无法加载配置文件: {}", config_path.display()))?;

            let base_dir = std::env::current_dir().expect("无法获取当前目录");

            // 收集所有文件路径
            let all_files = collect_all_files(&base_dir)?;

            // 应用忽略规则
            let ignore_rules = IgnoreRules::new(&config.get_ignore_rules(), &base_dir)?;
            let files_to_compress = ignore_rules.filter_files(all_files.into_iter())?;

            if files_to_compress.is_empty() {
                println!("没有需要压缩的文件。");
                return Ok(());
            }

            let output_archive_path =
                compressor::compress_directory(&config, &base_dir, files_to_compress)?;
            println!("压缩文件已创建: {}", output_archive_path.display());
        }
    }

    Ok(())
}

/// 递归地收集指定目录中所有文件的路径。
fn collect_all_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}
