use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// 表示 ZTR 压缩工具的配置。
/// 包含压缩格式、输出文件名、忽略规则和忽略文件路径。
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
    /// 已经解析的忽略文件内容 (在加载配置时读取并存储)
    #[serde(skip)]
    pub resolved_ignore_file_content: Option<String>,
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
            resolved_ignore_file_content: None, // 默认初始化为 None
        }
    }
}

impl Config {
    /// 从指定路径加载配置文件并解析为 Config 结构体。
    ///
    /// 如果配置中指定了 `ignore_file`，则会尝试读取其内容并存储在 `resolved_ignore_file_content` 字段中。
    ///
    /// # 参数
    /// - `path`: 配置文件的路径。
    ///
    /// # 返回
    /// `Result<Self>`: 成功时返回解析后的 Config 结构体，失败时返回错误信息。
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;

        let mut config: Config = toml::from_str(&content).with_context(|| "解析配置文件失败")?;

        // 验证压缩格式
        if !["zip", "tar.gz", "7z"].contains(&config.format.as_str()) {
            anyhow::bail!(
                "不支持的压缩格式: {}，支持的格式: zip, tar.gz, 7z",
                config.format
            );
        }

        // 如果指定了忽略文件路径，则读取其内容
        if let Some(ignore_file_path) = &config.ignore_file {
            if let Ok(file_content) = std::fs::read_to_string(ignore_file_path) {
                config.resolved_ignore_file_content = Some(file_content);
            }
        }

        Ok(config)
    }

    /// 创建一个默认的 `ztr.toml` 配置文件。
    ///
    /// # 参数
    /// - `output_path`: 配置文件的输出路径。如果为 `None`，则默认为当前目录下的 `ztr.toml`。
    ///
    /// # 返回
    /// `Result<()>`: 成功时返回 `Ok(())`，失败时返回错误信息。
    pub fn create_default_config_file(output_path: Option<&Path>) -> Result<()> {
        let config = Config::default();
        let toml_content = toml::to_string_pretty(&config).context("无法序列化默认配置")?;

        let path = output_path.unwrap_or(&Path::new("ztr.toml"));
        std::fs::write(path, toml_content)
            .with_context(|| format!("无法写入配置文件: {}", path.display()))?;

        Ok(())
    }

    /// 获取压缩包的输出名称。
    /// 如果配置中指定了输出名称，则使用该名称；否则，使用当前目录名作为输出名称。
    ///
    /// # 返回
    /// `String`: 压缩包的输出名称。
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

    /// 获取忽略规则列表，优先使用 `ignore` 字段，其次是 `resolved_ignore_file_content`。
    pub fn get_ignore_rules(&self) -> Vec<String> {
        let mut all_rules: HashSet<String> = HashSet::new();

        if let Some(ignore_list) = &self.ignore {
            for rule in ignore_list {
                all_rules.insert(rule.clone());
            }
        }

        if let Some(content) = &self.resolved_ignore_file_content {
            for line in content.lines() {
                let trimmed_line = line.trim();
                if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                    all_rules.insert(trimmed_line.to_string());
                }
            }
        }

        all_rules.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load() -> Result<()> {
        let toml_content = r#"
            format = "zip"
            output_name = "test_archive"
            ignore = [".test_ignore"]
            ignore_file = "./test_ignore_file.txt"
        "#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", toml_content)?;
        let config = Config::load(file.path())?;

        assert_eq!(config.format, "zip");
        assert_eq!(config.output_name, Some("test_archive".to_string()));
        assert_eq!(config.ignore, Some(vec![".test_ignore".to_string()]));
        assert_eq!(
            config.ignore_file,
            Some("./test_ignore_file.txt".to_string())
        );
        assert_eq!(config.resolved_ignore_file_content, None); // ignore_file.txt 不存在，所以内容应为 None
        Ok(())
    }

    #[test]
    fn test_config_load_with_ignore_file_content() -> Result<()> {
        let mut ignore_file = NamedTempFile::new()?;
        writeln!(ignore_file, "file_from_ignore.txt")?;
        let ignore_file_path = ignore_file.path().to_string_lossy().to_string();

        let toml_content = format!(
            r#"
            format = "zip"
            ignore_file = "{}"
        "#,
            ignore_file_path
        );
        let mut config_file = NamedTempFile::new()?;
        write!(config_file, "{}", toml_content)?;
        let config = Config::load(config_file.path())?;

        assert_eq!(config.format, "zip");
        assert_eq!(config.ignore_file, Some(ignore_file_path.clone()));
        assert_eq!(
            config.resolved_ignore_file_content,
            Some("file_from_ignore.txt\n".to_string())
        );

        let rules = config.get_ignore_rules();
        assert!(rules.contains(&"file_from_ignore.txt".to_string()));
        assert_eq!(rules.len(), 1);
        Ok(())
    }

    #[test]
    fn test_config_load_invalid_format() -> Result<()> {
        let toml_content = r#"
            format = "rar"
        "#;
        let mut file = NamedTempFile::new()?;
        write!(file, "{}", toml_content)?;
        let err = Config::load(file.path()).unwrap_err();
        assert!(err.to_string().contains("不支持的压缩格式"));
        Ok(())
    }

    #[test]
    fn test_get_output_name_from_config() {
        let config = Config {
            format: "zip".to_string(),
            output_name: Some("my_custom_name".to_string()),
            ignore: None,
            ignore_file: None,
            resolved_ignore_file_content: None,
        };
        assert_eq!(config.get_output_name(), "my_custom_name");
    }

    #[test]
    fn test_get_output_name_default() {
        let config = Config::default();
        // 假设当前目录名不是 "archive"，这里需要一个更健壮的测试，可能需要模拟当前目录
        // 为了测试目的，我们只检查它不是 None 并且不是空字符串
        let output_name = config.get_output_name();
        assert!(!output_name.is_empty());
        assert_ne!(output_name, "archive"); // 除非当前目录是根目录，否则不会是 "archive"
    }

    #[test]
    fn test_get_ignore_rules_from_config() {
        let config = Config {
            format: "zip".to_string(),
            output_name: None,
            ignore: Some(vec!["rule1".to_string(), "rule2".to_string()]),
            ignore_file: None,
            resolved_ignore_file_content: None,
        };
        let rules = config.get_ignore_rules();
        assert!(rules.contains(&"rule1".to_string()));
        assert!(rules.contains(&"rule2".to_string()));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_get_ignore_rules_from_resolved_file_content() {
        let mut config_with_file_content = Config::default();
        config_with_file_content.resolved_ignore_file_content =
            Some("# 注释\nrule_from_file1\n\nrule_from_file2".to_string());
        let rules = config_with_file_content.get_ignore_rules();
        assert!(rules.contains(&"rule_from_file1".to_string()));
        assert!(rules.contains(&"rule_from_file2".to_string()));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_get_ignore_rules_priority() {
        let mut config = Config {
            format: "zip".to_string(),
            output_name: None,
            ignore: Some(vec![
                "rule_from_config".to_string(),
                "common_rule".to_string(),
            ]),
            ignore_file: None,
            resolved_ignore_file_content: None,
        };
        config.resolved_ignore_file_content = Some("rule_from_file\ncommon_rule".to_string());
        let rules = config.get_ignore_rules();
        assert!(rules.contains(&"rule_from_config".to_string()));
        assert!(rules.contains(&"rule_from_file".to_string()));
        assert!(rules.contains(&"common_rule".to_string()));
        assert_eq!(rules.len(), 3); // "common_rule" 不会重复
    }
}
