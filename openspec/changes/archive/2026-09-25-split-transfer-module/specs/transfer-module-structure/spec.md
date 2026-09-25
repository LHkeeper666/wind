# transfer-module-structure Specification

## Purpose
Define the internal module structure of the transfer subsystem after refactoring from a single file to a directory module.

## Requirements
### Requirement: Transfer module directory structure
The transfer module SHALL be organized as a directory module `transfer/` containing the following sub-modules:

#### Scenario: Module layout
- **WHEN** the refactoring is complete
- **THEN** `src-tauri/src/transfer.rs` is replaced by `src-tauri/src/transfer/` directory
- **AND** the directory contains: `mod.rs`, `scheduler.rs`, `ftp.rs`, `local.rs`, `conflict.rs`, `helpers.rs`

### Requirement: Public API preservation
The transfer module SHALL expose the same public API as before the refactoring.

#### Scenario: External imports unchanged
- **WHEN** `commands/transfer_cmd.rs` imports `crate::transfer::TransferScheduler`
- **THEN** the import resolves without changes
- **AND** all public types (TransferScheduler, EnqueueTask, TransferTask, TransferType, TaskStatus, TransferHistoryRecord) remain accessible at `crate::transfer::*`

### Requirement: Module responsibility boundaries
Each sub-module SHALL have a single, well-defined responsibility.

#### Scenario: scheduler.rs owns queue state
- **WHEN** code manages transfer queue, slots, or history
- **THEN** it resides in `scheduler.rs`

#### Scenario: ftp.rs owns FTP execution
- **WHEN** code executes FTP download/upload/delete operations
- **THEN** it resides in `ftp.rs`

#### Scenario: local.rs owns local execution
- **WHEN** code executes local copy/move/delete operations
- **THEN** it resides in `local.rs`

#### Scenario: conflict.rs owns conflict detection
- **WHEN** code scans for file conflicts before transfer
- **THEN** it resides in `conflict.rs`

#### Scenario: helpers.rs owns shared utilities
- **WHEN** code provides utility functions used across modules
- **THEN** it resides in `helpers.rs`
