pub mod archive;

pub use archive::{ArchiveEntry, BenchmarkResult, FilePreview, SecurityReport};

#[tauri::command]
fn list_archive(archive_path: String) -> Result<Vec<ArchiveEntry>, String> {
    archive::list_archive(&archive_path)
}

#[tauri::command]
fn get_security_report(archive_path: String) -> Result<SecurityReport, String> {
    archive::analyze_security(&archive_path)
}

#[tauri::command]
fn get_file_preview(archive_path: String, file_inside: String, max_bytes: Option<usize>) -> Result<FilePreview, String> {
    let limit = max_bytes.unwrap_or(256 * 1024);
    archive::get_file_preview(&archive_path, &file_inside, limit)
}

#[tauri::command]
fn extract_archive(
    archive_path: String,
    destination: String,
    selected_files: Option<Vec<String>>,
) -> Result<usize, String> {
    archive::extract_archive(&archive_path, &destination, selected_files)
}

#[tauri::command]
fn create_archive(
    output_path: String,
    format: String,
    source_paths: Vec<String>,
    compression_level: Option<u32>,
) -> Result<usize, String> {
    archive::create_archive(&output_path, &format, source_paths, compression_level.unwrap_or(3))
}

#[tauri::command]
fn pick_archive_file() -> Option<String> {
    let file = rfd::FileDialog::new()
        .add_filter(
            "All Supported Archives (*.rar, *.zip, *.7z, *.tar...)",
            &[
                "rar", "zip", "7z", "tar", "gz", "tgz", "zst", "tzst",
                "RAR", "ZIP", "7Z", "TAR", "GZ", "TGZ", "ZST", "TZST",
            ],
        )
        .add_filter("WinRAR Archives (*.rar)", &["rar", "RAR"])
        .add_filter("ZIP Archives (*.zip)", &["zip", "ZIP"])
        .add_filter("7-Zip Archives (*.7z)", &["7z", "7Z"])
        .add_filter("Tar Archives (*.tar, *.tar.gz, *.tar.zst)", &["tar", "gz", "tgz", "zst", "tzst"])
        .add_filter("All Files (*.*)", &["*"])
        .pick_file();

    file.map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn pick_source_files() -> Vec<String> {
    let files = rfd::FileDialog::new().pick_files();
    match files {
        Some(paths) => paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect(),
        None => Vec::new(),
    }
}

#[tauri::command]
fn pick_source_folder() -> Option<String> {
    rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn pick_destination_folder() -> Option<String> {
    rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn pick_save_archive(default_name: String, format: String) -> Option<String> {
    let ext = match format.to_lowercase().as_str() {
        "zip" => "zip",
        "tar.gz" => "tar.gz",
        "tar.zst" => "tar.zst",
        "7z" => "7z",
        _ => "zip",
    };

    let mut dialog = rfd::FileDialog::new()
        .set_file_name(&format!("{}.{}", default_name, ext));

    if ext == "zip" {
        dialog = dialog.add_filter("ZIP Archive (*.zip)", &["zip"]);
    } else if ext == "tar.gz" {
        dialog = dialog.add_filter("Gzipped Tar (*.tar.gz)", &["tar.gz", "tgz"]);
    } else if ext == "tar.zst" {
        dialog = dialog.add_filter("Zstandard Tar (*.tar.zst)", &["tar.zst", "tzst"]);
    } else if ext == "7z" {
        dialog = dialog.add_filter("7-Zip Archive (*.7z)", &["7z"]);
    }

    dialog.save_file().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn run_benchmark(data_size_mb: Option<f64>) -> Vec<BenchmarkResult> {
    archive::benchmark_compression(data_size_mb.unwrap_or(20.0))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_archive,
            get_security_report,
            get_file_preview,
            extract_archive,
            create_archive,
            pick_archive_file,
            pick_source_files,
            pick_source_folder,
            pick_destination_folder,
            pick_save_archive,
            run_benchmark,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
