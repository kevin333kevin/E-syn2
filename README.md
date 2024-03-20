# E-syn2

## Usage

```bash
chmod +x ./run.sh && ./run.sh
```

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

### Parameters Explanation

- `iteration times`: the number of iterations for runner in egg.
- `multi_round` feature: heuristic algorithm that allows egg to run multiple rounds of rewriting.
- `dag_cost` feature: using DAG cost to guide the rewriting process.
- `random` extraction pattern: using random extraction pattern to explore the search space.