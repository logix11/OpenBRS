use openbrs_backup::{backup_diff, backup_full};
use openbrs_main_structs::FilePath;
use std::{fs, path::Path};
fn main() {
    // Get path
    let target_path = Path::new("test");

    // Ensure that the path is not absolute.
    if target_path.is_absolute() {
        panic!("The path is absolute; it must not be absolute, it must be relative")
    }

    // Make an instance of paths
    let paths = FilePath::new(&target_path.to_path_buf());

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

    let _passwd = "test_passwd".as_bytes();

    //backup_full(&paths);
    backup_diff(&paths, false);
}
