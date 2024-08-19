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


# Function to set up required directories
setup_directories() {
    echo -e "${GREEN}Setting up required directories...${RESET}"
    ensure_dir "e-rewriter/rewritten_circuit"
    ensure_dir "extraction-gym/input"
    ensure_dir "extraction-gym/out_dag_json"
    ensure_dir "extraction-gym/out_json"
    ensure_dir "extraction-gym/output_log"
    echo -e "${GREEN}Setup complete.${RESET}\n"
}

# Function to get user input
get_user_input() {
    read -p "Enter the number of iteration times (optional, default: 30): " iteration_times
    iteration_times=${iteration_times:-30}

    read -p "Enter the cost function for extraction-gym (optional, could be 'area' or 'delay', default: 'area'): " cost_function
    cost_function=${cost_function:-"area"}

    read -p "Enter the extraction pattern for e-rewriter (optional, default: 'faster-bottom-up'): " pattern
    pattern=${pattern:-"faster-bottom-up"}

    # if cost_function is 'area', replace it with 'node_sum_cost', if it is 'delay', replace it with 'node_depth_cost'
    if [ "$cost_function" == "area" ]; then
        cost_function="node_sum_cost"
    elif [ "$cost_function" == "delay" ]; then
        cost_function="node_depth_cost"
    fi
}

# Function to rewrite the circuit
rewrite_circuit() {
    echo -e "${YELLOW}<-----------------------------Process 1: Rewrite the Circuit----------------------------->${RESET}"
    change_dir "e-rewriter/"
    copy_file "circuit0.eqn" "../abc/circuit0_opt.eqn"
    change_dir "../abc"
    execute_command "./abc -c \"read_eqn circuit0_opt.eqn; st; write_eqn circuit0_opt.eqn\""
    copy_file "circuit0_opt.eqn" "../e-rewriter/circuit0_opt.eqn"
    change_dir "../e-rewriter"
    start_time_process_rw=$(date +%s.%N)
    execute_command "$feature_cmd circuit0_opt.eqn $iteration_times"
    change_dir ".."
    copy_file "e-rewriter/rewritten_circuit/rewritten_egraph_with_weight_cost_serd.json" "extraction-gym/input/"

    end_time_process_rw=$(date +%s.%N)
    runtime_process_rw=$(echo "$end_time_process_rw - $start_time_process_rw" | bc)
    echo -e "${GREEN}Process 1 - Rewrite circuit completed.${RESET}"
}

# Function to extract the DAG
extract_dag() {
    echo -e "${YELLOW}<-----------------------------Process 2: Extract DAG------------------------------>${RESET}"
    start_time_process_extract=$(date +%s.%N)
    echo -e "${YELLOW}Running extraction gym...${RESET}"
    change_dir "extraction-gym/"

    OUTPUT_DIR="output_log"
    mkdir -p ${OUTPUT_DIR}

    data="input/rewritten_egraph_with_weight_cost_serd.json"
    base_name=$(basename "${data}" .json)
    out_file="${OUTPUT_DIR}/log-${base_name}-${pattern}.json"

    echo "Running extractor for ${data} with ${pattern}"

    target/release/extraction-gym "${data}" --cost-function="${cost_function}" --extractor="${pattern}" --out="${out_file}"

    change_dir ".."
    end_time_process_extract=$(date +%s.%N)
    runtime_process_extract=$(echo "$end_time_process_extract - $start_time_process_extract" | bc)
    echo -e "${GREEN}Process 2 - Extract DAG completed.${RESET}"
}

run_abc() {
    echo -e "${YELLOW}<-----------------------------Process 3: Run ABC------------------------------>${RESET}"
    copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
    copy_file "extraction-gym/src/extract/tmp/output.eqn" "abc/opt.eqn"
    change_dir "abc"

    # baline - single operator - if -g
    execute_command "./abc -c \"read_eqn ori.eqn; read_lib asap7_clean.lib; if -g; st;dch; ps; map; topo; upsize; dnsize; stime;\""

    # baseline - single operator
    execute_command "./abc -c \"read_eqn opt.eqn; read_lib asap7_clean.lib; st; dch; ps; map; topo; upsize; dnsize; stime;\""
    
    # combinational equivalence checking
    execute_command "./abc -c \"cec ori.eqn opt.eqn\""

    change_dir ".."
    echo -e "${GREEN}Process 3 - Run ABC completed.${RESET}"
}

# Function to report total runtime
report_runtime() {
    echo -e "${GREEN}All processes completed successfully.${RESET}"

    echo -e "${GREEN}Rewrite circuit completed in ${RED}$runtime_process_rw${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Extract DAG completed in ${RED}$runtime_process_extract${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Total runtime: ${RED}$(echo "scale=2; $runtime_process_rw + $runtime_process_extract" | bc)${GREEN} seconds.${RESET}"
}

# Main script
feature_cmd="./target/x86_64-unknown-linux-musl/release/e-rewriter"
echo -e "${YELLOW}Using feature label: ${feature}${RESET}"

setup_directories
get_user_input 
rewrite_circuit
extract_dag
run_abc
report_runtime