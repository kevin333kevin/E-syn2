import os
import csv

# folder_path = "/data/cchen/esyn2_base/E-syn2/CSV_45_p/"
# output_folder = "/data/cchen/esyn2_base/E-syn2/CSV_45_p/total_csv"
# output_file = "total_45.csv"


#folder_path = "/data/cchen/esyn2_base/E-syn2/CSV_130_p_wo/"
folder_path = "/data/cchen/esyn2_base/E-syn2/CSV_test_if_w/"
#output_folder = "/data/cchen/esyn2_base/E-syn2/CSV_130_p_wo/total_csv"
output_folder = "/data/cchen/esyn2_base/E-syn2/CSV_test_if_w/total_csv"
output_file = "total_130.csv"
# folder_path = "/data/cchen/esyn2_base/E-syn2/CSV_7_p_wo/"
# output_folder = "/data/cchen/esyn2_base/E-syn2/CSV_7_p_wo/total_csv"
# output_file = "total_7.csv"
# folder_path = "/data/cchen/esyn2_base/E-syn2/CSV_45_p_wo/"
# output_folder = "/data/cchen/esyn2_base/E-syn2/CSV_45_p_wo/total_csv"
# output_file = "total_45.csv"
# 获取文件夹下所有 CSV 文件的路径
csv_files = [os.path.join(folder_path, filename) for filename in os.listdir(folder_path) if filename.endswith(".csv")]

# 定义空的数据列表
data = []

# 遍历每个 CSV 文件
for csv_file in csv_files:
    with open(csv_file, "r") as file:
        reader = csv.reader(file)
        next(reader)  # 跳过表头行
        file_name = os.path.basename(csv_file).replace(".csv", "")  # 获取文件名并去除后缀 ".csv"
        row_data = [file_name]  # 添加文件名作为第一列
        for row in reader:
            row_data.extend(row)
        data.append(row_data)

# 创建输出文件夹
os.makedirs(output_folder, exist_ok=True)

# 写入到文件
output_path = os.path.join(output_folder, output_file)
with open(output_path, "a", newline="") as file:  # 使用追加模式打开文件
    writer = csv.writer(file)
    writer.writerow([","] + ["lev", "delay", "runtime"] * 6)  # 写入表头行
    for row in data:
        writer.writerow(row)

print(f"表格已成功写入到文件 {output_path}")