//use chrono::prelude::Utc;
use std::fs::{self, File};
use std::io::BufReader;
use std::io::prelude::*;
use std::{panic, string};
use xz::write::XzEncoder;
//use text_io;
//use chrono;
use tar::Builder;
// Need

//extern crate proc_mounts;
//use proc_mounts::MountIter;

fn main() {
    // CREATE A NEW ARCHIVE
    // Set the path
    let archive_path = "test/backup.tar.xz";

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

    /*
    // Getting source. First, initialize the `source` to keep it in scope.
    let mut source: String = String::new();
    'source: loop {
        // Reading source
        print!("Enter the path to the file OR the directory you want to backup (Postscriptum: it MUST be a relative path):: ");

        // Storing the read value in a buffer
        let path:String = text_io::read!("{}\n");

        // pushing the buffer's value to the `source`
        source.push_str(&path);

        // Check whether the path is valid or not.
        if Path::new(&source).is_file() || Path::new(&source).is_dir() {
            break 'source;
        }
        println!("Invalid path.");
        source.clear();
    }

    let mut destination:String = String::new();
    'destination: loop{
        print!("Enter the path to the directory in which you want to backup to (make sure it is a mounted block device) :: ");
        let path: String = text_io::read!("{}\n");
        destination.push_str(&path);

        // Parse the /proc/mounts file to ensure that the destination is a valid
        // mount point.
        for mount in MountIter::new().unwrap(){
            // pattern matching and to get the Result only, without the Error.
            if let Ok(mount) = mount {
                if Path::new(&destination) == mount.dest {
                    break 'destination;
                }
            }
        }
        println!("Invalid path.");
        destination.clear();
    }

    // Archiving it. Create a timestamped name for the archive.
    // The name will be "<destination>/archive-<timestamp>.tar"
    let name = format!(
        "{}/archive-{}.tar",
        destination,
        Utc::now().to_string()
    );

    // Create the archive file then building it as an archive.
    let archive = File::create(name).unwrap();
    let mut _archive: Builder<File> = Builder::new(archive);

    if path::Path::new(&source).is_file() {
        // If the path is a file, append it with the name "archive" and path "path"
        _archive.append_file(
                "archive",
                &mut File::open(source.clone())
                .unwrap()
            ).unwrap();

    } else if Path::new(&source).is_dir() {
        // Otherwise, if the path is a directory append it with the name "archive"
        // and path "path"
        _archive.append_dir(
                "archive",
                source.clone()
            ).unwrap();

        // Then, we'll need to append its content, too.
        _archive.append_dir_all(
                "archive",
                source.clone()
            ).unwrap();

    } else {
        println!("Invalid path");
    }
     */
}
