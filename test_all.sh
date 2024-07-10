#!/bin/bash
input_dir="/data/cchen/esyn2_base/E-syn2/EPFL/"  # 输入目录
output_dir="/data/cchen/esyn2_base/E-syn2/e-rewriter/"  # 输出目录
output_dir="/data/cchen/esyn2_base/E-syn2/abc/"  # 输出目录
#log_dir="/data/cchen/esyn2_base/E-syn2/Log_7_p/"  # 日志目录
log_dir="/data/cchen/esyn2_base/E-syn2/Log_test_if_map/"  # 日志目录
# log_dir="/data/cchen/esyn2_base/E-syn2/Log_130_wo/"  # 日志目录
# csv_dir="/data/cchen/esyn2_base/E-syn2/csv/"  # 日志目录

# 创建日志目录
mkdir -p "$log_dir"

# 遍历输入目录下的所有 .eqn 文件
count=0
for file in "$input_dir"/*.eqn; do
    # ((count++))

    # # # 跳过前4个文件
    # if ((count <= 3)); then
    #     continue
    # fi

    # 提取文件名（不含扩展名）
    filename=$(basename "$file" .eqn)

    # 创建与文件名相同的子目录
    # mkdir -p "$csv_dir/$filename"

    # 复制文件到目标路径
    cp "$file" "$output_dir/ori.eqn"

    # 日志文件路径
    log_file="$log_dir/$filename.txt"

    # 运行脚本并将终端信息保存到日志文件
    cd /data/cchen/esyn2_base/E-syn2/
    #echo -e "5\ndelay\nfaster-bottom-up\n" | bash run_test_abc.sh > "$log_file" 2>&1
    echo -e "5\ndelay\nfaster-bottom-up\n" | bash run.sh > "$log_file" 2>&1
    bash clean.sh 
done