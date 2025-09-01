/*use chrono::prelude::Utc;
//use fileops::archive;
use proc_mounts::MountIter;
use std::{
    fs::File,
    path::{self, Path},
};
use tar::Builder;

#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn take_input() {
        let query = "test";

    }*/
}

mod fileops {
    use std::fs::File; use crate::path; use crate::Path; use crate::MountIter;
    use crate::Builder; use crate::Utc;

    pub fn find_source(path: &String) -> u8 {
        /*
        This function will search for the given path.
        If it finds it, and it is a file, it return 1. Otherwise, if is it a
        directory, it returns 2 (in binary.) Otherwise, it returns a 0.
        */
        if Path::new(path).is_file() {
            return 0b1
        } else if Path::new(path).is_dir() {
            return 0b10
        } else {
            return 0b0
        }
    }

    pub fn find_destination(path: &String) -> bool {
        for mount in MountIter::new().unwrap(){
            // pattern matching and to get the Result only, without the Error.
            if let Ok(mount) = mount {
                if Path::new(&path) == mount.dest {
                    return true
                }
            }
        }
        return false;
    }

    pub fn archive(file: String, source: String){
        // First, we create the file
        let file = File::create(file).unwrap();

        // Second, we make it an archive
        let mut file: Builder<File> = Builder::new(file);

        // Then, if the source is a file, we append it to the archive. The first
        // argument is the path to the archive. The second argument is the path
        // to the file.
        if path::Path::new(&source).is_file() {
            file.append_file(
                    "archive",
                    &mut File::open(source.clone()).unwrap()
                ).unwrap();

        // If the source is a directory, then we append the directory, then append
        // its conent.
        } else if Path::new(&source).is_dir() {
            file.append_dir(
                    "archive",
                    source.clone()
                ).unwrap();

            file.append_dir_all(
                    "archive",
                    source.clone()
                ).unwrap();

        }
    }
}

pub fn api(){
    use crate::fileops;
    let mut source: String = String::new();
    loop {

        // Reading source
        print!("Enter the path to the file OR the directory you want to backup (Postscriptum: it MUST be a relative path):: ");

        // Storing the read value in a buffer
        let path:String = text_io::read!("{}\n");

        // Check whether the path is valid or not.
        let ftype = fileops::find_source(&path);
        match ftype{
            0b1 => {
                println!("File is found.");
                source.push_str(&path);
                break;
            },
            0b10 => {
                println!("Directory is found.");
                source.push_str(&path);
                break},
            0b0 => println!("Invalid path."),
            _ => {}
        }
    }
    let mut destination:String = String::new();

    loop{
        print!("Enter the path to the directory in which you want to backup to (make sure it is a mounted block device) :: ");
        let path: String = text_io::read!("{}\n");

        // Parse the /proc/mounts file to ensure that the destination is a valid
        // mount point.
        match fileops::find_destination(&path){
            true => {
                println!("Destination is set.");
                destination.push_str(&path);
                break;
            },
            false => {
                println!("Invalid path--it either does not exist or is not a
                mount point.");
                destination.clear();
            }
        }
    }
    // Archiving it. Create a timestamped name for the archive. The name will be
    // "<destination>/archive-<timestamp>.tar"
    let name = format!(
        "{}/archive-{}.tar",
        destination,
        Utc::now().to_string()
    );
    archive(name, source);

}
 */
