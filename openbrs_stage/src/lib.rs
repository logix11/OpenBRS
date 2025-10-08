use openbrs_archv_cmprss::archive_compress;
use openbrs_main_structs::{Change, ChangeType, FilePath};
pub fn stage(changes: Vec<Change>, paths: FilePath) {
    // Parse changes
    for change in changes {
        // Match changes, to stage what was added and what was modified only.
        match change.change_type {
            ChangeType::Added | ChangeType::Modified => {
                archive_compress(&paths.target, &paths.archive);
            }
            ChangeType::Removed => {}
        }
    }
}
