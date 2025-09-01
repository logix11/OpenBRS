//use chrono::prelude::Utc;
use std::fs::{self, File};
use std::io::BufReader;
use std::io::prelude::*;
use std::panic;
use xz::read::XzEncoder;
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
    let archive_file = File::create(archive_path);
    // To replace the unwrap... For now, I just want it to panic
    let archive_file = match archive_file {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file :: {error}"),
    };
    // Build the archive
    let mut archive = Builder::new(archive_file);
    //
    // add a file to the archive
    archive.append_path(path)


    // Read the file before encoding it
    let file_to_encode = fs::File::open(archive_path).unwrap();
    // Buffer it to gain te Read trait
    let buffer = BufReader::new(file_to_encode);
    // Encode it
    let mut encoder = XzEncoder::new(buffer, 9);
    // prepare the file to write to
    let mut compressed_file = File::create("{archive_file}.zx").unwrap();
    // prepare the buffer to read the compressed data
    let mut buffer = Vec::new();
    // Push the comressed data to the buffer
    encoder.read_to_end(&mut buffer).unwrap(); // panic if it fails
    // Turn the buffer to a &[u8]
    let buffer: &[u8] = &buffer;
    compressed_file.write_all(buffer).unwrap(); // panic if it fails
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
