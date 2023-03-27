import json
import pathlib
import re
from collections import defaultdict
from pprint import pprint

import matplotlib.pyplot as plt
from matplotlib.ticker import ScalarFormatter

bench_dir = pathlib.Path(__file__).parent.parent.parent / "target" / "criterion" / "path_index"


def read_bench(dir: pathlib.Path, baseline: str) -> tuple[float, float]:
    estimates = json.loads((dir / baseline / "estimates.json").read_bytes());
    return estimates["mean"]["point_estimate"] / 1e9, estimates["std_dev"]["point_estimate"] / 1e9


# load data
RE = re.compile(r"(\d+)k_ratio_([\d.]+)_(flip|noflip)")
data = defaultdict(lambda: defaultdict(dict))
for path in bench_dir.glob("*k_ratio_*_*flip"):
    m = RE.match(path.name)
    if m:
        n = int(m.group(1)) * 1000
        ratio = float(m.group(2))
        flip = m.group(3)
        data[n][flip][ratio] = read_bench(path, "new")

    else:
        print("WRONG FILENAME", path.name)


# pprint(data)

def make_plot(data) -> tuple[plt.Figure, plt.Axes]:
    """Make a plot grid for the given data. Each row of the grid corresponds to a value of n. The first column
    contains the plot for the noflip case, the second column contains the plot for the flip case.

    Each plot shows the mean and standard deviation of the benchmark for different values of the ratio parameter.
    """
    fig, axes = plt.subplots(len(data), 2, figsize=(10, 10), sharex=True)
    for i, n in enumerate(sorted(data)):
        for j, flip in enumerate(["noflip", "flip"]):
            ax = axes[i, j]
            ax.set_title(f"n={n}, {flip}")
            ax.set_xlabel("ratio")
            ax.set_ylabel("time (s)")
            #ax.set_xscale("log")
            #ax.set_yscale("log")

            ratios = sorted(data[n][flip])
            means = [data[n][flip][ratio][0] for ratio in ratios]
            stds = [data[n][flip][ratio][1] for ratio in ratios]
            ax.errorbar(ratios, means, yerr=stds)


    fig.tight_layout()
    return fig, axes

fig, ax = make_plot(data)
fig.savefig("path_index_benches.png")
