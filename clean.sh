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
  "e-rewriter/dot_graph"
  "extraction-gym/data/my_data"
  "extraction-gym/out_json"
  "extraction-gym/out_process_dag_result"
  "extraction-gym/out_process_result"
  "extraction-gym/output"
  "extraction-gym/out_dag_json"
  "extraction-gym/data"
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

echo -e "${GREEN}Cleaning complete.${RESET}"