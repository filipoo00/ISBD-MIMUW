"""
Usage (run from projects/proj1; benchmarks/files/test.bin is created on the first run):

  All experiments, file in page cache (warm-up run + 5 repeats):
    python3 benchmarks/bench.py

  All experiments, page cache dropped before every run:
    sudo -v && python3 benchmarks/bench.py --cold

  Under memory pressure, process + page cache limited to 128 MB (Linux only):
    sudo -v && python3 benchmarks/bench.py --memory-max 128M

  Memory pressure combined with dropped cache:
    sudo -v && python3 benchmarks/bench.py --cold --memory-max 128M

  Only selected experiments (latency, throughput, pipeline, pipeline_k):
    python3 benchmarks/bench.py --only latency
    python3 benchmarks/bench.py --only throughput pipeline

  Find K_opt for M = 1 MB, with and without page cache:
    python3 benchmarks/bench.py --only pipeline_k
    sudo -v && python3 benchmarks/bench.py --only pipeline_k --cold

  File that fits under the memory limit, to compare with the one that does not:
    sudo -v && python3 benchmarks/bench.py benchmarks/files/small.bin --size 64M --memory-max 128M --out benchmarks/results/small

Results: <out>/<experiment>_<mode>.csv (one row per repeat) and <out>/meta_<mode>.txt,
where <mode> is warm, cold, warm_mem128M, cold_mem128M, ...
"""
import argparse
import csv
import os
import platform
import resource
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BIN_DIR = ROOT / "src" / "target" / "release"
OUT_DIR = ROOT / "benchmarks" / "results"
FILES_DIR = ROOT / "benchmarks" / "files"

KB = 1024
MB = 1024 * KB

LATENCY_M = [512, 4 * KB, 16 * KB, 64 * KB]
LATENCY_THREADS = [1, 2, 4, 8, 16]
THROUGHPUT_M = [512, 1 * KB, 4 * KB, 16 * KB, 64 * KB, 256 * KB, 1 * MB]
PIPELINE_M = [64 * KB, 256 * KB, 1 * MB, 4 * MB]
PIPELINE_K = [1, 10, 100, 1000, 10000]
PIPELINE_K_SWEEP = [1, 10, 100, 1000, 2000, 5000, 10000, 20000, 50000, 100000]

FIELDS = [
    "program", "M", "threads", "K", "repeat",
    "total_s", "read_s", "compute_s", "reads", "avg_read_us",
    "user_s", "sys_s", "hash",
]


def parse_size(text):
    units = {"K": KB, "M": MB, "G": 1024 * MB}
    unit = text[-1].upper()
    return int(text[:-1]) * units[unit] if unit in units else int(text)


def create_file(path, size):
    path.parent.mkdir(parents=True, exist_ok=True)
    print(f"Tworzę {path} ({size // MB} MB)", flush=True)
    with open(path, "wb") as f:
        left = size
        while left > 0:
            chunk = min(left, 16 * MB)
            f.write(os.urandom(chunk))
            left -= chunk
        f.flush()
        os.fsync(f.fileno())


def drop_caches():
    if sys.platform == "darwin":
        cmd = ["sudo", "purge"]
    else:
        cmd = ["sudo", "sh", "-c", "sync && sysctl -w vm.drop_caches=3"]
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)


def run_once(program, args, memory_max):
    cmd = [str(BIN_DIR / program), *map(str, args)]
    if memory_max:
        cmd = ["sudo", "systemd-run", "--scope", "--quiet", "-p", f"MemoryMax={memory_max}", *cmd]

    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)

    stats = dict(kv.split("=", 1) for kv in proc.stderr.split() if "=" in kv)
    return {
        **stats,
        "user_s": f"{after.ru_utime - before.ru_utime:.6f}",
        "sys_s": f"{after.ru_stime - before.ru_stime:.6f}",
        "hash": proc.stdout.split()[-1],
    }


def measure(program, args, params, opts):
    if not opts.cold:
        run_once(program, args, opts.memory_max)

    rows = []
    for repeat in range(opts.repeats):
        if opts.cold:
            drop_caches()
        rows.append({"program": program, **params, "repeat": repeat, **run_once(program, args, opts.memory_max)})

    hashes = {row["hash"] for row in rows}
    assert len(hashes) == 1, f"{program} {params}: różne hashe {hashes}"
    return rows


def same_hash(baseline, rows):
    assert rows[0]["hash"] == baseline[0]["hash"], f"{rows[0]['program']}: hash różny od baseline"


def bench_latency(file, opts):
    rows = []
    for m in LATENCY_M:
        base = measure("latency_baseline", [file, m], {"M": m}, opts)
        rows += base
        for threads in LATENCY_THREADS:
            new = measure("latency", [file, m, threads], {"M": m, "threads": threads}, opts)
            same_hash(base, new)
            rows += new
    return rows


def bench_throughput(file, opts):
    rows = []
    for m in THROUGHPUT_M:
        base = measure("throughput_baseline", [file, m], {"M": m}, opts)
        new = measure("throughput", [file, m], {"M": m}, opts)
        same_hash(base, new)
        rows += base + new
    return rows


def bench_pipeline(file, opts, ms, ks):
    rows = []
    for m in ms:
        for k in ks:
            base = measure("pipeline_baseline", [file, m, k], {"M": m, "K": k}, opts)
            new = measure("pipeline", [file, m, k], {"M": m, "K": k}, opts)
            same_hash(base, new)
            rows += base + new
    return rows


EXPERIMENTS = {
    "latency": bench_latency,
    "throughput": bench_throughput,
    "pipeline": lambda file, opts: bench_pipeline(file, opts, PIPELINE_M, PIPELINE_K),
    "pipeline_k": lambda file, opts: bench_pipeline(file, opts, [1 * MB], PIPELINE_K_SWEEP),
}


def write_csv(path, rows):
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=FIELDS, restval="")
        writer.writeheader()
        writer.writerows(rows)


def write_meta(path, file, opts):
    path.write_text(
        f"date: {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
        f"platform: {platform.platform()}\n"
        f"cpu_count: {os.cpu_count()}\n"
        f"file: {file}\n"
        f"file_size_bytes: {file.stat().st_size}\n"
        f"cold: {opts.cold}\n"
        f"memory_max: {opts.memory_max}\n"
        f"repeats: {opts.repeats}\n"
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("file", type=Path, nargs="?", default=FILES_DIR / "test.bin")
    parser.add_argument("--size", default="256M")
    parser.add_argument("--cold", action="store_true")
    parser.add_argument("--memory-max")
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--only", nargs="+", choices=EXPERIMENTS, default=list(EXPERIMENTS))
    parser.add_argument("--out", type=Path, default=OUT_DIR)
    opts = parser.parse_args()

    file = opts.file.resolve()
    if not file.exists():
        create_file(file, parse_size(opts.size))
    if opts.memory_max and sys.platform != "linux":
        sys.exit("--memory-max działa tylko na Linuxie (systemd-run)")

    subprocess.run(["make", "-C", str(ROOT)], check=True, stdout=subprocess.DEVNULL)

    mode = "cold" if opts.cold else "warm"
    if opts.memory_max:
        mode += f"_mem{opts.memory_max}"

    opts.out.mkdir(parents=True, exist_ok=True)
    write_meta(opts.out / f"meta_{mode}.txt", file, opts)

    for name in opts.only:
        print(f"{name} ({mode})", flush=True)
        rows = EXPERIMENTS[name](file, opts)
        write_csv(opts.out / f"{name}_{mode}.csv", rows)


if __name__ == "__main__":
    main()
