import re
import csv

def parse_txt_to_csv(input_txt, output_csv):
    # 定义正则表达式，匹配 lev 和 delay 的两种格式
    lev_pattern = re.compile(r"lev\s*=\s*(\d+)")
    delay_pattern = re.compile(r"Delay\s*=\s*([\d.]+)\s*ps")

    # 用于存储提取的结果
    extracted_data = []

    with open(input_txt, 'r') as file:
        sequence = 0  # 用于记录序号
        lev = None
        delay = None

        for line in file:
            # 匹配 lev
            lev_match = lev_pattern.search(line)
            if lev_match:
                lev = lev_match.group(1)

            # 匹配 delay
            delay_match = delay_pattern.search(line)
            if delay_match:
                delay = delay_match.group(1)

            # 如果同时找到了 lev 和 delay，记录数据
            if lev and delay:
                extracted_data.append([sequence, lev, delay])
                sequence += 1  # 更新序号
                lev = None  # 重置 lev
                delay = None  # 重置 delay

    # 将结果写入 CSV 文件
    with open(output_csv, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        # 写入表头
        writer.writerow(["Sequence", "Level", "Delay (ps)"])
        # 写入数据
        writer.writerows(extracted_data)
    
    print(f"Data successfully extracted to {output_csv}")

# 输入和输出文件路径
input_txt_file = "/home/cchen099/E-syn2/motivation/test.txt"  # 替换为你的文本文件路径
output_csv_file = "output.csv"  # 替换为你的输出 CSV 文件路径

# 调用函数
parse_txt_to_csv(input_txt_file, output_csv_file)