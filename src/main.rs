use blake3::{Hash, Hasher};
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

fn main() -> Result<(), Box<dyn Error>> {
    let mut hasher = Hasher::new();
    let mut results: HashMap<Hash, Vec<PathBuf>> = HashMap::new();
    for file in fs::read_dir("./test-files")? {
        let path = file?.path();
        let file = fs::File::open(&path)?;
        hasher.reset();
        copy_wide(file, &mut hasher)?;
        let file_hash = hasher.finalize();
        let counter = results.entry(file_hash).or_insert(Vec::new());
        counter.push(path);
    }

    for (_, files) in &results {
        if files.len() > 1 {
            println!("Those files are duplicates: {:?}", files);
        }
    }
    Ok(())
}

//A 16 KiB buffer is enough to take advantage of all the SIMD instruction sets
// that we support, but `std::io::copy` currently uses 8 KiB. Most platforms
// can support at least 64 KiB, and there's some performance benefit to using
// bigger reads, so that's what we use here.
fn copy_wide(mut reader: impl Read, hasher: &mut blake3::Hasher) -> io::Result<u64> {
    let mut buffer = [0; 65536];
    let mut total = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(n) => {
                hasher.update(&buffer[..n]);
                total += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}
