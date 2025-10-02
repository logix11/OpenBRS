use std::{fs::File, path::PathBuf};
use tar::Builder;
use xz::write::XzEncoder;

pub fn archive_compress_file(target_path: &PathBuf, blob_file: &PathBuf) {
    // Create the file before turning it to an archive
    let archive_file = File::create(blob_file).unwrap();

    // create an XzEncoder that wraps the file (this implements Write)
    // Thus, we can compress on the fly
    let encoder = XzEncoder::new(archive_file, 9); // 0..9 compression level

    // Build the archive to stream INTO the encoder to compress it directly
    let mut archive = Builder::new(encoder);

    // add a file to the archive
    archive.append_path(target_path).unwrap();

    // finish the tar stream
    archive.finish().unwrap();

    // Unwrap this archive, returning the underlying object which is the compressed data.
    let encoder = archive.into_inner().unwrap();

    // finish compression and get the inner File back
    let file = encoder.finish().unwrap();

    // ensure data is flushed to disk
    file.sync_all().unwrap();
}

pub fn archive_compress_dir(target_path: &PathBuf, blob_file: &PathBuf) {
    // Create the file before turning it to an archive
    let archive_file = File::create(&blob_file).unwrap();

    // create an XzEncoder that wraps the file (this implements Write)
    // Thus, we can compress on the fly
    let encoder = XzEncoder::new(archive_file, 9); // 0..9 compression level

    // Build the archive to stream INTO the encoder to compress it directly
    let mut archive = Builder::new(encoder);

    // add a file to the archive
    archive
        .append_dir_all(target_path.file_name().unwrap(), target_path)
        .unwrap();

    // finish the tar stream
    archive.finish().unwrap();

    // Unwrap this archive, returning the underlying object which is the compressed data.
    let encoder = archive.into_inner().unwrap();

    // finish compression and get the inner File back
    let file = encoder.finish().unwrap();

    // ensure data is flushed to disk
    file.sync_all().unwrap();
}
