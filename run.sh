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
    ensure_dir "e-rewriter/random_graph"
    ensure_dir "extraction-gym/input"
    ensure_dir "extraction-gym/out_dag_json"
    ensure_dir "extraction-gym/out_json"
    ensure_dir "extraction-gym/output_log"
    ensure_dir "process_json/input_saturacted_egraph"
    ensure_dir "process_json/input_extracted_egraph"
    ensure_dir "process_json/out_process_dag_result"
    ensure_dir "extraction-gym/random_out_dag_json/"
    echo -e "${GREEN}Setup complete.${RESET}\n"
}

# Function to get user input
get_user_input() {
    read -p "Enter the number of iteration times (optional): " iteration_times
    read -p "Enter the cost function for extraction-gym (optional, could be 'area' or 'delay'): " cost_function
    read -p "Enter the extraction pattern for e-rewriter (optional, could be 'faster-bottom-up' or 'random-based-faster-bottom-up', etc): " pattern

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
    start_time_process_rw=$(date +%s.%N)
    change_dir "e-rewriter/"
    execute_command "$feature_cmd circuit0.eqn $iteration_times"
    change_dir ".."
    copy_file "e-rewriter/rewritten_circuit/rewritten_egraph_with_weight_cost_serd.json" "extraction-gym/input/"

    echo -e "${YELLOW}Running extraction gym...${RESET}"
    change_dir "extraction-gym/"

    # Creating the output directory if it doesn't exist
    OUTPUT_DIR="output_log"
    #ext="faster-bottom-up"
    mkdir -p ${OUTPUT_DIR}

    # running the extraction process
    data="input/rewritten_egraph_with_weight_cost_serd.json"
    base_name=$(basename "${data}" .json)
    out_file="${OUTPUT_DIR}/log-${base_name}-${ext}.json"

    echo "Running extractor for ${data} with ${ext}"
    target/release/extraction-gym "${data}" --cost-function="${cost_function}" --extractor="${pattern}" --out="${out_file}"

    change_dir ".."

    end_time_process_rw=$(date +%s.%N)
    runtime_process_rw=$(echo "$end_time_process_rw - $start_time_process_rw" | bc)
    echo -e "${GREEN}Process 1 - Rewrite circuit completed.${RESET}"
}

# Function to extract the DAG and process JSON
extract_dag_and_process_json() {
    echo -e "${YELLOW}<-----------------------------Process 2: Extract the DAG and Process JSON----------------------------->${RESET}"
    start_time_process_process_json=$(date +%s.%N)

    #change_dir "extraction-gym/random_result/"
    #change_dir "-"

    # copy saturated egraph from extraction-gym/input/ to process_json/input_saturacted_egraph
    copy_file "extraction-gym/input/rewritten_egraph_with_weight_cost_serd.json" "process_json/input_saturacted_egraph/"

    # if pattern contains `random`

    if [[ "$pattern" == *"random"* ]]; then
        # copy all the file from extraction-gym/random_out_dag_json/ to process_json/input_extracted_egraph/
        for file in extraction-gym/random_out_dag_json/*; do
            copy_file "$file" "process_json/input_extracted_egraph/"
        done
        change_dir "process_json/"
        input_saturacted_egraph_path="input_saturacted_egraph/rewritten_egraph_with_weight_cost_serd.json"
        # make a for loop to parallel execute process_json for each extracted egraph
        for file in input_extracted_egraph/*; do
            input_extracted_egraph_path="$file"
            output_path="out_process_dag_result/${file##*/}"
            execute_command "target/release/process_json -s ${input_saturacted_egraph_path} -e ${input_extracted_egraph_path} -o ${output_path} -g"
        done
        change_dir ".."

        # Copying the output of process_json to the extraction-gym/out_json/rewritten_egraph_with_weight_cost_serd_${pattern}_${file##*/}.json
        echo -e "${YELLOW}Copying rewritten and extracted egraph files ... Prepare graph for Equation conversion.${RESET}"
        for file in process_json/out_process_dag_result/*; do
            copy_file "$file" "graph2eqn/${file##*/}"
        done
    else
        # copy extracted extraction-gym/out_dag_json/* to process_json/input_extracted_egraph/
        copy_file "extraction-gym/out_dag_json/rewritten_egraph_with_weight_cost_serd_${pattern}.json" "process_json/input_extracted_egraph/"
        change_dir "process_json/"

        input_saturacted_egraph_path="input_saturacted_egraph/rewritten_egraph_with_weight_cost_serd.json"
        input_extracted_egraph_path="input_extracted_egraph/rewritten_egraph_with_weight_cost_serd_${pattern}.json"
        output_path="out_process_dag_result/rewritten_egraph_with_weight_cost_serd_${pattern}.json"

        execute_command "target/release/process_json -s ${input_saturacted_egraph_path} -e ${input_extracted_egraph_path} -o ${output_path} -g"
        change_dir ".."

        # Copying the output of process_json to the extraction-gym/out_json/rewritten_egraph_with_weight_cost_serd_${pattern}.json
        echo -e "${YELLOW}Copying rewritten and extracted egraph files ... Prepare graph for Equation conversion.${RESET}"
        copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_${pattern}.json" "graph2eqn/result.json"
    fi

    

    end_time_process_process_json=$(date +%s.%N)
    runtime_process_process_json=$(echo "$end_time_process_process_json - $start_time_process_process_json" | bc)
    echo -e "${GREEN}Process 2 - Extract DAG and Process JSON completed.${RESET}"
}

# Function to convert graph to equation
graph_to_equation() {
    echo -e "${YELLOW}<-----------------------------Process 3: Graph to Equation ----------------------------------------------->${RESET}"
    start_time_process_graph2eqn=$(date +%s.%N)
    change_dir "graph2eqn/"
    # if pattern contains `random`
    if [[ "$pattern" == *"random"* ]]; then
        # for file of .json in current directory
        for file in ./*.json; do
            execute_command "target/release/graph2eqn $file" # 0 means do not check cyclic
            # change name for circuit0.eqn to circuit0_{i}.eqn
            # extract the number of filename
            index=$(echo "$file" | grep -oP '(?<=_)\d+(?=\.json)')
            mv "circuit0.eqn" "circuit0_$index.eqn"
            copy_file "circuit0_$index.eqn" "../abc/opt_$index.eqn"
        done
        change_dir ".."
    else
        execute_command "target/release/graph2eqn result.json" # 0 means do not check cyclic
        change_dir ".."
        copy_file "graph2eqn/circuit0.eqn" "abc/opt.eqn"
    fi
    end_time_process_graph2eqn=$(date +%s.%N)
    runtime_process_graph2eqn=$(echo "$end_time_process_graph2eqn - $start_time_process_graph2eqn" | bc)
    echo -e "${GREEN}Process 3 - Graph to Equation completed.${RESET}"
}

# Function to run ABC on the original and optimized circuit
run_abc() {
    echo -e "${YELLOW}<------------------------------Process 4: Run ABC on the original and optimized circuit, and conduct equivalent checking------------------->${RESET}"
    copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
    start_time_process_abc=$(date +%s.%N)

    change_dir "abc/"

    execute_command "./abc -c \"read_eqn ori.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    
    if [[ "$pattern" == *"random"* ]]; then
        # make a file in ../tmp_log/abc_opt_all.log with head `    i   o  Gates Area  Delay`
        echo "    i   o  Gates Area  Delay" > ../tmp_log/abc_opt_all.log
        for file in ./opt_*.eqn; do
            execute_command "./abc -c \"read_eqn $file;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime -d\"" >> ../tmp_log/abc_opt_all.log
        done
    else
        execute_command "./abc -c \"read_eqn opt.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    fi

    end_time_process_abc=$(date +%s.%N)
    runtime_process_abc=$(echo "$end_time_process_abc - $start_time_process_abc" | bc)
    echo -e "${GREEN}Process 4 - Run ABC on the original and optimized circuit completed.${RESET}"
}

# Function to compare original and optimized circuit
compare_circuits() {
    echo -e "${YELLOW}<-----------------------------Final Step: Comparing Original and Optimized Circuit----------------------------->${RESET}"
    if [[ "$pattern" == *"random"* ]]; then
        for file in ./opt_*.eqn; do
            execute_command "./abc -c \"cec ori.eqn $file\""
        done
    else
        execute_command "./abc -c \"cec ori.eqn opt.eqn\""
    fi

    change_dir ".."
}

# Function to report total runtime
report_runtime() {
    echo -e "${GREEN}All processes completed successfully.${RESET}"

    echo -e "${GREEN}Rewrite circuit completed in ${RED}$runtime_process_rw${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Extract DAG and Process JSON completed in ${RED}$runtime_process_process_json${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Graph to Equation in ${RED}$runtime_process_graph2eqn${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Run ABC on the original and optimized circuit completed in ${RED}$runtime_process_abc${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Total runtime: ${RED}$(echo "scale=2; $runtime_process_rw + $runtime_process_process_json + $runtime_process_graph2eqn + $runtime_process_abc" | bc)${GREEN} seconds.${RESET}"
}

# Main script
feature_cmd="./target/release/e-rewriter"
echo -e "${YELLOW}Using feature label: ${feature}${RESET}"

setup_directories
get_user_input 
rewrite_circuit # eqn2egraph, rewrite
extract_dag_and_process_json # extract from saturated egraph, process json
graph_to_equation # egraph2eqn
run_abc
compare_circuits  # logic equivalence check
report_runtime # report runtime