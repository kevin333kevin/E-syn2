import os
import csv

log_directory = '/data/cchen/E-syn2/Log/'
output_csv = 'log_times.csv'

# 获取Log目录下的所有文件
file_list = [f for f in os.listdir(log_directory) if f.endswith('.txt')]

# 创建CSV文件并写入表头
with open(output_csv, 'w', newline='') as csvfile:
    writer = csv.writer(csvfile)
    writer.writerow(['文件名', 'eqn_graph时间', 'graph2eqn时间'])

    # 遍历文件列表
    for file_name in file_list:
        file_path = os.path.join(log_directory, file_name)

        # 读取文件内容
        with open(file_path, 'r') as file:
            lines = file.readlines()

        eqn_graph_time = None
        graph2eqn_time = None

        # 查找特定行并提取时间
        for line in lines:
            if line.startswith('eqn_to_sexpr:'):
                eqn_graph_time = line.split(':')[1].strip()
            elif line.startswith('Process 2.3 graph2eqn:'):
                graph2eqn_time = line.split(':')[1].strip()

        # 写入CSV文件
        writer.writerow([file_name, eqn_graph_time, graph2eqn_time])

print("记录完成。")

# import csv

# file_names = [
#     "i2c.txt", "dec.txt", "log2.txt", "multiplier.txt", "sqrt.txt",
#     "ctrl.txt", "priority.txt", "sin.txt", "div.txt", "cavlc.txt",
#     "int2float.txt", "adder.txt", "mem_ctrl.txt", "arbiter.txt", "bar.txt",
#     "voter.txt", "hyp.txt", "max.txt", "router.txt", "square.txt"
# ]

# eqn_graph_times = [
#     "326.22ms", "79.17ms", "6.20s", "5.57s", "4.84s",
#     "41.29ms", "252.28ms", "1.14s", "11.72s", "131.64ms",
#     "56.68ms", "295.73ms", "9.91s", "2.65s", "700.82ms",
#     "", "5.08s", "726.02ms", "64.91ms", "4.47s"
# ]

# graph2eqn_times = [
#     "1.62s", "0.96s", "22.11s", "11.45s", "13.57s",
#     "1.11s", "1.44s", "4.96s", "20.13s", "1.27s",
#     "1.15s", "1.26s", "18.83s", "6.80s", "1.82s",
#     "6.96s", "", "2.19s", "1.24s", "6.08s"
# ]

# # 打开CSV文件
# with open('data.csv', mode='w', newline='') as file:
#     writer = csv.writer(file)

#     # 写入表头
#     writer.writerow(['文件名', 'eqn_graph时间', '', 'graph2eqn时间'])

#     # 写入数据行
#     for i in range(len(file_names)):
#         writer.writerow([file_names[i], eqn_graph_times[i], '', graph2eqn_times[i]])