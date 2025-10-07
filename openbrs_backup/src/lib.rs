use openbrs_main_structs::{Commit, FilePath, Tree};
use serde_json;
use std::fs;

// Function to run a full backup.
pub fn backup_full(paths: &FilePath) {
    let tree = Tree::build(paths, true);

    // Write off the tree as a JSON
    tree.write_tree(&paths);

    // Make the commit which will point to the blob and tree.
    // If the work is not committed, it'll be some trash that may need to be cleaned later
    let commit = Commit::new(tree.id, None, String::from("First commit"));

    // Write off the commit as a JSON
    commit.write(paths);

    // Create the file for the HEAD
    fs::File::create(paths.main.join("HEAD")).unwrap();

    // Write off the commit's ID
    fs::write(paths.main.join("HEAD"), commit.id).unwrap();
}

pub fn _backup_diff(paths: &FilePath, first_backup: bool) {
    match first_backup {
        true => {
            // Upon first backup, we run a full backup
            backup_full(paths)
        }
        false => {
            // We run a differential backup
            // Make the backup, this will prepare the tree
            let tree = Tree::build(&paths, false);

            // Write off the tree as a JSON
            tree.write_tree(paths);

            // Read the latest commit's ID before reading its tree
            let most_recent_commit = fs::read_to_string(paths.main.join("HEAD")).unwrap();

            // Read the latest commit's content, and get the tree's ID
            let latest_commit_json =
                fs::read_to_string(paths.commits.join(format!("{}.json", most_recent_commit)))
                    .unwrap();

            // Convert it to Commit instance
            let latest_commit: Commit = serde_json::from_str(&latest_commit_json).ok().unwrap();

            // Get the tree's ID
            let latest_tree_id = latest_commit.tree_id;

            // Read the latest tree
            let latest_tree_json =
                fs::read_to_string(paths.trees.join(format!("{}.json", latest_tree_id))).unwrap();

            // Convert it to a Tree instance
            let _latest_tree: Tree = serde_json::from_str(&latest_tree_json).unwrap();
        }
    };
}
