#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Read a CSV that contains Sequence, Level, Delay (ps)
Draw the figure exactly as requested and save to PDF with updated styles.
"""

import sys
import pathlib
import argparse

import pandas as pd
import matplotlib.pyplot as plt
from matplotlib import cm


def draw_figure(df: pd.DataFrame, pdf_path: pathlib.Path) -> None:
    """Create figure and save as pdf_path."""

    # ------------------------------------------------------------
    # Data split
    # ------------------------------------------------------------
    first10 = df.iloc[:10]          # blue cluster
    special = df.iloc[10]           # red point
    last3   = df.iloc[11:]          # green points

    # ------------------------------------------------------------
    # Base scatter points
    # ------------------------------------------------------------
    plt.style.use("seaborn-v0_8-whitegrid")
    fig, ax = plt.subplots(figsize=(8, 5), dpi=120)

    blue = ax.scatter(first10["Level"], first10["Delay (ps)"],
                      s=80, color="steelblue", edgecolor="black",
                      linewidth=.7,
                      label=("near-optimal level structural exploration"))

    red = ax.scatter(special["Level"], special["Delay (ps)"],
                     s=120, color="red", edgecolor="black",
                     linewidth=1.0, zorder=3,
                     label="baseline : level-oriented independent optimization")

    green = ax.scatter(last3["Level"], last3["Delay (ps)"],
                       s=100, color=cm.Greens(.55), edgecolor="black",
                       linewidth=.8,
                       label="continuous level-oriented independent optimization")

    # ------------------------------------------------------------
    # Red → Green arrows (segment by segment)
    # ------------------------------------------------------------
    line_df = pd.concat([special.to_frame().T, last3])  # 11,12,13,14
    for i in range(len(line_df) - 1):
        x0, y0 = line_df.iloc[i][["Level", "Delay (ps)"]]
        x1, y1 = line_df.iloc[i + 1][["Level", "Delay (ps)"]]
        ax.annotate("",
                    xy=(x1, y1),
                    xytext=(x0, y0),
                    arrowprops=dict(arrowstyle="->",
                                    color="red",
                                    lw=1.8))

    # ------------------------------------------------------------
    # Annotation next to red point
    # ------------------------------------------------------------
    ax.annotate("near independent\nlogic-level optimal",
                xy=(special["Level"], special["Delay (ps)"]),
                xytext=(special["Level"] + 2,
                        special["Delay (ps)"] - 40),
                fontsize=10, fontweight="bold",
                ha="left", va="bottom",
                arrowprops=dict(arrowstyle="-",
                                color="gray"))

    # ------------------------------------------------------------
    # Broad arrow from red point to blue cluster centre
    # ------------------------------------------------------------
    blue_x = first10["Level"].mean()
    blue_y = first10["Delay (ps)"].mean()

    ax.annotate("",
                xy=(blue_x, blue_y),
                xytext=(special["Level"], special["Delay (ps)"]),
                arrowprops=dict(arrowstyle="->",
                                color="black",
                                lw=1.5,
                                linestyle="--"))

    # ------------------------------------------------------------
    # Decorations
    # ------------------------------------------------------------
    ax.set_xlabel("Level", fontsize=13, fontweight="bold")
    ax.set_ylabel("Delay (ps)", fontsize=13, fontweight="bold")
    ax.set_title("A Case Study on Multiplier Design: Structural Differences in Mapping Impact",
                 fontsize=14, fontweight="bold")

    # Use custom handles because arrows are not part of legend
    handles, _ = ax.get_legend_handles_labels()
    ax.legend(handles=handles, frameon=True, fontsize=10)

    ax.set_xlim(df["Level"].min() - 2, df["Level"].max() + 2)
    ax.set_ylim(df["Delay (ps)"].min() - 70, df["Delay (ps)"].max() + 70)

    plt.tight_layout()
    fig.savefig(pdf_path, format="pdf")
    plt.close(fig)
    print(f"[INFO] Figure saved to {pdf_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot Level-Delay from CSV and save PDF")
    parser.add_argument("csv", help="input CSV file")
    parser.add_argument("--pdf", help="output PDF file (optional)")
    args = parser.parse_args()

    csv_path = pathlib.Path(args.csv).expanduser().resolve()
    if not csv_path.is_file():
        sys.exit(f"[ERROR] cannot find CSV: {csv_path}")

    pdf_path = pathlib.Path(args.pdf).expanduser().resolve() if args.pdf \
               else csv_path.with_suffix(".pdf")

    try:
        df = pd.read_csv(csv_path)
    except Exception as e:
        sys.exit(f"[ERROR] failed to read CSV: {e}")

    required = {"Sequence", "Level", "Delay (ps)"}
    if not required.issubset(df.columns):
        sys.exit(f"[ERROR] CSV must contain columns: {required}")

    draw_figure(df, pdf_path)


if __name__ == "__main__":
    main()