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
use openbrs_crypto::encrypt_archive;
use sha3::{Digest, Sha3_256};
use std::{
    fs::{self, metadata},
    path::{Path, PathBuf},
};

pub struct FilePath {
    target: PathBuf,
    main: PathBuf,
    objects: PathBuf,
    blobs: PathBuf,
    trees: PathBuf,
    commits: PathBuf,
    archive: PathBuf,
    encarch: PathBuf,
}

impl FilePath {
    pub fn new(target_path: PathBuf) -> Self {
        let archive_name = format!(
            "{}.tar.xz",
            target_path.file_name().unwrap().to_str().unwrap()
        );

        let main = if metadata(&target_path).unwrap().is_dir() {
            target_path.to_path_buf().join(".openbrs")
        } else {
            target_path.parent().unwrap().to_path_buf().join(".openbrs")
        };

        Self {
            target: target_path,
            main: main.clone(),
            objects: main.join("objects"),
            blobs: main.join("objects/blobs"),
            trees: main.join("objects/trees"),
            commits: main.join("objects/commits"),
            archive: main.join(format!("objects/blobs/{archive_name};")),
            encarch: main.join(format!("objects/blobs/{archive_name}.enc")),
        }
    }
}

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
    fn build(target: &PathBuf, paths: &FilePath, passwd: &[u8]) -> Self {
        let mut entries = Vec::new();

        // Read the directory, if it is indeed a directory
        if let Ok(dir_entries) = fs::read_dir(target) {
            // Iterate through children
            for entry in dir_entries.flatten() {
                // Get the path
                let path = entry.path();

                // Get the item's name
                let name = entry.file_name().to_string_lossy().to_string();

                // If it is a directory, iterate through it
                if path.is_dir() {
                    // If it is a subtree, create a Tree object
                    let subtree = Tree::build(&path, paths, passwd);

                    // get its id, from a hash. This is recursive.
                    let tree_id = subtree.id.clone();

                    // Push it into our main entries variable
                    entries.push(TreeEntry::Dir { name, tree_id })
                } else if path.is_file() {
                    // If it is a file, then create a blob
                    // Archive, compress, then encrypt it, and return the file content
                    archive_compress(&paths.target, &paths.archive);
                    let file_content = encrypt_archive(&paths.archive, passwd);

                    let blob_id = Blob::new(&paths.archive, &file_content).id.unwrap();

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

        // The ID is now a hash of a serialization of FileType:Name:Id; where Name is the file/dir name, and ID
        // is the hash of the content
        // Encode it in base64
        general_purpose::STANDARD.encode(hasher.finalize())
    }
}

/// A commit ties everything together
struct Commit<'message> {
    id: String,             // Unique identifier
    tree_id: String,        // Root tree id, which is the hash of its content
    parent: Option<String>, // Previous commit (None for the initial backup)
    message: &'message str, // Commit message
}

impl<'message> Commit<'message> {
    fn new(tree_id: String, parent: Option<String>, message: &'message str) -> Self {
        // Create a hasher to create the ID
        let mut hasher = Sha3_256::new();

        // Append the tree_id first
        hasher.update(tree_id.as_bytes());

        // Append the parent's id, of any
        if let Some(ref p) = parent {
            hasher.update(p.as_bytes());
        }

        // Append the commit's message
        hasher.update(message.as_bytes());

        // Hash the serial, and encode it in Base64
        let id = general_purpose::STANDARD.encode(hasher.finalize());

        // Return the commit
        Self {
            id,
            tree_id,
            parent,
            message,
        }
    }
}

// Function to run a full backup.
pub fn backup_full(paths: &FilePath, passwd: &[u8]) {
    // Make the backup, this will create the blob, and prepare the tree
    let tree = Tree::build(&paths.target, &paths, passwd);

    // Make the commit which will point to the blob and tree.
    // If the work is not committed, it'll be some trash that may need to be cleaned later
    let commit = Commit::new(tree.id, None, "First commit");
}

// Differential backup
// Needs the target, and the metadata file (if any)
pub fn backup_diff(paths: &FilePath, passwd: &[u8]) {
    // Create the blob
    let tree = Tree::build(&paths.target, &paths, passwd);
}

fn backup_incr() {}
