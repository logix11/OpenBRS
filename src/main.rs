use text_io; 
use std::{fs, os::unix::fs::FileTypeExt, path}; 
extern crate tar; 
use tar::Builder;
use chrono;
extern crate proc_mounts;
use proc_mounts::{MountIter};

fn main() {
	// Getting source
	let mut source: String = String::new();
	'source: loop {
		print!("Enter the path to the file OR the directory you want to backup (Postscriptum: it MUST be a relative path):: ");
		let path:String = text_io::read!("{}\n"); // reading until the first new line
		source.push_str(&path);

		// Check whether the path is valid or not.
		if path::Path::new(&source).is_file() || path::Path::new(&source).is_dir() {
			break 'source;
		}
		println!("Invalid path.");
		source.clear();
	}

	let mut destination:String = String::new();
	'destination: loop{
		print!("Enter the path to the directory in which you want to backup to (make sure it is a mounted block device) :: ");
		let path: String = text_io::read!(); 
		destination.push_str(&path);
		
		// Will parse the /proc/mounts file to ensure that the destination is a ]
		// valid mount point.
		for mount in MountIter::new().unwrap(){
			if let Ok(mount) = mount { // pattern matching and to get
				// the result only
				if path::Path::new(&destination) == mount.dest {
					break 'destination;
				}
			}
		}
		println!("Invalid path.");
		destination.clear();
	}

	// Archiving it. Create a timestamped name for the archive.
	let name = format!("./archive-{}.tar", chrono::prelude::Utc::now().to_string());

	// Create the archive file.
	let archive: fs::File = fs::File::create(name).unwrap();
	let mut _archive = Builder::new(archive);

	if path::Path::new(&source).is_file() {
		// If the path is a file, append it with the name "archive" and path 
		// "path"
		_archive.append_file("archive", &mut fs::File::open(source.clone()).unwrap()).unwrap();

	} else if path::Path::new(&source).is_dir() {
		// Otherwise, if the path is a directory append it with the name "archive"
		// and path "path"
		_archive.append_dir("archive", source.clone()).unwrap();

		// Then, we'll need to append its content, too.
		_archive.append_dir_all("archive", source.clone()).unwrap();

	} else {
		println!("Invalid path"); 
	}
}