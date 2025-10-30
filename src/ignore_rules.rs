use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};

/// 管理文件和目录的忽略规则。
/// 使用 Gitignore 语法来匹配路径。
pub struct IgnoreRules {
    gitignore: Gitignore,
    base_dir: PathBuf,
}

impl IgnoreRules {
    /// 创建新的 `IgnoreRules` 实例。
    ///
    /// # 参数
    /// - `rules`: 忽略规则的字符串切片。
    /// - `base_dir`: 基础目录，所有路径都将相对于此目录进行匹配。
    ///
    /// # 返回
    /// `Result<Self>`: 成功时返回 `IgnoreRules` 实例，失败时返回错误信息。
    pub fn new(rules: &[String], base_dir: &Path) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(base_dir);

        for rule in rules {
            builder
                .add_line(None, rule)
                .with_context(|| format!("无效的忽略规则: {}", rule))?;
        }

        let gitignore = builder.build().with_context(|| "构建忽略规则失败")?;

        Ok(Self {
            gitignore,
            base_dir: base_dir.to_path_buf(),
        })
    }

    /// 检查给定的路径是否应该被忽略。
    ///
    /// # 参数
    /// - `path`: 要检查的路径。
    /// - `is_dir`: 指示路径是否是目录。
    ///
    /// # 返回
    /// `bool`: 如果路径应该被忽略，则返回 `true`，否则返回 `false`。
    pub fn should_ignore(&self, path: &Path, is_dir: bool) -> bool {
        let relative_path = match path.strip_prefix(&self.base_dir) {
            Ok(p) => p,
            Err(_) => return false, // 如果无法获取相对路径，则不忽略
        };

        self.gitignore
            .matched_path_or_any_parents(relative_path, is_dir)
            .is_ignore()
    }

    /// 过滤给定的文件路径列表，移除所有应该被忽略的文件。
    ///
    /// # 参数
    /// - `files_to_filter`: 一个包含文件路径的迭代器。
    ///
    /// # 返回
    /// `Result<Vec<PathBuf>>`: 包含所有不被忽略的文件路径的向量。
    pub fn filter_files<I>(&self, files_to_filter: I) -> Result<Vec<PathBuf>>
    where
        I: Iterator<Item = PathBuf>,
    {
        let mut files = Vec::new();
        for path in files_to_filter {
            if !self.should_ignore(&path, path.is_dir()) {
                files.push(path);
            }
        }
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_ignore_rules_creation() -> Result<()> {
        let dir = tempdir()?;
        let base_dir = dir.path();
        let rules = vec!["*.txt".to_string(), "temp_dir/".to_string()];
        let ignore_rules = IgnoreRules::new(&rules, base_dir)?;

        assert!(!ignore_rules.should_ignore(&base_dir.join("file.rs"), false));
        assert!(ignore_rules.should_ignore(&base_dir.join("file.txt"), false));
        assert!(ignore_rules.should_ignore(&base_dir.join("temp_dir"), true));
        assert!(!ignore_rules.should_ignore(&base_dir.join("another_dir"), true));

        Ok(())
    }

    #[test]
    fn test_filter_files() -> Result<()> {
        let dir = tempdir()?;
        let base_dir = dir.path();

        // Create some files and directories
        fs::write(base_dir.join("file1.txt"), "content")?;
        fs::write(base_dir.join("file2.rs"), "content")?;
        fs::create_dir(base_dir.join("ignore_me"))?;
        fs::write(base_dir.join("ignore_me/file3.txt"), "content")?;
        fs::write(base_dir.join("ignore_me/file4.rs"), "content")?;

        let rules = vec!["*.txt".to_string(), "ignore_me/".to_string()];
        let ignore_rules = IgnoreRules::new(&rules, base_dir)?;

        let all_files = vec![
            base_dir.join("file1.txt"),
            base_dir.join("file2.rs"),
            base_dir.join("ignore_me"),
            base_dir.join("ignore_me/file3.txt"),
            base_dir.join("ignore_me/file4.rs"),
        ];

        let files = ignore_rules.filter_files(all_files.into_iter())?;

        assert_eq!(files.len(), 1);
        assert!(files.contains(&base_dir.join("file2.rs")));
        assert!(!files.contains(&base_dir.join("file1.txt")));
        assert!(!files.contains(&base_dir.join("ignore_me/file3.txt")));
        assert!(!files.contains(&base_dir.join("ignore_me/file4.rs")));

        Ok(())
    }

    #[test]
    fn test_should_ignore_nested_dir() -> Result<()> {
        let dir = tempdir()?;
        let base_dir = dir.path();

        fs::create_dir_all(base_dir.join("src/components"))?;
        fs::write(base_dir.join("src/components/foo.txt"), "content")?;
        fs::write(base_dir.join("src/components/bar.js"), "content")?;

        let rules = vec!["src/components/".to_string()];
        let ignore_rules = IgnoreRules::new(&rules, base_dir)?;

        assert!(ignore_rules.should_ignore(&base_dir.join("src/components"), true));
        assert!(ignore_rules.should_ignore(&base_dir.join("src/components/foo.txt"), false));
        assert!(ignore_rules.should_ignore(&base_dir.join("src/components/bar.js"), false));

        Ok(())
    }
}
