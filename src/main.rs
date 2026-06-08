#![feature(portable_simd)]

use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::process;
use std::simd::u8x64;
use clap::Parser;

#[derive(Parser)]
struct Args {
    base_file: String,

    target_file: String,

    #[arg(short = 's', long = "silent")]
    silent: bool,
}

fn main() {
    let args = Args::parse();

    if args.base_file == args.target_file {
        if !args.silent {
            println!("Same file");
        }
        process::exit(0);
    }

    match compare_file(&args.base_file, &args.target_file, args.silent) {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn compare_file(base_file_path: &str, target_file_path: &str, silent: bool) -> io::Result<()> {
    let mut base_file = File::open(base_file_path)?;
    let mut target_file = File::open(target_file_path)?;

    const BUFFER_SIZE: usize = 4096;
    let mut base_file_buffer = [0u8; BUFFER_SIZE];
    let mut target_file_buffer = [0u8; BUFFER_SIZE];

    let mut offset = 0u64;

    loop {
        let base_file_bytes = read_chunk(&mut base_file, &mut base_file_buffer)?;
        let target_file_bytes = read_chunk(&mut target_file, &mut target_file_buffer)?;

        if base_file_bytes == 0 && target_file_bytes == 0 {
            return Ok(());
        }

        if base_file_bytes != target_file_bytes {
            if !silent {
                println!("cmp: EOF on {}", if base_file_bytes < target_file_bytes {base_file_path} else {target_file_path});
            }
            return Err(io::Error::new(ErrorKind::Other, "Files have different sizes"));
        }

        match compare_buffers_simd(&base_file_buffer[..base_file_bytes], &target_file_buffer[..target_file_bytes]) {
            Some(pos) => {
                if !silent {
                    let byte_pos = offset + pos as u64;
                    println!("{} {} {} differ: {:o} {:o}", 
                        base_file_path, target_file_path, byte_pos + 1, base_file_buffer[pos], target_file_buffer[pos]);
                }
                return Err(io::Error::new(ErrorKind::Other, "Files differ"));
            }
            None => {
                offset += base_file_bytes as u64;
            }
        }
    }
}

fn read_chunk(file: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut total_read = 0;

    while total_read < buffer.len() {
        match file.read(&mut buffer[total_read..]) {
            Ok(0) => break,
            Ok(n) => total_read += n,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(total_read)
}

fn compare_buffers_simd(buffer1: &[u8], buffer2: &[u8]) -> Option<usize> {
    let len = buffer1.len();
    let mut index = 0;

    while index + 64 <= len {
        let v1 = u8x64::from_slice(&buffer1[index..index+64]);
        let v2 = u8x64::from_slice(&buffer2[index..index+64]);

        let mask = v1 != v2;

        if mask {
            for offset in 0..64 {
                if v1[offset] != v2[offset] {
                    return Some(index + offset);
                }
            }
        }

        index += 64;
    }
    
    for offset in index..len {
        if buffer1[offset] != buffer2[offset] {
            return Some(offset);
        }
    }

    None
}