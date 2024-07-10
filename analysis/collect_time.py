import os
import csv

folder_path = '/data/cchen/esyn2_base/E-syn2/Log_parse_time/'
output_file = 'output.csv'

def extract_info_from_file(file_path):
    with open(file_path, 'r') as file:
        lines = file.readlines()

    file_name = os.path.basename(file_path)
    times = []
    
    for line in lines:
        if 'eqn2egraph finished in:' in line:
            times.append(line.split(' ')[-1].strip())
        elif 'Extract DAG completed in' in line:
            times.append(line.split(' ')[-2].strip())
        elif 'Process JSON completed in' in line:
            times.append(line.split(' ')[-2].strip())
        elif 'Graph to Equation in' in line:
            times.append(line.split(' ')[-2].strip())

    return [file_name] + times

file_list = []
for root, dirs, files in os.walk(folder_path):
    for file in files:
        if file.endswith('.txt'):
            file_list.append(os.path.join(root, file))

result = []
for file_path in file_list:
    result.append(extract_info_from_file(file_path))
result = sorted(result, key=lambda x: x[0][0])

with open(output_file, 'w', newline='') as csv_file:
    writer = csv.writer(csv_file)
    writer.writerow(['File Name', 'eqn2egraph Time', 'Extract DAG Time', 'Process JSON Time', 'Graph to Equation Time'])
    writer.writerows(result)


# Write sorted data to output CSV file
