import grpc
import json
from concurrent import futures
import service_pb2
import service_pb2_grpc
import sys
import os
import argparse
import os
import shutil
import re
import gzip
sys.path.append("/data/cchen/esynturbo/E-syn2/HOGA")
from model import SynthNet
from main_qor_predict import  MyOwnDataset_4test
from torch_geometric.data import Dataset, Data
import torch
from tqdm import tqdm
import time
from dataset_prep import PygNodePropPredDataset, Evaluator
class VectorServiceServicer(service_pb2_grpc.VectorServiceServicer):
    def __init__(self, model, device):
        self.model = model
        self.device = device

    def ProcessJson(self, request, context):
        json_data = json.loads(request.json_data)
        # Process the JSON data and generate a vector response
        vector = [1.0, 2.0, 3.0]  # Example vector response
        return service_pb2.VectorResponse(vector=vector)

    def ProcessCircuitFiles(self, request, context):
        el_content = request.el_content
        csv_content = request.csv_content
        json_content = request.json_content
        print("start server")
        root_path = os.path.abspath(os.path.join(os.getcwd(), "..", "..", "HOGA", "dataset_4_test"))
        print("root_path: ",root_path)
        target_path = os.path.abspath(os.path.join(os.getcwd(), "..", "..", "HOGA", "dataset_4_test_process"))
        if not os.path.exists(target_path):
            os.makedirs(target_path)
        copy_files(root_path, target_path)
        process_dataset(target_path)
        processed_dir1 = os.path.join(target_path, 'processed')
        if not os.path.exists(processed_dir1):
           os.makedirs(processed_dir1, exist_ok=True)    
        dataset = MyOwnDataset_4test(root=target_path)
        pred = evaluate_4_test(self.model, self.device, dataset)
        pred = float(pred[0])
        print(f"pred: {pred}")      
        delay =1
        return service_pb2.CircuitProcessingResponse(delay=delay)


def evaluate_plot(model, device, dataloader):
    model.eval()
    batchData = []
    with torch.no_grad():
        for _, batch in enumerate(tqdm(dataloader, desc="Iteration",file=sys.stdout)):
            batch = batch.to(device)
            pred = model(batch)
            predArray = pred.view(-1,1).detach().cpu().numpy()
            batchData.append([predArray])
        
        # for i, data in enumerate(batchData):
        #     predArray, actualArray, desName, synID = data
        #     print(f"Batch {i} shape: predArray={predArray.shape}, actualArray={actualArray.shape}, desName={len(desName)}, synID={len(synID)}")
    return batchData

def evaluate_4_test(model, device, dataloader):
    model.eval()
    batchData = []
    with torch.no_grad():
        for _, batch in enumerate(tqdm(dataloader, desc="Iteration",file=sys.stdout)):
            batch = batch.to(device)
            pred = model(batch)
            predArray = pred.view(-1,1).detach().cpu().numpy()
            numInputs = pred.view(-1,1).size(0)
    return predArray


def copy_files(source_dir, target_dir):
    # 创建目标目录
    if not os.path.exists(target_dir):
        os.makedirs(target_dir)

    # 遍历源目录下的每个文件夹
    for folder in sorted(os.listdir(source_dir)):
        source_folder = os.path.join(source_dir, folder)
        
        #print(f"source_folder: {source_folder}")
        if os.path.isdir(source_folder):
            for subfolder in ["edgelist", "feature"]:
                source_subfolder = os.path.join(source_folder, subfolder)
                for filename in sorted(os.listdir(source_subfolder)):
                    source_file = os.path.join(source_subfolder, filename)
                    target_prob_folder = os.path.join(target_dir, f"{folder}", "raw")
                    os.makedirs(target_prob_folder, exist_ok=True)
                    target_file = os.path.join(target_prob_folder, filename)
                    shutil.copy(source_file, target_file)


def process_dataset(dataset_dir):
    for folder in os.listdir(dataset_dir):
        folder_path = os.path.join(dataset_dir, folder)
        if os.path.isdir(folder_path):
            process_folder(folder_path, folder)

def process_folder(folder_path, folder_name):
    raw_folder = os.path.join(folder_path, "raw")
    processed_folder = os.path.join(folder_path, "processed")
    if os.path.isdir(raw_folder):
        os.makedirs(processed_folder, exist_ok=True)
    if os.path.isdir(raw_folder):
        processed_files = set()
        for filename in os.listdir(raw_folder):
            source_file = os.path.join(raw_folder, filename)
            if filename.endswith(".csv"):
                new_filename = "node-feat.csv"
                if filename != new_filename and new_filename not in processed_files:
                    target_file = os.path.join(raw_folder, new_filename)
                    os.rename(source_file, target_file)
                    processed_files.add(new_filename)
                num_nodes = count_lines(target_file)
                num_node_file = os.path.join(raw_folder, "num-node-list.csv")
                with open(num_node_file, "w") as f:
                    f.write(str(num_nodes))

            elif filename.endswith(".el"):
                new_filename = "edge.csv"
                if filename != new_filename and new_filename not in processed_files:
                    target_file = os.path.join(raw_folder, new_filename)
                    edge_data = []
                    with open(source_file, "r") as f:
                        for line in f:
                            source, target = line.strip().split()
                            edge_data.append(f"{source},{target}")
                    with open(target_file, "w", newline="") as csvfile:
                        for line in edge_data:
                            csvfile.write(line + "\n")
                    os.remove(source_file)
                    processed_files.add(new_filename)
                num_edges = count_lines(target_file)
                num_edge_file = os.path.join(raw_folder, "num-edge-list.csv")
                with open(num_edge_file, "w") as f:
                    f.write(str(num_edges))

        for filename in os.listdir(raw_folder):
            if filename.endswith(".csv"):
                csv_file = os.path.join(raw_folder, filename)
                gz_file = os.path.join(raw_folder, f"{filename}.gz")
                with open(csv_file, "rb") as f_in, gzip.open(gz_file, "wb") as f_out:
                    shutil.copyfileobj(f_in, f_out)
                


def count_lines(file_path):
    with open(file_path, "r") as f:
        return sum(1 for _ in f)




def serve():
    parser = argparse.ArgumentParser(description='mult16')
    parser.add_argument('--bits', type=int, default=8)
    parser.add_argument('--bits_test', type=int, default=64)
    parser.add_argument('--device', type=int, default=0)
    parser.add_argument('--log_steps', type=int, default=1)
    parser.add_argument('--num_layers', type=int, default=1)
    parser.add_argument('--hidden_channels', type=int, default=256)
    parser.add_argument('--heads', type=int, default=8)
    parser.add_argument('--dropout', type=float, default=0.5)
    parser.add_argument('--weight_decay', type=float, default=5e-5)
    parser.add_argument('--lr', type=float, default=5e-4)
    parser.add_argument('--epochs', type=int, default=500)
    parser.add_argument('--batch_size', type=int, default=64)
    parser.add_argument('--num_hops', type=int, default=5)
    parser.add_argument('--runs', type=int, default=1)
    parser.add_argument('--mapped', type=int, default=0)
    parser.add_argument('--lda1', type=int, default=5)
    parser.add_argument('--lda2', type=int, default=1)
    parser.add_argument('--root_dir', type=str, default='data_ml')
    parser.add_argument('--directed', action='store_true')
    parser.add_argument('--test_all_bits', action='store_true')
    parser.add_argument('--save_model', action='store_true')
    parser.add_argument('--num_fc_layer', type=int, default=2)
    parser.add_argument('--gnn_embedding_dim', type=int, default=128)
    parser.add_argument('--num_epochs', type=int, default=80)
    parser.add_argument('--feature_size', type=int, default=4)
    args = parser.parse_args()
    device = f'cuda:{args.device}' if torch.cuda.is_available() else 'cpu'
    model = SynthNet(args).to(device)
    model.load_state_dict(torch.load('/data/cchen/esynturbo/E-syn2/HOGA/dump/hoga-epoch-74-val_loss-308515.542.pt'))

    start_time = time.time()
    root_path = os.path.abspath(os.path.join(os.getcwd(), "..", "..", "HOGA", "dataset_4_test"))
    print("root_path: ",root_path)
    target_path = os.path.abspath(os.path.join(os.getcwd(), "..", "..", "HOGA", "dataset_4_test_process"))
    if not os.path.exists(target_path):
        os.makedirs(target_path)
    copy_files(root_path, target_path)
    process_dataset(target_path)
    processed_dir1 = os.path.join(target_path, 'processed')
    if not os.path.exists(processed_dir1):
       os.makedirs(processed_dir1, exist_ok=True)    
    dataset = MyOwnDataset_4test(root=target_path)
    pred = evaluate_4_test(self.model, self.device, dataset)
    pred = float(pred[0])
    print(f"pred: {pred}")      
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    service_pb2_grpc.add_VectorServiceServicer_to_server(VectorServiceServicer(model, device), server)
    #service_pb2_grpc.add_VectorServiceServicer_to_server(VectorServiceServicer(), server)
    server.add_insecure_port('[::]:50051')
    server.start()
    server.wait_for_termination()


if __name__ == '__main__':
    serve()