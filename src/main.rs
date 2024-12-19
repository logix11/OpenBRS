use text_io; 
use std::fs::{self, Metadata}; 
extern crate tar; 
use tar::Builder;
fn main() {
	let _path:String = String::new();
    print!("Hello, enter the path to the file OR the directory you want to backup (Postscriptum: it MUST be a relative path):: ");
	let path:String = text_io::read!(); // reading until the first white space

	// Verifying whether the path is valid or invalid, then archiving it.
	// First, create the archive file.
	let archive: fs::File = fs::File::create("./archive.tar").unwrap();
	
	// Second, create the archive
	let mut _archive = Builder::new(archive);

	// Third, verifying whether the given path is a file or a directory.
	let path_verify: Metadata = fs::metadata(path.clone()).unwrap();
	if path_verify.is_file(){
		// If the path is valid, and is a file, append it with the name 
		// "archive" and path "path"
		_archive.append_file("archive", &mut fs::File::open(path.clone()).unwrap()).unwrap();
	} else if path_verify.is_dir(){
		// Otherwise, if the path is valid is valid, and is a directory,
		// append it with the name "archive" and path "path"
		_archive.append_dir("archive", path.clone()).unwrap();

		// Then, we'll need to append its content, too.
		_archive.append_dir_all("archive", path.clone()).unwrap();
	} else {
		println!("Invalid path"); 
	}
}
