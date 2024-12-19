use text_io; 
use std::{fs::File, path::{self, Path}}; 
use chrono::prelude::Utc;
extern crate tar; 
use tar::Builder;
use chrono;
extern crate proc_mounts;
use proc_mounts::MountIter;

fn main() {
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
/*
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
	}*/
}