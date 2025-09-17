use openbrs_versioning::backup;
use std::fs::metadata;
use std::path::{Path, PathBuf};

// TODO: FINISH ADAPTING THE CODE TO THE NEW STRUCTURE OF THE BACKUP FOLDER

fn main() {
    // Get path
    let target_path = Path::new("test/TOAD.png");

    // Ensure that the path is not absolute.
    if target_path.is_absolute() {
        panic!("The path is absolute; it must not be absolute, it must be relative")
    }

    // Is it a file, or a directory?
    let (main_dir, is_dir) = if metadata(target_path).unwrap().is_dir() {
        (target_path.to_path_buf().join(".openbrs"), 0b1)
    } else {
        (
            target_path.parent().unwrap().to_path_buf().join(".openbrs"),
            0b0,
        )
    };

    // Create the main directory
    std::fs::create_dir(&main_dir).unwrap();

    // Create the objects directory, that will hold the blobs
    let objects_path = main_dir.join("objects");
    std::fs::create_dir(&objects_path).unwrap();

    // Name the archive after the file's name, and add .xz.tar
    let file_name = target_path.file_name().unwrap().to_str().unwrap();
    let archive_name = format!("{}.tar.xz", file_name);
    let archive_path = objects_path.join(file_name);

    // Name the encrypted archive after the archive's name, and add .enc
    let encr_archive_name = format!("{}.enc", archive_name);
    let encr_archive_path = archive_path.join(encr_archive_name);
    let passwd = "test passwd".as_bytes();
    let backup_type = 0b1;
    backup(
        backup_type,
        is_dir,
        &target_path,
        &archive_path,
        &encr_archive_path,
        passwd,
    );
}
