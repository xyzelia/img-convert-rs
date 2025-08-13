use std::fs;
use std::path::{Path, PathBuf};
use std::io;

mod path_filter {}

/// 图片文件扩展名
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif",
    "webp", "svg", "ico", "raw", "heic", "heif", "avif"
];

/// 图片文件信息结构
#[derive(Debug, Clone)]
pub struct ImageFileInfo {
    /// 相对路径
    pub relative_path: PathBuf,
    /// 文件名
    pub filename: String,
    /// 完整路径
    pub full_path: PathBuf,
}

/// 检查文件是否为图片文件
pub fn is_image_file<P: AsRef<Path>>(file_path: P) -> bool {
    let path = file_path.as_ref();

    if !path.is_file() {
        return false;
    }

    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return IMAGE_EXTENSIONS.iter().any(|&ext| ext == ext_lower);
        }
    }

    false
}

/// 获取文件夹下所有图片文件的信息，结果放在 Vec 中
///
/// # 参数
/// * `folder_path` - 文件夹路径
///
/// # 返回值
/// * `Result<Vec<ImageFileInfo>, io::Error>` - 成功返回图片文件信息列表，失败返回 IO 错误
pub fn get_image_files<P: AsRef<Path>>(folder_path: P) -> io::Result<Vec<ImageFileInfo>> {
    let folder_path = folder_path.as_ref();

    if !folder_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("文件夹不存在: {}", folder_path.display()),
        ));
    }

    if !folder_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("路径不是文件夹: {}", folder_path.display()),
        ));
    }

    let mut image_files = Vec::new();
    collect_image_files_recursive(folder_path, folder_path, &mut image_files)?;

    Ok(image_files)
}

/// 递归收集图片文件信息
fn collect_image_files_recursive(
    base_path: &Path,
    current_path: &Path,
    image_files: &mut Vec<ImageFileInfo>,
) -> io::Result<()> {
    let entries = fs::read_dir(current_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_image_files_recursive(base_path, &path, image_files)?;
        } else if is_image_file(&path) {
            let relative_path = path.strip_prefix(base_path)
                .unwrap_or(&path)
                .to_path_buf();

            let filename = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let image_info = ImageFileInfo {
                relative_path: relative_path.clone(),
                filename,
                full_path: path,
            };

            image_files.push(image_info);
        }
    }

    Ok(())
}
