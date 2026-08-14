# ftp-folder-transfer Specification

## Purpose
FTP folder upload/download — recursively transfer entire directory trees between local filesystem and FTP servers through the TransferScheduler batch system.

## ADDED Requirements

### Requirement: FTP folder download
The system SHALL allow users to download entire directories from an FTP server to the local filesystem, expanding the directory tree into individual file transfer tasks within a single batch.

#### Scenario: Download folder via cross-panel paste (copy)
- **WHEN** user yanks (y) a directory from an FTP panel, switches to a local panel, and presses p
- **THEN** the system recursively lists the remote directory tree using MLSD
- **AND** creates the local directory structure matching the remote tree
- **AND** enqueues all files as individual FtpDownload tasks in a single batch via TransferScheduler
- **AND** the batch appears in TransferManager with the folder name as context

#### Scenario: Download folder via cross-panel paste (cut)
- **WHEN** user cuts (x) a directory from an FTP panel, switches to a local panel, and presses p
- **THEN** after all files in the batch download successfully, the remote directory tree is deleted from the FTP server
- **AND** if any file fails, the remote content is preserved (no partial cleanup)

#### Scenario: Download progress reflects total folder size
- **WHEN** a folder download batch is enqueued
- **THEN** total_bytes for the batch equals the sum of all file sizes from the recursive MLSD listing
- **AND** TransferManager displays aggregated progress across all files in the batch

#### Scenario: Cancel folder download
- **WHEN** user cancels all transfers in a folder download batch
- **THEN** all running downloads are cancelled and partial local files are cleaned up
- **AND** successfully downloaded files remain on disk

#### Scenario: Pre-create local directories before download
- **WHEN** a folder download is initiated
- **THEN** all target local directories are created before the first file transfer starts
- **AND** directory creation errors are reported immediately and abort the entire batch

### Requirement: FTP folder upload
The system SHALL allow users to upload entire directories from the local filesystem to an FTP server, expanding the directory tree into individual file transfer tasks within a single batch.

#### Scenario: Upload folder via cross-panel paste (copy)
- **WHEN** user yanks (y) a local directory, switches to an FTP panel, and presses p
- **THEN** the system recursively scans the local directory to build a file list with sizes
- **AND** creates the remote directory structure on the FTP server using MKD commands
- **AND** enqueues all files as individual FtpUpload tasks in a single batch via TransferScheduler

#### Scenario: Upload folder via cross-panel paste (cut)
- **WHEN** user cuts (x) a local directory, switches to an FTP panel, and presses p
- **THEN** after all files in the batch upload successfully, the local source directory is deleted
- **AND** if any file fails, the local source is preserved

#### Scenario: Pre-create remote directories before upload
- **WHEN** a folder upload is initiated
- **THEN** all remote directories in the tree are created on the FTP server using MKD before the first file transfer starts
- **AND** MKD errors for missing parent directories are handled by creating intermediate directories as needed

#### Scenario: Cancel folder upload
- **WHEN** user cancels all transfers in a folder upload batch
- **THEN** all running uploads are cancelled and partial remote files are cleaned up via RM on the FTP server

### Requirement: Recursive FTP directory listing
The system SHALL recursively list all files and directories under a given remote path using MLSD (with LIST fallback).

#### Scenario: List shallow directory
- **WHEN** `list_dir_recursive("/var/www")` is called and the directory contains only files
- **THEN** the function returns a flat list of `(path, size, is_dir)` for each entry

#### Scenario: List nested directory tree
- **WHEN** `list_dir_recursive("/var/www")` is called and the directory contains subdirectories with files
- **THEN** the function returns all files from all subdirectory levels with their full remote paths
- **AND** directory entries (is_dir=true) are included so the caller can pre-create the structure

#### Scenario: MLSD fallback to LIST
- **WHEN** the FTP server does not support MLSD
- **THEN** the system falls back to LIST and parses the output with `parse_list_line()`
- **AND** file sizes from LIST are used when available, defaulting to 0 for entries without size info

#### Scenario: Empty directory
- **WHEN** `list_dir_recursive` is called on an empty directory
- **THEN** the function returns an empty list with no error
- **AND** the pre-create step creates the directory itself as a leaf

#### Scenario: Permission error during listing
- **WHEN** a subdirectory in the tree returns a permission error during MLSD
- **THEN** the system reports the error including the specific failing path
- **AND** the entire folder transfer is aborted before any file transfers begin

### Requirement: Partial failure continuation
The system SHALL continue transferring remaining files when individual file transfers fail within a folder batch.

#### Scenario: Single file fails, rest continue
- **WHEN** one file in a folder upload/download batch fails (e.g., connection timeout)
- **THEN** the failed file entry shows "failed" status with the error message in TransferManager
- **AND** remaining queued files continue to transfer
- **AND** already-completed files are not affected

#### Scenario: All files fail
- **WHEN** all files in a folder batch fail (e.g., connection lost)
- **THEN** the batch divider remains visible showing all entries as failed
- **AND** cut-mode cleanup is NOT triggered (source content is preserved)
