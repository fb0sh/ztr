use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};

pub struct IgnoreRules {
    gitignore: Gitignore,
    base_dir: PathBuf,
}

impl IgnoreRules {
    pub fn new(rules: &[String], base_dir: &Path) -> Result<Self> {
        let mut builder = GitignoreBuilder::new(base_dir);

        for rule in rules {
            // 添加规则到构建器
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

    pub fn should_ignore(&self, path: &Path, is_dir: bool) -> bool {
        let relative_path = match path.strip_prefix(&self.base_dir) {
            Ok(p) => p,
            Err(_) => return false, // 如果无法获取相对路径，则不忽略
        };

        // 使用 matched_path_or_any_parents 来检查路径或任何父目录是否匹配
        self.gitignore
            .matched_path_or_any_parents(relative_path, is_dir)
            .is_ignore()
    }

    pub fn get_files_to_compress<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let dir = dir.as_ref();

        self.collect_files(dir, &mut files)?;

        Ok(files)
    }

    fn collect_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries =
            std::fs::read_dir(dir).with_context(|| format!("无法读取目录: {}", dir.display()))?;

        for entry in entries {
            let entry = entry.with_context(|| "读取目录条目失败")?;
            let path = entry.path();

            // 检查是否应该忽略
            if self.should_ignore(&path, path.is_dir()) {
                continue;
            }

            if path.is_dir() {
                // 递归处理子目录
                self.collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }

        Ok(())
    }
}
