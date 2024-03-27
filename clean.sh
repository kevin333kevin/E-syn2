#!/bin/bash
RED="\e[31m"
GREEN="\e[32m"
YELLOW="\e[1;33m"
RESET="\e[0m"
# warning for user that this script will remove all output files in the directories specified
echo -e "${RED}WARNING: This script will remove all output files in the directories specified.${RESET}"

# Here the question is colored with YELLOW and the input prompt is in default color
echo -ne "${YELLOW}Do you want to continue? (y/n)${RESET} "
read answer

if [ "$answer" != "y" ]; then
  echo -e "${RED}Aborting.${RESET}"
  exit 1
fi

echo -e "${GREEN}Cleaning output folders and files...${RESET}"

# Define directories to clean
directories=(
  "e-rewriter/random_result"
  "e-rewriter/random_dot"
  "e-rewriter/dot_graph"
  "extraction-gym/data/my_data"
  "extraction-gym/out_json"
  "extraction-gym/out_process_dag_result"
  "extraction-gym/out_process_result"
  "extraction-gym/output"
  "extraction-gym/out_dag_json"
  "extraction-gym/data"
  "extraction-gym/input"
  "process_json/out_process_dag_result"
  "process_json/out_process_result"
)

# Remove directories if they exist
for dir in "${directories[@]}"; do
  if [ -d "$dir" ]; then
    rm -rf "$dir"
    echo -e "Removed $dir"
  fi
done

# Special case for graph2eqn: remove .json and .eqn files if they exist
if [ -d "graph2eqn" ]; then
  rm -f graph2eqn/*.json 2>/dev/null
  rm -f graph2eqn/*.eqn 2>/dev/null
  echo -e "${GREEN}Cleaned graph2eqn directory${RESET}"
fi

# Special case for abc: remove .eqn files if they exist
if [ -d "abc" ]; then
  rm -f abc/*.eqn 2>/dev/null
  echo -e "${GREEN}Cleaned abc directory${RESET}"
fi

# Special case for extraction-gym/random_result
if [ -d "extraction-gym/random_result" ]; then
  rm -f extraction-gym/random_result/* 2>/dev/null
  echo -e "${GREEN}Cleaned extraction-gym/random_result directory${RESET}"
fi

# Special case for tmp_log/, remove files starts with "log_+ number" if they exist
if [ -d "tmp_log" ]; then
  rm -f tmp_log/log_[0-9]*_* 2>/dev/null
  echo -e "${GREEN}Cleaned tmp_log directory${RESET}"
fi

# ask user whether to execute cargo clean in each directory
echo -ne "${YELLOW}Do you want to execute cargo clean in each directory? (y/n)${RESET} "
read answer

if [ "$answer" != "y" ]; then
  echo -e "${RED}Aborting.${RESET}"
  exit 1
fi

# Execute cargo clean in e-rewriter directory
if [ -d "e-rewriter" ]; then
  (cd e-rewriter && cargo clean)
  echo -e "${GREEN}Ran cargo clean in e-rewriter directory${RESET}"
fi

# Execute cargo clean in process_json directory
if [ -d "process_json" ]; then
  (cd process_json && cargo clean)
  echo -e "${GREEN}Ran cargo clean in process_json directory${RESET}"
fi

# Execute cargo clean in graph2eqn directory
if [ -d "graph2eqn" ]; then
  (cd graph2eqn && cargo clean)
  echo -e "${GREEN}Ran cargo clean in graph2eqn directory${RESET}"
fi

# Execute cargo clean in extraction-gym directory
if [ -d "extraction-gym" ]; then
  (cd extraction-gym && cargo clean)
  echo -e "${GREEN}Ran cargo clean in extraction-gym directory${RESET}"
fi


echo -e "${GREEN}Cleaning complete.${RESET}"
