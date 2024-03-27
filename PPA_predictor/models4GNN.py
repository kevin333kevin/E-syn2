# GNN_model.py
import dgl
import torch
import torch.nn as nn
import torch.nn.functional as F
from dgl.nn import GINConv

# Define the GNN model
class GINModel(nn.Module):
    def __init__(self, input_dim, hidden_dim, output_dim, num_layers):
        super(GINModel, self).__init__()
        self.layers = nn.ModuleList()
        self.batch_norms = nn.ModuleList()

        for i in range(num_layers):
            if i == 0:
                mlp = nn.Sequential(
                    nn.Linear(input_dim, hidden_dim),
                    nn.ReLU(),
                    nn.Linear(hidden_dim, hidden_dim)
                )
            else:
                mlp = nn.Sequential(
                    nn.Linear(hidden_dim, hidden_dim),
                    nn.ReLU(),
                    nn.Linear(hidden_dim, hidden_dim)
                )
            self.layers.append(GINConv(mlp, 'mean'))
            self.batch_norms.append(nn.BatchNorm1d(hidden_dim))

        self.linear_pred = nn.Linear(hidden_dim, output_dim)

    def forward(self, g, features):
        h = features
        for layer, batch_norm in zip(self.layers, self.batch_norms):
            h = layer(g, h)
            h = batch_norm(h)
            h = F.relu(h)
        g.ndata['h'] = h
        hg = dgl.mean_nodes(g, 'h')
        return self.linear_pred(hg)

# Save and load functions for the model
def save_model(model, filename):
    torch.save(model.state_dict(), filename)

def load_model(model, filename):
    model.load_state_dict(torch.load(filename))
    model.eval()
    return model