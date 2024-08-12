import json
import re
import pygraphviz as pgv # conda install --channel conda-forge pygraphviz

sub_times = 0 # substitute times for replacing variables with output index
record_visite_output_index = []

def read_json_data(file_path):
    with open(file_path, 'r') as file:
        return json.load(file)

def create_graph(data):
    dag = pgv.AGraph(directed=True)
    for node, attributes in data['nodes'].items():
        dag.add_node(node, label=f"{node}\n{attributes['op']}")  
        if attributes['children'] != []:
            for child in attributes['children']:
                dag.add_edge(node, child)
    return dag

def resolve_op(node_id, data):
    node_data = data['nodes'][node_id]
    if not node_data['children']:
        return node_data['op']
    if node_data['op'] == '!':
        child_id = node_data['children'][0]
        if data['nodes'][child_id]['op'] == '!':
            #assert False, "Double negation is not supported"
            return resolve_op(child_id, data)
            #return f"new_n_{child_id}"
        else:
            #return f"!new_n_{child_id}"
            return f"!{resolve_op(child_id, data)}"
    return f"new_n_{node_id}"

def format_connection(node_id, node_data, data):
    operation = node_data['op']
    new_node_id = f"new_n_{node_id}"
    
    if len(node_data['children']) == 1 and operation == '!':
        #child_id = node_data['children'][0]
        solved_children = resolve_op(node_data['children'][0], data)
        return f"{new_node_id} = {operation}{solved_children}"
    elif len(node_data['children']) == 0:
        return f"{new_node_id} = {operation}"
    elif len(node_data['children']) == 2:
        if operation == '&':
            return
        left_id, right_id = node_data['children']
        left_resolved_op = resolve_op(left_id, data)
        right_resolved_op = resolve_op(right_id, data)
        return f"{new_node_id} = {left_resolved_op} {operation} {right_resolved_op}"
    
    else:
        return ""

def replace_variables(entry, output_index_lst):
    global sub_times
    global record_visite_output_index
    sub = False
    for index, value in enumerate(output_index_lst):
        pattern = re.compile(r'\bnew_n_' + str(value) + r'\b')
        new_entry = pattern.sub(f"p[{index}]", entry)
        # if pattern.sub successfully replaces the pattern, then sub is True
        if new_entry != entry:
            sub = True
            record_visite_output_index.append(value)
            entry = new_entry
    if sub: sub_times += 1
    entry += ";"
    return entry

def find_all_output_index(root_eclass, data):
    output_index_lst = []
    amp_node_lst = [(root_eclass, 0)]
    amp_node = (root_eclass, 0)
    index = 1

    while True:
        children = data['nodes'][amp_node[0]]['children']
        if data['nodes'][children[0]]['op'] == '&':
            successors_id = children[0]
            #if data['nodes'][children[1]]['op'] == '0' or data['nodes'][children[1]]['op'] == '1': # output = 0 or 1 in original circuit
            output_index_lst.append(children[1])
        elif data['nodes'][children[1]]['op'] == '&':
            assert False, "& on the right side of the root node is not supported"
            successors_id = children[1]
            output_index_lst.append(children[0])
        else:
            output_index_lst.extend(children[::-1])
            break
            
        amp_node_lst.append((successors_id, index))
        index += 1
        amp_node = amp_node_lst[-1]
        
    return [int(i) for i in output_index_lst[::-1]], amp_node_lst

def create_table(data, output_index_lst):
    table = [format_connection(node_id, node_data, data) for node_id, node_data in data['nodes'].items()]
    return [replace_variables(entry, output_index_lst) for entry in table if entry]

def replace_symbols_in_text(text, symbol_mapping):
    pattern = re.compile(r'(?<!\w)(' + '|'.join(re.escape(key) for key in sorted(symbol_mapping.keys(), key=len, reverse=True)) + r')(?!\w)')
    
    def get_replacement(match):
        return symbol_mapping[match.group(0)]
    
    return pattern.sub(get_replacement, text)

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

def write_equations_to_file(file_name, prefix_mapping, variables):
    with open(file_name, 'w') as file:
        write_inorder(file, variables)
        output_variables = get_output_variables(prefix_mapping)
        write_outorder(file, output_variables) 
        write_equations(file, prefix_mapping)

def write_inorder(file, variables):
    file.write(f"INORDER = {' '.join(variables)};\n")

def get_output_variables(prefix_mapping):
    output_variables = []
    for line in table:
        if line.startswith("p["):
            index = extract_index(line)
            output_variables.append(prefix_mapping.get(f"p[{index}]", f"p[{index}]"))
    return output_variables

def write_outorder(file, output_variables):
    file.write(f"OUTORDER = {' '.join(output_variables)};\n")  

def write_equations(file, prefix_mapping):
    for equation in table:
        file.write(replace_symbols_in_text(equation, prefix_mapping) + "\n")
        
def extract_index(line):
    return int(line.split("[")[1].split("]")[0])

# if __name__ == "__main__":
#     sub_times = 0
#     data = read_json_data('test_for_graph2eqn.json')
#     dag = create_graph(data)
#     root_eclass = data['root_eclasses'][0]
#     output_index_lst, amp_node_lst = find_all_output_index(root_eclass, data)
#     table = create_table(data, output_index_lst)
#     output_prefix_mapping, input_variables_name = read_prefix_mapping("test_for_graph2eqn.eqn")
#     #assert len(table) == 15
#     write_equations_to_file("circuit0.eqn", output_prefix_mapping, input_variables_name)


if __name__ == "__main__":
    sub_times = 0
    data = read_json_data('graph2eqn/result.json')
    dag = create_graph(data)
    root_eclass = data['root_eclasses'][0]
    output_index_lst, amp_node_lst = find_all_output_index(root_eclass, data)
    table = create_table(data, output_index_lst)
    output_prefix_mapping, input_variables_name = read_prefix_mapping("e-rewriter/circuit0.eqn")
    write_equations_to_file("circuit0.eqn", output_prefix_mapping, input_variables_name)