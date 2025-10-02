use openbrs_archv_cmprss::{archive_compress_dir, archive_compress_file};
use serde::{Deserialize, Serialize};
use serde_json;
use sha3::{Digest, Sha3_256};
use std::fs::metadata;
use std::{
    fs::{self, File},
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

/// A commit ties everything together
#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub id: String,             // Unique identifier
    pub tree_id: String,        // Root tree id, which is the hash of its content
    pub parent: Option<String>, // Previous commit (None for the initial backup)
    pub message: String,        // Commit message
}

impl Commit {
    pub fn new(tree_id: String, parent: Option<String>, message: String) -> Self {
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

    pub fn write(&self, paths: &FilePath) {
        // Write off the commit as a JSON
        // Turn the tree to JSON String format
        let json = serde_json::to_string_pretty(&self).unwrap();

        // Prepare the path
        let path = paths.commits.join(format!("{}.json", self.id));

        // Write it off
        fs::write(path, json).unwrap();
    }
}

/// A blob is the path to the data with its hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    pub id: Option<String>, // SHA3-256 hash of content
}

impl Blob {
    pub fn new(file_content: &Vec<u8>) -> Self {
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
    pub id: String,             // Hash of serialized tree
    pub name: String,           // Name of directory
    pub entries: Vec<EntryRef>, // IDs of contents.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRef {
    pub name: String,
    pub id: String,
}

impl Tree {
    pub fn build(paths: &FilePath, first_backup: bool) -> Self {
        if paths.target.is_dir() {
            let tree = Tree::build_dir(&paths, &paths);
            archive_compress_dir(&paths.target, &paths.archive);
            tree
        } else {
            let tree = Tree::build_file(paths, first_backup);
            tree
        }
    }

    fn build_dir(main_paths: &FilePath, current_paths: &FilePath) -> Self {
        // Create a vector for the IDs:name string pairs.
        let mut entries = Vec::new();

        // Collect entries first, so the iterator (and its FD) is dropped
        let entries_vec: Vec<_> = fs::read_dir(&current_paths.target)
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
                let subtree = Tree::build_dir(&main_paths, &FilePath::new(path.target));

                // Push its id into our main entries variable
                entries.push(EntryRef {
                    name: name,
                    id: subtree.id,
                });
            } else if path.target.is_file() {
                // Parse the item, hash their content, to build the tree.
                let mut file_content = Vec::new();
                let mut file = File::open(path.target).unwrap();
                file.read_to_end(&mut file_content).unwrap();

                // Get its hash (ID)
                let blob = Blob::new(&file_content);

                // push it to the tree
                entries.push(EntryRef {
                    name: name,
                    id: blob.id.unwrap(),
                });
            }
        }

        // Set the ID of the file/main target directory.
        let id = Self::calc_dir_id(entries.clone());

        // Return the ID, the filename, and the entries.
        let tree = Tree {
            id,
            name: current_paths
                .target
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            entries,
        };

        tree.write_tree(main_paths);

        tree
    }

    fn build_file(paths: &FilePath, first_backup: bool) -> Self {
        // If it is a file, then create a blob
        let name = paths
            .target
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        if first_backup {
            archive_compress_file(&paths.target, &paths.archive);

            // Read the file and create a Blob instance
            let mut file_content = Vec::new();
            let mut file = File::open(&paths.target).unwrap();
            file.read_to_end(&mut file_content).unwrap();

            let blob_id = Blob::new(&file_content).id.unwrap();

            // Return the ID and the tree itself
            Tree {
                id: blob_id.clone(),
                name: name.clone(),
                entries: vec![EntryRef { name, id: blob_id }],
            }
        } else {
            // Read file
            let mut file_content = Vec::new();
            let mut file = File::open(paths.target.to_path_buf()).unwrap();
            file.read_to_end(&mut file_content).unwrap();

            // Get its hash (ID)
            let blob_id = Blob::new(&file_content).id.unwrap();

            // Return the ID and the tree itself
            Tree {
                id: blob_id.clone(),
                name: name.clone(),
                entries: vec![EntryRef { name, id: blob_id }],
            }
        }
    }

    fn calc_dir_id(mut entries: Vec<EntryRef>) -> String {
        // Create the hasher
        let mut hasher = Sha3_256::new();

        // Sort it, to have determnistic IDs
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        // For each entry, append its name and ID to the string before hashing it.
        for entry in entries {
            hasher.update(format!("{}:{}", entry.name, entry.id))
        }

        // The ID is now a hash of a serialization of FileType:Name:Id; where Name is the file/dir name, and ID
        // is the hash of the content
        // Encode it in base64
        hex::encode(hasher.finalize())
    }

    fn write_tree(&self, paths: &FilePath) {
        // Write off the tree as a JSON
        // Turn the tree to JSON String format
        let json = serde_json::to_string_pretty(&self).unwrap();

        // Prepare the path
        let path = paths.trees.join(format!("{}.json", &self.id));
        println!("Path :: {:?}", path);
        // Create the file
        fs::File::create(&path).unwrap();
        // Write it off
        fs::write(path, json).unwrap();
    }
}
