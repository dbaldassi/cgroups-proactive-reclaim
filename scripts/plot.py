import polars as pl
import matplotlib.pyplot as plt
import glob
import os
import sys
import collections

# Dossier contenant les CSV
csv_dir = sys.argv[1] if len(sys.argv) > 1 else "."
csv_files = glob.glob(os.path.join(csv_dir, "*.csv"))

all_points = []
all_swap_points = []

for csv_file in csv_files:
    df = pl.read_csv(csv_file)
    timestamps = df["timestamp"].to_numpy()
    mem = (df["current_memory_usage"] / (1024 * 1024)).to_numpy()  # Convertir en MB
    swap = (df["current_swap_usage"] / (1024 * 1024)).to_numpy()
    all_points.extend(zip(timestamps, mem))
    all_swap_points.extend(zip(timestamps, swap))

mem_x, mem_y = zip(*all_points)
swap_x, swap_y = zip(*all_swap_points)

fig, ax1 = plt.subplots(figsize=(10, 6))

# Scatter mémoire (MB) sur axe principal
ax1.scatter(mem_x, mem_y, color='tab:blue', alpha=0.2, s=8, label="Mémoire (points)")

mem_dict = collections.defaultdict(list)
for x, y in all_points:
    mem_dict[x].append(y)
mem_mean_x = sorted(mem_dict.keys())
mem_mean_y = [sum(mem_dict[t])/len(mem_dict[t]) for t in mem_mean_x]
# Courbe moyenne mémoire bien visible
ax1.plot(mem_mean_x, mem_mean_y, color='darkblue', linewidth=2.5, label="Mémoire (moyenne)")

ax1.set_xlabel("Timestamp")
ax1.set_ylabel("Mémoire (MB)", color='darkblue')
ax1.tick_params(axis='y', labelcolor='darkblue')

# Scatter swap sur axe secondaire - petits points peu visibles
ax2 = ax1.twinx()
ax2.scatter(swap_x, swap_y, color='tab:orange', alpha=0.2, s=8, label="Swap (points)")

swap_dict = collections.defaultdict(list)
for x, y in all_swap_points:
    swap_dict[x].append(y)
swap_mean_x = sorted(swap_dict.keys())
swap_mean_y = [sum(swap_dict[t])/len(swap_dict[t]) for t in swap_mean_x]
# Courbe moyenne swap bien visible
ax2.plot(swap_mean_x, swap_mean_y, color='red', linewidth=2.5, label="Swap (moyenne)")

ax2.set_ylabel("Swap (MB)", color='tab:orange')
ax2.tick_params(axis='y', labelcolor='tab:orange')

fig.suptitle("Mémoire (MB) et swap (MB) en fonction du temps (tous CSV)")
fig.tight_layout()

# Sauvegarder l'image dans le dossier des CSV
img_file = os.path.join(csv_dir, "mem_swap_vs_time.png")
plt.savefig(img_file)
plt.close()

for csv_file in csv_files:
    df = pl.read_csv(csv_file)
    timestamps = df["timestamp"].to_numpy()
    active_anon = (df["active_anon"] / (1024 * 1024)).to_numpy()
    inactive_anon = (df["inactive_anon"] / (1024 * 1024)).to_numpy()
    active_file = (df["active_file"] / (1024 * 1024)).to_numpy()
    inactive_file = (df["inactive_file"] / (1024 * 1024)).to_numpy()

    plt.figure(figsize=(10, 6))
    plt.plot(timestamps, active_anon, label="active_anon (MB)", color="tab:blue")
    plt.plot(timestamps, inactive_anon, label="inactive_anon (MB)", color="tab:orange")
    plt.plot(timestamps, active_file, label="active_file (MB)", color="tab:green")
    plt.plot(timestamps, inactive_file, label="inactive_file (MB)", color="tab:red")

    plt.xlabel("Timestamp")
    plt.ylabel("Valeur (MB)")
    plt.title(f"Listes actives/inactives ({os.path.basename(csv_file)})")
    plt.legend()
    plt.tight_layout()

    img_file = os.path.splitext(csv_file)[0] + "_active_inactive.png"
    plt.savefig(img_file)
    plt.close()