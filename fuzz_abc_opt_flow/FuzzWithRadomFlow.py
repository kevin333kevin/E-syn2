import os
from concurrent.futures import ProcessPoolExecutor, as_completed

# Function to execute a single ABC flow
def execute_flow(flow):
    # Prepare the command with the given flow
    command = "../abc/abc -c \"abcrc; read_eqn ../abc/ori.eqn; strash; {} ;read_lib ../abc/asap7_clean.lib ; map ; topo; upsize; dnsize; stime -d\"".format(flow)
    # Execute the command
    os.system(command)
    
    return "Completed: {}".format(flow)

def main():
    # Load the flows from file
    with open('GeneratedFlows.txt', 'r') as file:
        flows = [line.strip() for line in file]
    
    # Number of parallel processes (up to the number of cores/cpus you have)
    max_workers = 64
    
    # Create a process pool and execute the flows in parallel
    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        # Submit all the flows for execution
        future_to_flow = {executor.submit(execute_flow, flow): flow for flow in flows}
        
        # Process the results as they complete
        for future in as_completed(future_to_flow):
            flow = future_to_flow[future]
            try:
                result = future.result()
                print(result)  # Or handle the result in another way
            except Exception as exc:
                print('%r generated an exception: %s' % (flow, exc))

if __name__ == '__main__':
    main()
