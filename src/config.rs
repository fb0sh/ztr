use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 压缩格式: "zip", "tar.gz", "7z"
    pub format: String,
    /// 输出文件名 (可选)
    pub output_name: Option<String>,
    /// 忽略规则列表
    pub ignore: Option<Vec<String>>,
    /// 忽略文件路径
    pub ignore_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: "tar.gz".to_string(),
            output_name: None,
            ignore: Some(vec![
                "target/".to_string(),
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                ".git/".to_string(),
                ".svn/".to_string(),
                ".hg/".to_string(),
                "node_modules/".to_string(),
                "__pycache__/".to_string(),
                ".pytest_cache/".to_string(),
                ".venv/".to_string(),
                "venv/".to_string(),
                "env/".to_string(),
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                "*.pyd".to_string(),
                ".idea/".to_string(),
                ".vscode/".to_string(),
                "*.iml".to_string(),
            ]),
            ignore_file: None,
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;

        let config: Config = toml::from_str(&content).with_context(|| "解析配置文件失败")?;

        // 验证压缩格式
        if !["zip", "tar.gz", "7z"].contains(&config.format.as_str()) {
            anyhow::bail!(
                "不支持的压缩格式: {}，支持的格式: zip, tar.gz, 7z",
                config.format
            );
        }

        Ok(config)
    }

    pub fn get_output_name(&self) -> String {
        if let Some(name) = &self.output_name {
            name.clone()
        } else {
            // 使用当前目录名
            match std::env::current_dir() {
                Ok(path) => path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive")
                    .to_string(),
                Err(_) => "archive".to_string(),
            }
        }
    }

    pub fn get_ignore_rules(&self) -> Vec<String> {
        let mut rules = Vec::new();

        // 如果配置中有 ignore 规则，优先使用
        if let Some(ignore_list) = &self.ignore {
            rules.extend(ignore_list.clone());
        } else if let Some(ignore_file_path) = &self.ignore_file {
            // 否则尝试从指定的忽略文件读取
            if let Ok(content) = std::fs::read_to_string(ignore_file_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        rules.push(line.to_string());
                    }
                }
            }
        }

        rules
    }
}
