import csv

existing_table = {}

# 读取现有的CSV文件
with open('res_var1.csv', 'r') as csvfile:
    reader = csv.reader(csvfile)
    for row in reader:
        op = row[0]
        data = row[1:]
        if op in existing_table:
            existing_table[op].extend(data)
        else:
            existing_table[op] = data

# 将字典转换为列表
merged_table = [[op, *data] for op, data in existing_table.items()]

# 写入新的CSV文件
with open('res_var2.csv', 'w', newline='') as csvfile:
    writer = csv.writer(csvfile)
    writer.writerows(merged_table)

    