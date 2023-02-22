use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::unistd::{close, lseek, read, write, Whence};
use std::error::Error;

use lpirs::chapter_4_fileio::seek_io;
use std::fs::File;
use std::io::Read;

fn generate_testdata(path: &str) -> Result<(), Box<dyn Error>> {
    File::create(path)?;
    seek_io::seek_io(
        path,
        vec![
            seek_io::SeekMode::Write("abcde".to_string()),
            seek_io::SeekMode::Seek(10),
            seek_io::SeekMode::Write("z".to_string()),
        ],
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let in_file_path = "./file_with_hole";
    let out_file_path = "./file_with_hole_out";

    generate_testdata(in_file_path)?;

    let mut buf = vec![0u8; 1024];

    let fd_in = open(in_file_path, OFlag::O_RDWR, Mode::all())?;
    let fd_out = open(out_file_path, OFlag::O_CREAT | OFlag::O_WRONLY, Mode::all())?;

    let mut offset = 0;

    'outer: loop {
        let start_hole = lseek(fd_in, offset, Whence::SeekHole)?;

        match lseek(fd_in, start_hole, Whence::SeekData) {
            Ok(end_hole) => {
                lseek(fd_in, offset, Whence::SeekSet)?;

                loop {
                    let num_read = read(fd_in, &mut buf[..])?;
                    if num_read == 0 {
                        break 'outer;
                    }

                    if offset + num_read as i64 == start_hole {
                        write(fd_out, &buf[..(start_hole - offset) as usize])?;

                        offset = lseek(fd_out, end_hole, Whence::SeekSet)?;
                        break;
                    }

                    write(fd_out, &buf[..])?;
                    offset = offset + buf.len() as i64;
                }
            }
            Err(err) => {
                eprintln!("{err}");
                break;
            }
        }
    }

    close(fd_in)?;
    close(fd_out)?;

    let mut buf_in = Vec::new();
    let mut buf_out = Vec::new();

    File::open(in_file_path)?.read_to_end(&mut buf_in)?;
    File::open(out_file_path)?.read_to_end(&mut buf_out)?;

    assert_eq!(buf_in, buf_out);

    return Ok(());
}
