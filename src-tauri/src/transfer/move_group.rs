use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::ftp::FtpManager;
use super::TransferTask;

fn ftp_parts(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix("ftp://")?;
    let slash = rest.find('/')?;
    Some((&rest[..slash], &rest[slash..]))
}

async fn size_compat(client: &mut suppaftp::tokio::AsyncRustlsFtpStream, remote: &str) -> Result<u64, String> {
    match client.size(remote).await {
        Ok(size) => Ok(size as u64),
        Err(absolute_error) => {
            let (parent, name) = remote.rsplit_once('/').ok_or("Invalid FTP file path")?;
            if name.is_empty() { return Err("Invalid FTP file name".into()); }
            client.cwd(if parent.is_empty() { "/" } else { parent }).await
                .map_err(|e| format!("CWD {parent} after SIZE failure ({absolute_error}): {e}"))?;
            client.size(name).await.map(|size| size as u64)
                .map_err(|e| format!("SIZE {name} after CWD: {e}"))
        }
    }
}

async fn remote_size(manager: &Arc<TokioMutex<FtpManager>>, path: &str) -> Result<u64, String> {
    let (conn, remote) = ftp_parts(path).ok_or("Invalid FTP path")?;
    let mgr = manager.lock().await;
    let session = mgr.create_independent(conn).await?;
    drop(mgr);
    let mut ftp = session.lock().await;
    let size = size_compat(&mut ftp.client, remote).await?;
    let _ = ftp.client.quit().await;
    Ok(size)
}

async fn path_size(manager: &Arc<TokioMutex<FtpManager>>, path: &str) -> Result<u64, String> {
    if ftp_parts(path).is_some() { remote_size(manager, path).await }
    else { std::fs::metadata(path).map(|m| m.len()).map_err(|e| format!("Metadata {path}: {e}")) }
}

fn local_files(root: &Path) -> Result<HashMap<String, u64>, String> {
    let mut result = HashMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|e| format!("Read {}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() { return Err(format!("Symbolic link in move source: {}", path.display())); }
            if meta.is_dir() { pending.push(path); }
            else { result.insert(path.to_string_lossy().to_string(), meta.len()); }
        }
    }
    Ok(result)
}

async fn remote_files(manager: &Arc<TokioMutex<FtpManager>>, path: &str) -> Result<HashMap<String, u64>, String> {
    let (conn, remote) = ftp_parts(path).ok_or("Invalid FTP path")?;
    let mgr = manager.lock().await;
    let session = mgr.create_independent(conn).await?;
    drop(mgr);
    let mut ftp = session.lock().await;
    let entries = crate::ftp::list_dir_recursive(&mut ftp.client, remote).await?;
    let mut result = HashMap::new();
    for (remote, listed_size, is_dir) in entries {
        if !is_dir {
            let size = if listed_size > 0 { listed_size } else {
                size_compat(&mut ftp.client, &remote).await?
            };
            result.insert(format!("ftp://{conn}{remote}"), size);
        }
    }
    let _ = ftp.client.quit().await;
    Ok(result)
}

async fn delete_remote(manager: &Arc<TokioMutex<FtpManager>>, path: &str, is_dir: bool, expected: &HashMap<String, u64>) -> Result<(), String> {
    let (conn, remote) = ftp_parts(path).ok_or("Invalid FTP path")?;
    let mgr = manager.lock().await;
    let session = mgr.create_independent(conn).await?;
    drop(mgr);
    let mut ftp = session.lock().await;
    if is_dir {
        let entries = crate::ftp::list_dir_recursive(&mut ftp.client, remote).await?;
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        for (path, size, dir) in entries {
            if dir { dirs.push(path); }
            else {
                let key = format!("ftp://{conn}{path}");
                let actual_size = if size > 0 { size } else {
                    size_compat(&mut ftp.client, &path).await?
                };
                if expected.get(&key).is_none_or(|expected_size| *expected_size != actual_size) {
                    return Err(format!("Source changed before cleanup: {path}"));
                }
                files.push(path);
            }
        }
        if files.len() != expected.len() { return Err("Source changed before cleanup".into()); }
        for file in files { ftp.client.rm(&file).await.map_err(|e| format!("Delete {file}: {e}"))?; }
        dirs.sort_by_key(|dir| std::cmp::Reverse(dir.matches('/').count()));
        for dir in dirs { ftp.client.rmdir(&dir).await.map_err(|e| format!("Remove {dir}: {e}"))?; }
        ftp.client.rmdir(remote).await.map_err(|e| format!("Remove {remote}: {e}"))?;
    } else {
        ftp.client.rm(remote).await.map_err(|e| format!("Delete {remote}: {e}"))?;
    }
    let _ = ftp.client.quit().await;
    Ok(())
}

fn delete_local_tree(source: &str, expected: &HashMap<String, u64>) -> Result<(), String> {
    let root = Path::new(source);
    if local_files(root)? != *expected { return Err("Source changed before cleanup".into()); }
    let mut dirs = vec![root.to_path_buf()];
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() { pending.push(path.clone()); dirs.push(path); }
        }
    }
    for file in expected.keys() { std::fs::remove_file(file).map_err(|e| format!("Delete {file}: {e}"))?; }
    dirs.sort_by_key(|dir| std::cmp::Reverse(dir.components().count()));
    for dir in dirs { std::fs::remove_dir(&dir).map_err(|e| format!("Remove {}: {e}", dir.display()))?; }
    Ok(())
}

pub async fn verify_and_cleanup(
    source: &str,
    destination: &str,
    is_dir: bool,
    tasks: &[TransferTask],
    manager: &Arc<TokioMutex<FtpManager>>,
) -> Result<(), String> {
    for task in tasks {
        let source_size = path_size(manager, &task.source).await?;
        let dest_size = path_size(manager, &task.destination).await?;
        if source_size != dest_size || (task.total_bytes > 0 && source_size != task.total_bytes) {
            return Err(format!("Move verification failed for {}: source={source_size}, destination={dest_size}, expected={}", task.source, task.total_bytes));
        }
    }
    if is_dir {
        let mut expected = HashMap::new();
        for task in tasks { expected.insert(task.source.clone(), path_size(manager, &task.destination).await?); }
        let actual = if ftp_parts(source).is_some() { remote_files(manager, source).await? }
            else { local_files(Path::new(source))? };
        if actual != expected { return Err("Source directory changed during transfer; source preserved".into()); }
        if ftp_parts(destination).is_some() {
            let (conn, remote) = ftp_parts(destination).ok_or("Invalid FTP destination")?;
            let mgr = manager.lock().await;
            let session = mgr.create_independent(conn).await?;
            drop(mgr);
            let mut ftp = session.lock().await;
            ftp.client.cwd(remote).await.map_err(|e| format!("Destination directory missing: {e}"))?;
        } else if !Path::new(destination).is_dir() {
            return Err("Destination directory missing".into());
        }
    }
    let mut expected_sources = HashMap::new();
    for task in tasks { expected_sources.insert(task.source.clone(), path_size(manager, &task.destination).await?); }
    let cleanup = if ftp_parts(source).is_some() { delete_remote(manager, source, is_dir, &expected_sources).await }
    else if is_dir { delete_local_tree(source, &expected_sources) }
    else { std::fs::remove_file(source).map_err(|e| format!("Source cleanup failed: {e}")) };
    cleanup.map_err(|e| format!("PARTIAL: destination verified, source cleanup failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::{ftp_parts, size_compat, verify_and_cleanup};
    use crate::ftp::FtpManager;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn parses_ftp_paths_without_losing_nested_directories() {
        assert_eq!(ftp_parts("ftp://one/a/b/file.txt"), Some(("one", "/a/b/file.txt")));
        assert_eq!(ftp_parts("C:\\local\\file.txt"), None);
    }

    #[test]
    #[ignore = "requires WIND_FTP_TEST_ADDR and WIND_FTP_TEST_FILE"]
    fn live_size_compat_accepts_nested_ftp_path() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let addr = std::env::var("WIND_FTP_TEST_ADDR").expect("WIND_FTP_TEST_ADDR");
            let path = std::env::var("WIND_FTP_TEST_FILE").expect("WIND_FTP_TEST_FILE");
            let mut ftp = suppaftp::tokio::AsyncRustlsFtpStream::connect(&addr).await.unwrap();
            ftp.login("anonymous", "anonymous").await.unwrap();
            let size = size_compat(&mut ftp, &path).await.unwrap();
            assert!(size > 0);
            let _ = ftp.quit().await;
        });
    }

    #[test]
    fn changed_source_tree_blocks_cleanup() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transfer");
        let path = root.to_string_lossy().to_string();
        let manager = Arc::new(Mutex::new(FtpManager::new()));
        let result = verify_and_cleanup(&path, &path, true, &[], &manager).await;
        assert!(result.unwrap_err().contains("Source directory changed"));
        assert!(root.is_dir());
        });
    }

    #[test]
    fn destination_mismatch_blocks_file_cleanup() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let source = root.join("Cargo.toml").to_string_lossy().to_string();
        let destination = root.join("src/lib.rs").to_string_lossy().to_string();
        let task = super::TransferTask {
            id: 1, batch_id: 1, op_type: super::super::TransferType::FtpDownload,
            status: super::super::TaskStatus::Done,
            source: source.clone(), destination: destination.clone(), total_bytes: 1, bytes_done: 1,
            conn_name: None, cancel_flag: None, skip_rel_paths: Vec::new(),
            rename_rel_paths: Default::default(), keep_copy: false, permanent: false,
            password: None, internal_paths: None,
        };
        let manager = Arc::new(Mutex::new(FtpManager::new()));
        assert!(verify_and_cleanup(&source, &destination, false, &[task], &manager).await.is_err());
        assert!(std::path::Path::new(&source).is_file());
        });
    }
}
