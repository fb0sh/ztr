# ZTR - 智能压缩工具

一个基于配置文件的智能压缩工具，支持多种压缩格式和灵活的文件忽略规则。它既可以作为命令行工具使用，也可以作为 Rust 库集成到其他项目中。

## 🚀 功能特性

- **多格式支持**: 支持 ZIP、TAR.GZ、7Z 三种主流压缩格式
- **智能配置**: 通过配置文件自定义压缩选项和忽略规则
- **Gitignore风格**: 使用类似 .gitignore 的语法来忽略不需要的文件
- **进度显示**: 实时显示压缩进度和文件大小信息
- **简洁易用**: 默认情况下自动使用配置文件进行压缩
- **可复用库**: 核心功能已封装为 Rust 库 `ztr_lib`，方便二次开发和集成

## 📦 安装

### 作为命令行工具

#### 从源码编译

```bash
git clone <repository-url>
cd ztr
cargo build --release
```

编译后的可执行文件位于 `target/release/ztr.exe` (Windows) 或 `target/release/ztr` (Linux/macOS)。

### 作为 Rust 库

将以下内容添加到你的 `Cargo.toml` 中：

```toml
[dependencies]
ztr_lib = "2.0.0" # 替换为实际版本号
```

## 🛠️ 使用方法

### 作为命令行工具

#### 基本用法

```bash
# 直接压缩当前目录（如果存在 ztr.toml 配置文件）
ztr

# 创建默认配置文件
ztr init

# 查看支持的压缩格式
ztr show

# 使用指定配置文件压缩
ztr compress --config my-config.toml

# 压缩指定目录
ztr compress --path /path/to/my/folder

# 使用指定配置文件压缩指定目录
ztr compress --config my-config.toml --path /path/to/my/folder
```

#### 命令说明

- `ztr` (默认行为)
  检查当前目录是否存在 `ztr.toml` 配置文件，如果存在则直接进行压缩。

- `ztr init`
  在当前目录创建默认的 `ztr.toml` 配置文件。

- `ztr show`
  显示所有支持的压缩格式。

- `ztr compress`
  压缩当前目录。可通过 `--config` 参数指定配置文件路径，通过 `--path` 参数指定要压缩的目录。如果未指定 `--path`，则默认为当前目录。

### 作为 Rust 库

以下是一个简单的示例，展示了如何在 Rust 项目中使用 `ztr_lib` 进行压缩：

```rust
use ztr_lib::config::Config;
use ztr_lib::compressor;
use ztr_lib::ignore_rules::IgnoreRules;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    // 假设 ztr.toml 配置文件存在于当前目录
    let config_path = PathBuf::from("ztr.toml");
    let config = Config::load(&config_path)?;

    // 指定要压缩的目录，例如当前目录
    let base_dir = std::env::current_dir()?;

    // 收集所有文件路径
    let mut all_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&base_dir).into_iter().filter_map(|e| e.ok()) {
        all_files.push(entry.path().to_path_buf());
    }

    // 应用忽略规则
    let ignore_rules = IgnoreRules::new(
        &config.get_ignore_rules(),
        &base_dir,
    )?;
    let files_to_compress = ignore_rules.filter_files(all_files.into_iter())?;

    // 执行压缩操作
    let output_archive_path = compressor::compress_directory(
        &config,
        &base_dir,
        files_to_compress,
    )?;
    println!("压缩文件已创建: {}", output_archive_path.display());

    Ok(())
}
```

更详细的库使用文档可以通过 `cargo doc --open` 命令生成并查看。

## ⚙️ 配置文件

`ztr.toml` 配置文件示例：

```toml
# ZTR 压缩工具配置文件

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
```

### 配置选项说明

| 选项 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `format` | 字符串 | 是 | 压缩格式，支持 "zip"、"tar.gz"、"7z" |
| `output_name` | 字符串 | 否 | 输出文件名，默认使用当前目录名 |
| `ignore` | 数组 | 否 | 忽略规则列表，优先级高于 `ignore_file` |
| `ignore_file` | 字符串 | 否 | 指定忽略文件路径，如 `.gitignore` |

### 忽略规则优先级

1. 如果配置了 `ignore` 数组，则优先使用数组中的规则
2. 如果没有配置 `ignore` 但配置了 `ignore_file`，则从指定文件读取规则
3. 两者都没有配置则不忽略任何文件

## 📝 忽略规则语法

忽略规则支持以下语法模式：

| 模式 | 说明 | 示例 |
|------|------|------|
| `*` | 匹配任意字符 | `*.tmp` 匹配所有 .tmp 文件 |
| `/` | 目录分隔符 | `target/` 匹配 target 目录及其内容 |
| `?` | 单个字符 | `???.txt` 匹配三个字符的 .txt 文件 |
| `[]` | 字符范围 | `[abc]*.txt` 匹配以 a、b 或 c 开头的 .txt 文件 |
| `!` | 否定规则 | `!important.log` 不忽略 important.log 文件 |
| `#` | 注释 | `# 这是注释` |

## 🎯 支持的压缩格式

| 格式 | 特点 | 适用场景 |
|------|------|----------|
| **zip** | 兼容性最好，几乎所有系统都支持 | 跨平台文件传输 |
| **tar.gz** | Linux 常用格式，压缩率适中 | Linux/Unix 环境部署 |
| **7z** | 压缩率最高，支持多种算法 | 需要最大压缩率的场景 |

## 💡 使用示例

### 示例 1: 压缩 Rust 项目

```bash
# 在 Rust 项目根目录
ztr init
# 编辑 ztr.toml，确保包含 target/ 忽略规则
ztr
```

### 示例 2: 压缩 Node.js 项目

```toml
# ztr.toml
format = "zip"
output_name = "my-node-app"
ignore = [
    "node_modules/",
    "*.log",
    ".env",
    "dist/",
    "coverage/",
]
```

### 示例 3: 使用现有的 .gitignore

```toml
# ztr.toml
format = "tar.gz"
ignore_file = "./.gitignore"
```

## 🔧 开发

### 项目结构

```
ztr/
├── src/
│   ├── main.rs          # 主程序入口
│   ├── lib.rs           # 库入口
│   ├── config.rs        # 配置文件解析
│   ├── compressor.rs    # 压缩功能实现
│   └── ignore_rules.rs  # 忽略规则处理
├── Cargo.toml           # 项目依赖配置
├── ztr.toml           # 默认配置文件示例
└── README.md           # 项目说明文档
```

### 依赖库

- `clap`: 命令行参数解析
- `serde`: 序列化/反序列化
- `toml`: TOML 配置文件解析
- `zip`: ZIP 格式支持
- `tar`: TAR 格式支持
- `flate2`: GZIP 压缩支持
- `sevenz-rust`: 7Z 格式支持
- `ignore`: Gitignore 风格的文件过滤
- `indicatif`: 进度条显示
- `anyhow`: 错误处理

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📞 反馈

如果您在使用过程中遇到问题或有改进建议，请：

- 提交 [Issue](../../issues)
- 发送邮件至：[your-email@example.com]

---

**ZTR** - 让压缩变得简单而智能！ 🎉