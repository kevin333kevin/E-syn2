import torch
import torch.nn as nn
import torch.nn.functional as F
import dgl
import dgl.function as fn
from dgl.dataloading import GraphDataLoader
from sklearn.model_selection import train_test_split

# GCN model
class GCN(nn.Module):
    def __init__(self, in_feats, hid_feats, out_feats):
        super(GCN, self).__init__()
        self.conv1 = dgl.nn.GraphConv(in_feats, hid_feats)
        self.conv2 = dgl.nn.GraphConv(hid_feats, out_feats)

    def forward(self, g, in_feat):
        h = self.conv1(g, in_feat)
        h = F.relu(h)
        h = self.conv2(g, h)
        g.ndata['h'] = h
        return dgl.mean_nodes(g, 'h')

# GAT model
class GAT(nn.Module):
    def __init__(self, in_feats, hid_feats, out_feats, num_heads):
        super(GAT, self).__init__()
        self.conv1 = dgl.nn.GATConv(in_feats, hid_feats, num_heads)
        self.conv2 = dgl.nn.GATConv(hid_feats * num_heads, out_feats, 1)

    def forward(self, g, in_feat):
        h = self.conv1(g, in_feat).flatten(1)
        h = F.relu(h)
        h = self.conv2(g, h).mean(1)
        g.ndata['h'] = h
        return dgl.mean_nodes(g, 'h')

# GraphSAGE model
class GraphSAGE(nn.Module):
    def __init__(self, in_feats, hid_feats, out_feats):
        super(GraphSAGE, self).__init__()
        self.conv1 = dgl.nn.SAGEConv(in_feats, hid_feats, 'mean')
        self.conv2 = dgl.nn.SAGEConv(hid_feats, out_feats, 'mean')

    def forward(self, g, in_feat):
        h = self.conv1(g, in_feat)
        h = F.relu(h)
        h = self.conv2(g, h)
        g.ndata['h'] = h
        return dgl.mean_nodes(g, 'h')

# APPNP model
class APPNP(nn.Module):
    def __init__(self, in_feats, hid_feats, out_feats, k, alpha):
        super(APPNP, self).__init__()
        self.lin1 = nn.Linear(in_feats, hid_feats)
        self.lin2 = nn.Linear(hid_feats, out_feats)
        self.prop1 = dgl.nn.APPNPConv(k, alpha)

    def forward(self, g, in_feat):
        h = self.lin1(in_feat)
        h = F.relu(h)
        h = self.lin2(h)
        h = self.prop1(g, h)
        return h

# Load the dataset
dataset = dgl.load_graphs('./graph_dataset.bin')[0]

# Split the dataset into training and testing sets
train_graphs, test_graphs = train_test_split(dataset, test_size=0.2, random_state=42)

# Create dataloaders
train_dataloader = GraphDataLoader(train_graphs, batch_size=32, shuffle=True)
test_dataloader = GraphDataLoader(test_graphs, batch_size=32)

# Models, loss function, and optimizer
models = {
    'GCN': GCN(train_graphs[0].ndata['op'].shape[1], 16, 1),
    'GAT': GAT(train_graphs[0].ndata['op'].shape[1], 8, 1, num_heads=8),
    'GraphSAGE': GraphSAGE(train_graphs[0].ndata['op'].shape[1], 16, 1),
    'APPNP': APPNP(train_graphs[0].ndata['op'].shape[1], 16, 1, k=10, alpha=0.1)
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