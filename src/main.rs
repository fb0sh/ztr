use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod compressor;
mod config;
mod ignore_rules;

use config::Config;

#[derive(Parser)]
#[command(name = "ztr")]
#[command(about = "一个基于配置文件的智能压缩工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建默认配置文件 ztr.toml
    Init,
    /// 显示支持的压缩格式
    Show,
    /// 压缩当前目录
    Compress {
        /// 指定配置文件路径
        #[arg(short, long, default_value = "ztr.toml")]
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            create_default_config()?;
            println!("✓ 已创建默认配置文件: ztr.toml");
        }
        Some(Commands::Show) => {
            show_supported_formats();
        }
        Some(Commands::Compress { config }) => {
            let config = Config::load(&config)?;
            compressor::compress_directory(&config)?;
        }
        None => {
            // 没有命令时，检查ztr.toml是否存在，存在则压缩
            if std::path::Path::new("ztr.toml").exists() {
                println!("找到配置文件 ztr.toml，开始压缩...");
                let config = Config::load("ztr.toml")?;
                compressor::compress_directory(&config)?;
            } else {
                println!("未找到配置文件 ztr.toml");
                println!("使用 'ztr init' 创建配置文件");
                println!("使用 'ztr show' 查看支持的格式");
                println!("使用 'ztr compress' 压缩当前目录");
            }
        }
    }

    Ok(())
}

fn create_default_config() -> Result<()> {
    let default_config = r#"# ZTR 压缩工具配置文件

# 压缩格式: 支持 "zip", "tar.gz", "7z"
format = "tar.gz"

# 输出文件名 (可选，默认使用当前目录名)
# output_name = "my_archive"

# 忽略规则列表 (类似 .gitignore)
ignore = [
    "target/",
    "*.tmp",
    "*.log",
    ".DS_Store",
    "Thumbs.db",
    "*.swp",
    "*.swo",
    "*~",
    ".git/",
    ".svn/",
    ".hg/",
    "node_modules/",
    "__pycache__/",
    ".pytest_cache/",
    ".venv/",
    "venv/",
    "env/",
    "*.pyc",
    "*.pyo",
    "*.pyd",
    ".idea/",
    ".vscode/",
    "*.iml",
]

# 指定忽略文件路径 (可选，默认使用 .gitignore)
# ignore_file = "./.gitignore"
"#;

    std::fs::write("ztr.toml", default_config)?;
    Ok(())
}

fn show_supported_formats() {
    println!("支持的压缩格式:");
    println!("  zip     - ZIP 格式 (兼容性好)");
    println!("  tar.gz  - TAR.GZ 格式 (Linux常用)");
    println!("  7z      - 7-Zip 格式 (压缩率高)");
}
