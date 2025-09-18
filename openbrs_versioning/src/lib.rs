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

use base64::{Engine as _, engine::general_purpose};
use openbrs_archv_cmprss::{self, archive_compress};
use openbrs_crypto::{CryptoMetadata, encrypt_archive};
use sha3::{Digest, Sha3_256};
use std::{fs, path::Path};

/// A blob is the path to the data with its hash
struct Blob<'path> {
    id: Option<String>, // SHA3-256 hash of content
    path: &'path Path,  // Path to the blob
}

impl<'path> Blob<'path> {
    fn new(path: &'path Path, file_content: &Vec<u8>) -> Self {
        Self {
            id: Blob::calc_id(file_content),
            path,
        }
    }

    fn calc_id(file_content: &Vec<u8>) -> Option<String> {
        // Get SHA3-256 hash of the file
        // Create the hasher
        let mut hasher = Sha3_256::new();

        // Hash the content
        hasher.update(file_content);

        // Consume the hash
        let file_hash = hasher.finalize();

        // Convert it to bytes
        let digest_bytes: &[u8] = file_hash.as_ref();

        // Convert it to base 64, and return it
        Some(general_purpose::STANDARD.encode(digest_bytes))
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

impl Tree {
    fn build(target_path: &Path, archive_path: &Path, blob_path: &Path, passwd: &[u8]) -> Self {
        let mut entries = Vec::new();

        // Read the directory, if it is indeed a directory
        if let Ok(dir_entries) = fs::read_dir(target_path) {
            // Iterate through children
            for entry in dir_entries.flatten() {
                // Get the path
                let path = entry.path();

                // Get the item's name
                let name = entry.file_name().to_string_lossy().to_string();

                // If it is a directory, iterate through it
                if path.is_dir() {
                    // If it is a subtree, create a Tree object
                    let subtree = Tree::build(&path, blob_path, blob_path, passwd);

                    // get its id, from a hash.
                    // This is recursive.
                    let tree_id = subtree.id.clone();

                    // Push it into our main entries variable
                    entries.push(TreeEntry::Dir { name, tree_id })
                } else if path.is_file() {
                    // If it is a file, then create a blob
                    // Archive, compress, then encrypt it, and return the file content
                    archive_compress(&target_path, &archive_path);
                    let file_content = encrypt_archive(&blob_path, &archive_path, passwd);

                    let blob_id = Blob::new(blob_path, &file_content).id.unwrap();

                    entries.push(TreeEntry::File { name, blob_id });
                }
            }
        }
        // Set the ID of the file/main target directory.
        let id = Self::calc_dir_id(&entries);

        // Return the ID and the tree itself
        Tree { id, entries }
    }

    fn calc_dir_id(entries: &Vec<TreeEntry>) -> String {
        // Create the hasher
        let mut hasher = Sha3_256::new();

        // For each entry, append its name and ID to the string before hashing it.
        for entry in entries {
            match entry {
                TreeEntry::File { name, blob_id } => {
                    hasher.update(format!("file:{}:{}", name, blob_id))
                }
                TreeEntry::Dir { name, tree_id } => {
                    hasher.update(format!("dir:{}:{}", name, tree_id))
                }
            }
        }

        // Encode it in base64
        general_purpose::STANDARD.encode(hasher.finalize())
    }
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

pub fn backup(
    backup_type: u8,
    target_path: &Path,
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
    let tree = Tree::build(target_path, archive_path, encr_archive_path, passwd);
}

// Differential backup
// Needs the target, and the metadata file (if any)
fn backup_diff(target_path: &Path, archive_path: &Path, encr_archive_path: &Path, passwd: &[u8]) {
    // Create the blob
    let tree = Tree::build(target_path, archive_path, encr_archive_path, passwd);
}

fn backup_incr() {}
