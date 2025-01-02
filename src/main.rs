#![feature(prelude_2024)]
use blake3;
use clap::Parser;
use sled;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::prelude::rust_2024::Future;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::{debug, info, instrument, trace};
use walkdir::WalkDir;

static APP_NAME: &str = "dedupe";

#[derive(Debug)]
struct FileEntry {
    path: String,
    hash: String,
    mtime: u64,
}

impl FileEntry {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let s = String::from_utf8(bytes.to_vec()).ok()?;
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(FileEntry {
            path: parts[0].to_string(),
            hash: parts[1].to_string(),
            mtime: parts[2].parse().ok()?,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        format!("{}|{}|{}", self.path, self.hash, self.mtime).into_bytes()
    }
}

#[derive(Debug)]
struct FileTask {
    path: PathBuf,
    mtime: u64,
    counter: u64,
}

#[derive(Parser, Debug)]
#[command(
    name = APP_NAME,
    author,
    version,
    about = "Find and manage duplicate files using blake3 hashes",
    long_about = "A command line tool to find and manage duplicate files by calculating blake3 hashes.
It can cache results between runs and selectively move duplicates to a separate folder.
")]
struct Args {
    /// Folder to scan
    #[arg(short, long)]
    path: String,

    /// Cache file for storing index
    #[arg(short, long, default_value = ".dedupe.cache")]
    cache: Option<String>,

    /// Folder to move duplicates to
    #[arg(short, long, default_value = "duplicates")]
    duplicates_folder: String,

    /// Force reindexing even if cache exists
    #[arg(short, long, default_value_t = true)]
    reindex: bool,

    /// Disable recursive directory traversal
    #[arg(long, default_value_t = false)]
    no_recursive: bool,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn get_mtime<P: AsRef<Path>>(path: P) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(mtime)
}

#[instrument(skip(db))]
fn find_duplicates(db: &sled::Db) -> HashMap<String, Vec<String>> {
    let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();

    for item in db.iter() {
        if let Ok((_, value)) = item {
            let file_entry = FileEntry::from_bytes(&value).unwrap();
            hash_map
                .entry(file_entry.hash)
                .or_insert_with(Vec::new)
                .push(file_entry.path);
        }
    }
    

    hash_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect()
}

#[instrument(skip(db))]
async fn index_folder(db: &sled::Db, path: PathBuf, recursive: bool) -> Result<(), std::io::Error> {
    info!("Indexing folder: {}", path.display());
    trace!("Recursive: {}", recursive);

    let db = Arc::new(db.clone());
    let (tx, rx) = mpsc::channel(100);
    let mut tasks = JoinSet::new();
    let mut counter = 0u64;
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // File discovery task
    tasks.spawn(async move {
        let walker = if recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(mtime) = get_mtime(entry.path()) {
                    let task = FileTask {
                        path: entry.path().to_owned(),
                        mtime,
                        counter,
                    };
                    if tx.send(task).await.is_err() {
                        break;
                    }
                    counter += 1;
                }
            }
        }
    });

    // Process files in parallel
    let worker_count = 4;
    for _ in 0..worker_count {
        let rx = Arc::clone(&rx);
        let db = Arc::clone(&db);

        tasks.spawn(async move {
            let mut rx_guard = rx.lock().await;
            while let Some(task) = rx_guard.recv().await {
                let needs_hash = if let Ok(Some(existing)) = db.get(task.counter.to_be_bytes()) {
                    if let Some(entry) = FileEntry::from_bytes(&existing) {
                        entry.mtime != task.mtime
                    } else {
                        true
                    }
                } else {
                    true
                };

                if needs_hash {
                    if let Ok(hash) = calculate_hash_async(task.path.clone()).await {
                        let entry = FileEntry {
                            path: task.path.to_string_lossy().into_owned(),
                            hash,
                            mtime: task.mtime,
                        };
                        let _ = db.insert(task.counter.to_be_bytes(), entry.to_bytes());
                    }
                }
            }
        });
    }

    while let Some(result) = tasks.join_next().await {
        result?;
    }

    Ok(())
}

async fn move_duplicates(
    duplicates: HashMap<String, Vec<String>>,
    duplicates_folder: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::fs::create_dir_all(duplicates_folder).await?;

    let mut tasks = JoinSet::new();

    for (_, paths) in duplicates {
        let _original = &paths[0];
        for duplicate in &paths[1..] {
            let dup = duplicate.clone();
            let folder = duplicates_folder.to_owned();

            tasks.spawn(async move {
                let file_name = Path::new(&dup)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let new_path = Path::new(&folder).join(file_name);
                tokio::fs::rename(dup, new_path).await
            });
        }
    }

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}

#[instrument]
async fn calculate_hash_async(path: PathBuf) -> Result<String, std::io::Error> {
    trace!("Calculating hash for: {}", path.display());

    let mut file = tokio::fs::File::open(&path).await?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0; 1024 * 1024]; // 1MB buffer

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    // Setup tracing
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(
            tracing::Level::INFO
        )
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_ansi(true)
        .pretty()
        .init();

    info!("Starting file deduplication");
    debug!("Arguments: {:?}", args);

    let db = if let Some(cache) = args.cache.as_ref() {
        debug!("Using cache file: {}", cache);
        let db = sled::open(cache)?;
        if args.reindex {
            info!("Reindexing files...");
            index_folder(&db, Path::new(&args.path).to_path_buf(), !args.no_recursive).await?;
        } else {
            debug!("Using existing index");
        }
        db
    } else {
        debug!("Using temporary database");
        let db = sled::Config::new().temporary(true).open()?;
        index_folder(&db, Path::new(&args.path).to_path_buf(), !args.no_recursive).await?;
        db
    };
    info!("Found files: {:#?}", db.iter()
        .filter_map(|r| r.ok())
        .filter_map(|(_, v)| FileEntry::from_bytes(&v))
        .collect::<Vec<FileEntry>>());

    let duplicates = find_duplicates(&db);

    if duplicates.is_empty() {
        println!("No duplicates found");
        return Ok(());
    }


    println!("Found duplicates:");
    for (hash, paths) in &duplicates {
        println!("\nHash: {}", hash);
        for path in paths {
            println!("  {}", path);
        }
    }

    print!("Move duplicates to {}? (y/N): ", args.duplicates_folder);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        move_duplicates(duplicates, &args.duplicates_folder).await?;
        println!("Duplicates moved successfully");
    }

    Ok(())
}
