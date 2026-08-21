import csv
import json
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from matplotlib.gridspec import GridSpec

from read_frames import read_frames

with open("constants.json") as f:
    constants = json.load(f)

WORLD_W, WORLD_H = constants["world_shape"]
SEED = constants["seed"]
FRAMES_PATH = f"runs/frames/{SEED}.bin"
STATS_PATH = f"runs/seed{SEED}.csv"

# ---- load the full stats log once; we reveal it progressively as the animation plays ----
ages, prey_counts, pred_counts = [], [], []
with open(STATS_PATH) as f:
    for row in csv.DictReader(f):
        ages.append(int(row["age"]))
        prey_counts.append(int(row["prey"]))
        pred_counts.append(int(row["predators"]))
ages = np.array(ages)
prey_counts = np.array(prey_counts)
pred_counts = np.array(pred_counts)

# ---- figure: world on top, growing stats plot underneath ----
fig = plt.figure(figsize=(10, 8))
gs = GridSpec(2, 1, height_ratios=[2, 1], hspace=0.25)
ax_world = fig.add_subplot(gs[0])
ax_stats = fig.add_subplot(gs[1])

ax_world.set_xlim(0, WORLD_W)
ax_world.set_ylim(0, WORLD_H)
ax_world.set_aspect("equal")

# scatter's `s` is an area in points^2, a screen-space unit with no inherent relationship
# to data coordinates. To make plotted circle size == sqrt(energy) (the exact radius Rust's
# Blob::radius() computes, and the exact value check_pred_interactions sums to test
# collisions), convert data-unit radius -> points using the axes' own transform. This only
# stays valid as long as xlim/ylim/aspect/figure-size don't change after this point, which
# is why it's computed once, up front, rather than per frame.
p0 = ax_world.transData.transform((0, 0))
p1 = ax_world.transData.transform((1, 0))
points_per_data_unit = np.hypot(*(p1 - p0)) * 72.0 / fig.dpi

prey_scatter = ax_world.scatter([], [], c="green", alpha=0.8, label="prey")
pred_scatter = ax_world.scatter([], [], c="red", alpha=0.8, label="predator")
# ax_world.legend(loc="upper right")
title = ax_world.set_title("")

ax_stats.set_xlim(0, ages.max() if len(ages) else 1)
ax_stats.set_ylim(0, max(prey_counts.max(), pred_counts.max(), 1) * 1.1)
ax_stats.set_xlabel("age")
ax_stats.set_ylabel("count")
prey_line, = ax_stats.plot([], [], c="green", label="prey")
pred_line, = ax_stats.plot([], [], c="red", label="predators")
ax_stats.legend(loc="upper right")


def radius_to_size(radius_data_units):
    # area in points^2 for a scatter marker of the given radius in data (world) units
    return np.pi * (radius_data_units * points_per_data_unit) ** 2


def update(indexed_frame):
    idx, (age, blobs) = indexed_frame
    prey = blobs[blobs["type"] == 0]
    pred = blobs[blobs["type"] == 1]

    prey_scatter.set_offsets(np.column_stack([prey["x"], prey["y"]]))
    prey_scatter.set_sizes(radius_to_size(np.sqrt(prey["energy"])))

    pred_scatter.set_offsets(np.column_stack([pred["x"], pred["y"]]))
    pred_scatter.set_sizes(radius_to_size(np.sqrt(pred["energy"])))

    title.set_text(f"age {age}   prey {len(prey)}   predators {len(pred)}")

    prey_line.set_data(ages[: idx + 1], prey_counts[: idx + 1])
    pred_line.set_data(ages[: idx + 1], pred_counts[: idx + 1])

    progress(idx, 1)

    return prey_scatter, pred_scatter, title, prey_line, pred_line

def progress(current_frame, total_frames):
    print(f"\rFrame {current_frame}/{total_frames}", end="", flush=True)

anim = animation.FuncAnimation(
    fig,
    update,
    frames=enumerate(read_frames(FRAMES_PATH)),  # (idx, (age, blobs)) per frame
    interval=30,
    blit=True,
    cache_frame_data=False,
)

# plt.show()

anim.save("evolution.mp4", writer="ffmpeg", fps=15, extra_args=["-preset", "ultrafast"])
