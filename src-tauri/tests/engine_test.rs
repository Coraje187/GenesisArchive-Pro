#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tauri_app_lib::archive;

    #[test]
    fn test_benchmark_speed() {
        let results = archive::benchmark_compression(5.0);
        assert_eq!(results.len(), 2);
        for res in &results {
            println!(
                "Algorithm: {} | Compress: {} MB/s | Decompress: {} MB/s | Ratio: {:.1}%",
                res.algorithm, res.compress_speed_mb_s, res.decompress_speed_mb_s, res.ratio_percent
            );
        }
    }

    #[test]
    fn test_create_and_read_zip() {
        let temp_dir = std::env::temp_dir().join("nova_test_zip");
        let _ = std::fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("hello.txt");
        let mut f = File::create(&test_file).unwrap();
        writeln!(f, "Hello NovaArchive! Better than WinRAR.").unwrap();

        let zip_out = temp_dir.join("test.zip");
        let count = archive::create_archive(
            zip_out.to_str().unwrap(),
            "zip",
            vec![test_file.to_str().unwrap().to_string()],
            6,
        ).unwrap();
        assert_eq!(count, 1);

        // List
        let entries = archive::list_archive(zip_out.to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "hello.txt");

        // In-memory preview
        let preview = archive::get_file_preview(
            zip_out.to_str().unwrap(),
            "hello.txt",
            1024,
        ).unwrap();
        assert_eq!(preview.preview_type, "text");
        assert!(preview.text_content.unwrap().contains("Better than WinRAR"));

        // Security scan
        let sec = archive::analyze_security(zip_out.to_str().unwrap()).unwrap();
        assert!(sec.is_safe);
        assert_eq!(sec.threat_level, "safe");

        // Extract
        let extract_dir = temp_dir.join("extracted");
        let extracted_count = archive::extract_archive(
            zip_out.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            None,
        ).unwrap();
        assert_eq!(extracted_count, 1);

        let read_back = std::fs::read_to_string(extract_dir.join("hello.txt")).unwrap();
        assert!(read_back.contains("Better than WinRAR"));
    }

    #[test]
    fn test_create_and_read_tar_zst() {
        let temp_dir = std::env::temp_dir().join("nova_test_zst");
        let _ = std::fs::create_dir_all(&temp_dir);

        let test_file = temp_dir.join("speed.json");
        let mut f = File::create(&test_file).unwrap();
        writeln!(f, "{{\"engine\": \"zstandard\", \"speed\": \"3gb/s\"}}").unwrap();

        let zst_out = temp_dir.join("speed.tar.zst");
        let count = archive::create_archive(
            zst_out.to_str().unwrap(),
            "tar.zst",
            vec![test_file.to_str().unwrap().to_string()],
            3,
        ).unwrap();
        assert_eq!(count, 1);

        // List
        let entries = archive::list_archive(zst_out.to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "speed.json");

        // Extract
        let extract_dir = temp_dir.join("extracted_zst");
        let extracted_count = archive::extract_archive(
            zst_out.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            None,
        ).unwrap();
        assert_eq!(extracted_count, 1);
    }

    #[test]
    fn test_read_user_rar() {
        let user_rar = r"C:\Users\Coraj\Downloads\Project Ultimate Reshade - ETS2_ATS.rar";
        if std::path::Path::new(user_rar).exists() {
            let entries = archive::list_archive(user_rar).expect("Failed to list user RAR archive");
            println!("User RAR entries count: {}", entries.len());
            assert!(!entries.is_empty(), "Archive should contain entries");
            for e in entries.iter().take(5) {
                println!(" - Entry: {} (Size: {} bytes, Dir: {})", e.name, e.size, e.is_dir);
            }
        }
    }
}
