import json
import networkx as nx
import matplotlib.pyplot as plt
import pygraphviz as pgv
from IPython.display import Image
import re

# Your JSON data
# read data from json file
with open('/data/guangyuh/coding_env/E-syn2_power/graph2eqn/result.json', 'r') as f:
    data = json.load(f)

# Create a directed graph
dag = pgv.AGraph(directed=True)

# Add nodes and edges to the graph
for node, attributes in data['nodes'].items():
    dag.add_node(node, label=f"{node}\n{attributes['op']}")  
    for child in attributes['children']:
        dag.add_edge(node, child)  

# Helper function to resolve the 'op' for terminal nodes
def resolve_op(node_id):
    node_data = data['nodes'][node_id]
    if not node_data['children']:  # Terminal node
        return node_data['op']
    if node_data['op'] == '!':  # Unary operation
        child_id = node_data['children'][0]
        #return f"!{resolve_op(child_id)}"
        if data['nodes'][child_id]['op'] == '!':  # Double negation
            return f"{resolve_op(child_id)}"
        else:
            return f"!{resolve_op(child_id)}"
    # Prepend 'new_n_' to non-terminal node ID
    return f"new_n_{node_id}"

# Function to analyze the node connection and return the formatted string
def format_connection(node_id, node_data):
    operation = node_data['op']
    new_node_id = f"new_n_{node_id}"  # Modify the node ID for non-terminals
    
    # Check if the operation is unary (only one child)
    if len(node_data['children']) == 1:
        child_id = node_data['children'][0]
        #resolved_op = resolve_op(child_id)
            
    # If the node is binary (two children)
    elif len(node_data['children']) == 2:
        if operation == '&':  # Inline the connection for '&' nodes
            return
        left_id, right_id = node_data['children']
        left_resolved_op = resolve_op(left_id)
        right_resolved_op = resolve_op(right_id)
        return f"{new_node_id} = {left_resolved_op} {operation} {right_resolved_op}"
    
    # If the node has no children
    else:
        return ""
def replace_variables(entry, output_index_lst):
    # Sort the output_index_lst in descending order to handle longest names first
    #output_index_lst_sorted = sorted(output_index_lst, key=lambda x: -x)

    for index, value in enumerate(output_index_lst):
        # Create a pattern that matches "new_n_<value>" exactly, not as a substring
        pattern = re.compile(r'\bnew_n_' + str(value) + r'\b')
        # Replace with "p[index]"
        entry = pattern.sub(f"p[{index}]", entry)

    # Append `;` to the end of each entry
    entry += ";"
    return entry

def find_all_output_index(node_id, node_data):
    # recursivly find all the '&' node's children and inline connections
    output_index_lst = []
    amp_node_lst = [(data['root_eclasses'][0], 0)]  # initialize the amp_node to the root_eclasses and its index
    amp_node = (data['root_eclasses'][0], 0)  # initialize the amp_node to the root_eclasses and its index
    index = 1

    if data['nodes'][data['nodes'][amp_node[0]]['children'][0]]['op'] == '&':
        successors_id = data['nodes'][amp_node[0]]['children'][0]
        output_index_lst.append(data['nodes'][amp_node[0]]['children'][1])
    elif data['nodes'][data['nodes'][amp_node[0]]['children'][1]]['op'] == '&':
        successors_id = data['nodes'][amp_node[0]]['children'][1]
        output_index_lst.append(data['nodes'][amp_node[0]]['children'][0])

    while True:
        amp_node_lst.append((successors_id, index))  # append the left child of the amp_node
        index += 1
        amp_node = amp_node_lst[-1]  # move to the left child
        if data['nodes'][data['nodes'][amp_node[0]]['children'][0]]['op'] == '&':
            successors_id = data['nodes'][amp_node[0]]['children'][0]
            output_index_lst.append(data['nodes'][amp_node[0]]['children'][1])
        elif data['nodes'][data['nodes'][amp_node[0]]['children'][1]]['op'] == '&':
            successors_id = data['nodes'][amp_node[0]]['children'][1]  # find the left child of the amp_node
            output_index_lst.append(data['nodes'][amp_node[0]]['children'][0])
        else:
            output_index_lst.extend(
                (
                    data['nodes'][amp_node[0]]['children'][1],
                    data['nodes'][amp_node[0]]['children'][0],
                )
            )
            break
    return output_index_lst, amp_node_lst
        
output_index_lst, amp_node_lst = find_all_output_index(data['root_eclasses'][0], data['nodes'][data['root_eclasses'][0]])
# convert string to int in output_index_lst
output_index_lst = [int(i) for i in output_index_lst]
# reverse the order of output_index_lst
output_index_lst = output_index_lst[::-1] # #output_index_lst = [10, 23, 24]

# Create the table of connections
table = [format_connection(node_id, node_data) for node_id, node_data in data['nodes'].items() if node_data['children']]

table_modified = []
# Output the table from the script
for entry in table:
    if entry:  # Only print non-empty entries
        #for value in output_index_lst:
            # if entry string constants new_n_output_index, replace it with the actual output index
        entry = replace_variables(entry, output_index_lst)
        # append `;` to the end of each entry`
        #entry += ";"
        #print(entry)
        table_modified.append(entry)

# print all the amp_node in amp_node_lst
#for amp_node in amp_node_lst:
    #print(amp_node)

#print(output_index_lst)

# remove None from table
table = table_modified

# extract all the 

# if amp_node:
#     amp_children = data['nodes'][amp_node]['children']
#     inlined_connections = [resolve_op(child) for child in amp_children]
#     print(f"\nThe '&' node's children are: {', '.join(inlined_connections)}")
# else:
#     print("\nThere is no '&' node present in the DAG.")



def replace_symbols(text, mapping):
    # Create a regular expression pattern that matches all keys in the mapping dict
    # The pattern is adjusted to capture keys even if surrounded by non-word characters
    pattern = re.compile(r'(?<!\w)(' + '|'.join(re.escape(key) for key in sorted(mapping.keys(), key=len, reverse=True)) + r')(?!\w)')
    
    # The replacement function takes a match object and returns the corresponding replacement
    def replace_func(match):
        return mapping[match.group(0)]
    
    # Use re.sub() to substitute occurrences using the pattern and replacement function
    modified_text = pattern.sub(replace_func, text)
    
    return modified_text


def read_prefix_mapping(file_path):
    output_mapping = {}
    with open(file_path, 'r') as file:
        for line in file:
            line = line.strip()
            if line.startswith("INORDER = "):
                input_variables = line.replace("INORDER = ", "").replace(';', '').split()
            if line.startswith("OUTORDER = "):
                parts = line.replace("OUTORDER = ", "").replace(';', '').split()
                output_mapping = {f"p[{index}]": part for index, part in enumerate(parts)}
                break
    return output_mapping, input_variables

def write_to_file(file_name, prefix_mapping, variables):
    with open(file_name, 'w') as file:
        file.write(f"INORDER = {' '.join(variables)};\n")
        #outorder = [prefix_mapping.get(f"p[{i}]", f"p[{i}]") for i in range(len(table))]
        outorder = []
        for line in table:
            if line.startswith("p["):
                index = int(line.split("[")[1].split("]")[0])
                outorder.append(prefix_mapping.get(f"p[{index}]", f"p[{index}]"))
        file.write(f"OUTORDER = {' '.join(outorder)};\n")

        for part in table:
            # extract all po[index] in string, find mapping in prefix_mapping, replace with new mapping
            file.write(replace_symbols(part, prefix_mapping) + "\n")
                
                
            # else:
            #     file.write(f"{part};\n")

        # for expr in table:
        #     file.write(expr + "\n")

output_prefix_mapping, input_variables_name = read_prefix_mapping("/data/guangyuh/coding_env/E-syn2_power/e-rewriter/circuit0.eqn")
#print(output_prefix_mapping)
write_to_file("/data/guangyuh/coding_env/E-syn2_power/circuit0.eqn", output_prefix_mapping, input_variables_name)