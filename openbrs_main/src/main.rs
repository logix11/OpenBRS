use openbrs_versioning::backup_full;
use std::fs::metadata;
use std::path::Path;

// TODO: FINISH ADAPTING THE CODE TO THE NEW STRUCTURE OF THE BACKUP FOLDER

enum FilePath {
    TargetPath,
    MainDir,
    ObjectsPath,
    BlobsPath,
    TreesPath,
    CommitPath,
    ArchivePath,
    EncrArchPath,
}

impl FilePath {
    fn as_path(&self, target_path: &Path) -> PathBuf {
        // Create the main_dir for later
        let main_dir = if metadata(target_path).unwrap().is_dir() {
            target_path.to_path_buf().join(".openbrs").unwrap()
        } else {
            target_path
                .parent()
                .unwrap()
                .to_path_buf()
                .join(".openbrs")
                .unwrap()
        };

        // Create the archive name for later
        let archive_name = format!(
            "{}.tar.xz",
            target_path.file_name().unwrap().to_str().unwrap()
        );

        match self {
            FilePath::TargetPath => target_path,
            FilePath::MainDir => main_dir,
            FilePath::ObjectsPath => main_dir.join("objects").unwrap(),
            FilePath::BlobsPath => main_dir.join("objects/blobs").unwrap(),
            FilePath::TreesPath => main_dir.join("objects/trees").unwrap(),
            FilePath::CommitPath => main_dir.join("objects/commits").unwrap(),
            FilePath::ArchivePath => main_dir.join(format!("objects/blobs/{archive_name};")),
            FilePath::EncrArchPath => main_dir.join(format!("objects/blobs/{archive_name}.enc")),
        }
    }
}

fn main() {
    // Get path
    let target_path = Path::new("test/TOAD.png");

    // Ensure that the path is not absolute.
    if target_path.is_absolute() {
        panic!("The path is absolute; it must not be absolute, it must be relative")
    }
    let paths = FilePath::new(target_path);

    backup_full(&target_path, &archive_path, &encr_archive_path, passwd);
}
