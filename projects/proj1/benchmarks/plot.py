"""
Turns every <experiment>_<mode>.csv in the given directory into tables in summary.md
and charts in plots/ (best = minimum total_s of the repeats of each configuration):

  latency_<mode>.csv     -> tables: total time and avg pread time per M x threads
                            charts: latency_total, latency_read
  throughput_<mode>.csv  -> table:  total/sys/read time, read vs mmap per M
                            charts: throughput_total, throughput_sys
  pipeline_<mode>.csv    -> table:  total time baseline -> pipeline per K x M
                            chart:  pipeline
  pipeline_k_<mode>.csv  -> table:  total/read/compute time per K
                            chart:  pipeline_k
  meta_<mode>.txt        -> copied into summary.md above the mode's tables
"""
import csv
import sys
from collections import defaultdict
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter

SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#4a3aa7", "#e34948"]
MARKERS = ["o", "s", "^", "D", "v", "P", "X", "*"]
SURFACE = "#fcfcfb"
INK = "#0b0b0b"
INK_2 = "#52514e"
MUTED = "#898781"
GRID = "#e1e0d9"
AXIS = "#c3c2b7"

plt.rcParams.update({
    "figure.facecolor": SURFACE,
    "axes.facecolor": SURFACE,
    "savefig.facecolor": SURFACE,
    "font.size": 10,
    "text.color": INK,
    "axes.labelcolor": INK_2,
    "axes.titlesize": 12,
    "axes.titleweight": "bold",
    "axes.titlecolor": INK,
    "axes.titlelocation": "left",
    "axes.edgecolor": AXIS,
    "axes.linewidth": 0.8,
    "axes.spines.top": False,
    "axes.spines.right": False,
    "axes.grid": True,
    "axes.axisbelow": True,
    "grid.color": GRID,
    "grid.linewidth": 0.8,
    "xtick.color": AXIS,
    "ytick.color": AXIS,
    "xtick.labelcolor": INK_2,
    "ytick.labelcolor": INK_2,
    "legend.frameon": False,
    "legend.labelcolor": INK_2,
    "lines.linewidth": 2,
    "lines.markersize": 6,
})

EXPERIMENTS = ["pipeline_k", "latency", "throughput", "pipeline"]


def load(path):
    groups = defaultdict(list)
    with open(path) as f:
        for row in csv.DictReader(f):
            groups[(row["program"], row["M"], row["threads"], row["K"])].append(row)
    return [min(rows, key=lambda r: float(r["total_s"])) for rows in groups.values()]


def pick(rows, program, **params):
    return [r for r in rows if r["program"] == program and all(r[k] == str(v) for k, v in params.items())]


def size(n):
    n = int(n)
    for unit, factor in [("MB", 1024 * 1024), ("KB", 1024)]:
        if n >= factor:
            return f"{n // factor} {unit}"
    return f"{n} B"


def kcount(k):
    k = int(k)
    return f"{k // 1000}k" if k >= 1000 and k % 1000 == 0 else str(k)


def series(i):
    return {"color": SERIES[i % len(SERIES)], "marker": MARKERS[i % len(MARKERS)]}


def save(fig, path):
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)


def log2_axis(ax, values, fmt):
    ax.set_xscale("log", base=2)
    ax.set_xticks(values, [fmt(v) for v in values])
    ax.minorticks_off()


def log_y(ax):
    ax.set_yscale("log")
    ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: f"{v:g}"))
    ax.yaxis.set_minor_formatter(FuncFormatter(lambda v, _: f"{v:g}" if f"{v:g}".lstrip("0.")[:1] in "25" else ""))


def legend(ax):
    ax.legend(loc="upper left", bbox_to_anchor=(1.01, 1))


def log10_axis(ax, values, fmt):
    ax.set_xscale("log")
    ax.set_xticks(values, [fmt(v) for v in values])
    ax.minorticks_off()
    if len(values) > 6:
        ax.tick_params(axis="x", labelrotation=45)


def plot_latency(rows, plots, mode):
    ms = sorted({int(r["M"]) for r in rows})
    threads = sorted({int(r["threads"]) for r in rows if r["threads"]})
    xs = list(range(1, len(threads) + 1))
    charts = [
        ("total_s", "czas programu [s]", "Latency: czas całego programu", "latency_total"),
        ("avg_read_us", "średni czas pread [µs]", "Latency: średni czas jednego odczytu", "latency_read"),
    ]
    files = []
    for col, ylabel, title, name in charts:
        fig, ax = plt.subplots(figsize=(8.5, 4.2))
        for i, m in enumerate(ms):
            new = sorted(pick(rows, "latency", M=m), key=lambda r: int(r["threads"]))
            ax.plot(xs, [float(r[col]) for r in new], label=f"M = {size(m)}", **series(i))
            for r in pick(rows, "latency_baseline", M=m):
                ax.plot([0], [float(r[col])], linestyle="none", **series(i))
        ax.axvline(0.5, color=GRID, linewidth=0.8)
        ax.set_xticks([0, *xs], ["baseline", *map(str, threads)])
        ax.set_xlabel("liczba wątków czytających")
        ax.set_ylabel(ylabel)
        ax.set_ylim(bottom=0)
        ax.set_title(title)
        legend(ax)
        files.append((title, f"{name}_{mode}.png"))
        save(fig, plots / files[-1][1])
    return files


def plot_throughput(rows, plots, mode):
    ms = sorted({int(r["M"]) for r in rows})
    charts = [
        ("total_s", "czas programu [s]", "Throughput: czas całego programu", "throughput_total"),
        ("sys_s", "czas CPU w jądrze (sys) [s]", "Throughput: czas systemowy (sys)", "throughput_sys"),
    ]
    files = []
    for col, ylabel, title, name in charts:
        fig, ax = plt.subplots(figsize=(8.5, 4.2))
        for i, (program, label) in enumerate([("throughput_baseline", "read()"), ("throughput", "mmap()")]):
            data = sorted(pick(rows, program), key=lambda r: int(r["M"]))
            ax.plot([int(r["M"]) for r in data], [float(r[col]) for r in data], label=label, **series(i))
        log2_axis(ax, ms, size)
        ax.set_xlabel("rozmiar bloku M")
        ax.set_ylabel(ylabel)
        ax.set_ylim(bottom=0)
        ax.set_title(title)
        legend(ax)
        files.append((title, f"{name}_{mode}.png"))
        save(fig, plots / files[-1][1])
    return files


def plot_pipeline(rows, plots, mode):
    ms = sorted({int(r["M"]) for r in rows})
    ks = sorted({int(r["K"]) for r in rows})
    fig, axes = plt.subplots(1, 2, figsize=(12, 4.2), sharey=True)
    for ax, (program, label) in zip(axes, [("pipeline_baseline", "baseline"), ("pipeline", "pipeline (2 wątki)")]):
        for i, m in enumerate(ms):
            data = sorted(pick(rows, program, M=m), key=lambda r: int(r["K"]))
            ax.plot([int(r["K"]) for r in data], [float(r["total_s"]) for r in data], label=f"M = {size(m)}", **series(i))
        log10_axis(ax, ks, kcount)
        log_y(ax)
        ax.set_xlabel("liczba iteracji K")
        ax.set_title(f"Pipeline: {label}")
    axes[0].set_ylabel("czas programu [s]")
    legend(axes[1])
    title = "Pipeline: czas w funkcji M i K"
    save(fig, plots / f"pipeline_{mode}.png")
    return [(title, f"pipeline_{mode}.png")]


def plot_pipeline_k(rows, plots, mode):
    m = rows[0]["M"]
    ks = sorted({int(r["K"]) for r in rows})
    base = sorted(pick(rows, "pipeline_baseline"), key=lambda r: int(r["K"]))
    new = sorted(pick(rows, "pipeline"), key=lambda r: int(r["K"]))
    lines = [
        (base, "total_s", "baseline: czas programu"),
        (new, "total_s", "pipeline: czas programu"),
        (new, "read_s", "pipeline: odczyt (read_s)"),
        (new, "compute_s", "pipeline: obliczenia (compute_s)"),
    ]
    fig, ax = plt.subplots(figsize=(8.5, 4.2))
    for i, (data, col, label) in enumerate(lines):
        ax.plot([int(r["K"]) for r in data], [float(r[col]) for r in data], label=label, **series(i))
    log10_axis(ax, ks, kcount)
    log_y(ax)
    ax.set_xlabel("liczba iteracji K")
    ax.set_ylabel("czas [s]")
    title = f"Pipeline (M = {size(m)}): odczyt vs obliczenia"
    ax.set_title(title)
    legend(ax)
    save(fig, plots / f"pipeline_k_{mode}.png")
    return [(title, f"pipeline_k_{mode}.png")]


def table(header, body):
    lines = ["| " + " | ".join(header) + " |", "|" + "---:|" * len(header)]
    lines += ["| " + " | ".join(row) + " |" for row in body]
    return "\n".join(lines)


def value(rows, col, fmt="{:.3f}"):
    return fmt.format(float(rows[0][col])) if rows and rows[0][col] else "-"


def summary_latency(rows):
    ms = sorted({int(r["M"]) for r in rows})
    threads = sorted({int(r["threads"]) for r in rows if r["threads"]})
    header = ["M", "baseline", *[f"{t} wątk." for t in threads]]
    out = []
    for col, caption, fmt in [("total_s", "Czas całego programu [s]", "{:.3f}"), ("avg_read_us", "Średni czas jednego pread [µs]", "{:.2f}")]:
        body = [
            [size(m), value(pick(rows, "latency_baseline", M=m), col, fmt)]
            + [value(pick(rows, "latency", M=m, threads=t), col, fmt) for t in threads]
            for m in ms
        ]
        out.append(f"**{caption}**\n\n" + table(header, body))
    return out


def summary_throughput(rows):
    ms = sorted({int(r["M"]) for r in rows})
    header = ["M", "read: czas [s]", "mmap: czas [s]", "read: sys [s]", "mmap: sys [s]", "read: same odczyty [s]"]
    body = []
    for m in ms:
        base, new = pick(rows, "throughput_baseline", M=m), pick(rows, "throughput", M=m)
        body.append([size(m), value(base, "total_s"), value(new, "total_s"), value(base, "sys_s"), value(new, "sys_s"), value(base, "read_s")])
    return [table(header, body)]


def summary_pipeline(rows):
    ms = sorted({int(r["M"]) for r in rows})
    ks = sorted({int(r["K"]) for r in rows})
    header = ["K", *[f"M = {size(m)}" for m in ms]]
    body = [
        [kcount(k)] + [f"{value(pick(rows, 'pipeline_baseline', M=m, K=k), 'total_s')} → {value(pick(rows, 'pipeline', M=m, K=k), 'total_s')}" for m in ms]
        for k in ks
    ]
    return ["**Czas programu [s]: baseline → pipeline**\n\n" + table(header, body)]


def summary_pipeline_k(rows):
    ks = sorted({int(r["K"]) for r in rows})
    header = ["K", "baseline: czas [s]", "pipeline: czas [s]", "pipeline: read_s", "pipeline: compute_s"]
    body = []
    for k in ks:
        base, new = pick(rows, "pipeline_baseline", K=k), pick(rows, "pipeline", K=k)
        body.append([kcount(k), value(base, "total_s"), value(new, "total_s"), value(new, "read_s"), value(new, "compute_s")])
    return [table(header, body)]


PLOTS = {"latency": plot_latency, "throughput": plot_throughput, "pipeline": plot_pipeline, "pipeline_k": plot_pipeline_k}
SUMMARIES = {"latency": summary_latency, "throughput": summary_throughput, "pipeline": summary_pipeline, "pipeline_k": summary_pipeline_k}
TITLES = {"latency": "Latency", "throughput": "Throughput", "pipeline": "Pipeline: siatka M × K", "pipeline_k": "Pipeline: M = 1 MB, przemiatanie K"}


def split_name(path):
    for exp in EXPERIMENTS:
        if path.stem.startswith(exp + "_"):
            return exp, path.stem[len(exp) + 1:]
    return None, None


MANUAL_MARKER = "<!-- Wszystko poniżej tej linii plot.py zachowuje przy ponownym generowaniu. -->"
MANUAL_TEMPLATE = MANUAL_MARKER + "\n\n## Podsumowanie\n\n_Tu wpisz własne wnioski._\n"


def manual_part(path):
    if path.exists():
        text = path.read_text()
        if MANUAL_MARKER in text:
            return text[text.index(MANUAL_MARKER):]
    return MANUAL_TEMPLATE


def process(directory):
    plots = directory / "plots"
    plots.mkdir(exist_ok=True)

    found = defaultdict(dict)
    for path in sorted(directory.glob("*.csv")):
        exp, mode = split_name(path)
        if exp:
            found[mode][exp] = load(path)

    sections = [
        f"# Wyniki: {directory.name}",
        "> Wyniki poniżej są generowane automatycznie przez `benchmarks/plot.py` z plików CSV "
        "i nadpisywane przy każdym uruchomieniu.",
        "Dla każdej konfiguracji pokazane jest najszybsze z powtórzeń (najmniejszy `total_s`); "
        "wszystkie kolumny w wierszu pochodzą z tego samego uruchomienia.",
    ]
    for mode in sorted(found):
        meta = directory / f"meta_{mode}.txt"
        sections.append(f"## Tryb: {mode}")
        if meta.exists():
            sections.append("```\n" + meta.read_text().strip() + "\n```")
        for exp in ["latency", "throughput", "pipeline", "pipeline_k"]:
            if exp not in found[mode]:
                continue
            rows = found[mode][exp]
            sections.append(f"### {TITLES[exp]}")
            sections += SUMMARIES[exp](rows)
            sections += [f"![{title}](plots/{name})" for title, name in PLOTS[exp](rows, plots, mode)]

    summary = directory / "summary.md"
    sections += ["---", manual_part(summary)]
    summary.write_text("\n\n".join(sections))
    print(f"{directory}/summary.md, {len(list(plots.glob('*.png')))} wykresów w {plots}")


def main():
    if len(sys.argv) < 2:
        sys.exit("Użycie: uv run benchmarks/plot.py KATALOG_Z_WYNIKAMI [...]")
    for arg in sys.argv[1:]:
        process(Path(arg))


if __name__ == "__main__":
    main()
