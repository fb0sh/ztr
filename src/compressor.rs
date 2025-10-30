use crate::config::Config;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::path::{Path, PathBuf};

/// 根据配置压缩指定文件列表。

/// # 参数
/// - `config`: 压缩配置。
/// - `base_dir`: 基础目录，所有文件路径都将相对于此目录进行计算。
/// - `files_to_compress`: 要压缩的文件路径列表。

/// # 返回
/// `Result<PathBuf>`: 成功时返回输出文件的路径，失败时返回错误信息。
pub fn compress_directory(
    config: &Config,
    base_dir: &Path,
    files_to_compress: Vec<PathBuf>,
) -> Result<PathBuf> {
    let output_name = config.get_output_name();
    let output_path = match config.format.as_str() {
        "zip" => base_dir.join(format!("{}.zip", output_name)),
        "tar.gz" => base_dir.join(format!("{}.tar.gz", output_name)),
        "7z" => base_dir.join(format!("{}.7z", output_name)),
        _ => anyhow::bail!("不支持的压缩格式: {}", config.format),
    };

    println!("正在压缩目录: {}", base_dir.display());
    println!("输出文件: {}", output_path.display());
    println!("压缩格式: {}", config.format);

    let files = files_to_compress;

    if files.is_empty() {
        println!("没有需要压缩的文件。");
        return Ok(output_path);
    }

    println!("找到 {} 个文件要压缩", files.len());

    // 创建进度条
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("正在压缩...");

    // 根据格式选择压缩方法
    let result = match config.format.as_str() {
        "zip" => compress_zip(&files, base_dir, &output_path, &pb),
        "tar.gz" => compress_tar_gz(&files, base_dir, &output_path, &pb),
        "7z" => compress_7z(&files, base_dir, &output_path, &pb),
        _ => anyhow::bail!("不支持的压缩格式: {}", config.format),
    };

    pb.finish_with_message("压缩完成");

    match result {
        Ok(_) => {
            println!("✓ 压缩完成: {}", output_path.display());

            // 显示文件大小
            if let Ok(metadata) = std::fs::metadata(&output_path) {
                let size = metadata.len();
                if size > 1024 * 1024 {
                    println!("文件大小: {:.2} MB", size as f64 / (1024.0 * 1024.0));
                } else if size > 1024 {
                    println!("文件大小: {:.2} KB", size as f64 / 1024.0);
                } else {
                    println!("文件大小: {} bytes", size);
                }
            }
        }
        Err(e) => {
            println!("✗ 压缩失败: {}", e);
            return Err(e);
        }
    }

    Ok(output_path)
}

/// 将文件压缩为 ZIP 格式。

/// # 参数
/// - `files`: 要压缩的文件路径列表。
/// - `base_dir`: 基础目录，用于计算文件中相对路径。
/// - `output_path`: 输出 ZIP 文件的路径。
/// - `pb`: 进度条。

/// # 返回
/// `Result<()>`: 成功时返回 `Ok(())`，失败时返回错误信息。
pub fn compress_zip(
    files: &[PathBuf],
    base_dir: &Path,
    output_path: &Path,
    pb: &ProgressBar,
) -> Result<()> {
    use std::io::Write;
    use zip::{ZipWriter, write::FileOptions};

    let file = File::create(output_path).context("创建ZIP文件失败")?;
    let mut zip = ZipWriter::new(file);

    for file_path in files {
        pb.inc(1);

        let relative_path = file_path
            .strip_prefix(base_dir)
            .with_context(|| format!("计算相对路径失败: {}", file_path.display()))?;

        let mut file = File::open(file_path)
            .with_context(|| format!("打开文件失败: {}", file_path.display()))?;

        let relative_path_str = relative_path.to_string_lossy().replace("\\", "/");
        zip.start_file(&relative_path_str, FileOptions::default())
            .with_context(|| format!("添加文件到ZIP失败: {}", file_path.display()))?;

        let mut buffer = Vec::new();
        std::io::copy(&mut file, &mut buffer)
            .with_context(|| format!("读取文件失败: {}", file_path.display()))?;

        zip.write_all(&buffer)
            .with_context(|| format!("写入ZIP失败: {}", file_path.display()))?;
    }

    zip.finish().context("完成ZIP写入失败")?;

    Ok(())
}

/// 将文件压缩为 TAR.GZ 格式。

/// # 参数
/// - `files`: 要压缩的文件路径列表。
/// - `base_dir`: 基础目录，用于计算文件中相对路径。
/// - `output_path`: 输出 TAR.GZ 文件的路径。
/// - `pb`: 进度条。

/// # 返回
/// `Result<()>`: 成功时返回 `Ok(())`，失败时返回错误信息。
pub fn compress_tar_gz(
    files: &[PathBuf],
    base_dir: &Path,
    output_path: &Path,
    pb: &ProgressBar,
) -> Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::Builder;

    let file = File::create(output_path).context("创建TAR.GZ文件失败")?;
    let gz_encoder = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(gz_encoder);

    for file_path in files {
        pb.inc(1);

        let relative_path = file_path
            .strip_prefix(base_dir)
            .with_context(|| format!("计算相对路径失败: {}", file_path.display()))?;

        tar.append_path_with_name(file_path, relative_path)
            .with_context(|| format!("添加文件到TAR失败: {}", file_path.display()))?;
    }

    tar.finish().context("完成TAR.GZ写入失败")?;

    Ok(())
}

/// 将文件压缩为 7Z 格式。

/// # 参数
/// - `files`: 要压缩的文件路径列表。
/// - `base_dir`: 基础目录，用于计算文件中相对路径。
/// - `output_path`: 输出 7Z 文件的路径。
/// - `pb`: 进度条。

/// # 返回
/// `Result<()>`: 成功时返回 `Ok(())`，失败时返回错误信息。
pub fn compress_7z(
    files: &[PathBuf],
    base_dir: &Path,
    output_path: &Path,
    pb: &ProgressBar,
) -> Result<()> {
    use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
    use std::io::Read;

    let mut sz_writer = SevenZWriter::create(output_path).context("创建7Z文件失败")?;

    for file_path in files {
        pb.inc(1);

        let relative_path = file_path
            .strip_prefix(base_dir)
            .with_context(|| format!("计算相对路径失败: {}", file_path.display()))?;

        if file_path.is_file() {
            let mut file = File::open(file_path)
                .with_context(|| format!("打开文件失败: {}", file_path.display()))?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)
                .with_context(|| format!("读取文件内容失败: {}", file_path.display()))?;

            let mut entry = SevenZArchiveEntry::default();
            entry.name = relative_path.to_string_lossy().replace("\\", "/");
            entry.size = content.len() as u64;

            sz_writer
                .push_archive_entry(entry, Some(content.as_slice()))
                .with_context(|| format!("添加文件到7Z失败: {}", file_path.display()))?;
        }
    }

    sz_writer.finish().context("完成7Z写入失败")?;

    Ok(())
}
