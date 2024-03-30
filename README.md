# E-syn2

## Usage

```bash
make
chmod +x ./run.sh && ./run.sh
```

run with iterations experiments:

```bash
for i in $(seq 5 5 50); do
  echo -e "${i}\ndelay\nrandom" | bash run.sh > tmp_log/log_${i}_no_feature.txt
  wait
done
```

### Directory Structure

```
./
├── abc
├── converted_circuit_strash # benchmark circuits
├── data_process_script
├── deprecated
├── e-rewriter # rewriter and parser
|   ├── rewritten_circuit # rewriten circuits -> wait for extraction
│   ├── circuit0.eqn # put your circuit here
│   ├── src
│   ├── target
├── extraction-gym # extractor
│   ├── data
│   │   ├── egg
│   │   └── my_data # 1. saturacted circuits graphs for extraction (copied from e-rewriter)
│   ├── out_dag_json
│   │   └── my_data # 4. raw json marked the extracted nodes - dag based extraction
│   ├── out_json
│   │   └── my_data # 4. raw json marked the extracted nodes - tree based extraction
│   ├── output
│   │   ├── egg
│   │   └── my_data # 3. log files during extraction
│   ├── random_result # 2. random extraction results -> raw json (will copy to out_json and out_dag_json)
│   ├── src
│   └── target
├── process_json # post-processing script for extracted results
│   ├── out_process_dag_result # processed json (handled extracted nodes raw json)
│   ├── out_process_result # processed json (handled extracted nodes raw json)
│   ├── src
│   └── target
├── graph2eqn # convert extracted circuits to eqn format
│   ├── circuit0.eqn
│   ├── src
│   └── target
├── fuzz_abc_opt_flow # explore the optimization space of circuits
├── PPA_predictor # PPA predictor for E-graph formed circuits
├── clean.sh
├── collect.sh
├── README.md
├── run.sh
├── test.sh
├── Makefile
└── tmp_log
```