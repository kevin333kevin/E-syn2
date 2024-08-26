import argparse

import torch
import torch.utils.data as Data
from torch.utils.data import random_split
import torch_geometric.transforms as T
import torch_geometric
#from logger import Logger
import os
import sys
import numpy as np
import pandas as pd
import copy
import torch
from utils import *
from dataset_prep import PygNodePropPredDataset, Evaluator,PygNodePropPredDataset_4test
from model import HOGA,SynthNet
from torch_geometric.data import Dataset, Data
from functools import partial
import os.path as osp
from torch_geometric.loader import DataLoader
from tqdm import tqdm
from torch.optim.lr_scheduler import ReduceLROnPlateau
import pickle

import matplotlib.pyplot as plt


def plotChart(x,y,xlabel,ylabel,leg_label,title,DUMP_DIR):
    fig = plt.figure(figsize=(10,6))
    ax = fig.add_subplot(1, 1, 1)
    plt.plot(x,y, label=leg_label)
    leg = plt.legend(loc='best', ncol=2, shadow=True, fancybox=True)
    leg.get_frame().set_alpha(0.5)
    plt.xlabel(xlabel, weight='bold')
    plt.ylabel(ylabel, weight='bold')
    plt.title(title,weight='bold')
    plt.savefig(osp.join(DUMP_DIR,title+'.png'), format='png', bbox_inches='tight')

def evaluate_plot(model, device, dataloader):
    model.eval()
    totalMSE = AverageMeter()
    batchData = []
    with torch.no_grad():
        for _, batch in enumerate(tqdm(dataloader, desc="Iteration",file=sys.stdout)):
            batch = batch.to(device)
            pred = model(batch)
            lbl = batch.y.float().reshape(-1, 1)
            desName = batch.desName
            synID = batch.synID
            predArray = pred.view(-1,1).detach().cpu().numpy()
            actualArray = lbl.view(-1,1).detach().cpu().numpy()
            batchData.append([predArray,actualArray,desName,synID])
            mseVal = mse(pred, lbl)
            numInputs = pred.view(-1,1).size(0)
            totalMSE.update(mseVal,numInputs)
        # for i, data in enumerate(batchData):
        #     predArray, actualArray, desName, synID = data
        #     print(f"Batch {i} shape: predArray={predArray.shape}, actualArray={actualArray.shape}, desName={len(desName)}, synID={len(synID)}")
    return totalMSE.avg,batchData



class MyOwnDataset(Dataset):
    def __init__(self, root):
        super().__init__(root)
        self.root = root

    @property
    def raw_file_names(self):
        return os.listdir(self.root)

    @property
    def processed_file_names(self):
        file_names = os.listdir(self.processed_dir)
        return [f for f in file_names if f not in ['pre_transform.pt', 'pre_filter.pt']]

    def download(self):
        # Download to `self.raw_dir`.
        pass
        ...

    def process(self):
        idx = 0
        synID_dict = {}
        for design_name in os.listdir(self.root):
            if design_name != 'processed' and design_name != 'raw' and design_name != 'dataset_esyn':
                master = pd.read_csv('dataset_prep/master.csv', index_col=0)
                if design_name not in master:
                    os.system(f"python dataset_prep/make_master_file.py --design_name {design_name}")
                dataset_r = PygNodePropPredDataset(name=f'{design_name}', root=self.root)
                data_r = dataset_r[0]
                data_r = T.ToSparseTensor()(data_r)
                dataset = PygNodePropPredDataset(name=f'{design_name}', root=self.root)
                data = dataset[0]
                data = preprocess(data)
                data = T.ToSparseTensor()(data)
    
                # Add 'desName' and 'synID' key-value pairs
                design_name_parts = design_name.split('_')
                data['desName'] = design_name_parts[0]
    
                if design_name_parts[0] not in synID_dict:
                    synID_dict[design_name_parts[0]] = 1
                data['synID'] = synID_dict[design_name_parts[0]]
                synID_dict[design_name_parts[0]] += 1
    
                torch.save(data, osp.join(self.processed_dir, f'data_{idx}.pt'))
                idx += 1
            
    def len(self):
        return len(self.processed_file_names)

    def get(self, idx):
        data = torch.load(osp.join(self.processed_dir, f'data_{idx}.pt'))
        return data


class MyOwnDataset_4test(Dataset):
    def __init__(self, root):
        super().__init__(root)
        self.root = root

    @property
    def raw_file_names(self):
        return os.listdir(self.root)

    @property
    def processed_file_names(self):
        file_names = os.listdir(self.processed_dir)
        return [f for f in file_names if f not in ['pre_transform.pt', 'pre_filter.pt']]

    def download(self):
        pass
        ...

    def process(self):
        idx = 0
        synID_dict = {}
        for design_name in os.listdir(self.root):

            if design_name != 'processed' and design_name != 'raw' :
                master_path = os.path.join(os.path.dirname(self.root), 'dataset_prep', 'master.csv')
                master = pd.read_csv(master_path, index_col=0)
                os.system(f"python dataset_prep/make_master_file.py --design_name {design_name}")
                dataset = PygNodePropPredDataset_4test(name=f'{design_name}', root=self.root)
                data = dataset[0]
                print(data)
                data = preprocess(data)
                data = T.ToSparseTensor()(data)
    
                # Add 'desName' and 'synID' key-value pairs
                design_name_parts = design_name.split('_')
                data['desName'] = design_name_parts[0]
    
                if design_name_parts[0] not in synID_dict:
                    synID_dict[design_name_parts[0]] = 1
                data['synID'] = synID_dict[design_name_parts[0]]
                synID_dict[design_name_parts[0]] += 1
    
                torch.save(data, osp.join(self.processed_dir, f'data_{idx}.pt'))
                idx += 1
            
    def len(self):
        return len(self.processed_file_names)

    def get(self, idx):
        data = torch.load(osp.join(self.processed_dir, f'data_{idx}.pt'))
        return data


def main():
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
    args = parser.parse_args()

    
    print(args)
    device = f'cuda:{args.device}' if torch.cuda.is_available() else 'cpu'
    # device = torch.device('cpu') ## cpu for now only

    if not os.path.exists(f'models/'):
        os.makedirs(f'models/')

    if args.mapped == 1:
        suffix ="_7nm_mapped"
    elif args.mapped == 2:
        suffix ="_mapped"
    else:
        suffix = ''

    root_path = "dataset_esyn"
    DUMP_DIR = "dump"
    if not os.path.exists(DUMP_DIR):
        os.makedirs(DUMP_DIR)
    print(f"Loading dataset from {root_path}")


    processed_dir1 = os.path.join(root_path, 'processed')
    if not os.path.exists(processed_dir1):
       os.makedirs(processed_dir1, exist_ok=True)    
    dataset = MyOwnDataset(root=root_path)
   # print(f"dataset size: {len(dataset)}")

    #saving processed data
    current_dir = os.getcwd()
    
    processed_dir = os.path.join(current_dir, 'processed_data')
    if not os.path.exists(processed_dir):
       os.makedirs(processed_dir, exist_ok=True)
 
 
 
    #torch.save(dataset, os.path.join(processed_dir, 'data_processed.pt'))
    
    
    
    #reading processed data dataset
    dataset1 = torch.load(os.path.join(processed_dir, 'data_processed.pt'))

    
    
    
    
    #split dataset into train, val, test
    train_size = int(0.8 * len(dataset1))
    val_size = int(0.1 * len(dataset1))
    test_size = len(dataset1) - train_size - val_size
    train_dataset, val_dataset, test_dataset = random_split(dataset1, [train_size, val_size, test_size])
    print(f"Training dataset size: {len(train_dataset)}")
    print(f"Validation dataset size: {len(val_dataset)}")
    print(f"Test dataset size: {len(test_dataset)}")
    print(train_dataset[0])
    args.feature_size = dataset1[0].num_features
    print(f"Feature size: {args.feature_size}")
    # load dataset into dataloader
    train_dl = DataLoader(train_dataset, batch_size=args.batch_size, shuffle=True, num_workers=10)
    batch_next = next(iter(train_dl))
    
    print(f"First batch data:")
    for key, value in batch_next.items():
        if key == 'x' or key == 'y' or key == 'batch':
            print(f"{key} shape: {value.shape}")
            print(f"{key} value: {value}")
    for batch in train_dl:
       print(batch.x.shape, batch.y.shape,batch.batch.shape)               

    valid_dl = DataLoader(val_dataset, batch_size=args.batch_size, shuffle=True, num_workers=10)
    test_dl = DataLoader(test_dataset, batch_size=args.batch_size, shuffle=True, num_workers=10)
    model = SynthNet(args).to(device)
    # print(train_dl[0])



    
    learning_rate = args.lr #0.001
    optimizer = torch.optim.Adam(model.parameters(),lr=learning_rate)
    scheduler = ReduceLROnPlateau(optimizer, 'min',verbose=True)
    valid_curve = []
    train_loss = []
    validLossOpt = 0
    bestValEpoch = 0
    # for ep in range(1, args.num_epochs + 1):
    #     print("\nEpoch [{}/{}]".format(ep, args.num_epochs))
    #     print("\nTraining..")
    #     trainLoss = train(model, device, train_dl, optimizer)
    #     print("\nEvaluation..")
    #     validLoss = evaluate(model, device, valid_dl)
    #     if ep > 1:
    #         if validLossOpt > validLoss:
    #             validLossOpt = validLoss
    #             bestValEpoch = ep
    #             torch.save(model.state_dict(), osp.join(DUMP_DIR, 'hoga-epoch-{}-val_loss-{:.3f}.pt'.format(bestValEpoch, validLossOpt)))
    #     else:
    #         validLossOpt = validLoss
    #         torch.save(model.state_dict(), osp.join(DUMP_DIR, 'hoga-epoch-{}-val_loss-{:.3f}.pt'.format(bestValEpoch, validLossOpt)))
    #     print({'Train loss': trainLoss,'Validation loss': validLoss})
    #     valid_curve.append(validLoss)
    #     train_loss.append(trainLoss)
    #     scheduler.step(validLoss)
    # with open(osp.join(DUMP_DIR,'valid_curve.pkl'),'wb') as f:
    #     pickle.dump(valid_curve,f)

    # with open(osp.join(DUMP_DIR,'train_loss.pkl'),'wb') as f:
    #     pickle.dump(train_loss,f)
    ##### EVALUATION ######
    
    plotChart([i+1 for i in range(len(valid_curve))],valid_curve,"# Epochs","Loss","test_acc","Validation loss",DUMP_DIR)
    plotChart([i+1 for i in range(len(train_loss))],train_loss,"# Epochs","Loss","train_loss","Training loss",DUMP_DIR)
    # bestValEpoch = 74
    # validLossOpt = 308515.542
    # Loading best validation model
    model.load_state_dict(torch.load(osp.join(DUMP_DIR, 'hoga-epoch-{}-val_loss-{:.3f}.pt'.format(bestValEpoch, validLossOpt))))

    batchSize = args.batch_size #64
    # Evaluate on train data
    trainMSE,trainBatchData = evaluate_plot(model, device, train_dl)
    NUM_BATCHES_TRAIN = len(train_dl)

    doScatterAndTopKRanking(NUM_BATCHES_TRAIN,batchSize,trainBatchData,DUMP_DIR,"train")

    # Evaluate on validation data
    validMSE,validBatchData = evaluate_plot(model, device, valid_dl)
    NUM_BATCHES_VALID = len(valid_dl)
    doScatterAndTopKRanking(NUM_BATCHES_VALID,batchSize,validBatchData,DUMP_DIR,"valid")

    # Evaluate on test data
    testMSE,testBatchData = evaluate_plot(model, device, test_dl)
    NUM_BATCHES_TEST = len(test_dl)
    doScatterAndTopKRanking(NUM_BATCHES_TEST,batchSize,testBatchData,DUMP_DIR,"test")
    
    num_params = sum(p.numel() for p in model.parameters())
    
    print("********************")
    print("Final run statistics")
    print("********************")
    print(f'Total Params: {num_params}')
    print("Training loss per sample:{}".format(trainMSE))
    print("Validation loss per sample:{}".format(validMSE))
    print("Test loss per sample:{}".format(testMSE))
    print("********************")


criterion = torch.nn.MSELoss()
def train(model,device,dataloader,optimizer):
    epochLoss = AverageMeter()
    model.train()
    for _, batch in enumerate(tqdm(dataloader, desc="Iteration",file=sys.stdout)):
        batch = batch.to(device)
        # print(f"Batch data:")
        # for key, value in batch.__dict__.items():
        #     print(f"{key}: {value}")        
        lbl = batch.y.float()
        optimizer.zero_grad()
        pred = model(batch)
        loss = criterion(pred,lbl)
        loss.backward()
        optimizer.step()
        numInputs = pred.view(-1,1).size(0)
        epochLoss.update(loss.detach().item(),numInputs)
    return epochLoss.avg

def evaluate(model, device, dataloader):
    model.eval()
    validLoss = AverageMeter()
    with torch.no_grad():
        for _, batch in enumerate(tqdm(dataloader, desc="Iteration",file=sys.stdout)):
            batch = batch.to(device)
            pred = model(batch)
            lbl = batch.y.float()
            mseVal = mse(pred, lbl)
            numInputs = pred.view(-1,1).size(0)
            validLoss.update(mseVal,numInputs)
    return validLoss.avg



if __name__ == "__main__":
    main()