/*
* Plan and design:
* To design a versioning system, I'll use three types of objects, just like Git:
*  1. Blob: represents the content of a file (after optional compression + encryption).
       * Stored by hash (e.g., SHA-256).
       * Identical content -> identical blob -> deduplication for free.
   2. Tree: represents a directory.
       * Maps file/directory names -> blob IDs or sub-tree IDs.
       * Basically the "snapshot of a folder."
   3. Commit: represents a backup session.
       * Points to a root tree.
       * Contains metadata (timestamp, backup type, salts, nonces, et cetera.).
       * May reference a parent commit (for differential/incremental backups).
*/

/// A blob is just binary data with a hash
struct Blob {
    id: String,   // SHA3-256 hash of content
    path: String, // Path to the blob
}

impl Blob {
    fn new(id: String, path: String) -> Self {
        Blob { id, path }
    }
}

/// A tree maps names to blobs/trees
struct Tree {
    id: String,              // Hash of serialized tree
    entries: Vec<TreeEntry>, // files/subdirs in this folder
}

enum TreeEntry {
    File { name: String, blob_id: String },
    Dir { name: String, tree_id: String },
}
/// A commit ties everything together
struct Commit {
    id: String,             // Hash of commit data
    tree_id: String,        // Root tree
    parent: Option<String>, // Previous commit (None for full backup)
    backup_type: BackupType,
    timestamp: u64,
    metadata: CryptoMetadata, // salts, nonces, digests
}

enum BackupType {
    Full,
    Differential,
    Incremental,
}

use base64::{Engine as _, engine::general_purpose};
use openbrs_archv_cmprss::{self, archive_compress};
use openbrs_crypto::{CryptoMetadata, encrypt_archive};
use sha3::{Digest, Sha3_256};
use std::path::Path;

pub fn backup(
    backup_type: u8,
    target_path: &Path,
    main_dir: &Path,
    archive_path: &Path,
    encr_archive_path: &Path,
    passwd: &[u8],
) {
    // Select the appropriate function
    if backup_type == 0b1 {
        backup_full(target_path, archive_path, encr_archive_path, passwd);
    } else if backup_type == 0b10 {
        backup_diff(target_path, archive_path, encr_archive_path, passwd);
    } else if backup_type == 0b11 {
        backup_incr();
    } else {
        panic!("Error: no such backup type");
    }
}

// Function to run a full backup.
fn backup_full(target_path: &Path, archive_path: &Path, encr_archive_path: &Path, passwd: &[u8]) {
    // Archive and compress
    archive_compress(&target_path, &archive_path);

    // Encrypt
    encrypt_archive(&encr_archive_path, &archive_path, passwd);
}

// Differential backup
// Needs the target, and the metadata file (if any)
fn backup_diff(target_path: &Path, archive_path: &Path, encr_archive_path: &Path, passwd: &[u8]) {
    // Archive and compress
    archive_compress(&target_path, &archive_path);

    // Encrypt
    let file_content = encrypt_archive(&encr_archive_path, &archive_path, passwd);

    // Get SHA3-256 hash of the file
    // Create the hasher
    let mut hasher = Sha3_256::new();

    // Hash the content
    hasher.update(file_content);

    // Consume the hash
    let file_hash = hasher.finalize();

    // Convert it to bytes
    let digest_bytes: &[u8] = file_hash.as_ref();

    // Convert it to base 64
    let digest_base64 = general_purpose::STANDARD.encode(digest_bytes);

    // Create the blob
    let blob = Blob::new(
        digest_base64,
        String::from("./test/.openbrs/objects/backup1"),
    );
}

fn backup_incr() {}
