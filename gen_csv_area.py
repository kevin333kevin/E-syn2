import os
import re
import csv

lev = []
delay = []
and_gate = []
area = []
time_values=[] 
def extract_values(output_text):
    pattern_lev = r"lev\s*=\s*(\d+)"
    pattern_and = r"and\s*=\s*(\d+)"
    pattern_delay = r"Delay\s*=\s*([\d.]+) ps"
    pattern_area = r"Area\s*=\s*([\d.]+)"
    matches_lev = re.findall(pattern_lev, output_text)
    matches_delay = re.findall(pattern_delay, output_text)
    matches_and = re.findall(pattern_and, output_text)
    matches_area = re.findall(pattern_area, output_text)
    if matches_lev:
        lev_value = int(matches_lev[0])
       # print("lev:", lev_value)
        lev.append(lev_value)
    if matches_and:
        and_value = int(matches_and[0])
       # print("lev:", lev_value)
        and_gate.append(and_value)

    if matches_delay:
       # print(matches_delay)
        delay_value = round(float(matches_delay[-1]), 2)
       # print("delay:", delay_value)
        delay.append(delay_value)

    if matches_area:
       # print(matches_delay)
        area_value = round(float(matches_area[0]), 2)
       # print("delay:", delay_value)
        area.append(area_value)

    pattern_time = r"Run\s*ABC\s*on\s*the\s*ori\s{2}in\s*([\d.]+)\s*seconds"

    matches = re.findall(pattern_time, output_text)
    
    for match in matches:
           print("found")
           rounded_time = round(float(match), 2)
           time_values.append(rounded_time)
        

    pattern_time1 = r"Total runtime \w+: ([\d.]+) seconds"

    matches = re.findall(pattern_time1, output_text)
    for match1 in matches:
        time_value = float(match1)
        time_values.append(round(float(match1), 2))




def process_text_file(file_path):
    with open(file_path, 'r') as file:
        text = file.read()
        lines = text.splitlines()  # 按行拆分文本内容

        for line in lines:
            # Extract values from the line
            extract_values(line)

def save_to_csv(data, csv_path):
    with open(csv_path, 'w', newline='') as file:
        writer = csv.writer(file)
        writer.writerow(['and','area','lev', 'delay','runtime'])
        writer.writerows(data)

# 指定目录路径
# directory = '/data/cchen/esyn2_base/E-syn2/Log_45_p/'
# output_directory = '/data/cchen/esyn2_base/E-syn2/CSV_45_p/'

# directory = '/data/cchen/esyn2_base/E-syn2/Log_130_p/'
# output_directory = '/data/cchen/esyn2_base/E-syn2/CSV_130_p/'
# directory = '/data/cchen/esyn2_base/E-syn2/Log_7_p_wo/'
# output_directory = '/data/cchen/esyn2_base/E-syn2/CSV_7_p_wo/'
# directory = '/data/cchen/esyn2_base/E-syn2/Log_45_p_wo/'
# output_directory = '/data/cchen/esyn2_base/E-syn2/CSV_45_p_wo_area/'
# directory = '/data/cchen/esyn2_base/E-syn2/Log_test_if_w/'
# output_directory = '/data/cchen/esyn2_base/E-syn2/CSV_test_if_w/'
directory = '/data/cchen/esyn2_base/E-syn2/Log_test_if_abc/'
output_directory = '/data/cchen/esyn2_base/E-syn2/CSV_test_if_abc/'

# 创建输出目录
os.makedirs(output_directory, exist_ok=True)

# 遍历目录下的每个txt文件
for filename in os.listdir(directory):
    if filename.endswith('.txt'):
        file_path = os.path.join(directory, filename)
        
        # 清空数据列表
        lev.clear()
        delay.clear()
        time_values.clear()
        and_gate.clear()
        area.clear()
        # 处理文本文件
        process_text_file(file_path)

        print("lev:", len(lev))
        print("delay:", len(delay))
        print("and:", len(and_gate))
        print("area:", len(area))
        print(time_values)
        print(len(time_values))

        # 生成对应的CSV文件名
        csv_filename = os.path.splitext(filename)[0] + '.csv'
        csv_path = os.path.join(output_directory, csv_filename)
        print(csv_filename)

        # 保存到CSV文件
        data = [[and_gate[i],area[i],lev[i], delay[i],time_values[i]] for i in range(len(lev))]  # 按索引对应组合数据
        save_to_csv(data, csv_path)