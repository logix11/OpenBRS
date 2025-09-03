use std::fs::File;
use tar::Builder;
use xz::write::XzEncoder;

pub fn archive_compress(archive_path: &str) {
    // ARCHIVING AND COMRPESSING

    // Create the file before turning it to an archive
    let archive_file = File::create(archive_path).unwrap();

    // create an XzEncoder that wraps the file (this implements Write)
    // This means we can compress on the fly
    let encoder = XzEncoder::new(archive_file, 9); // 0..9 compression level

    // Build the archive to stream INTO the encoder to compress it directly
    let mut archive = Builder::new(encoder);

    // add a file to the archive
    let path = "test/TOAD.png";
    archive.append_path(path).unwrap();

    // finish the tar stream
    archive.finish().unwrap();

    // Unwrap this archive, returning the underlying object which is the compressed data.
    let encoder = archive.into_inner().unwrap();

    // finish compression and get the inner File back
    let file = encoder.finish().unwrap();

    // ensure data is flushed to disk
    file.sync_all().unwrap();
}
