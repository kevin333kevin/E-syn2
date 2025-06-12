# E-syn2

## Prequisites  

- Rust environment
- Berkeley ABC tool (replaced with the `abc/src/opt/dar/darRefact.c`)

## Usage

```bash
make
chmod +x ./run.sh && ./run.sh
```


run with Simulated annealing extraction with a golden model for delay tuning:

```bash
 echo -e "5\ndelay\nrandom-sim-ann-based-faster-bottom-up-fast-par\n10\n0.8\n" | bash run_sa.sh 
```


