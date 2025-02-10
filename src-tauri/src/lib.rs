#![feature(os_str_display)]
use dashmap::DashMap;
use keccak_asm::{Digest, Sha3_256};
use rayon::prelude::*;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn index_folder(path: &str) -> DashMap<String, Vec<String>> {
    let path = PathBuf::from(path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path));
    let mut file_entries: DashMap<String, Vec<String>> = DashMap::new();
    println!("Indexing folder: {:?}", path);

    let walker = WalkDir::new(path);
    walker
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .for_each(|entry| {
            let path = entry.into_path();
            let hash = hash_file(&path).unwrap();
            file_entries.entry(hash).or_default().push(
                path.to_string_lossy()
                    .trim_start_matches(r"\\?\")
                    .to_string(),
            );
        });

    file_entries = file_entries
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();

    file_entries
}

fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(&path)?;
    let mut hasher = Sha3_256::new();
    let mut buffer = [0; 512];

    loop {
        let n = file.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, index_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
