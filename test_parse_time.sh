#!/bin/bash

input_dir="/data/cchen/E-syn2/converted_circuit_strash/EPFL/"  # 输入目录
output_dir="/data/cchen/E-syn2/e-rewriter/"  # 输出目录
log_dir="/data/cchen/E-syn2/Log/"  # 日志目录

mkdir -p "$log_dir"  # 创建日志目录

for file in "$input_dir"/*.eqn; do
    filename=$(basename "$file" .eqn)  # 提取文件名（不含扩展名）
    log_file="$log_dir/$filename.txt"  # 日志文件路径

    # 复制文件到目标路径
    cp "$file" "$output_dir/circuit0.eqn"

    # 运行脚本并将终端信息保存到日志文件
    cd /data/cchen/E-syn2/
    ./run2_fast.sh > "$log_file" 2>&1

    echo "Completed: $filename"  # 打印已完成的文件名
done