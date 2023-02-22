use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{close, lseek, read, write, Whence};
use std::error::Error;

pub enum PrintMode {
    Unicode,
    Hex,
}

pub enum SeekMode {
    Write(String),
    Read(usize, PrintMode),
    Seek(i64),
}

pub fn seek_io(file_path: &str, seek_modes: Vec<SeekMode>) -> Result<(), Box<dyn Error>> {
    let fd = open(file_path, OFlag::O_CREAT | OFlag::O_RDWR, Mode::all())?;

    for seek_mode in seek_modes {
        match seek_mode {
            SeekMode::Write(word) => {
                let num_written = write(fd, word.as_bytes())?;
                println!("wrote {num_written} bytes");
            }
            SeekMode::Read(bytes, print_mode) => {
                let mut buf = vec![0u8; bytes];
                match read(fd, &mut buf[..])? {
                    0 => {
                        println!("EOF")
                    }
                    _ => match print_mode {
                        PrintMode::Unicode => {
                            let text = std::str::from_utf8(&buf[..]).unwrap();
                            println!("{text}");
                        }
                        PrintMode::Hex => println!("{buf:#04X?}"),
                    },
                };
            }
            SeekMode::Seek(offset) => {
                lseek(fd, offset, Whence::SeekSet)?;
                println!("seek to {offset} succeeded.")
            }
        }
    }

    close(fd)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::chapter_4_fileio::seek_io::{seek_io, PrintMode, SeekMode};
    use std::fs::File;

    #[test]
    fn create_huge_file_with_big_hole() {
        let file_path = "/dev/shm/file_with_hole";

        File::create(file_path).unwrap();

        seek_io(
            file_path,
            vec![
                SeekMode::Seek(1000000000000000),
                SeekMode::Write("abc".to_string()),
            ],
        )
        .unwrap();

        let metadata = std::fs::metadata(file_path).unwrap();

        // file has size of 1PB
        // exa -ll /dev/shm/file_with_hole
        //    .rw-r--r-- 1,0P sebastian 12 Feb 00:10 /dev/shm/file_with_hole
        assert_eq!(metadata.len(), 1000000000000003);

        seek_io(
            file_path,
            vec![SeekMode::Seek(100000), SeekMode::Read(5, PrintMode::Hex)],
        )
        .unwrap();

        seek_io(
            file_path,
            vec![
                SeekMode::Seek(1000000000000000),
                SeekMode::Read(10, PrintMode::Unicode),
            ],
        )
        .unwrap();
    }
}
