#!/bin/bash

# Check if the rewritten_circuit directory exists under e-rewriter; if not, create it
if [ ! -d "e-rewriter/rewritten_circuit" ]; then
    mkdir -p e-rewriter/rewritten_circuit
fi

# Check if the required folders exist under extraction-gym; if not, create them
if [ ! -d "extraction-gym/input" ]; then
    mkdir -p extraction-gym/input
    mkdir -p extraction-gym/input/egg
fi

if [ ! -d "extraction-gym/out_dag_json" ]; then
    mkdir -p extraction-gym/out_dag_json
fi

# if [ ! -d "extraction-gym/out_process_result" ]; then
#     mkdir -p extraction-gym/out_process_result
# fi

if [ ! -d "extraction-gym/out_json" ]; then
    mkdir -p extraction-gym/out_json
fi

# if [ ! -d "extraction-gym/out_process_dag_result" ]; then
#     mkdir -p extraction-gym/out_process_dag_result
# fi

if [ ! -d "extraction-gym/output" ]; then
    mkdir -p extraction-gym/output/egg
    mkdir -p extraction-gym/output
fi


# print the process - rewrite - process
echo "-----------------------------Process 1: Rewrite the circuit-----------------------------"
start_time_process1=$(date +%s.%N)
cd e-rewriter/ && cargo run circuit0.eqn 
cd ..

end_time_process1=$(date +%s.%N)
runtime_process1=$(echo "$end_time_process1 - $start_time_process1" | bc)


echo "-----------------------------Process 2: Extract the DAG-----------------------------"
start_time_process2=$(date +%s.%N)
#cd extraction-gym/ && make
end_time_process2=$(date +%s.%N)
#cd ..
start_time_process2_2=$(date +%s.%N)

for i in {0..19}; do
  cp e-rewriter/random_result/result${i}.json extraction-gym/out_json/
done

cd process_json/ && target/release/process_json
cd ..



echo "-----------------------------Process 3: Evaluate-----------------------------"

# cd abc/ && ./abc -c "cec ori.eqn op.eqn"
# cd ..
# cd abc/ && ./abc -c "read_eqn ori.eqn;st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
# cd abc/ && ./abc -c "read_eqn op.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
cp e-rewriter/circuit0.eqn abc/ori.eqn
cd abc/ && ./abc -c "read_eqn ori.eqn;st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
cd ..








#cp -r process_json/out_process_result extraction-gym/  && cp -r process_json/out_process_dag_result extraction-gym/
#----------------select&test extract alogrithm---------------------
#cp process_json/out_process_dag_result/graph_internal_serd_bottom-up.json graph2eqn/graph_internal_serd_bottom-up.json
for i in {0..19}; do
  cp process_json/out_process_result/result${i}.json graph2eqn/result.json
  cd graph2eqn/ && target/release/graph2eqn result.json
  cd ..
  cp graph2eqn/circuit0.eqn abc/op2.eqn
  rm graph2eqn/circuit0.eqn
  # rm graph2eqn/circuit0.eqn
  cd abc/ && ./abc -c "read_eqn op2.eqn; st; dch -f;st; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
  rm op2.eqn
  cd ..



done
#cp process_json/out_process_dag_result/graph_internal_serd_faster-bottom-up.json graph2eqn/graph_internal_serd_faster-bottom-up.json
# cp process_json/out_process_dag_result/graph_internal_serd_faster-greedy-dag.json graph2eqn/graph_internal_serd_faster-greedy-dag.json
# cp process_json/out_process_dag_result/graph_internal_serd_global-greedy-dag.json graph2eqn/graph_internal_serd_global-greedy-dag.json
#cp process_json/out_process_dag_result/graph_internal_serd_greedy-dag.json graph2eqn/graph_internal_serd_greedy-dag.json
end_time_process2_2=$(date +%s.%N)
echo "-----------------------------Process 3: graph to eqn-----------------------------"
start_time_process2_3=$(date +%s.%N)
# cd graph2eqn/ && target/release/graph2eqn graph_internal_serd_faster-bottom-up.json
# 
# cd ..
# cp graph2eqn/circuit0.eqn abc/op.eqn
# cp e-rewriter/circuit0.eqn abc/ori.eqn


# cd graph2eqn/ && target/release/graph2eqn graph_internal_serd_bottom-up.json
# cd ..
# cp graph2eqn/circuit0.eqn abc/op1.eqn
# rm graph2eqn/circuit0.eqn
# cd abc/ && ./abc -c "read_eqn op1.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
# echo the current directory

# echo cec
echo "-----------------------------CEC of original circuit and optimized circuit-----------------------------"
./abc -c "cec ori.eqn op2.eqn"

cd ..



end_time_process2_3=$(date +%s.%N)


# cd graph2eqn/ && target/release/graph2eqn graph_internal_serd_faster-bottom-up.json
# cd ..
# cp graph2eqn/circuit0.eqn abc/op1.eqn
# rm graph2eqn/circuit0.eqn
# cd abc/ && ./abc -c "read_eqn op1.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
# end_time_process2_3=$(date +%s.%N)
# cd graph2eqn/ && target/release/graph2eqn graph_internal_serd_faster-greedy-dag.json
# cd ..
# cp graph2eqn/circuit0.eqn abc/op2.eqn
# rm graph2eqn/circuit0.eqn
# cd abc/ && ./abc -c "read_eqn op2.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
#end_time_process2_3=$(date +%s.%N)
# cd graph2eqn/ && target/release/graph2eqn graph_internal_serd_global-greedy-dag.json
# cd ..
# cp graph2eqn/circuit0.eqn abc/op3.eqn
# rm graph2eqn/circuit0.eqn
# cd abc/ && ./abc -c "read_eqn op3.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..
# cd graph2eqn/ && target/release/graph2eqn graph2eqn/graph_internal_serd_greedy-dag.json
# cd ..
# cp graph2eqn/circuit0.eqn abc/op4.eqn
# rm graph2eqn/circuit0.eqn
# cd abc/ && ./abc -c "read_eqn op4.eqn; st; rewrite; balance; print_stats -p; read_lib asap7_clean.lib ; map ; topo; upsize; dnsize; stime"
# cd ..



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