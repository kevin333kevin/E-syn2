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
ensure_dir "e-rewriter/dot_graph"
ensure_dir "extraction-gym/data/my_data"
ensure_dir "extraction-gym/data/egg"
ensure_dir "extraction-gym/out_dag_json/my_data"
ensure_dir "extraction-gym/out_json/my_data"
ensure_dir "extraction-gym/output/egg"
ensure_dir "extraction-gym/output/my_data"

echo -e "${GREEN}Setup complete.${RESET}\n"

# Get user input for iteration times and feature label
read -p "Enter the number of iteration times: " iteration_times
read -p "Enter the feature label (optional): " feature

# Helper info for user input
if [ -z "$feature" ]; then
    echo -e "${YELLOW}Feature label not provided, proceeding without it.${RESET}"
    feature_cmd="./target/release/e-rewriter"
else
    feature_cmd="./target/release/e-rewriter-${feature}"
    echo -e "${YELLOW}Using feature label: ${feature}${RESET}"
fi


# Process 1: Rewrite the circuit
echo -e "${YELLOW}<-----------------------------Process 1: Rewrite the Circuit----------------------------->${RESET}"
start_time_process_rw=$(date +%s.%N)
change_dir "e-rewriter/"
execute_command "$feature_cmd circuit0.eqn $iteration_times"
change_dir ".."
copy_file "e-rewriter/dot_graph/graph_internal_serd.json" "extraction-gym/data/my_data/"
end_time_process_rw=$(date +%s.%N)
runtime_process_rw=$(echo "$end_time_process_rw - $start_time_process_rw" | bc)
echo -e "${GREEN}Process 1 - Rewrite circuit completed.${RESET}"

# Process 2: Extract the DAG and Process JSON
echo -e "${YELLOW}<-----------------------------Process 2: Extract the DAG and Process JSON----------------------------->${RESET}"
start_time_process_process_json=$(date +%s.%N)
copy_file "e-rewriter/result.json" "extraction-gym/out_json/my_data"
change_dir "process_json/"
execute_command "target/release/process_json"
change_dir ".."
copy_file "process_json/out_process_result/result.json" "graph2eqn/result.json"
end_time_process_process_json=$(date +%s.%N)
runtime_process_process_json=$(echo "$end_time_process_process_json - $start_time_process_process_json" | bc)
echo -e "${GREEN}Process 2 - Extract DAG and Process JSON completed.${RESET}"

# Process 3: Convert Graph to Equation and Evaluate
echo -e "${YELLOW}<-----------------------------Process 3: Graph to Equation ----------------------------------------------->${RESET}"
start_time_process_graph2eqn=$(date +%s.%N)
change_dir "graph2eqn/"
execute_command "target/release/graph2eqn result.json"
change_dir ".."
copy_file "graph2eqn/circuit0.eqn" "abc/op2.eqn"
end_time_process_graph2eqn=$(date +%s.%N)
runtime_process_graph2eqn=$(echo "$end_time_process_graph2eqn - $start_time_process_graph2eqn" | bc)
echo -e "${GREEN}Process 3 - Graph to Equation completed.${RESET}"

# Process 4: Run ABC on the original and optimized circuit
echo -e "${YELLOW}<------------------------------Process 4: Run ABC on the original and optimized circuit, and conduct equivalent checking------------------->${RESET}"
copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
start_time_process_abc=$(date +%s.%N)
change_dir "abc/"
execute_command "./abc -c \"read_eqn ori.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
execute_command "./abc -c \"read_eqn op2.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""

end_time_process_abc=$(date +%s.%N)
runtime_process_abc=$(echo "$end_time_process_abc - $start_time_process_abc" | bc)
echo -e "${GREEN}Process 4 - Run ABC on the original and optimized circuit completed.${RESET}"

# Final Step: Compare Original and Optimized Circuit
echo -e "${YELLOW}<-----------------------------Final Step: Comparing Original and Optimized Circuit----------------------------->${RESET}"
execute_command "./abc -c \"cec ori.eqn op2.eqn\""

change_dir ".."

# Report total runtime
echo -e "${GREEN}All processes completed successfully.${RESET}"

echo -e "${GREEN}Rewrite circuit completed in $runtime_process_rw seconds.${RESET}"
echo -e "${GREEN}Extract DAG and Process JSON completed in $runtime_process_process_json seconds.${RESET}"
echo -e "${GREEN}Graph to Equation in $runtime_process_graph2eqn seconds.${RESET}"
echo -e "${GREEN}Run ABC on the original and optimized circuit completed in $runtime_process_abc seconds.${RESET}"
echo -e "${GREEN}Total runtime: $(echo "scale=2; $runtime_process_rw + $runtime_process_process_json + $runtime_process_graph2eqn + $runtime_process_abc" | bc) seconds.${RESET}"