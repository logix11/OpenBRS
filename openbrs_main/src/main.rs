use openbrs_versioning::{Commit, FilePath, Tree};
use serde_json;
use std::{fs, path::Path};

fn main() {
    // Get path
    let target_path = Path::new("test/TOAD.png");

    // Ensure that the path is not absolute.
    if target_path.is_absolute() {
        panic!("The path is absolute; it must not be absolute, it must be relative")
    }

    // Make and instante of paths
    let paths = FilePath::new(target_path.to_path_buf());

    //Create paths if they dont exist
    let first_backup = match fs::exists(&paths.main) {
        Ok(_) => false,
        Err(_) => {
            FilePath::create_dirs(&paths);
            true
        }
    };

    let passwd = "test_passwd".as_bytes();

    backup_full(&paths, passwd);
}

// Function to run a full backup.
fn backup_full(paths: &FilePath, passwd: &[u8]) {
    // Make the backup, this will create the blob, and prepare the tree
    let tree = Tree::build(&paths, passwd, true);

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
}

fn backup_diff(paths: &FilePath, passwd: &[u8], first_backup: bool) {
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

            // We won't commit, because we'll need to check the tree, decide which files to backup, then backup them off
            // and finally commit
        }
    };
}
