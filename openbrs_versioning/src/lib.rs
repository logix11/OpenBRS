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
use openbrs_archv_cmprss::{self, archive_compress_dir, archive_compress_file};
use openbrs_crypto::encrypt_archive;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::{
    fs::{self, File, metadata},
    io::Read,
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePath {
    pub target: PathBuf,
    pub main: PathBuf,
    pub objects: PathBuf,
    pub blobs: PathBuf,
    pub trees: PathBuf,
    pub commits: PathBuf,
    pub archive: PathBuf,
    pub encarch: Option<PathBuf>,
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
            archive: main.join(format!("objects/blobs/{archive_name}")),
            encarch: Some(main.join(format!("objects/blobs/{archive_name}.enc"))),
        }
    }

    pub fn create_dirs(&self) {
        fs::create_dir(&self.main).unwrap();
        fs::create_dir(&self.objects).unwrap();
        fs::create_dir(&self.blobs).unwrap();
        fs::create_dir(&self.trees).unwrap();
        fs::create_dir(&self.commits).unwrap();
    }
}

/// A blob is the path to the data with its hash
struct Blob {
    id: Option<String>, // SHA3-256 hash of content
                        //path: PathBuf,      // Path to the blob
}

impl Blob {
    fn new(file_content: &Vec<u8>) -> Self {
        Self {
            id: Blob::calc_id(file_content),
            //path,
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

        // Convert it to hexa, and return it
        Some(hex::encode(digest_bytes))
    }
}

/// A tree maps names to blobs/trees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub id: String,              // Hash of serialized tree
    pub entries: Vec<TreeEntry>, // files/subdirs in this folder
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TreeEntry {
    File {
        name: String,
        blob_id: String,
    },
    Dir {
        name: String,       // Normal directory name
        tree_id: String,    // Tree's content's hash
        subtree: Box<Tree>, // To store sub-trees
    },
}

#[derive(Debug)]
pub enum Change {
    Added(TreeEntry),
    Modified(TreeEntry, Vec<Change>),
    Unchanged(TreeEntry),
    Removed(String), // Name of the file/directory removed
}

pub type Store = HashMap<String, Tree>;

impl Tree {
    pub fn build(paths: &FilePath, passwd: &[u8], first_backup: bool) -> Self {
        if paths.target.is_dir() {
            let tree = Tree::build_dir(paths, passwd);
            archive_compress_dir(&paths.target, &paths.archive);
            tree
        } else {
            let tree = Tree::build_file(paths, passwd, first_backup);
            tree
        }
    }

    fn build_dir(paths: &FilePath, passwd: &[u8]) -> Self {
        let mut entries = Vec::new();

        // Collect entries first, so the iterator (and its FD) is dropped
        let entries_vec: Vec<_> = fs::read_dir(&paths.target)
            .unwrap()
            .flatten()
            .map(|entry| {
                let path = FilePath::new(entry.path());
                let name = entry.file_name().to_string_lossy().to_string();
                (path, name)
            })
            .collect(); // <-- FD closed here

        // Now process the collected entries
        for (path, name) in entries_vec {
            // If it is a directory, iterate through it
            if path.target.is_dir() {
                // If it is a subtree, check first whether it is the .openbrs workplace
                // If yes, skip it
                if path.target.to_string_lossy().contains("/.openbrs") {
                    continue;
                }

                // create a Tree instance
                let new_paths = FilePath::new(path.target);
                let subtree = Tree::build_dir(&new_paths, passwd);
                let subtree_id = subtree.id.clone();

                // Push it into our main entries variable
                entries.push(TreeEntry::Dir {
                    name,
                    tree_id: subtree_id,
                    subtree: Box::new(subtree),
                });
            } else if path.target.is_file() {
                // If it is a file, then create a blob if it's the first (or a full backup) backup; otherwise, only parse
                // the item, and get their hashes, to build the tree and compare it to the reference's tree, and decide which
                // files changed.
                // Archive, compress, then encrypt it, and return the file content

                // Read file
                let mut file_content = Vec::new();
                let mut file = File::open(path.target).unwrap();
                file.read_to_end(&mut file_content).unwrap();

                // Get its hash (ID)
                let blob_id = Blob::new(&file_content).id.unwrap();

                // push it to the tree
                entries.push(TreeEntry::File { name, blob_id });
            }
        }

        // Set the ID of the file/main target directory.
        let id = Self::calc_dir_id(&entries);

        // Return the ID and the tree itself
        Tree { id, entries }
    }
    fn build_file(paths: &FilePath, passwd: &[u8], first_backup: bool) -> Self {
        let mut entries = Vec::new();

        // If it is a file, then create a blob
        let name = paths
            .target
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        if first_backup {
            archive_compress_file(&paths.target, &paths.archive);
            let file_content = encrypt_archive(&paths.archive, passwd);

            //let blob_id = Blob::new(paths.archive.clone(), &file_content).id.unwrap();
            let blob_id = Blob::new(&file_content).id.unwrap();
            let blod_id_clone = blob_id.clone();

            // push it to the tree
            entries.push(TreeEntry::File { name, blob_id });

            // Return the ID and the tree itself
            Tree {
                id: blod_id_clone,
                entries,
            }
        } else {
            // Read file
            let mut file_content = Vec::new();
            let mut file = File::open(paths.target.to_path_buf()).unwrap();
            file.read_to_end(&mut file_content).unwrap();

            // Get its hash (ID)
            let blob_id = Blob::new(&file_content).id.unwrap();
            let blod_id_clone = blob_id.clone();

            // push it to the tree
            entries.push(TreeEntry::File { name, blob_id });

            // Return the ID and the tree itself
            Tree {
                id: blod_id_clone,
                entries,
            }
        }
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
                TreeEntry::Dir { name, tree_id, .. } => {
                    hasher.update(format!("dir:{}:{}", name, tree_id))
                }
            }
        }

        // The ID is now a hash of a serialization of FileType:Name:Id; where Name is the file/dir name, and ID
        // is the hash of the content
        // Encode it in base64
        hex::encode(hasher.finalize())
    }

    // Function to calculate difference between trees
    pub fn diff_trees(
        new_tree: &Tree,
        old_tree: &Tree,
        store: &HashMap<String, Tree>,
    ) -> Vec<Change> {
        // We'll store the changes here
        let mut changes: Vec<Change> = Vec::new();

        // Prepare a hash map to index old entires by name
        let mut old_map: HashMap<String, &TreeEntry> = HashMap::new();
        for entry in &old_tree.entries {
            match entry {
                TreeEntry::File { name, .. } | TreeEntry::Dir { name, .. } => {
                    old_map.insert(name.clone(), entry);
                }
            }
        }

        // check new entries
        for entry in &new_tree.entries {
            match entry {
                TreeEntry::File { name, blob_id } => {
                    if let Some(old_entry) = old_map.remove(name) {
                        match old_entry {
                            TreeEntry::File {
                                blob_id: old_blob, ..
                            } => {
                                if blob_id == old_blob {
                                    // Same name, same ID (content's hash)
                                    changes.push(Change::Unchanged(entry.clone()));
                                } else {
                                    // Same name, different ID (hash)
                                    changes.push(Change::Modified(entry.clone(), vec![]));
                                }
                            }
                            _ => {
                                // Type changed from directory to file
                                changes.push(Change::Modified(entry.clone(), vec![]));
                            }
                        }
                    } else {
                        // New entry
                        changes.push(Change::Added(entry.clone()));
                    }
                }

                TreeEntry::Dir { name, tree_id, .. } => {
                    if let Some(old_entry) = old_map.remove(name) {
                        match old_entry {
                            TreeEntry::Dir {
                                tree_id: old_tree_id,
                                ..
                            } => {
                                if tree_id == old_tree_id {
                                    // Same directory name, same ID (content's hash)
                                    changes.push(Change::Unchanged(entry.clone()));
                                } else {
                                    // Same name, but different ID
                                    // Get the older subtree
                                    let old_subtree = store.get(old_tree_id).unwrap();

                                    // Get the newer subtree
                                    let new_subtree = store.get(tree_id).unwrap();

                                    // Calculate the changes
                                    let sub_changes =
                                        Tree::diff_trees(new_subtree, old_subtree, store);

                                    // Push changes
                                    changes.push(Change::Modified(entry.clone(), sub_changes));
                                }
                            }
                            _ => {
                                //Type change from file to directory
                                changes.push(Change::Modified(entry.clone(), vec![]))
                            }
                        }
                    } else {
                        // New directory
                        changes.push(Change::Added(entry.clone()))
                    }
                }
            }
        }

        // Remaining old entries are removed
        for (name, _) in old_map {
            changes.push(Change::Removed(name))
        }

        changes
    }

    fn _populate_store(&self, store: &mut Store) {
        store.insert(self.id.clone(), self.clone());
        for entry in &self.entries {
            if let TreeEntry::Dir { tree_id, .. } = entry {
                if let Some(subtree) = store.clone().get(tree_id) {
                    subtree._populate_store(store);
                }
            }
        }
    }
}

/// A commit ties everything together
#[derive(Serialize, Deserialize)]
pub struct Commit<'message> {
    pub id: String,             // Unique identifier
    pub tree_id: String,        // Root tree id, which is the hash of its content
    pub parent: Option<String>, // Previous commit (None for the initial backup)
    pub message: &'message str, // Commit message
}

impl<'message> Commit<'message> {
    pub fn new(tree_id: String, parent: Option<String>, message: &'message str) -> Self {
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
        let id = hex::encode(hasher.finalize());

        // Return the commit
        Self {
            id,
            tree_id,
            parent,
            message,
        }
    }
}
