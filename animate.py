import json
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.animation as animation

from read_frames import read_frames

with open("constants.json") as f:
    constants = json.load(f)

WORLD_W, WORLD_H = constants["world_shape"]
FRAMES_PATH = f"runs/frames/{constants['seed']}.bin"


fig, ax = plt.subplots(figsize=(10, 6))
ax.set_xlim(0, WORLD_W)
ax.set_ylim(0, WORLD_H)
ax.set_aspect("equal")

p0 = ax.transData.transform((0, 0))
p1 = ax.transData.transform((1, 0))
points_per_data_unit = np.hypot(*(p1 - p0)) * 72.0 / fig.dpi

def radius_to_size(radius_data_units):
    # area in points^2 for a scatter marker of the given radius in data (world) units
    return np.pi * (radius_data_units * points_per_data_unit) ** 2

prey_scatter = ax.scatter([], [], c="green", alpha=0.8, label="prey")
pred_scatter = ax.scatter([], [], c="red", alpha=0.8, label="predator")
# ax.legend(loc="upper right")
title = ax.set_title("")


def update(frame):
    age, blobs = frame
    prey = blobs[blobs["type"] == 0]
    pred = blobs[blobs["type"] == 1]

    prey_scatter.set_offsets(np.column_stack([prey["x"], prey["y"]]))
    prey_scatter.set_sizes(radius_to_size(np.sqrt(prey["energy"])))  # area ~ energy, since radius ~ sqrt(energy)

    pred_scatter.set_offsets(np.column_stack([pred["x"], pred["y"]]))
    pred_scatter.set_sizes(radius_to_size(np.sqrt(pred["energy"])))

    title.set_text(f"age {age}   prey {len(prey)}   predators {len(pred)}")

    print(f"\rFrame {age}", end="", flush=True)
    return prey_scatter, pred_scatter, title


anim = animation.FuncAnimation(
    fig,
    update,
    frames=read_frames(FRAMES_PATH),  # generator: reads the .bin lazily, doesn't load it all at once
    interval=30,
    blit=True,
    cache_frame_data=False,  # required with a generator, otherwise matplotlib tries to cache every frame
)

# plt.show()
# to save instead of (or as well as) showing it live:
anim.save(f"evolution(seed{constants['seed']}).mp4", writer="ffmpeg", fps=20)
