use openbrs_archv_cmprss::{self, archive_compress};
use openbrs_crypto::encrypt_archive;
//use openbrs_crypto;

fn main() {
    // Archive and compress
    let path = "test/backup.xz.tar";
    archive_compress(&path);
    encrypt_archive(&path, "password".as_ref());

    // Encrypt
}
