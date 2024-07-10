# Experimental testing script (e.g., different dag extraction results, hyperparameters tuning for egg, random extraction testing, etc.)

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
ensure_dir "extraction-gym/input/egg"
ensure_dir "extraction-gym/out_dag_json"
ensure_dir "extraction-gym/out_json"
ensure_dir "extraction-gym/output/egg"
ensure_dir "extraction-gym/output"

echo -e "${GREEN}Setup complete.${RESET}\n"

# Test type prompt for the developer to conduct experimental testing
echo -e "Enter the type of tests you want to run:"
echo -e "1 for Random Graph Tests"
echo -e "2 for Extraction Gym Tests"
read -p "Test type (1/2): " test_type

if [ $test_type -eq 1 ]; then
    echo -e "${GREEN}Conduct Experimental Testing for Random Graph Extraction...${RESET}"
    # Process 1: Rewrite the circuit
    # initialize variablles - feature, iteration_times, pattern
    feature=""
    iteration_times=10
    pattern="random"
    feature_cmd="./target/release/e-rewriter"
    echo -e "${YELLOW}<-----------------------------Process 1: Rewrite the Circuit----------------------------->${RESET}"
    start_time_process_rw=$(date +%s.%N)
    change_dir "e-rewriter/"
    execute_command "$feature_cmd circuit0.eqn $iteration_times $pattern"
    change_dir ".."
    copy_file "e-rewriter/rewritten_circuit/graph_internal_serd.json" "extraction-gym/input/"

    # if feature is feature2, run the extraction gym -> cd extraction-gym/ && make

    if [ ! -z "$feature" ] && [ "$feature" == "dag_cost" ]; then
        echo -e "${YELLOW}Running extraction gym...${RESET}"
        change_dir "extraction-gym/"
        execute_command "make"
        change_dir ".."
    fi

    end_time_process_rw=$(date +%s.%N)
    runtime_process_rw=$(echo "$end_time_process_rw - $start_time_process_rw" | bc)
    echo -e "${GREEN}Process 1 - Rewrite circuit completed.${RESET}"

    # Process 2: Extract the DAG and Process JSON
    echo -e "${YELLOW}<-----------------------------Process 2: Extract the DAG and Process JSON----------------------------->${RESET}"
    start_time_process_process_json=$(date +%s.%N)
    copy_file "e-rewriter/result.json" "extraction-gym/out_json"

    for i in $(seq 0 19); do
        copy_file "e-rewriter/random_result/result${i}.json" "extraction-gym/out_json/"
    done

    change_dir "process_json/"
    execute_command "target/release/process_json"
    change_dir ".."

    # if feature is not dag_cost, copy the result.json to graph2eqn/result.json

    if [ -z "$feature" ] || [ "$feature" != "dag_cost" ]; then
        echo -e "${YELLOW}Copying result.json ... Prepare graph for Equation conversion.${RESET}"
        copy_file "process_json/out_process_result/result.json" "graph2eqn/result.json" 
        for i in $(seq 0 19); do
            copy_file "process_json/out_process_result/result${i}.json" "graph2eqn/result${i}.json" 
        done
    elif [ "$feature" == "dag_cost" ]; then
        echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json ... Prepare graph for Equation conversion.${RESET}"
        copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json" "graph2eqn/result.json" 
    fi


    end_time_process_process_json=$(date +%s.%N)
    runtime_process_process_json=$(echo "$end_time_process_process_json - $start_time_process_process_json" | bc)
    echo -e "${GREEN}Process 2 - Extract DAG and Process JSON completed.${RESET}"

    # Process 3: Convert Graph to Equation and Evaluate
    echo -e "${YELLOW}<-----------------------------Process 3: Graph to Equation ----------------------------------------------->${RESET}"
    start_time_process_graph2eqn=$(date +%s.%N)
    change_dir "graph2eqn/"
    execute_command "target/release/graph2eqn result.json"
    for i in $(seq 0 19); do
        execute_command "target/release/graph2eqn result${i}.json"
        copy_file "circuit0.eqn" "../abc/opt_${i}.eqn"
    done

    change_dir ".."
    
    end_time_process_graph2eqn=$(date +%s.%N)
    runtime_process_graph2eqn=$(echo "$end_time_process_graph2eqn - $start_time_process_graph2eqn" | bc)
    echo -e "${GREEN}Process 3 - Graph to Equation completed.${RESET}"

    # Process 4: Run ABC on the original and optimized circuit
    echo -e "${YELLOW}<------------------------------Process 4: Run ABC on the original and optimized circuit, and conduct equivalent checking------------------->${RESET}"
    copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
    start_time_process_abc=$(date +%s.%N)
    change_dir "abc/"
    

    for i in $(seq 0 19); do
        execute_command "./abc -c \"read_eqn opt_${i}.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    done

    execute_command "./abc -c \"read_eqn ori.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""

    end_time_process_abc=$(date +%s.%N)
    runtime_process_abc=$(echo "$end_time_process_abc - $start_time_process_abc" | bc)
    echo -e "${GREEN}Process 4 - Run ABC on the original and optimized circuit completed.${RESET}"

    # Final Step: Compare Original and Optimized Circuit
    echo -e "${YELLOW}<-----------------------------Final Step: Comparing Original and Optimized Circuit----------------------------->${RESET}"

    for i in $(seq 0 19); do
        execute_command "./abc -c \"cec ori.eqn opt_${i}.eqn\""
    done

    change_dir ".."

    # Report total runtime
    echo -e "${GREEN}All processes completed successfully.${RESET}"

    echo -e "${GREEN}Rewrite circuit completed in ${RED}$runtime_process_rw${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Extract DAG and Process JSON completed in ${RED}$runtime_process_process_json${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Graph to Equation in ${RED}$runtime_process_graph2eqn${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Run ABC on the original and optimized circuit completed in ${RED}$runtime_process_abc${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Total runtime: ${RED}$(echo "scale=2; $runtime_process_rw + $runtime_process_process_json + $runtime_process_graph2eqn + $runtime_process_abc" | bc)${GREEN} seconds.${RESET}"

elif [ $test_type -eq 2 ]; then
    echo -e "${GREEN}Conduct Experimental Testing for Extraction Gym...${RESET}"
    feature=""
    iteration_times=10
    pattern=""
    feature_cmd="./target/release/e-rewriter-dag_cost"
    echo -e "${YELLOW}<-----------------------------Process 1: Rewrite the Circuit----------------------------->${RESET}"
    start_time_process_rw=$(date +%s.%N)
    change_dir "e-rewriter/"
    execute_command "$feature_cmd circuit0.eqn $iteration_times $pattern"
    change_dir ".."
    copy_file "e-rewriter/rewritten_circuit/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json" "extraction-gym/input/"

    # if feature is feature2, run the extraction gym -> cd extraction-gym/ && make

    echo -e "${YELLOW}Running extraction gym...${RESET}"
    change_dir "extraction-gym/"
    execute_command "make"
    change_dir ".."

    end_time_process_rw=$(date +%s.%N)
    runtime_process_rw=$(echo "$end_time_process_rw - $start_time_process_rw" | bc)
    echo -e "${GREEN}Process 1 - Rewrite circuit completed.${RESET}"

    # Process 2: Extract the DAG and Process JSON
    echo -e "${YELLOW}<-----------------------------Process 2: Extract the DAG and Process JSON----------------------------->${RESET}"
    start_time_process_process_json=$(date +%s.%N)

    change_dir "process_json/"
    execute_command "target/release/process_json"
    change_dir ".."

    echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up_bottom-up.json ... Prepare graph for Equation conversion.${RESET}"
    copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_bottom-up.json" "graph2eqn/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_bottom-up.json"
    echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json ... Prepare graph for Equation conversion.${RESET}"
    copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json" "graph2eqn/rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json"
    echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up_faster-greedy-dag.json ... Prepare graph for Equation conversion.${RESET}"
    copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_faster-greedy-dag.json" "graph2eqn/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_faster-greedy-dag.json"
    echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up_global-greedy-dag.json ... Prepare graph for Equation conversion.${RESET}"
    copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_global-greedy-dag.json" "graph2eqn/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_global-greedy-dag.json"
    echo -e "${YELLOW}Copying rewritten_egraph_with_weight_cost_serd_faster-bottom-up_greedy-dag.json ... Prepare graph for Equation conversion.${RESET}"
    copy_file "process_json/out_process_dag_result/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_greedy-dag.json" "graph2eqn/rewritten_egraph_with_weight_cost_serd_faster-bottom-up_greedy-dag.json"

    end_time_process_process_json=$(date +%s.%N)
    runtime_process_process_json=$(echo "$end_time_process_process_json - $start_time_process_process_json" | bc)
    echo -e "${GREEN}Process 2 - Extract DAG and Process JSON completed.${RESET}"

    # Process 3: Convert Graph to Equation and Evaluate
    echo -e "${YELLOW}<-----------------------------Process 3: Graph to Equation ----------------------------------------------->${RESET}"
    start_time_process_graph2eqn=$(date +%s.%N)
    change_dir "graph2eqn/"
    execute_command "target/release/graph2eqn rewritten_egraph_with_weight_cost_serd_faster-bottom-up_bottom-up.json"
    execute_command "target/release/graph2eqn rewritten_egraph_with_weight_cost_serd_faster-bottom-up.json"
    execute_command "target/release/graph2eqn rewritten_egraph_with_weight_cost_serd_faster-bottom-up_faster-greedy-dag.json"
    execute_command "target/release/graph2eqn rewritten_egraph_with_weight_cost_serd_faster-bottom-up_global-greedy-dag.json"
    execute_command "target/release/graph2eqn rewritten_egraph_with_weight_cost_serd_faster-bottom-up_greedy-dag.json"

    copy_file "circuit0.eqn" "../abc/opt_bottom-up.eqn"
    copy_file "circuit0.eqn" "../abc/opt_faster-bottom-up.eqn"
    copy_file "circuit0.eqn" "../abc/opt_faster-greedy-dag.eqn"
    copy_file "circuit0.eqn" "../abc/opt_global-greedy-dag.eqn"
    copy_file "circuit0.eqn" "../abc/opt_greedy-dag.eqn"

    change_dir ".."
    
    end_time_process_graph2eqn=$(date +%s.%N)
    runtime_process_graph2eqn=$(echo "$end_time_process_graph2eqn - $start_time_process_graph2eqn" | bc)
    echo -e "${GREEN}Process 3 - Graph to Equation completed.${RESET}"

    # Process 4: Run ABC on the original and optimized circuit
    echo -e "${YELLOW}<------------------------------Process 4: Run ABC on the original and optimized circuit, and conduct equivalent checking------------------->${RESET}"
    copy_file "e-rewriter/circuit0.eqn" "abc/ori.eqn"
    start_time_process_abc=$(date +%s.%N)
    change_dir "abc/"
    

    execute_command "./abc -c \"read_eqn opt_bottom-up.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    execute_command "./abc -c \"read_eqn opt_faster-bottom-up.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    execute_command "./abc -c \"read_eqn opt_faster-greedy-dag.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    execute_command "./abc -c \"read_eqn opt_global-greedy-dag.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""
    execute_command "./abc -c \"read_eqn opt_greedy-dag.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""


    execute_command "./abc -c \"read_eqn ori.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime\""

    end_time_process_abc=$(date +%s.%N)
    runtime_process_abc=$(echo "$end_time_process_abc - $start_time_process_abc" | bc)
    echo -e "${GREEN}Process 4 - Run ABC on the original and optimized circuit completed.${RESET}"

    # Final Step: Compare Original and Optimized Circuit
    echo -e "${YELLOW}<-----------------------------Final Step: Comparing Original and Optimized Circuit----------------------------->${RESET}"

    for i in $(seq 0 19); do
        execute_command "./abc -c \"cec ori.eqn opt_${i}.eqn\""
    done

    change_dir ".."

    # Report total runtime
    echo -e "${GREEN}All processes completed successfully.${RESET}"

    echo -e "${GREEN}Rewrite circuit completed in ${RED}$runtime_process_rw${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Extract DAG and Process JSON completed in ${RED}$runtime_process_process_json${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Graph to Equation in ${RED}$runtime_process_graph2eqn${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Run ABC on the original and optimized circuit completed in ${RED}$runtime_process_abc${GREEN} seconds.${RESET}"
    echo -e "${GREEN}Total runtime: ${RED}$(echo "scale=2; $runtime_process_rw + $runtime_process_process_json + $runtime_process_graph2eqn + $runtime_process_abc" | bc)${GREEN} seconds.${RESET}"

fi 

Unfortunately, I fell ill and my state of mind was not clear.