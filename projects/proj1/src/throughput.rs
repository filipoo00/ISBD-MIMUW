use std::env;
use std::fs::File;
use std::time::Instant;
use memmap2::MmapOptions;

mod hash;
use hash::sum_block_hash;


fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: throughput <filepath> <block_size>");
        return;
    }

    let block_size: usize = args[2].parse().expect("Invalid block size");
    assert!(block_size > 0, "Block size must be positive.");

    let start = Instant::now();

    let file_res = File::open(args[1].clone());
    if file_res.is_err() {
        eprintln!("File not found: {}", args[1]);
        return;
    }
    let file = file_res.unwrap();
    let total_size = file.metadata().unwrap().len();

    let mut result = vec![0u8; 16];
    if total_size > 0 {
        let mmap = unsafe {
            MmapOptions::new().map(&file).unwrap()
        };
        for block in mmap.chunks(block_size)
        {
            let block_hash = md5::compute(block);
            sum_block_hash(&mut result, &block_hash);
        }
    }

    let total = start.elapsed();
    eprintln!("total_s={:.6}", total.as_secs_f64());

    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!("");
}