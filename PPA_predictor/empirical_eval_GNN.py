import torch
import torch.nn as nn
import torch.nn.functional as F
import dgl
import dgl.function as fn
from dgl.dataloading import GraphDataLoader
from sklearn.model_selection import train_test_split
from models4GNN import GCNModel, GATModel, GraphSAGEModel, APPNPModel, GINModel

# Load the dataset
dataset = dgl.load_graphs('./graph_dataset.bin')[0]

# Split the dataset into training and testing sets
train_graphs, test_graphs = train_test_split(dataset, test_size=0.2, random_state=42)

# Create dataloaders
train_dataloader = GraphDataLoader(train_graphs, batch_size=32, shuffle=True)
test_dataloader = GraphDataLoader(test_graphs, batch_size=32)

# Models, loss function, and optimizer
models = {
    'GCN': GCNModel(train_graphs[0].ndata['op'].shape[1], 16, 1),
    'GAT': GATModel(train_graphs[0].ndata['op'].shape[1], 8, 1, num_heads=8),
    'GraphSAGE': GraphSAGEModel(train_graphs[0].ndata['op'].shape[1], 16, 1),
    'APPNP': APPNPModel(train_graphs[0].ndata['op'].shape[1], 16, 1, k=10, alpha=0.1),
    'GIN': GINModel(train_graphs[0].ndata['op'].shape[1], 16, 1, num_layers=2)
}
loss_func = nn.MSELoss()

# Training and evaluation
for model_name, model in models.items():
    optimizer = torch.optim.Adam(model.parameters(), lr=0.01)
    
    # Training loop
    for epoch in range(100):
        model.train()
        for batched_graph in train_dataloader:
            pred = model(batched_graph, batched_graph.ndata['op'].float())
            loss = loss_func(pred, batched_graph.ndata['labels'].float())
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
    
    # Evaluation
    model.eval()
    total_error = 0
    for batched_graph in test_dataloader:
        pred = model(batched_graph, batched_graph.ndata['op'].float())
        total_error += loss_func(pred, batched_graph.ndata['labels'].float()).item()
    
    mean_error = total_error / len(test_dataloader)
    print(f'{model_name} - Mean Squared Error on test set: {mean_error:.4f}')