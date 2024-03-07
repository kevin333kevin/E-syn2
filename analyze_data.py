import csv
import re

headers = ['WireLoad', 'Gates', 'Cap', 'Area', 'Delay', 'IterationLimit', 'Classes', 'Nodes', 'Rebuilds']

table = []

i = 0
with open('log_adder.txt', 'r') as f:
    lines = f.readlines()
    for line in lines:
        if 'WireLoad' in line:
            row = []
            i += 1
            row.append("op" + str(i - 1))
            gates_match = re.search(r'Gates\s*=\s*(\d+)', line)
            if gates_match:
                gates = gates_match.group(1)
                row.append(gates)
            cap_match = re.search(r'Cap\s*=\s*([\d.]+)\s+ff', line)
            if cap_match:
                cap = cap_match.group(1)
                row.append(cap)
            area_match = re.search(r'Area\s*=\s*(\d+)', line)
            if area_match:
                area = area_match.group(1)
                row.append(area)
            delay_match = re.search(r'Delay\s*=\s*([\d.]+)\s+ps', line)
            if delay_match:
                delay = delay_match.group(1)
                row.append(delay)

            table.append(row)

i=0
with open('log_adder.txt', 'r') as f:
    lines = f.readlines()
    for line in lines:
        if 'Time take for runner:' in line:
            row = []
            i += 1
            row.append("op" + str(i - 1))
            time_match = re.search(r'Time take for runner:\s*([\d.]+)\s*s', line)
            if time_match:
                time = time_match.group(1)
                row.append(time)
                classes_match = re.search(r'Classes:\s+(\d+)', line)
                if classes_match:
                    classes = classes_match.group(1)
                    row.append(classes)
                nodes_match = re.search(r'Nodes:\s+(\d+)', line)
                if nodes_match:
                    nodes = nodes_match.group(1)
                    row.append(nodes)
                table.append(row)

i=0
with open('log_adder.txt', 'r') as f:
    lines = f.readlines()
    for line in lines:
        if 'Rebuilds:' in line:
            row = []
            i += 1
            row.append("op" + str(i - 1))
            rebuilds_match = re.search(r'Rebuilds:\s+(\d+)', line)
            if rebuilds_match:
                    rebuilds = rebuilds_match.group(1)
                    row.append(rebuilds)
            table.append(row)

with open('res_var1.csv', 'w', newline='') as csvfile:
    writer = csv.writer(csvfile)
    writer.writerow(headers)  
    writer.writerows(table)  