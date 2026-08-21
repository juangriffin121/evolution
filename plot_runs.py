import argparse
from pathlib import Path

import pandas as pd
import matplotlib.pyplot as plt


PREY_COLOR = "green"
PRED_COLOR = "red"

plot_each_run = True
plot_energy = True
figsize=(12, 8)



def plot_run(df, title, output_path=None, plot_energy=False):
    if plot_energy:
        fig, (ax_pop, ax_energy) = plt.subplots(
            2, 1,
            figsize=figsize,
            sharex=True,
        )
    else:
        fig, ax_pop = plt.subplots(
            figsize=figsize,
        )


    # Population
    ax_pop.plot(
        df["age"],
        df["predators"],
        color=PRED_COLOR,
        label="Predators",
    )
    ax_pop.plot(
        df["age"],
        df["prey"],
        color=PREY_COLOR,
        label="Prey",
    )

    ax_pop.set_ylabel("Population")
    ax_pop.set_title(title)
    ax_pop.legend()
    ax_pop.grid(alpha=0.2)

    if plot_energy:
        # Mean energy
        ax_energy.plot(
            df["age"],
            df["total_pred_energy"],
            color=PRED_COLOR,
            label="Total predator energy",
        )
        ax_energy.plot(
            df["age"],
            df["total_prey_energy"],
            color=PREY_COLOR,
            label="Total prey energy",
        )

        ax_energy.set_xlabel("Age")
        ax_energy.set_ylabel("Total energy")
        ax_energy.legend()
        ax_energy.grid(alpha=0.2)

    fig.tight_layout()

    if output_path is not None:
        fig.savefig(output_path, dpi=150)
        plt.close(fig)
    else:
        plt.show()


def plot_all(runs, title, output_path=None, plot_energy=False):
    alpha = 0.4
    linewidth = 2
    if plot_energy:
        fig, (ax_pop, ax_energy) = plt.subplots(
            2, 1,
            figsize=figsize,
            sharex=True,
        )

    else:
        fig, ax_pop = plt.subplots(
            figsize=figsize,
        )


    for name, df in runs:

        # Population
        ax_pop.plot(
            df["age"],
            df["predators"],
            color=PRED_COLOR,
            alpha=alpha,
            linewidth=linewidth,
        )
        ax_pop.plot(
            df["age"],
            df["prey"],
            color=PREY_COLOR,
            alpha=alpha,
            linewidth=linewidth,
        )

        # Mean energy
        if plot_energy:
            ax_energy.plot(
                df["age"],
                df["total_pred_energy"],
                color=PRED_COLOR,
                alpha=alpha,
                linewidth=linewidth,
            )
            ax_energy.plot(
                df["age"],
                df["total_prey_energy"],
                color=PREY_COLOR,
                alpha=alpha,
                linewidth=linewidth,
            )

    # Dummy lines for a clean legend
    ax_pop.plot([], [], color=PRED_COLOR, label="Predators")
    ax_pop.plot([], [], color=PREY_COLOR, label="Prey")

    if plot_energy:
        ax_energy.plot([], [], color=PRED_COLOR, label="Total predator energy")
        ax_energy.plot([], [], color=PREY_COLOR, label="Total prey energy")

    ax_pop.set_ylabel("Population")
    ax_pop.set_title(title)
    ax_pop.legend()
    ax_pop.grid(alpha=2*alpha)


    if plot_energy:
        ax_energy.set_xlabel("Age")
        ax_energy.set_ylabel("Total energy")
        ax_energy.legend()
        ax_energy.grid(alpha=2*alpha)

    fig.tight_layout()

    if output_path is not None:
        fig.savefig(output_path, dpi=150)
        plt.close(fig)
    else:
        plt.show()


def main():
    directory = Path("./")

    csv_files = sorted(directory.glob("*.csv"))

    if not csv_files:
        raise SystemExit(f"No CSV files found in {directory}")

    output_dir = directory / "runs/plots"
    output_dir.mkdir(exist_ok=True)

    runs = []


    for csv_file in csv_files:
        df = pd.read_csv(csv_file)

        required_columns = {
            "age",
            "prey",
            "predators",
            "mean_prey_energy",
            "mean_pred_energy",
        }

        missing = required_columns - set(df.columns)

        if missing:
            print(f"Skipping {csv_file}: missing columns {missing}")
            continue

        # Sort in case the CSV isn't already ordered
        df = df.sort_values("age")

        df["total_prey_energy"] = (
            df["mean_prey_energy"] * df["prey"]
        )
        df["total_pred_energy"] = (
            df["mean_pred_energy"] * df["predators"]
        )

        runs.append((csv_file.stem, df))

        # One plot per seed/run

        if plot_each_run:
            output_path = output_dir / f"{csv_file.stem}.png"

            plot_run(
                df,
                title=f"Population {'and Total energy' if plot_energy else ''} vs Age per species ({csv_file.stem})",
                output_path= output_path,
                plot_energy=plot_energy
            )

    if not runs:
        raise SystemExit("No valid CSV files found.")

    # Combined plot
    combined_path = output_dir / "all_runs.png"

    plot_all(
        runs,
        title=f"All runs ({len(runs)} seeds)",
        output_path= combined_path,
        plot_energy=plot_energy
    )

    print(f"Plotted {len(runs)} runs.")



if __name__ == "__main__":
    main()
