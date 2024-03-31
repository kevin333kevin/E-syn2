#!/bin/bash
RED="\e[31m"
GREEN="\e[32m"
YELLOW="\e[1;33m"
RESET="\e[0m"

# Utility function for creating directories if they do not exist
ensure_dir() {
    if [ ! -d "$1" ]; then
        mkdir -p "$1" || { echo -e "${RED}Failed to create directory $1${RESET}"; exit 1; }
    fi
}

# Utility function for changing directories safely
change_dir() {
    cd "$1" || { echo -e "${RED}Failed to change directory to $1${RESET}"; exit 1; }
}

# Utility function for copying files safely
copy_file() {
    cp "$1" "$2" || { echo -e "${RED}Failed to copy $1 to $2${RESET}"; exit 1; }
}

# Utility function to execute a command and handle failure
execute_command() {
    eval "$1" || { echo -e "${RED}Failed to execute command: $1${RESET}"; exit 1; }
}

echo -e "${GREEN}Setting up required directories...${RESET}"

# Setup directories
ensure_dir "e-rewriter/rewritten_circuit"
ensure_dir "e-rewriter/random_graph"
ensure_dir "extraction-gym/input"
#ensure_dir "extraction-gym/input/egg"
ensure_dir "extraction-gym/out_dag_json"
ensure_dir "extraction-gym/out_json"
#ensure_dir "extraction-gym/output/egg"
ensure_dir "extraction-gym/output_log"
ensure_dir "process_json/input_saturacted_egraph"
ensure_dir "process_json/input_extracted_egraph"
#ensure_dir "extraction-gym/input"
#ensure_dir "extraction-gym/input/egg"

echo -e "${GREEN}Setup complete.${RESET}\n"

# Get user input for iteration times and feature label
read -p "Enter the number of iteration times (optional): " iteration_times
read -p "Enter the cost function for extraction-gym (optional, could be 'area' or 'delay'): " cost_function
read -p "Enter the extraction pattern for e-rewriter (optional, could be 'random'): " pattern

# if cost_function is 'area', replace it with 'node_sum_cost', if it is 'delay', replace it with 'node_depth_cost'
if [ "$cost_function" == "area" ]; then
    cost_function="node_sum_cost"
elif [ "$cost_function" == "delay" ]; then
    cost_function="node_depth_cost"
fi


# feature="dag_cost"
# feature_cmd="./target/release/e-rewriter-${feature}"
feature_cmd="./target/release/e-rewriter"
echo -e "${YELLOW}Using feature label: ${feature}${RESET}"

# Process 1: Rewrite the circuit
echo -e "${YELLOW}<-----------------------------Process 1: Rewrite the Circuit----------------------------->${RESET}"
start_time_process_rw=$(date +%s.%N)
change_dir "e-rewriter/"
execute_command "$feature_cmd circuit0.eqn $iteration_times $pattern"
change_dir ".."
copy_file "e-rewriter/rewritten_circuit/rewritten_egraph_with_weight_cost_serd.json" "extraction-gym/input/"

echo -e "${YELLOW}Running extraction gym...${RESET}"
change_dir "extraction-gym/"

# Creating the output directory if it doesn't exist
OUTPUT_DIR="output_log"
ext="faster-bottom-up"
mkdir -p ${OUTPUT_DIR}

# running the extraction process
data="input/rewritten_egraph_with_weight_cost_serd.json"
base_name=$(basename "${data}" .json)
out_file="${OUTPUT_DIR}/log-${base_name}-${ext}.json"

echo "Running extractor for ${data} with ${ext}"
target/release/extraction-gym "${data}" --cost-function="${cost_function}" --extractor="${ext}" --out="${out_file}"

change_dir ".."

end_time_process_rw=$(date +%s.%N)
runtime_process_rw=$(echo "$end_time_process_rw - $start_time_process_rw" | bc)
echo -e "${GREEN}Process 1 - Rewrite circuit completed.${RESET}"

# Process 2: Extract the DAG and Process JSON
echo -e "${YELLOW}<-----------------------------Process 2: Extract the DAG and Process JSON----------------------------->${RESET}"
start_time_process_process_json=$(date +%s.%N)

#copy_file "extraction-gym/random_result/result9.json" "extraction-gym/out_dag_json/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json"

# randomly choose a result file under random_result/ to copy

# Navigate to the directory containing the result files
change_dir "extraction-gym/random_result/"

# Randomly choose one of the result*.json files and copy it
# find . -name 'result*.json' | shuf -n 1 | xargs -I{} cp {} ../out_dag_json/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json

change_dir "-"

# copy saturacted egraph from extraction-gym/input/ to process_json/input_saturacted_egraph
copy_file "extraction-gym/input/rewritten_egraph_with_weight_cost_serd.json" "process_json/input_saturacted_egraph/"
# copy extracted extraction-gym/out_dag_json/* to process_json/input_extracted_egraph/
copy_file "extraction-gym/out_dag_json/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json" "process_json/input_extracted_egraph/"

change_dir "process_json/"

execute_command "target/release/process_json --graph-file input_saturacted_egraph/rewritten_egraph_with_weight_cost_serd.json --extraction-result-dir input_extracted_egraph/ --extract-dag"
wait
change_dir ".."

# Copying the output of process_json to the extraction-gym/out_json/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json
echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json ... Prepare graph for Equation conversion.${RESET}"
copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json" "graph2eqn/result.json" 

end_time_process_process_json=$(date +%s.%N)
runtime_process_process_json=$(echo "$end_time_process_process_json - $start_time_process_process_json" | bc)
echo -e "${GREEN}Process 2 - Extract DAG and Process JSON completed.${RESET}"

# Process 3: Convert Graph to Equation and Evaluate
echo -e "${YELLOW}<-----------------------------Process 3: Graph to Equation ----------------------------------------------->${RESET}"
start_time_process_graph2eqn=$(date +%s.%N)
change_dir "graph2eqn/"
execute_command "target/release/graph2eqn result.json" # 0 means do not check cyclic
change_dir ".."
copy_file "graph2eqn/circuit0.eqn" "abc/opt.eqn"
end_time_process_graph2eqn=$(date +%s.%N)
runtime_process_graph2eqn=$(echo "$end_time_process_graph2eqn - $start_time_process_graph2eqn" | bc)
echo -e "${GREEN}Process 3 - Graph to Equation completed.${RESET}"

# Process 4: Run ABC on the original and optimized circuit
echo -e "${YELLOW}<------------------------------Process 4: Run ABC on the original and optimized circuit, and conduct equivalent checking------------------->${RESET}"
copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
start_time_process_abc=$(date +%s.%N)
change_dir "abc/"
execute_command "./abc -c \"read_eqn ori.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
execute_command "./abc -c \"read_eqn opt.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""

end_time_process_abc=$(date +%s.%N)
runtime_process_abc=$(echo "$end_time_process_abc - $start_time_process_abc" | bc)
echo -e "${GREEN}Process 4 - Run ABC on the original and optimized circuit completed.${RESET}"

# Final Step: Compare Original and Optimized Circuit
echo -e "${YELLOW}<-----------------------------Final Step: Comparing Original and Optimized Circuit----------------------------->${RESET}"
execute_command "./abc -c \"cec ori.eqn opt.eqn\""

change_dir ".."

# Report total runtime
echo -e "${GREEN}All processes completed successfully.${RESET}"

echo -e "${GREEN}Rewrite circuit completed in ${RED}$runtime_process_rw${GREEN} seconds.${RESET}"
echo -e "${GREEN}Extract DAG and Process JSON completed in ${RED}$runtime_process_process_json${GREEN} seconds.${RESET}"
echo -e "${GREEN}Graph to Equation in ${RED}$runtime_process_graph2eqn${GREEN} seconds.${RESET}"
echo -e "${GREEN}Run ABC on the original and optimized circuit completed in ${RED}$runtime_process_abc${GREEN} seconds.${RESET}"
echo -e "${GREEN}Total runtime: ${RED}$(echo "scale=2; $runtime_process_rw + $runtime_process_process_json + $runtime_process_graph2eqn + $runtime_process_abc" | bc)${GREEN} seconds.${RESET}"