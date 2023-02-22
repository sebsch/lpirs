use std::error::Error;

use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};

fn copy_file(from: &str, dest: &str) -> Result<(), Box<dyn Error>> {
    let fd_in = open(from, OFlag::O_RDONLY, Mode::empty())?;

    let fd_out = open(
        dest,
        OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
        Mode::S_IRUSR
            | Mode::S_IWUSR
            | Mode::S_IRGRP
            | Mode::S_IWGRP
            | Mode::S_IROTH
            | Mode::S_IWOTH,
    )?;

    let buf_size = 1024;
    let mut buf = vec![0u8; buf_size];

    loop {
        let num_read = read(fd_in, &mut buf[..])?;
        if num_read > 0 {
            write(fd_out, &buf[..num_read])?;
            continue;
        }
        break;
    }

    close(fd_in)?;
    close(fd_out)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, Rng};
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use super::*;

    fn initialize(from_path: &str) {
        let mut test_file = File::create(from_path).unwrap();
        let test_data: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(700000)
            .map(char::from)
            .collect();
        test_file.write_all(test_data.as_bytes()).unwrap();
    }

    #[test]
    fn test_copy_file_copies_content() {
        let from_path = Path::new("/tmp/somefile").display().to_string();
        let dest_path = Path::new("/tmp/somefile.bak").display().to_string();
        initialize(&from_path);

        copy_file(&from_path, &dest_path).unwrap();

        let mut from_content = String::new();
        let mut dest_content = String::new();

        File::open(&from_path)
            .unwrap()
            .read_to_string(&mut from_content)
            .unwrap();
        File::open(&dest_path)
            .unwrap()
            .read_to_string(&mut dest_content)
            .unwrap();

        assert_eq!(from_content, dest_content)
    }

    #[test]
    #[ignore]
    fn test_copy_file_creates_file_right_permissions() {
        let from_path = Path::new("/dev/shm/from_mode").display().to_string();
        let dest_path = Path::new("/dev/shm/dest_mode").display().to_string();
        initialize(&from_path);

        copy_file(&from_path, &dest_path).unwrap();

        let dest_file_perms = fs::metadata(&dest_path).unwrap().permissions().mode();
        println!("{dest_file_perms}");
        assert_eq!(format!("{dest_file_perms:o}"), "100666")
    }
}
