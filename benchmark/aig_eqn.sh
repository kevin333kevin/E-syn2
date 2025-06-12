#!/bin/bash

# 定义相对路径
SCRIPT_DIR=$(dirname "$0")        # 当前脚本所在目录
EPFL_DIR="$SCRIPT_DIR/epfl"       # 输入目录 (epfl 文件夹)
OUTPUT_DIR="$SCRIPT_DIR/epfl_eqn" # 输出目录 (epfl_eqn 文件夹)
ABC_PATH="$SCRIPT_DIR/../abc/abc" # abc 程序路径 (../abc/abc)

# 检查目标输出文件夹是否存在，如果不存在则创建
if [ ! -d "$OUTPUT_DIR" ]; then
    mkdir -p "$OUTPUT_DIR"
fi

# 遍历 epfl 文件夹下的所有 .aig 文件
for aig_file in "$EPFL_DIR"/*.aig; do
    # 检查是否有 .aig 文件
    if [ ! -f "$aig_file" ]; then
        echo "No .aig files found in $EPFL_DIR"
        exit 1
    fi

    # 提取文件名（不带路径和后缀）
    filename=$(basename -- "$aig_file")
    filename_no_ext="${filename%.*}"

    # 定义输出 .eqn 文件路径
    eqn_file="$OUTPUT_DIR/$filename_no_ext.eqn"

    # 执行 abc 命令
    "$ABC_PATH" -c "read $aig_file; write_eqn $eqn_file"

    # 打印转换结果
    if [ $? -eq 0 ]; then
        echo "Converted $aig_file to $eqn_file"
    else
        echo "Failed to convert $aig_file"
    fi
done