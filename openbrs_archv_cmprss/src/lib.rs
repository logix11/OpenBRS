use std::{
    fs::{self, File},
    path::{self, PathBuf},
};
use tar::Builder;
use xz::write::XzEncoder;

pub fn archive_compress(target_path: &PathBuf, blobs: &PathBuf) {
    // Set the path to archive:
    let archive_name = format!(
        "{}.tar.xz",
        target_path.file_name().unwrap().to_str().unwrap()
    );
    let archive = blobs.join(format!("{archive_name}"));

    // Create the file before turning it to an archive
    let archive_file = File::create(archive).unwrap();

    // create an XzEncoder that wraps the file (this implements Write)
    // Thus, we can compress on the fly
    let encoder = XzEncoder::new(archive_file, 9); // 0..9 compression level

    // Build the archive to stream INTO the encoder to compress it directly
    let mut archive = Builder::new(encoder);

    // add a file to the archive
    if target_path.is_dir() {
        // First, register the target directory, then archive its content
        let dir_name = target_path.file_name().unwrap();
        archive.append_dir(dir_name, target_path).unwrap();
        append_dir_all_excluding(&mut archive, target_path, target_path);
    } else {
        archive.append_path(target_path).unwrap();
    }

    // finish the tar stream
    archive.finish().unwrap();

    // Unwrap this archive, returning the underlying object which is the compressed data.
    let encoder = archive.into_inner().unwrap();

    // finish compression and get the inner File back
    let file = encoder.finish().unwrap();

    // ensure data is flushed to disk
    file.sync_all().unwrap();
}

// I use this function to exclude the .openbrs workspace:
// Signature explication:
//  * builder is the Tar builder;
//  * base is the root file system we're backing up;
//  * path is the current directory we're visiting during recursion, it starts equal to base.
fn append_dir_all_excluding(
    builder: &mut Builder<XzEncoder<File>>,
    base: &PathBuf,
    path: &PathBuf,
) {
    // What we want to exclude
    let exclude = ".openbrs";

    // Is it a directory, or a file?
    if path.is_dir() {
        // iterate through entries and get the the entry's name
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Is the target our workspace or does it contain it?
            if exclude.contains(&file_name_str.as_ref()) {
                continue;
            }

            // Get the full and relative paths
            let entry_path = entry.path();
            let rel_path = path::Path::new(base.file_name().unwrap())
                .join(entry_path.strip_prefix(base).unwrap());

            // is it a folder? Then recurse deeper; otherwise, simply archive it
            if entry_path.is_dir() {
                builder.append_dir(rel_path, &entry_path).unwrap();
                append_dir_all_excluding(builder, base, &entry_path);
            } else {
                builder
                    .append_path_with_name(&entry_path, rel_path)
                    .unwrap();
            }
        }
    }
}
