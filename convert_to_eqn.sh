#!/bin/bash

path="/data/cchen/esyn2_base/E-syn2/benchmarks/best_results"
depth_dir="$path/depth"
eqn_depth_dir="$path/eqn_depth"
abc_dir="/data/cchen/esyn2_base/E-syn2/abc"  # 替换为实际的 abc 目录路径

cd "$depth_dir"

for file in *.blif
do
  if [[ -f $file ]]; then
    file_path="$depth_dir/$file"
    new_file="${file%.blif}.eqn"
    new_file="$eqn_depth_dir/$new_file"
    mkdir -p "$(dirname "$new_file")"
    touch "$new_file"
    command="$abc_dir/abc -c \"read_blif $file_path; st; write_eqn $new_file\""
    eval "$command"
  fi
done