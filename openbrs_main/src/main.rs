use openbrs_versioning::{FilePath, backup_full};
use std::path::Path;

fn main() {
    // Get path
    let target_path = Path::new("test/TOAD.png");

    // Ensure that the path is not absolute.
    if target_path.is_absolute() {
        panic!("The path is absolute; it must not be absolute, it must be relative")
    }

    // Make and instante of paths
    let paths = FilePath::new(target_path.to_path_buf());

    //Create paths
    FilePath::create_dirs(&paths);

    let passwd = "test_passwd".as_bytes();

    backup_full(&paths, passwd);
}
