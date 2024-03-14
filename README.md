# E-syn2

- `run1.sh` - runner, using feature label to accept multiple runs
- `run_rad.sh` - testing script for testing random version
- `run2.sh` - using DAG (extraction - gym)


- `run.sh` - main runner, no featur label -> multiple iterations
- `run1_test_iterations.sh` - testing script for testing multiple iterations -> test various iterations effects for a single run
- `run_rad` - just add cp
- `run2.sh` - using DAG (extraction - gym)
- `run2_fast` - only one algorithm in extaction gym

- extraction_gym 

- `feature 2` -> using extraction gym (cp name different) - see `run2.sh`, `run2_fast.sh`
- `feature 1` 


run with iterations experiments:

```bash
for i in $(seq 5 5 50); do
  echo -e "${i},multi_round,\n" | bash run.sh > tmp_log/log_${i}_multi_round.txt
  wait
done
```

```bash
for i in $(seq 5 5 50); do
  echo -e "${i}\n\n\n" | bash run.sh > tmp_log/log_${i}_no_feature.txt
  wait
done
```