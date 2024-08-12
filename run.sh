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
    read -p "Enter the number of iteration times (optional, default: 1): " iteration_times
    iteration_times=${iteration_times:-30}

    read -p "Enter the cost function for extraction-gym (optional, could be 'area' or 'delay', default: 'area'): " cost_function
    cost_function=${cost_function:-"area"}

    read -p "Enter the extraction pattern for e-rewriter (optional, could be 'faster-bottom-up' or 'random-based-faster-bottom-up', default: 'faster-bottom-up'): " pattern
    pattern=${pattern:-"faster-bottom-up"}

    # if pattern is provided with *random*
    if [[ "$pattern" == *"random"* ]]; then
        read -p "Enter the number of samplings for random pattern (optional, default: 10): " num_samplings
        num_samplings=${num_samplings:-30}

        read -p "Enter the probability of randomization (optional, default: 0.5): " prob_randomization
        prob_randomization=${prob_randomization:-0.1}
    fi

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
    #execute_command "../abc/abc -c \"read_eqn circuit0.eqn; write_eqn circuit0.eqn\""
    copy_file "circuit0.eqn" "../abc/circuit0_opt.eqn"
    change_dir "../abc"
    # alan
    #execute_command "./abc -c \"read_eqn circuit0_opt.eqn; if -K 6 -g -C 8; st; write_eqn circuit0_opt.eqn\"" 
    # lazyman
    #execute_command "./abc -c \"read_eqn circuit0_opt.eqn; recadd3_opt ; write_eqn circuit0_opt.eqn\"" 

    # baseline
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

    # Creating the output directory if it doesn't exist
    OUTPUT_DIR="output_log"
    #ext="faster-bottom-up"
    ext=${pattern}
    mkdir -p ${OUTPUT_DIR}

    # running the extraction process
    data="input/rewritten_egraph_with_weight_cost_serd.json"
    base_name=$(basename "${data}" .json)
    out_file="${OUTPUT_DIR}/log-${base_name}-${ext}.json"

    echo "Running extractor for ${data} with ${ext}"

    if [[ "$pattern" == *"random"* ]]; then
        target/release/extraction-gym "${data}" --cost-function="${cost_function}" --extractor="${pattern}" --out="${out_file}" --num-samples="${num_samplings}" --random-prob="${prob_randomization}"
    else
        target/release/extraction-gym "${data}" --cost-function="${cost_function}" --extractor="${pattern}" --out="${out_file}"
    fi

    change_dir ".."
    end_time_process_extract=$(date +%s.%N)
    runtime_process_extract=$(echo "$end_time_process_extract - $start_time_process_extract" | bc)
    echo -e "${GREEN}Process 2 - Extract DAG completed.${RESET}"
}


# Function to process JSON
process_json() {
    echo -e "${YELLOW}<-----------------------------Process 3: Process JSON----------------------------->${RESET}"
    start_time_process_process_json=$(date +%s.%N)

    copy_file "extraction-gym/input/rewritten_egraph_with_weight_cost_serd.json" "process_json/input_saturacted_egraph/"

    if [[ "$pattern" == *"random"* ]]; then
        for file in extraction-gym/random_out_dag_json/*; do
            copy_file "$file" "process_json/input_extracted_egraph/"
        done
        change_dir "process_json/"
        input_saturacted_egraph_path="input_saturacted_egraph/rewritten_egraph_with_weight_cost_serd.json"
        
        # Parallel execution of process_json for each extracted egraph
        ls input_extracted_egraph/* | parallel --eta "target/x86_64-unknown-linux-musl/release/process_json -s ${input_saturacted_egraph_path} -e {} -o out_process_dag_result/{/} -g"
        
        change_dir ".."

        echo -e "${YELLOW}Copying rewritten and extracted egraph files ... Prepare graph for Equation conversion.${RESET}"
        for file in process_json/out_process_dag_result/*; do
            copy_file "$file" "graph2eqn/${file##*/}"
        done
    else
        copy_file "extraction-gym/out_dag_json/rewritten_egraph_with_weight_cost_serd_${pattern}.json" "process_json/input_extracted_egraph/"
        change_dir "process_json/"

        input_saturacted_egraph_path="input_saturacted_egraph/rewritten_egraph_with_weight_cost_serd.json"
        input_extracted_egraph_path="input_extracted_egraph/rewritten_egraph_with_weight_cost_serd_${pattern}.json"
        output_path="out_process_dag_result/rewritten_egraph_with_weight_cost_serd_${pattern}.json"

        execute_command "target/x86_64-unknown-linux-musl/release/process_json -s ${input_saturacted_egraph_path} -e ${input_extracted_egraph_path} -o ${output_path} -g"
        change_dir ".."

        echo -e "${YELLOW}Copying rewritten and extracted egraph files ... Prepare graph for Equation conversion.${RESET}"
        copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_${pattern}.json" "graph2eqn/result.json"
    fi

    end_time_process_process_json=$(date +%s.%N)
    runtime_process_process_json=$(echo "$end_time_process_process_json - $start_time_process_process_json" | bc)
    echo -e "${GREEN}Process 3 - Extract DAG and Process JSON completed.${RESET}"
}

# Function to convert graph to equation
graph_to_equation() {
    echo -e "${YELLOW}<-----------------------------Process 4: Graph to Equation ----------------------------------------------->${RESET}"
    start_time_process_graph2eqn=$(date +%s.%N)
    change_dir "graph2eqn/"
    
    if [[ "$pattern" == *"random"* ]]; then
        # Parallel execution of graph2eqn for each JSON file
        ls ./*.json | parallel --eta 'target/x86_64-unknown-linux-musl/release/graph2eqn {} circuit_opt_{/}.eqn'
        #ls ./*.json | parallel --eta 'echo {/} | sed "s/[^0-9]*\([0-9]\+\).*/\1/" | xargs -I{} target/x86_64-unknown-linux-musl/release/graph2eqn {1} circuit_opt_{}.eqn' ::: {} 
        
        # Rename circuit0.eqn to circuit0_{i}.eqn and copy to abc directory
        #ls ./*.json | parallel --eta 'index=$(echo "{}" | grep -oP "(?<=_)\d+(?=\.json)"); mv "circuit0.eqn" "circuit0_$index.eqn"; copy_file "circuit0_$index.eqn" "../abc/opt_$index.eqn"'
        
        # Copy optimized circuits to abc directory
        for file in ./*.eqn; do
            index=$(echo "$file" | awk -F'[_.]' '{print $(NF-2)}' )
            copy_file "$file" "../abc/opt_$index.eqn"
        done

        change_dir ".."
    else
        execute_command "target/x86_64-unknown-linux-musl/release/graph2eqn result.json circuit0.eqn"
        change_dir ".."
        copy_file "graph2eqn/circuit0.eqn" "abc/opt.eqn"
    fi
    
    end_time_process_graph2eqn=$(date +%s.%N)
    runtime_process_graph2eqn=$(echo "$end_time_process_graph2eqn - $start_time_process_graph2eqn" | bc)
    echo -e "${GREEN}Process 4 - Graph to Equation completed.${RESET}"
}

# Function to run ABC on the original and optimized circuit
run_abc() {
    echo -e "${YELLOW}<------------------------------Process 5: Run ABC on the original and optimized circuit, and conduct equivalent checking------------------->${RESET}"
    copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
    
    #python graph2eqn_ng.py
    change_dir "abc/"

    start_time_process_abc_original_cir=$(date +%s.%N)
    # baseline
    #execute_command "./abc -c \"read_eqn ori.eqn; read_lib asap7_clean.lib;  st; dch; ps; map; st; dch; ps; amap; st;dch; ps; amap; st;dch; ps; amap; st; topo; upsize; dnsize; stime;\"" 

    # lazy man
    # execute_command "./abc -c \"read_eqn ori.eqn; recadd3_ori; read_lib asap7_clean.lib;  st; dch; ps; map; topo; upsize; dnsize; stime;\"" 

    # alan
    #execute_command "./abc -c \"read_eqn ori.eqn;  if -K 6 -g -C 8; read_lib asap7_clean.lib;  ;st;dch;ps;  map ;topo;  upsize; dnsize; stime;st;dch;ps;  map ;topo;  upsize; dnsize; stime;st;dch;ps;  map ;topo;  upsize; dnsize; stime;st;dch;ps;  map ;topo;  upsize; dnsize; stime;\"" 
    
    # baline - single operator - if -g
    execute_command "./abc -c \"read_eqn ori.eqn; read_lib asap7_clean.lib; if -g; st;dch; ps; map; topo; upsize; dnsize; stime;\""

    end_time_process_abc_original_cir=$(date +%s.%N)
    runtime_process_abc_original_cir=$(echo "$end_time_process_abc_original_cir - $start_time_process_abc_original_cir" | bc)
    #execute_command "./abc -c \"read_eqn ori.eqn; st; dch -f; st; timing; timing; read_lib asap7_clean.lib; map; topo; upsize; dnsize; stime\""
    start_time_process_abc=$(date +%s.%N)
    if [[ "$pattern" == *"random"* ]]; then
        timestamp=$(date +%Y%m%d%H%M%S)
        echo "    i   o  Gates Area  Delay" > ../abc/stats.txt
        # Parallel execution of ABC for each optimized circuit
        # lazy man
        ls ./opt_*.eqn | parallel --eta "./abc -c \"read_eqn {}; read_lib asap7_clean.lib;  st; dch; ps; map; topo; upsize; dnsize; stime -d\"" >> ../tmp_log/abc_opt_all_${timestamp}.log
        
        # alan
        #ls ./opt_*.eqn | parallel --eta "./abc -c \"read_eqn {}; read_lib asap7_clean.lib; st;dch;ps;  map ;topo;  upsize; dnsize; stime;st;dch;ps;  map ;topo;  upsize; dnsize; stime;st;dch;ps;  map ;topo;  upsize; dnsize; stime;st;dch;ps;  map ;topo;  upsize; dnsize; stime -d\"" >> ../tmp_log/abc_opt_all_${timestamp}.log
        
        # copy right from ./stats.txt to ../tmp_log/abc_opt_all_{timestamp}.log
        mv "stats.txt" "../tmp_log/abc_opt_all_formatted_${timestamp}.log"
    else
        # alan
        # execute_command "./abc -c \"read_eqn opt.eqn; read_lib asap7_clean.lib;  st; dch; ps; map; topo; upsize; dnsize; stime; st; dch; ps; map; topo; upsize; dnsize; stime; st; dch; ps; map; topo; upsize; dnsize; stime; st; dch; ps; map; topo; upsize; dnsize; stime;\""
        # lazyman
        #execute_command "./abc -c \"read_eqn opt.eqn; read_lib asap7_clean.lib;  st; dch; ps; amap; topo; upsize; dnsize; stime;\""
        # baseline
        #execute_command "./abc -c \"read_eqn opt.eqn; read_lib asap7_clean.lib;  st; dch; ps; amap; topo; upsize; dnsize; stime;\""

        # baseline - single operator
        execute_command "./abc -c \"read_eqn opt.eqn; read_lib asap7_clean.lib;  st; dch; ps; map; topo; upsize; dnsize; stime;\""
        
        #execute_command "./abc -c \"read_eqn opt.eqn; st; dch -f; st; timing; timing; read_lib asap7_clean.lib; map; topo; upsize; dnsize; stime\""
        #execute_command "./abc -c \"read_eqn ../circuit0.eqn; print_stats -p; print_stats -p; read_lib asap7_clean.lib; map; topo; upsize; dnsize; stime\""
        #execute_command "./abc -c \"cec ori.eqn ../circuit0.eqn\""
    fi
    


    end_time_process_abc=$(date +%s.%N)
    runtime_process_abc=$(echo "$end_time_process_abc - $start_time_process_abc" | bc)
    echo -e "${GREEN}Process 5 - Run ABC on the original and optimized circuit completed.${RESET}"
}

# Function to compare original and optimized circuit
compare_circuits() {
    echo -e "${YELLOW}<-----------------------------Final Step: Comparing Original and Optimized Circuit----------------------------->${RESET}"
    if [[ "$pattern" == *"random"* ]]; then
        # Parallel execution of cec for each optimized circuit
        ls ./opt_*.eqn | parallel --eta "./abc -c \"cec ori.eqn {}\""
    else
        execute_command "./abc -c \"cec ori.eqn opt.eqn\""
    fi

    change_dir ".."
}

# Function to report total runtime
report_runtime() {
    echo -e "${GREEN}All processes completed successfully.${RESET}"

    echo -e "${GREEN}Rewrite circuit completed in ${RED}$runtime_process_rw${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Extract DAG completed in ${RED}$runtime_process_extract${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Process JSON completed in ${RED}$runtime_process_process_json${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Graph to Equation in ${RED}$runtime_process_graph2eqn${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Run ABC on the optimized circuit completed in ${RED}$runtime_process_abc${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Run ABC on the original circuit completed in ${RED}$runtime_process_abc_original_cir${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Total runtime: ${RED}$(echo "scale=2; $runtime_process_rw + $runtime_process_extract + $runtime_process_process_json + $runtime_process_graph2eqn + $runtime_process_abc" | bc)${GREEN} seconds.${RESET}"
}

# Main script
feature_cmd="./target/x86_64-unknown-linux-musl/release/e-rewriter"
echo -e "${YELLOW}Using feature label: ${feature}${RESET}"

setup_directories
get_user_input 
rewrite_circuit # eqn2egraph, rewrite
extract_dag # extract from saturated egraph, extract dag
process_json # extract from saturated egraph, process json
graph_to_equation # egraph2eqn
run_abc
compare_circuits  # logic equivalence check
report_runtime # report runtime