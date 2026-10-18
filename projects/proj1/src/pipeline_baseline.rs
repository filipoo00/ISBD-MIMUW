use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::time::{Duration, Instant};

mod hash;
use hash::sum_block_hash;
use hash::stretch_md5;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage: pipeline <filepath> <block_size> <k>");
        return;
    }

    let block_size: u64 = args[2].parse().expect("Invalid block size");
    let k: u64 = args[3].parse().expect("Invalid number K");

    assert!(block_size > 0, "Block size must be positive.");
    assert!(k > 0, "Number K must be positive.");

    let start = Instant::now();
    let mut read_time = Duration::ZERO;
    let mut compute_time = Duration::ZERO;

    let file_res = File::open(args[1].clone());
    if file_res.is_err() {
        eprintln!("File not found: {}", args[1]);
        return;
    }
    let file = file_res.unwrap();

    let metadata = file.metadata();
    if metadata.is_err() {
        eprintln!("Error opening file: {}", args[1]);
        return;
    }
    let total_size = metadata.unwrap().len();
    let num_blocks = (total_size + block_size - 1) / block_size;

    let mut buffer = vec![0u8; block_size as usize];
    let mut result = vec![0u8; 16];
    for block_idx in 0..num_blocks
    {
        let offset = block_idx * block_size;
        let bytes_to_read = std::cmp::min(block_size, total_size - offset) as usize;

        // read
        let target_buf = &mut buffer[..bytes_to_read];
        
        let read_start = Instant::now();
        let err = file.read_exact_at(target_buf, offset);
        if err.is_err() {
            eprintln!("Error reading file: {}", err.unwrap_err());
            return;
        }
        read_time += read_start.elapsed();

        // compute
        let compute_start = Instant::now();
        let block_hash = stretch_md5(&target_buf, k);
        sum_block_hash(&mut result, &block_hash);
        compute_time += compute_start.elapsed();
    }

    let total = start.elapsed();
    eprintln!("total_s={:.6} read_s={:.6} compute_s={:.6}", total.as_secs_f64(), read_time.as_secs_f64(), compute_time.as_secs_f64());

    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!("");

}