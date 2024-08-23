import os
import shutil
import re
import gzip
# 定义源和目标目录
# source_dir = "/data/cchen/HOGA/data_ml_aig"
# target_dir = "dataset_esyn"

# 创建目标目录
def copy_files(source_dir, target_dir):
    # 创建目标目录
    if not os.path.exists(target_dir):
        os.makedirs(target_dir)

    # 遍历源目录下的每个文件夹
    for folder in sorted(os.listdir(source_dir)):
        source_folder = os.path.join(source_dir, folder)
        if os.path.isdir(source_folder):
            for subfolder in ["edgelist", "feature"]:
                source_subfolder = os.path.join(source_folder, subfolder)
                for filename in sorted(os.listdir(source_subfolder)):
                    source_file = os.path.join(source_subfolder, filename)
                    i = int(filename.split('_')[-1].split('.')[0])
                    target_prob_folder = os.path.join(target_dir, f"{folder}_{i}", "raw")
                    os.makedirs(target_prob_folder, exist_ok=True)
                    target_file = os.path.join(target_prob_folder, filename)
                    shutil.copy(source_file, target_file)


def process_dataset(dataset_dir):
    for folder in os.listdir(dataset_dir):
        folder_path = os.path.join(dataset_dir, folder)
        if os.path.isdir(folder_path):
            process_folder(folder_path, folder)

def process_folder(folder_path, folder_name):
    raw_folder = os.path.join(folder_path, "raw")
    processed_folder = os.path.join(folder_path, "processed")
    if os.path.isdir(raw_folder):
        os.makedirs(processed_folder, exist_ok=True)
    if os.path.isdir(raw_folder):
        processed_files = set()
        for filename in os.listdir(raw_folder):
            source_file = os.path.join(raw_folder, filename)
            if filename.endswith(".csv"):
                new_filename = "node-feat.csv"
                if filename != new_filename and new_filename not in processed_files:
                    target_file = os.path.join(raw_folder, new_filename)
                    os.rename(source_file, target_file)
                    processed_files.add(new_filename)
                num_nodes = count_lines(target_file)
                num_node_file = os.path.join(raw_folder, "num-node-list.csv")
                with open(num_node_file, "w") as f:
                    f.write(str(num_nodes))

            elif filename.endswith(".el"):
                new_filename = "edge.csv"
                if filename != new_filename and new_filename not in processed_files:
                    target_file = os.path.join(raw_folder, new_filename)
                    edge_data = []
                    with open(source_file, "r") as f:
                        for line in f:
                            source, target = line.strip().split()
                            edge_data.append(f"{source},{target}")
                    with open(target_file, "w", newline="") as csvfile:
                        for line in edge_data:
                            csvfile.write(line + "\n")
                    os.remove(source_file)
                    processed_files.add(new_filename)
                num_edges = count_lines(target_file)
                num_edge_file = os.path.join(raw_folder, "num-edge-list.csv")
                with open(num_edge_file, "w") as f:
                    f.write(str(num_edges))

        for filename in os.listdir(raw_folder):
            if filename.endswith(".csv"):
                csv_file = os.path.join(raw_folder, filename)
                gz_file = os.path.join(raw_folder, f"{filename}.gz")
                with open(csv_file, "rb") as f_in, gzip.open(gz_file, "wb") as f_out:
                    shutil.copyfileobj(f_in, f_out)
                


def count_lines(file_path):
    with open(file_path, "r") as f:
        return sum(1 for _ in f)
# Usage
# dataset_dir = "/data/cchen/HOGA/dataset_esyn"
# process_dataset(dataset_dir)