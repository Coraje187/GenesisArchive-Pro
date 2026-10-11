use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub compressed_size: u64,
    pub modified: String,
    pub ratio: String,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub is_safe: bool,
    pub threat_level: String, // "safe", "warning", "critical"
    pub total_entries: usize,
    pub total_uncompressed_bytes: u64,
    pub total_compressed_bytes: u64,
    pub overall_ratio: String,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub path: String,
    pub preview_type: String, // "text", "image", "binary"
    pub text_content: Option<String>,
    pub base64_content: Option<String>,
    pub size: u64,
    pub is_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub algorithm: String,
    pub data_size_mb: f64,
    pub compress_time_ms: u128,
    pub decompress_time_ms: u128,
    pub compress_speed_mb_s: f64,
    pub decompress_speed_mb_s: f64,
    pub compressed_size_bytes: usize,
    pub ratio_percent: f64,
}

fn format_ratio(uncompressed: u64, compressed: u64) -> String {
    if uncompressed == 0 {
        return "0%".to_string();
    }
    let ratio = (compressed as f64 / uncompressed as f64) * 100.0;
    format!("{:.1}%", ratio)
}

fn format_date(dt: Option<zip::DateTime>) -> String {
    if let Some(t) = dt {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            t.year(),
            t.month(),
            t.day(),
            t.hour(),
            t.minute()
        )
    } else {
        "Unknown".to_string()
    }
}

pub fn list_archive(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let path = Path::new(archive_path);
    if !path.exists() {
        return Err(format!("Archive does not exist: {}", archive_path));
    }

    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    if file_name.ends_with(".zip") {
        list_zip(archive_path)
    } else if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        list_tar_gz(archive_path)
    } else if file_name.ends_with(".tar.zst") || file_name.ends_with(".tzst") {
        list_tar_zst(archive_path)
    } else if file_name.ends_with(".tar") {
        list_tar(archive_path)
    } else if file_name.ends_with(".7z") {
        list_7z(archive_path)
    } else {
        // Fallback: try zip first, then tar
        if let Ok(entries) = list_zip(archive_path) {
            Ok(entries)
        } else if let Ok(entries) = list_tar(archive_path) {
            Ok(entries)
        } else {
            Err("Unsupported archive format or corrupt file.".to_string())
        }
    }
}

fn list_zip(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;
    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        let entry_path = file.name().to_string();
        let name = Path::new(&entry_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&entry_path)
            .to_string();

        let is_dir = file.is_dir();
        let size = file.size();
        let compressed_size = file.compressed_size();
        let modified = format_date(file.last_modified());
        let ratio = format_ratio(size, compressed_size);
        let is_encrypted = file.encrypted();

        entries.push(ArchiveEntry {
            path: entry_path,
            name,
            is_dir,
            size,
            compressed_size,
            modified,
            ratio,
            is_encrypted,
        });
    }

    Ok(entries)
}

fn list_tar(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(file);
    let mut entries = Vec::new();

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let entry = entry_res.map_err(|e| e.to_string())?;
        let path_buf = entry.path().map_err(|e| e.to_string())?.into_owned();
        let entry_path = path_buf.to_string_lossy().replace('\\', "/");
        let name = path_buf.file_name().and_then(|s| s.to_str()).unwrap_or(&entry_path).to_string();
        let is_dir = entry.header().entry_type().is_dir();
        let size = entry.header().size().unwrap_or(0);
        let mtime = entry.header().mtime().unwrap_or(0);

        let modified = if mtime > 0 {
            chrono::DateTime::from_timestamp(mtime as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        entries.push(ArchiveEntry {
            path: entry_path,
            name,
            is_dir,
            size,
            compressed_size: size,
            modified,
            ratio: "100.0%".to_string(),
            is_encrypted: false,
        });
    }

    Ok(entries)
}

fn list_tar_gz(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let mut entries = Vec::new();

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let entry = entry_res.map_err(|e| e.to_string())?;
        let path_buf = entry.path().map_err(|e| e.to_string())?.into_owned();
        let entry_path = path_buf.to_string_lossy().replace('\\', "/");
        let name = path_buf.file_name().and_then(|s| s.to_str()).unwrap_or(&entry_path).to_string();
        let is_dir = entry.header().entry_type().is_dir();
        let size = entry.header().size().unwrap_or(0);
        let mtime = entry.header().mtime().unwrap_or(0);

        let modified = if mtime > 0 {
            chrono::DateTime::from_timestamp(mtime as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        entries.push(ArchiveEntry {
            path: entry_path,
            name,
            is_dir,
            size,
            compressed_size: 0,
            modified,
            ratio: "-".to_string(),
            is_encrypted: false,
        });
    }

    Ok(entries)
}

fn list_tar_zst(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let zst = zstd::Decoder::new(file).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(zst);
    let mut entries = Vec::new();

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let entry = entry_res.map_err(|e| e.to_string())?;
        let path_buf = entry.path().map_err(|e| e.to_string())?.into_owned();
        let entry_path = path_buf.to_string_lossy().replace('\\', "/");
        let name = path_buf.file_name().and_then(|s| s.to_str()).unwrap_or(&entry_path).to_string();
        let is_dir = entry.header().entry_type().is_dir();
        let size = entry.header().size().unwrap_or(0);
        let mtime = entry.header().mtime().unwrap_or(0);

        let modified = if mtime > 0 {
            chrono::DateTime::from_timestamp(mtime as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        entries.push(ArchiveEntry {
            path: entry_path,
            name,
            is_dir,
            size,
            compressed_size: 0,
            modified,
            ratio: "-".to_string(),
            is_encrypted: false,
        });
    }

    Ok(entries)
}

fn list_7z(archive_path: &str) -> Result<Vec<ArchiveEntry>, String> {
    let path = Path::new(archive_path);
    let reader = sevenz_rust::SevenZReader::open(path, sevenz_rust::Password::empty())
        .map_err(|e| format!("Failed to read 7z: {}", e))?;

    let mut entries = Vec::new();
    for entry in reader.archive().files.iter() {
        let entry_path = entry.name().to_string();
        let name = Path::new(&entry_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&entry_path)
            .to_string();

        let is_dir = entry.is_directory();
        let size = entry.size();
        let compressed_size = entry.compressed_size;
        let ratio = format_ratio(size, compressed_size);
        let is_encrypted = entry.has_stream() && entry.is_anti_item();

        entries.push(ArchiveEntry {
            path: entry_path,
            name,
            is_dir,
            size,
            compressed_size,
            modified: "Archive time".to_string(),
            ratio,
            is_encrypted,
        });
    }

    Ok(entries)
}

pub fn analyze_security(archive_path: &str) -> Result<SecurityReport, String> {
    let entries = list_archive(archive_path)?;
    let mut findings = Vec::new();
    let mut total_uncompressed: u64 = 0;
    let mut total_compressed: u64 = 0;
    let mut threat_level = "safe".to_string();

    let suspicious_extensions = [
        ".exe", ".scr", ".bat", ".cmd", ".vbs", ".ps1", ".jar", ".msi", ".dll", ".pif", ".cpl",
    ];

    for entry in &entries {
        total_uncompressed += entry.size;
        total_compressed += entry.compressed_size;

        // 1. Path Traversal check
        if entry.path.contains("..") || entry.path.starts_with('/') || entry.path.starts_with('\\') || entry.path.contains(':') {
            findings.push(format!(
                "⚠️ Path Traversal risk: '{}' attempts directory traversal or absolute path.",
                entry.path
            ));
            threat_level = "critical".to_string();
        }

        // 2. Suspicious executable check
        let lower = entry.name.to_lowercase();
        for ext in &suspicious_extensions {
            if lower.ends_with(ext) {
                findings.push(format!("⚠️ Executable binary detected: '{}'", entry.name));
                if threat_level != "critical" {
                    threat_level = "warning".to_string();
                }
            }
        }
    }

    // 3. Zip Bomb check
    if total_compressed > 0 {
        let ratio = total_uncompressed as f64 / total_compressed as f64;
        if ratio > 100.0 && total_uncompressed > 500 * 1024 * 1024 {
            findings.push(format!(
                "🚨 Possible Zip Bomb detected! Compression ratio is {:.1}:1 (Uncompressed: {:.2} GB vs Compressed: {:.2} MB)",
                ratio,
                total_uncompressed as f64 / (1024.0 * 1024.0 * 1024.0),
                total_compressed as f64 / (1024.0 * 1024.0)
            ));
            threat_level = "critical".to_string();
        }
    }

    if findings.is_empty() {
        findings.push("Archive structure is clean. No path traversals or dangerous patterns detected.".to_string());
    }

    let overall_ratio = format_ratio(total_uncompressed, total_compressed);

    Ok(SecurityReport {
        is_safe: threat_level == "safe",
        threat_level,
        total_entries: entries.len(),
        total_uncompressed_bytes: total_uncompressed,
        total_compressed_bytes: total_compressed,
        overall_ratio,
        findings,
    })
}

pub fn get_file_preview(archive_path: &str, file_inside: &str, max_bytes: usize) -> Result<FilePreview, String> {
    let path = Path::new(archive_path);
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    let mut buffer = Vec::new();
    let mut file_size: u64 = 0;

    if file_name.ends_with(".zip") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let mut target = archive.by_name(file_inside).map_err(|e| format!("Entry not found: {}", e))?;
        file_size = target.size();
        let mut limited = (&mut target).take(max_bytes as u64);
        limited.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    } else if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        let file = File::open(archive_path).map_err(|e| e.to_string())?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        for entry_res in archive.entries().map_err(|e| e.to_string())? {
            let mut entry = entry_res.map_err(|e| e.to_string())?;
            let p = entry.path().map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
            if p == file_inside {
                file_size = entry.header().size().unwrap_or(0);
                let mut limited = (&mut entry).take(max_bytes as u64);
                limited.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                break;
            }
        }
    } else {
        return Err("Previews currently supported for ZIP and TAR.GZ archives directly in-memory.".to_string());
    }

    if buffer.is_empty() {
        return Err("File is empty or could not be read.".to_string());
    }

    let is_truncated = (buffer.len() as u64) < file_size;
    let ext = Path::new(file_inside)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if image
    let image_exts = ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico"];
    if image_exts.contains(&ext.as_str()) {
        let mime = match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            _ => "image/png",
        };
        // Simple base64 encoding
        const BASE64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut b64 = String::new();
        for chunk in buffer.chunks(3) {
            let b0 = chunk[0];
            let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

            b64.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
            b64.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            if chunk.len() > 1 {
                b64.push(BASE64_ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
            } else {
                b64.push('=');
            }
            if chunk.len() > 2 {
                b64.push(BASE64_ALPHABET[(b2 & 0x3f) as usize] as char);
            } else {
                b64.push('=');
            }
        }

        return Ok(FilePreview {
            path: file_inside.to_string(),
            preview_type: "image".to_string(),
            text_content: None,
            base64_content: Some(format!("data:{};base64,{}", mime, b64)),
            size: file_size,
            is_truncated,
        });
    }

    // Try text utf-8
    match String::from_utf8(buffer.clone()) {
        Ok(text) => Ok(FilePreview {
            path: file_inside.to_string(),
            preview_type: "text".to_string(),
            text_content: Some(text),
            base64_content: None,
            size: file_size,
            is_truncated,
        }),
        Err(_) => {
            // Hex preview for binary
            let hex_preview: Vec<String> = buffer
                .chunks(16)
                .take(32)
                .map(|chunk| {
                    let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
                    let ascii: String = chunk
                        .iter()
                        .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
                        .collect();
                    format!("{:48}  |{}|", hex.join(" "), ascii)
                })
                .collect();

            Ok(FilePreview {
                path: file_inside.to_string(),
                preview_type: "binary".to_string(),
                text_content: Some(hex_preview.join("\n")),
                base64_content: None,
                size: file_size,
                is_truncated,
            })
        }
    }
}

pub fn extract_archive(
    archive_path: &str,
    destination: &str,
    selected_files: Option<Vec<String>>,
) -> Result<usize, String> {
    let dest_dir = Path::new(destination);
    if !dest_dir.exists() {
        std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    }

    let file_name = Path::new(archive_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if file_name.ends_with(".zip") {
        extract_zip(archive_path, dest_dir, selected_files)
    } else if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir, selected_files)
    } else if file_name.ends_with(".tar.zst") || file_name.ends_with(".tzst") {
        extract_tar_zst(archive_path, dest_dir, selected_files)
    } else if file_name.ends_with(".tar") {
        extract_tar(archive_path, dest_dir, selected_files)
    } else if file_name.ends_with(".7z") {
        extract_7z(archive_path, dest_dir)
    } else {
        Err("Unsupported archive format for extraction.".to_string())
    }
}

fn sanitize_path(base: &Path, relative: &str) -> Result<PathBuf, String> {
    let cleaned = relative.replace('\\', "/");
    let parts = cleaned.split('/').filter(|p| !p.is_empty() && *p != "." && *p != "..");
    let mut out = base.to_path_buf();
    for part in parts {
        out.push(part);
    }
    if !out.starts_with(base) {
        return Err("Directory traversal attempt prevented!".to_string());
    }
    Ok(out)
}

fn extract_zip(
    archive_path: &str,
    dest_dir: &Path,
    selected: Option<Vec<String>>,
) -> Result<usize, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut count = 0;

    let filter_set: Option<std::collections::HashSet<String>> =
        selected.map(|list| list.into_iter().collect());

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let entry_name = entry.name().to_string();

        if let Some(ref set) = filter_set {
            if !set.contains(&entry_name) {
                continue;
            }
        }

        let out_path = sanitize_path(dest_dir, &entry_name)?;

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}

fn extract_tar_gz(
    archive_path: &str,
    dest_dir: &Path,
    selected: Option<Vec<String>>,
) -> Result<usize, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let mut count = 0;

    let filter_set: Option<std::collections::HashSet<String>> =
        selected.map(|list| list.into_iter().collect());

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry_res.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");

        if let Some(ref set) = filter_set {
            if !set.contains(&path) {
                continue;
            }
        }

        let out_path = sanitize_path(dest_dir, &path)?;
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = out_path.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            entry.unpack(&out_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}

fn extract_tar_zst(
    archive_path: &str,
    dest_dir: &Path,
    selected: Option<Vec<String>>,
) -> Result<usize, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let zst = zstd::Decoder::new(file).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(zst);
    let mut count = 0;

    let filter_set: Option<std::collections::HashSet<String>> =
        selected.map(|list| list.into_iter().collect());

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry_res.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");

        if let Some(ref set) = filter_set {
            if !set.contains(&path) {
                continue;
            }
        }

        let out_path = sanitize_path(dest_dir, &path)?;
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = out_path.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            entry.unpack(&out_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}

fn extract_tar(
    archive_path: &str,
    dest_dir: &Path,
    selected: Option<Vec<String>>,
) -> Result<usize, String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(file);
    let mut count = 0;

    let filter_set: Option<std::collections::HashSet<String>> =
        selected.map(|list| list.into_iter().collect());

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry_res.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");

        if let Some(ref set) = filter_set {
            if !set.contains(&path) {
                continue;
            }
        }

        let out_path = sanitize_path(dest_dir, &path)?;
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = out_path.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            entry.unpack(&out_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}

fn extract_7z(archive_path: &str, dest_dir: &Path) -> Result<usize, String> {
    sevenz_rust::decompress_file(archive_path, dest_dir)
        .map_err(|e| format!("Failed to extract 7z archive: {}", e))?;
    Ok(1)
}

pub fn create_archive(
    output_path: &str,
    format: &str,
    source_paths: Vec<String>,
    compression_level: u32,
) -> Result<usize, String> {
    match format.to_lowercase().as_str() {
        "zip" => create_zip(output_path, source_paths, compression_level),
        "tar.gz" | "tgz" => create_tar_gz(output_path, source_paths, compression_level),
        "tar.zst" | "tzst" => create_tar_zst(output_path, source_paths, compression_level),
        "7z" => create_7z(output_path, source_paths),
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

fn create_zip(output_path: &str, source_paths: Vec<String>, _level: u32) -> Result<usize, String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut count = 0;

    for src in source_paths {
        let src_path = Path::new(&src);
        if !src_path.exists() {
            continue;
        }

        let base_dir = src_path.parent().unwrap_or(src_path);

        if src_path.is_file() {
            let rel_name = src_path.file_name().unwrap().to_string_lossy().replace('\\', "/");
            zip.start_file(rel_name, options).map_err(|e| e.to_string())?;
            let mut f = File::open(src_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
            count += 1;
        } else {
            for entry in WalkDir::new(src_path) {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                let rel = path
                    .strip_prefix(base_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");

                if path.is_file() {
                    zip.start_file(rel, options).map_err(|e| e.to_string())?;
                    let mut f = File::open(path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                    count += 1;
                } else if !rel.is_empty() {
                    zip.add_directory(rel, options).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(count)
}

fn create_tar_gz(output_path: &str, source_paths: Vec<String>, level: u32) -> Result<usize, String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::new(level.min(9)));
    let mut tar = tar::Builder::new(enc);
    let mut count = 0;

    for src in source_paths {
        let src_path = Path::new(&src);
        if !src_path.exists() {
            continue;
        }

        if src_path.is_file() {
            let rel_name = src_path.file_name().unwrap().to_string_lossy();
            let mut f = File::open(src_path).map_err(|e| e.to_string())?;
            tar.append_file(&*rel_name, &mut f).map_err(|e| e.to_string())?;
            count += 1;
        } else {
            let base_name = src_path.file_name().unwrap().to_string_lossy();
            tar.append_dir_all(&*base_name, src_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    tar.finish().map_err(|e| e.to_string())?;
    Ok(count)
}

fn create_tar_zst(output_path: &str, source_paths: Vec<String>, level: u32) -> Result<usize, String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    let enc = zstd::Encoder::new(file, level.min(22) as i32).map_err(|e| e.to_string())?.auto_finish();
    let mut tar = tar::Builder::new(enc);
    let mut count = 0;

    for src in source_paths {
        let src_path = Path::new(&src);
        if !src_path.exists() {
            continue;
        }

        if src_path.is_file() {
            let rel_name = src_path.file_name().unwrap().to_string_lossy();
            let mut f = File::open(src_path).map_err(|e| e.to_string())?;
            tar.append_file(&*rel_name, &mut f).map_err(|e| e.to_string())?;
            count += 1;
        } else {
            let base_name = src_path.file_name().unwrap().to_string_lossy();
            tar.append_dir_all(&*base_name, src_path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    tar.finish().map_err(|e| e.to_string())?;
    Ok(count)
}

fn create_7z(output_path: &str, source_paths: Vec<String>) -> Result<usize, String> {
    if let Some(first) = source_paths.first() {
        let p = Path::new(first);
        sevenz_rust::compress_to_path(p, output_path)
            .map_err(|e| format!("Failed to create 7z: {}", e))?;
        Ok(1)
    } else {
        Err("No source files specified".to_string())
    }
}

pub fn benchmark_compression(data_size_mb: f64) -> Vec<BenchmarkResult> {
    let size_bytes = (data_size_mb * 1024.0 * 1024.0) as usize;
    // Generate realistic repetitive sample data (HTML/JSON like text)
    let pattern = b"{\"user\":{\"id\":1024,\"name\":\"GenesisArchive Tester\",\"action\":\"compress\",\"speed\":\"blazing_fast\",\"metadata\":[\"winrar_killer\",\"zstandard_turbo\",\"open_source_freedom\"]},\n";
    let mut sample_data = Vec::with_capacity(size_bytes);
    while sample_data.len() < size_bytes {
        sample_data.extend_from_slice(pattern);
    }
    sample_data.truncate(size_bytes);

    let mut results = Vec::new();

    // 1. Zstandard (Level 3 - Default fast)
    {
        let start_c = std::time::Instant::now();
        let mut compressed = Vec::new();
        let mut encoder = zstd::Encoder::new(&mut compressed, 3).unwrap();
        encoder.write_all(&sample_data).unwrap();
        encoder.finish().unwrap();
        let compress_time = start_c.elapsed().as_millis();

        let start_d = std::time::Instant::now();
        let mut decompressed = Vec::new();
        let mut decoder = zstd::Decoder::new(Cursor::new(&compressed)).unwrap();
        decoder.read_to_end(&mut decompressed).unwrap();
        let decompress_time = start_d.elapsed().as_millis();

        let comp_s = (data_size_mb / (compress_time.max(1) as f64 / 1000.0)).round();
        let decomp_s = (data_size_mb / (decompress_time.max(1) as f64 / 1000.0)).round();
        let ratio = (compressed.len() as f64 / size_bytes as f64) * 100.0;

        results.push(BenchmarkResult {
            algorithm: "Zstandard (zstd - Level 3)".to_string(),
            data_size_mb,
            compress_time_ms: compress_time,
            decompress_time_ms: decompress_time,
            compress_speed_mb_s: comp_s,
            decompress_speed_mb_s: decomp_s,
            compressed_size_bytes: compressed.len(),
            ratio_percent: ratio,
        });
    }

    // 2. Standard Deflate / Gzip (Level 6 - WinRAR/Zip default)
    {
        let start_c = std::time::Instant::now();
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
        encoder.write_all(&sample_data).unwrap();
        let compressed = encoder.finish().unwrap();
        let compress_time = start_c.elapsed().as_millis();

        let start_d = std::time::Instant::now();
        let mut decoder = flate2::read::GzDecoder::new(Cursor::new(&compressed));
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        let decompress_time = start_d.elapsed().as_millis();

        let comp_s = (data_size_mb / (compress_time.max(1) as f64 / 1000.0)).round();
        let decomp_s = (data_size_mb / (decompress_time.max(1) as f64 / 1000.0)).round();
        let ratio = (compressed.len() as f64 / size_bytes as f64) * 100.0;

        results.push(BenchmarkResult {
            algorithm: "Standard Deflate/Gzip (WinRAR style - Level 6)".to_string(),
            data_size_mb,
            compress_time_ms: compress_time,
            decompress_time_ms: decompress_time,
            compress_speed_mb_s: comp_s,
            decompress_speed_mb_s: decomp_s,
            compressed_size_bytes: compressed.len(),
            ratio_percent: ratio,
        });
    }

    results
}
