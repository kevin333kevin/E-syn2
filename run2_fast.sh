#!/bin/bash

# Check if the dot_graph directory exists under e-rewriter; if not, create it
if [ ! -d "e-rewriter/dot_graph" ]; then
    mkdir -p e-rewriter/dot_graph
fi

# Check if the required folders exist under extraction-gym; if not, create them
if [ ! -d "extraction-gym/data" ]; then
    mkdir -p extraction-gym/data/my_data
    mkdir -p extraction-gym/data/egg
fi

if [ ! -d "extraction-gym/out_dag_json" ]; then
    mkdir -p extraction-gym/out_dag_json/my_data
fi

# if [ ! -d "extraction-gym/out_process_result" ]; then
#     mkdir -p extraction-gym/out_process_result
# fi

if [ ! -d "extraction-gym/out_json" ]; then
    mkdir -p extraction-gym/out_json/my_data
fi

# if [ ! -d "extraction-gym/out_process_dag_result" ]; then
#     mkdir -p extraction-gym/out_process_dag_result
# fi

if [ ! -d "extraction-gym/output" ]; then
    mkdir -p extraction-gym/output/egg
    mkdir -p extraction-gym/output/my_data
fi


# print the process - rewrite - process
echo "-----------------------------Process 1: Rewrite the circuit-----------------------------"
# start_time_process1=$(date +%s.%N)
# cd e-rewriter/ && cargo run  --features feature2 circuit0.eqn 
# #cd e-rewriter/ && target/release/e-rewriter --features feature2 circuit0.eqn 
# # Execute the steps
# #if [ "$display" -eq 0 ]; then
#     #cd e-rewriter/ && target/release/e-rewriter circuit0.eqn circuit1.eqn
# #else
# #    cd e-rewriter/ && target/release/e-rewriter --features display circuit0.eqn circuit1.eqn
# #fi

# cd ..
# cp e-rewriter/dot_graph/graph_cost_serd.json extraction-gym/data/my_data/
# end_time_process1=$(date +%s.%N)
# runtime_process1=$(echo "$end_time_process1 - $start_time_process1" | bc)


echo "-----------------------------Process 2: Extract the DAG-----------------------------"
start_time_process2=$(date +%s.%N)
cd extraction-gym/ && make
#rm extraction-gym/out_dag_json/my_data/graph_cost_serd_faster-bottom-up.json

end_time_process2=$(date +%s.%N)
cd ..
start_time_process2_2=$(date +%s.%N)

cp extraction-gym/random_result/result9.json extraction-gym/out_dag_json/my_data/graph_cost_serd_faster-bottom-up.json
cd process_json/ && target/release/process_json
cd ..
#cp -r process_json/out_process_result extraction-gym/  && cp -r process_json/out_process_dag_result extraction-gym/
#----------------select&test extract alogrithm---------------------

cp process_json/out_process_dag_result/graph_cost_serd_faster-bottom-up.json graph2eqn/graph_cost_serd_faster-bottom-up.json
#cp process_json/out_process_dag_result/graph_cost_serd_faster_greedy_dag.json graph2eqn/graph_cost_serd_faster_greedy_dag.json
end_time_process2_2=$(date +%s.%N)
echo "-----------------------------Process 3: graph to eqn-----------------------------"
 start_time_process2_3=$(date +%s.%N)
cd graph2eqn/ && target/release/graph2eqn graph_cost_serd_faster-bottom-up.json
#cd graph2eqn/ && target/release/graph2eqn graph_cost_serd_faster_greedy_dag.json
# 
cd ..
cp graph2eqn/circuit0.eqn abc/op.eqn
cp e-rewriter/circuit0.eqn abc/ori.eqn

echo "-----------------------------Process 3: Evaluate-----------------------------"

# cd abc/ && ./abc -c "cec ori.eqn op.eqn"
# cd ..
# cd abc/ && ./abc -c "read_eqn ori.eqn;st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
# cd abc/ && ./abc -c "read_eqn op.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..

# cd graph2eqn/ && target/release/graph2eqn graph_cost_serd_bottom-up.json
# cd ..
# cp graph2eqn/circuit0.eqn abc/op1.eqn
# rm graph2eqn/circuit0.eqn
echo "-----------------------------original-----------------------------"
cd abc/ && ./abc -c "read_eqn ori.eqn; st;ps; dch; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
# cd abc/ && ./abc -c "read_eqn ori.eqn;st; dch; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"

end_time_process2_3=$(date +%s.%N)



echo "-----------------------------graph_cost_serd_faster-bottom-up-----------------------------"
cd graph2eqn/ && target/release/graph2eqn graph_cost_serd_faster-bottom-up.json
cd ..
cp graph2eqn/circuit0.eqn abc/op5.eqn
rm graph2eqn/circuit0.eqn
cd abc/ && ./abc -c "read_eqn op5.eqn; st;ps; dch; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
cd ..
end_time_process2_3=$(date +%s.%N)



echo "-----------------------------CEC of original circuit and optimized circuit-----------------------------"

cd abc/ &&./abc -c "cec ori.eqn op5.eqn"

cd ..



# Return to the original directory


runtime_process2=$(echo "$end_time_process2 - $start_time_process2" | bc)
runtime_process2_2=$(echo "$end_time_process2_2 - $start_time_process2_2" | bc)
runtime_process2_3=$(echo "$end_time_process2_3 - $start_time_process2_3" | bc)
runtime_process3_1=$(echo "$runtime_process1 + $runtime_process2 +$runtime_process2_2 +$runtime_process2_3   " | bc)
echo "Process 1 Rewrite the circuit runtime: $runtime_process1 seconds"
echo "Process 2.1 Extract the DAG runtime: $runtime_process2 seconds"
echo "Process 2.2 Process json runtime: $runtime_process2_2 seconds"
echo "Process 2.3 graph2eqn: $runtime_process2_3 seconds"
echo "Process 3.1 total_time rw+extract: $runtime_process3_1 seconds"