use openbrs_versioning::{Commit, FilePath, Store, Tree};
use serde_json;
use std::{fs, path::Path};
fn main() {
    // Get path
    let target_path = Path::new("test");

    // Ensure that the path is not absolute.
    if target_path.is_absolute() {
        panic!("The path is absolute; it must not be absolute, it must be relative")
    }

    // Make an instance of paths
    let paths = FilePath::new(target_path.to_path_buf());

    // Create paths if they dont exist
    let _first_backup = match fs::exists(&paths.main) {
        Ok(response) => {
            if !response {
                paths.create_dirs();
            };
            !response
        }
        Err(_) => {
            paths.create_dirs();
            true
        }
    };

    let passwd = "test_passwd".as_bytes();

    backup_full(&paths, passwd);
}

// Function to run a full backup.
fn backup_full(paths: &FilePath, passwd: &[u8]) {
    let tree = Tree::build(paths, passwd, true);

    // Write off the tree as a JSON
    // Turn the tree to JSON String format
    let json = serde_json::to_string_pretty(&tree).unwrap();

    // Prepare the path
    let path = paths.trees.join(format!("{}.json", tree.id));

    // Create the file
    fs::File::create(&path).unwrap();
    // Write it off
    fs::write(path, json).unwrap();

    // Make the commit which will point to the blob and tree.
    // If the work is not committed, it'll be some trash that may need to be cleaned later
    let commit = Commit::new(tree.id, None, "First commit");

    // Write off the commit as a JSON
    // Turn the tree to JSON String format
    let json = serde_json::to_string_pretty(&commit).unwrap();

    // Prepare the path
    let path = paths.commits.join(format!("{}.json", commit.id));

    // Write it off
    fs::write(path, json).unwrap();

    // Create the file for the HEAD
    fs::File::create(paths.main.join("HEAD")).unwrap();

    // Write off the commit's ID
    fs::write(paths.main.join("HEAD"), commit.id).unwrap();
}

fn _backup_diff(paths: &FilePath, passwd: &[u8], first_backup: bool) {
    match first_backup {
        true => {
            // Upon first backup, we run a full backup
            backup_full(paths, passwd)
        }
        false => {
            // We run a differential backup
            // Make the backup, this will prepare the tree
            let tree = Tree::build(&paths, passwd, false);

            // Write off the tree as a JSON
            // Turn the tree to JSON String format
            let json = serde_json::to_string_pretty(&tree).unwrap();

            // Prepare the path
            let path = paths.trees.join(format!("{}.json", tree.id));

            // Create the file
            fs::File::create(&path).unwrap();
            // Write it off
            fs::write(path, json).unwrap();

            // Read the latest commit's ID before reading its tree
            let latest_commit_id = fs::read_to_string(paths.main.join("HEAD")).unwrap();

            // Read the latest commit's content, and get the tree's ID
            let latest_commit_json =
                fs::read_to_string(paths.commits.join(format!("{}.json", latest_commit_id)))
                    .unwrap();

            // Convert it to Commit instance
            let latest_commit: Commit = serde_json::from_str(&latest_commit_json).ok().unwrap();

            // Get the tree's ID
            let latest_tree_id = latest_commit.tree_id;

            // Read the latest tree
            let latest_tree_json =
                fs::read_to_string(paths.trees.join(format!("{}.json", latest_tree_id))).unwrap();

            // Convert it to a Tree instance
            let latest_tree: Tree = serde_json::from_str(&latest_tree_json).unwrap();

            // Create the store
            let store = Store::new();

            // Populate the store with the new tree
            // Calculate changes compared to the latest commit
            let _changes = Tree::diff_trees(&tree, &latest_tree, &store);
        }
    };
}
