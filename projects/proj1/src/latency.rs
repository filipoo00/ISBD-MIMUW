use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::time::{Duration, Instant};
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use rand::{rng, Rng, RngExt, SeedableRng};
use rand::rngs::StdRng;

mod hash;
use hash::sum_block_hash;

pub struct RandomOrderIterator {
    elements: Vec<u64>,
    rng: Box<dyn Rng>
}

impl RandomOrderIterator {
    pub fn with_seed(size: u64, seed: u64) -> Self {
        Self { elements: (0 as u64..size).collect() , rng: Box::new(StdRng::seed_from_u64(seed)), }
    }

    pub fn new(size: u64) -> Self {
        Self {
            elements: (0..size).collect(),
            rng: Box::new(rng()),
        }
    }
}

impl Iterator for RandomOrderIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements.is_empty() {
            None
        } else {
            let rand_idx = self.rng.random_range(0..self.elements.len());
            Some(self.elements.swap_remove(rand_idx))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.elements.len(), Some(self.elements.len()))
    }
}

const RAND_SEED: u64 = 1234;

fn worker_read_file(
    tx: mpsc::SyncSender<Vec<u8>>,
    file: Arc<File>,
    indices: Arc<Vec<u64>>,
    counter: Arc<AtomicUsize>,
    block_size: u64,
    total_size: u64,
) -> (Duration, u64) {
    let mut buffer = vec![0u8; block_size as usize];
    let mut read_time = Duration::ZERO;
    let mut reads: u64 = 0;

    loop {
        let i = counter.fetch_add(1, Ordering::Relaxed);

        if i >= indices.len() {
            break;
        }

        let block_idx = indices[i];

        let offset = block_size * block_idx;
        let bytes_to_read = std::cmp::min(block_size, total_size - offset) as usize;

        let target_buf = &mut buffer[..bytes_to_read];

        let read_start = Instant::now();
        file.read_exact_at(target_buf, offset).unwrap();
        read_time += read_start.elapsed();
        reads += 1;

        tx.send(target_buf.to_vec()).unwrap();
    }

    (read_time, reads)
}

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: latency <filepath> <block_size> <num_workers>");
        return;
    }

    let block_size: u64 = args[2].parse().expect("Invalid block size");
    let num_workers: usize = args[3].parse().expect("Invalid number of workers");

    assert!(block_size > 0, "Block size must be positive.");
    assert!(num_workers > 0, "Number of workers must be positive.");

    let start = Instant::now();

    let file_res = File::open(args[1].clone());
    if file_res.is_err() {
        eprintln!("File not found: {}", args[1]);
        return;
    }
    let file = Arc::new(file_res.unwrap());

    let metadata = file.metadata();
    if metadata.is_err() {
        eprintln!("Error opening file: {}", args[1]);
        return;
    }
    let total_size = metadata.unwrap().len();
    let num_blocks = (total_size + block_size - 1) / block_size;
    let random_iter = RandomOrderIterator::with_seed(num_blocks, RAND_SEED);
    let indices: Vec<u64> = random_iter.collect();
    let indices = Arc::new(indices);
    let counter = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(64);

    let mut handles = Vec::new();
    for _ in 0..num_workers {
        let file_clone = Arc::clone(&file);
        let tx_clone = tx.clone();
        let indices_clone = Arc::clone(&indices);
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            worker_read_file(
                tx_clone, 
                file_clone, 
                indices_clone,
                counter_clone,
                block_size,
                total_size,
            )
        });

        handles.push(handle);
    }

    drop(tx);


    let mut result = vec![0u8; 16];
    for block in rx {
        let block_hash = md5::compute(&block);
        sum_block_hash(&mut result, &block_hash);
    }

    let mut read_time = Duration::ZERO;
    let mut reads: u64 = 0;
    for handle in handles {
        let (t, n) = handle.join().unwrap();
        read_time += t;
        reads += n;
    }

    let total = start.elapsed();
    let avg_read_us = if reads > 0 { read_time.as_secs_f64() * 1e6 / reads as f64 } else { 0.0 };
    eprintln!("total_s={:.6} reads={} avg_read_us={:.3}", total.as_secs_f64(), reads, avg_read_us);

    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!("");

}
