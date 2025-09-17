use std::{fs::File, path::Path};
use tar::Builder;
use xz::write::XzEncoder;

pub fn archive_compress(target_path: &Path, archive_path: &Path) {
    // ARCHIVING AND COMRPESSING

    // Create the file before turning it to an archive
    let archive_file = File::create(archive_path).unwrap();

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
