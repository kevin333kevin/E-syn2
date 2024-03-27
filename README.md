# E-syn2

## Usage

```bash
make
chmod +x ./run.sh && ./run.sh
```

### Directory Structure

```
./
├── abc
├── converted_circuit_strash # benchmark circuits
├── data_process_script
│   ├── analyze_data.py
│   ├── deal_csv.py
│   └── extract2csv.sh
├── deprecated
├── e-rewriter
|   ├── dot_graph # rewriten circuits -> wait for extraction
│   ├── circuit0.eqn # put your circuit here
│   ├── random_dot 
│   ├── src
│   ├── target
├── extraction-gym
│   ├── data
│   │   ├── egg
│   │   └── my_data # 1. saturacted circuits graphs for extraction (copied from e-rewriter)
│   ├── out_dag_json
│   │   └── my_data # 4. raw json marked the extracted nodes
│   ├── out_json
│   │   └── my_data # 4. raw json marked the extracted nodes
│   ├── output
│   │   ├── egg
│   │   └── my_data # 3. log files during extraction
│   ├── random_result # 2. random extraction results -> raw json (will copy to out_json and out_dag_json)
│   ├── src
│   └── target
├── fuzz_abc_opt_flow # explore the optimization space of circuits
├── graph2eqn
│   ├── circuit0.eqn
│   ├── src
│   └── target
├── PPA_predictor # PPA predictor for E-graph formed circuits
├── process_json
│   ├── out_process_dag_result # processed json (from extraction-gym out_dag_json)
│   ├── out_process_result # processed json (from extraction-gym out_json) 
│   ├── process_json # json with updated eclasses (deprecated)
│   ├── src
│   └── target
├── clean.sh
├── collect.sh
├── README.md
├── run.sh
├── test.sh
├── Makefile
└── tmp_log
```