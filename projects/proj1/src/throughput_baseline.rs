use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::time::{Duration, Instant};

mod hash;
use hash::sum_block_hash;


fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: throughput <filepath> <block_size>");
        return;
    }

    let block_size = args[2].parse().expect("Invalid block size");
    assert!(block_size > 0, "Block size must be positive.");

    let start = Instant::now();

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
    let mut read_time = Duration::ZERO;
    for block_idx in 0..num_blocks
    {
        let offset = block_idx * block_size;
        let bytes_to_read = std::cmp::min(block_size, total_size - offset) as usize;

        // read
        let target_buf = &mut buffer[..bytes_to_read];
        let read_start = Instant::now();
        let err = file.read_exact_at(target_buf, offset);
        read_time += read_start.elapsed();
        if err.is_err() {
            eprintln!("Error reading file: {}", err.unwrap_err());
            return;
        }

        // compute
        let block_hash = md5::compute(target_buf);
        sum_block_hash(&mut result, &block_hash);
    }

    let total = start.elapsed();
    eprintln!("total_s={:.6} read_s={:.6}", total.as_secs_f64(), read_time.as_secs_f64());

    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!("");

}