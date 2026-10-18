use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::sync::{mpsc};
use std::thread;
use std::time::{Duration, Instant};

mod hash;
use hash::sum_block_hash;
use hash::stretch_md5;


fn worker_read_file(
    tx: mpsc::SyncSender<Vec<u8>>,
    file: File,
    block_size: u64,
    num_blocks: u64,
    total_size: u64,
) -> Duration {
    let mut buffer = vec![0u8; block_size as usize];
    let mut read_time = Duration::ZERO;

    for block_idx in 0..num_blocks {
        let offset = block_idx * block_size;
        let bytes_to_read = std::cmp::min(block_size, total_size - offset) as usize;

        let target_buf = &mut buffer[..bytes_to_read];

        let read_start = Instant::now();
        file.read_exact_at(target_buf, offset).unwrap();
        read_time += read_start.elapsed();

        tx.send(target_buf.to_vec()).unwrap();
    }

    read_time
}

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
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(2);

    let handle = thread::spawn(move || {
        worker_read_file(tx, file, block_size, num_blocks, total_size)
    });

    let mut result = vec![0u8; 16];
    let mut compute_time = Duration::ZERO;
    for block in rx {
        let compute_start = Instant::now();
        let block_hash = stretch_md5(&block, k);
        sum_block_hash(&mut result, &block_hash);
        compute_time += compute_start.elapsed();
    }

    let read_time = handle.join().unwrap();

    let total = start.elapsed();
    eprintln!("total_s={:.6} read_s={:.6} compute_s={:.6}", total.as_secs_f64(), read_time.as_secs_f64(), compute_time.as_secs_f64());

    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!("");

}