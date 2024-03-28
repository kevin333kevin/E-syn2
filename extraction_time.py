import os
import csv

log_directory = '/data/cchen/E-syn2/Log/'
output_csv = 'log_extraction.csv'

# Get all files in the Log directory
file_list = [f for f in os.listdir(log_directory) if f.endswith('.txt')]

# Create the CSV file and write the header
with open(output_csv, 'w', newline='') as csvfile:
    writer = csv.writer(csvfile)
    writer.writerow(['文件名', '提取DAG时间'])

    # Iterate over the file list
    for file_name in file_list:
        file_path = os.path.join(log_directory, file_name)

        # Read the file content
        with open(file_path, 'r') as file:
            lines = file.readlines()

        extract_dag_time = None

        # Find the specific line and extract the time
        for line in lines:
            if line.startswith('Process 2.1 Extract the DAG runtime:'):
                extract_dag_time = line.split(':')[1].strip()

        # Write to the CSV file
        writer.writerow([file_name, extract_dag_time])

print("Recording completed.")