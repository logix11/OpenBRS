use openbrs_archv_cmprss::archive_compress;
use openbrs_main_structs::{Change, ChangeType, FilePath};
use std::env;
pub fn stage(changes: Vec<Change>, paths: &FilePath) {
    // Parse changes
    for change in changes {
        println!("{}", change.name);
        if change.name == ".openbrs" {
            continue;
        }
        // Match changes, to stage what was added and what was modified only.
        match change.change_type {
            ChangeType::Added | ChangeType::Modified => {
                // One issue: I have the target's absolute path, not relative path.
                // Solution: get the relative path according to the current working directory.
                // CWD = Current Working Directory
                let cwd = env::current_dir().unwrap();
                let target_relative_path = change.path.strip_prefix(cwd).unwrap();
                archive_compress(&target_relative_path.to_path_buf(), &paths.blobs);
            }
            ChangeType::Removed => {}
        }
    }
}
