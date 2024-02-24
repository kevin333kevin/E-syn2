# clean the output folder

if [ -d "extraction-gym/out_json" ]; then rm -rf extraction-gym/out_json 
fi

if [ -d "extraction-gym/out_process_dag_result" ]; then rm -rf extraction-gym/out_process_dag_result 
fi

if [ -d "extraction-gym/out_process_result" ]; then rm -rf extraction-gym/out_process_result 
fi

if [ -d "extraction-gym/output" ]; then rm -rf extraction-gym/output 
fi

if [ -d "extraction-gym/out_dag_json" ]; then rm -rf extraction-gym/out_dag_json 
fi

if [ -d "extraction-gym/data" ]; then rm -rf extraction-gym/data
fi


if [ -d "process_json/out_process_dag_result" ]; then rm -rf process_json/out_process_dag_result
fi

if [ -d "process_json/out_process_result" ]; then rm -rf process_json/out_process_result
fi

# remove all the .json and .eqn in the graph2eqn folder
if [ -d "graph2eqn" ]; then rm graph2eqn/*.json && rm graph2eqn/*.eqn
fi

# remove all the .eqn in the abc folder
if [ -d "abc" ]; then rm abc/*.eqn
fi