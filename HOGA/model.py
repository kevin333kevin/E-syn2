import torch
import torch.nn.functional as F
from torch.nn import Linear, BatchNorm1d, LayerNorm, Dropout, Softmax
from torch_geometric.nn import global_mean_pool, global_max_pool

'''
Slightly modified multihead attention for Gamora
'''
class MultiheadAttentionMix(torch.nn.Module):
    def __init__(self, input_dim, num_heads, dropout=0.0):
        super(MultiheadAttentionMix, self).__init__()

        self.input_dim = input_dim
        self.num_heads = num_heads
        self.head_dim = input_dim // num_heads

        # Linear projections for queries, keys, and values
        self.query_projection = Linear(input_dim, input_dim)
        self.key_projection = Linear(input_dim, input_dim)
        self.value_projection = Linear(input_dim, input_dim)

        # Linear projection for the output of the attention heads
        self.output_projection = Linear(input_dim, input_dim)

        self.dropout = Dropout(dropout)
        self.softmax = Softmax(dim=-1)

    def forward(self, query, key, value, mask=None):
        batch_size = query.size(0)

        # Linear projections for queries, keys, and values
        query = self.query_projection(query)
        key = self.key_projection(key)
        value = self.value_projection(value)

        # Reshape the projected queries, keys, and values
        query = query.view(batch_size * self.num_heads, -1, self.head_dim)
        key = key.view(batch_size * self.num_heads, -1, self.head_dim)
        value = value.view(batch_size * self.num_heads, -1, self.head_dim)

        # Compute the scaled dot-product attention
        attention_scores = torch.bmm(query, key.transpose(1, 2))
        attention_scores = attention_scores / torch.sqrt(torch.tensor(self.head_dim, dtype=torch.float32))

        # Apply the mask (if provided)
        if mask is not None:
            mask = mask.unsqueeze(1)  # Add head dimension
            attention_scores = attention_scores.masked_fill(mask == 0, float("-inf"))

        attention_probs = self.softmax(attention_scores)
        attention_probs = self.dropout(attention_probs)

        # Compute the output of the attention heads
        attention_output = torch.bmm(attention_probs, value)

        # Reshape and project the output of the attention heads
        attention_output = attention_output.view(batch_size, -1, self.input_dim)
        attention_output = self.output_projection(attention_output)

        return attention_output, attention_probs

'''
Vanilla multihead attention (recommended for general use cases)
'''
class MultiheadAttention(torch.nn.Module):
    def __init__(self, input_dim, num_heads, dropout=0.0):
        super(MultiheadAttention, self).__init__()

        self.input_dim = input_dim
        self.num_heads = num_heads
        self.head_dim = input_dim // num_heads

        # Linear projections for queries, keys, and values
        self.query_projection = Linear(input_dim, input_dim)
        self.key_projection = Linear(input_dim, input_dim)
        self.value_projection = Linear(input_dim, input_dim)

        # Linear projection for the output of the attention heads
        self.output_projection = Linear(input_dim, input_dim)

        self.dropout = Dropout(dropout)
        self.softmax = Softmax(dim=-1)

    def forward(self, query, key, value, mask=None):
        batch_size, seq_len, _ = query.size()

        # Linear projections for queries, keys, and values
        query = self.query_projection(query)
        key = self.key_projection(key)
        value = self.value_projection(value)

        # Reshape the projected queries, keys, and values
        query = query.view(batch_size, seq_len, self.head_dim, -1)
        key = key.view(batch_size, seq_len, self.head_dim, -1)
        value = value.view(batch_size, seq_len, self.head_dim, -1)

        # Compute the scaled dot-product attention
        attention_scores = torch.einsum('bldh, bndh -> blnh', query, key)
        attention_scores = attention_scores / torch.sqrt(torch.tensor(self.head_dim, dtype=torch.float32))

        attention_probs = self.softmax(attention_scores)
        attention_probs = self.dropout(attention_probs)

        # Compute the output of the attention heads
        attention_output = torch.einsum('blnh, bndh -> bldh', attention_probs, value)

        # Reshape and project the output of the attention heads
        attention_output = attention_output.reshape(batch_size, seq_len, self.input_dim)
        attention_output = self.output_projection(attention_output)

        return attention_output, attention_probs

# class HOGA(torch.nn.Module):
#     # def __init__(self, in_channels, hidden_channels, out_channels, num_layers,
#     #              dropout, num_hops, heads, attn_dropout=0.0, attn_type="vanilla", use_bias=False):
#     def __init__(self, in_channels, hidden_channels, out_channels, num_layers,
#                  dropout, num_hops, heads, attn_dropout=0.0, attn_type="vanilla", use_bias=False):
#         super(HOGA, self).__init__()
#         self.num_layers = num_layers
#         self.num_hops = num_hops

#         self.lins = torch.nn.ModuleList()
#         self.gates = torch.nn.ModuleList()
#         self.trans = torch.nn.ModuleList()
#         self.lns = torch.nn.ModuleList()
#         self.lins.append(Linear(in_channels, hidden_channels, bias=use_bias))
#         self.lins.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
#         self.lins.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
#         self.gates.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
#         if attn_type == "vanilla":
#             self.trans.append(MultiheadAttention(hidden_channels, heads, dropout=attn_dropout))
#         else:
#             self.trans.append(MultiheadAttentionMix(hidden_channels, heads, dropout=attn_dropout))
#         self.lns.append(LayerNorm(hidden_channels))
#         for _ in range(num_layers - 1):
#             self.lins.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
#             self.gates.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
#             if attn_type == "vanilla":
#                 self.trans.append(MultiheadAttention(hidden_channels, heads, dropout=attn_dropout))
#             else:
#                 self.trans.append(MultiheadAttentionMix(hidden_channels, heads, dropout=attn_dropout))
#             self.lns.append(LayerNorm(hidden_channels))

#         # Linear layers for predictions
#         # self.linear = torch.nn.ModuleList()
#         # self.linear.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
#         # self.linear.append(Linear(hidden_channels, out_channels, bias=use_bias))
#         # self.linear.append(Linear(hidden_channels, out_channels, bias=use_bias))
#         # self.linear.append(Linear(hidden_channels, out_channels, bias=use_bias))

#         self.bn = BatchNorm1d(hidden_channels)
#         self.attn_layer = Linear(2 * hidden_channels, 1)

#         self.dropout = dropout

#     def reset_parameters(self):
#         for lin in self.lins:
#             lin.reset_parameters()
#         for gate in self.gates:
#             gate.reset_parameters()
#         for li in self.linear:
#             li.reset_parameters()
#         self.bn.reset_parameters()

#     def forward(self, x):
#         # Current implementation: use a shared linear layer for all hop-wise features
#         # Note: apply separate layers for different hop-wise features may further improve accuracy
#         x = self.lins[0](x)

#         for i, tran in enumerate(self.trans):
#             x = self.lns[i](self.gates[i](x)*(tran(x, x, x)[0]))
#             x = F.relu(x)
#             x = F.dropout(x, p=self.dropout, training=self.training)

#         target = x[:,0,:].unsqueeze(1).repeat(1,self.num_hops-1,1)
#         split_tensor = torch.split(x, [1, self.num_hops-1], dim=1)
#         node_tensor = split_tensor[0]
#         neighbor_tensor = split_tensor[1]
#         layer_atten = self.attn_layer(torch.cat((target, neighbor_tensor), dim=2))
#         layer_atten = F.softmax(layer_atten, dim=1)
#         neighbor_tensor = neighbor_tensor * layer_atten
#         neighbor_tensor = torch.sum(neighbor_tensor, dim=1, keepdim=True)
#         x = (node_tensor + neighbor_tensor).squeeze()
#         # x = self.linear[0](x)
#         # x = self.bn(F.relu(x))
#         # x = F.dropout(x, p=self.dropout, training=self.training)

#         # x1 = self.linear[1](x) # for xor
#         # x2 = self.linear[2](x) # for maj
#         # x3 = self.linear[3](x) # for roots

#         return x, layer_atten
class HOGA(torch.nn.Module):
    # def __init__(self, in_channels, hidden_channels, out_channels, num_layers,
    #              dropout, num_hops, heads, attn_dropout=0.0, attn_type="vanilla", use_bias=False):
    def __init__(self, in_channels, hidden_channels, num_layers,
                 dropout, num_hops, heads, attn_dropout=0.0, attn_type="vanilla", use_bias=False):
        super(HOGA, self).__init__()
        self.num_layers = num_layers
        self.num_hops = num_hops

        self.lins = torch.nn.ModuleList()
        self.gates = torch.nn.ModuleList()
        self.trans = torch.nn.ModuleList()
        self.lns = torch.nn.ModuleList()
        self.lins.append(Linear(in_channels, hidden_channels, bias=use_bias))
        self.lins.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
        self.lins.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
        self.gates.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
        if attn_type == "vanilla":
            self.trans.append(MultiheadAttention(hidden_channels, heads, dropout=attn_dropout))
        else:
            self.trans.append(MultiheadAttentionMix(hidden_channels, heads, dropout=attn_dropout))
        self.lns.append(LayerNorm(hidden_channels))
        for _ in range(num_layers - 1):
            self.lins.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
            self.gates.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
            if attn_type == "vanilla":
                self.trans.append(MultiheadAttention(hidden_channels, heads, dropout=attn_dropout))
            else:
                self.trans.append(MultiheadAttentionMix(hidden_channels, heads, dropout=attn_dropout))
            self.lns.append(LayerNorm(hidden_channels))

        # Linear layers for predictions
        # self.linear = torch.nn.ModuleList()
        # self.linear.append(Linear(hidden_channels, hidden_channels, bias=use_bias))
        # self.linear.append(Linear(hidden_channels, out_channels, bias=use_bias))
        # self.linear.append(Linear(hidden_channels, out_channels, bias=use_bias))
        # self.linear.append(Linear(hidden_channels, out_channels, bias=use_bias))

        self.bn = BatchNorm1d(hidden_channels)
        self.attn_layer = Linear(2 * hidden_channels, 1)

        self.dropout = dropout

    def reset_parameters(self):
        for lin in self.lins:
            lin.reset_parameters()
        for gate in self.gates:
            gate.reset_parameters()
        for li in self.linear:
            li.reset_parameters()
        self.bn.reset_parameters()

    def forward(self, x):
        # Current implementation: use a shared linear layer for all hop-wise features
        # Note: apply separate layers for different hop-wise features may further improve accuracy
        x = self.lins[0](x)

        for i, tran in enumerate(self.trans):
            x = self.lns[i](self.gates[i](x)*(tran(x, x, x)[0]))
            x = F.relu(x)
            x = F.dropout(x, p=self.dropout, training=self.training)

        target = x[:,0,:].unsqueeze(1).repeat(1,self.num_hops-1,1)
        split_tensor = torch.split(x, [1, self.num_hops-1], dim=1)
        node_tensor = split_tensor[0]
        neighbor_tensor = split_tensor[1]
        layer_atten = self.attn_layer(torch.cat((target, neighbor_tensor), dim=2))
        layer_atten = F.softmax(layer_atten, dim=1)
        neighbor_tensor = neighbor_tensor * layer_atten
        neighbor_tensor = torch.sum(neighbor_tensor, dim=1, keepdim=True)
        x = (node_tensor + neighbor_tensor).squeeze()
        # x = self.linear[0](x)
        # x = self.bn(F.relu(x))
        # x = F.dropout(x, p=self.dropout, training=self.training)

        # x1 = self.linear[1](x) # for xor
        # x2 = self.linear[2](x) # for maj
        # x3 = self.linear[3](x) # for roots

        return x
class GNN_node(torch.nn.Module):
    """
    Output:
        node representations
    """

    def __init__(self, in_channels, gnn_emb_dim, num_gnn_layers, heads, num_hops, dropout):
        '''
            emb_dim (int): node embedding dimensionality
            num_layer (int): number of GNN message passing layers
        '''
        super(GNN_node, self).__init__()
        self.num_layers = num_gnn_layers
        self.in_channels = in_channels
        self.hidden_channels = gnn_emb_dim
        self.heads = heads
        self.num_hops = num_hops
        self.dropout = dropout

        ###List of GNNs
        self.convs = torch.nn.ModuleList()
        self.batch_norms = torch.nn.ModuleList()
        self.convs.append(HOGA(in_channels, self.hidden_channels, num_gnn_layers, dropout, num_hops + 1, heads))
        self.batch_norms.append(torch.nn.BatchNorm1d(self.hidden_channels))
        for layer in range(1, self.num_layers):
            self.convs.append(HOGA(in_channels, self.hidden_channels, num_gnn_layers, dropout, num_hops + 1, heads))
            self.batch_norms.append(torch.nn.BatchNorm1d(self.hidden_channels))

    def forward(self, batched_data):
        h = batched_data
        for layer in range(self.num_layers):
            h = self.convs[layer](h)
            h = self.batch_norms[layer](h)
            if layer != self.num_layers - 1:
                h = F.relu(h)
        return h

class GNN(torch.nn.Module):
    #self.gnn = GNN( self.in_channels,self.gnn_emb_dim, self.num_gnn_layers, 
    # self.heads, self.num_hops, self.dropout)
    def __init__(self, in_channels, gnn_emb_dim, num_gnn_layers,heads,num_hops,dropout):
        super(GNN, self).__init__()
        self.gnn_node = GNN_node(in_channels, gnn_emb_dim, num_gnn_layers,heads,num_hops,dropout)
        self.pool1 = global_mean_pool
        self.pool2 = global_max_pool

    def forward(self, batched_data):
       # print("batched_data.x.shape: ",batched_data.x.shape)
        h = self.gnn_node(batched_data.x.float())
       # print("h shape",h.shape)
        batch =batched_data.batch
        #print(f"h_node shape: {h_node.shape}")
        h_graph1 = self.pool1(h, batched_data.batch)
       # print(f"h_graph1 shape: {h_graph1.shape}")
        h_graph2 = self.pool2(h,batched_data.batch)
       # print(f"h_graph2 shape: {h_graph2.shape}")
       # print("output shape: "  ,(torch.cat([h_graph1,h_graph2],dim=1)).shape)
        xF=torch.cat([h_graph1,h_graph2], dim=1)
       # print(f"xF shape: {xF.shape}")
        return xF

    # def forward(self, batched_data):
    #     batch = batched_data.batch
    #     h = batched_data.x.float()
    #     h = F.relu(self.batch_norm1(self.conv1(h, edge_index)))
    #     #h = F.relu(self.batch_norm2(self.conv2(h, edge_index)))
    #     h = self.batch_norm2(self.conv2(h, edge_index))

    #     xF = torch.cat([global_max_pool(h, batch), global_mean_pool(h, batch)], dim=1)

    #     return xF
class SynthNet(torch.nn.Module):
    def __init__(self, args):
        super(SynthNet, self).__init__()
        self.in_channels = args.feature_size
        self.num_fc_layers = args.num_fc_layer
        self.num_gnn_layers = args.num_layers
        
        
        self.gnn_emb_dim = args.gnn_embedding_dim


        self.hidden_dim = args.hidden_channels
        self.heads = args.heads
        self.num_hops=args.num_hops         
       
        self.dropout = args.dropout

        self.n_classes = 1


        self.gnn = GNN( self.in_channels,self.gnn_emb_dim, self.num_gnn_layers, self.heads, self.num_hops, self.dropout)
        self.fcs = torch.nn.ModuleList()
        self.batch_norms = torch.nn.ModuleList()
        self.fcs.append(torch.nn.Linear(self.gnn_emb_dim*2,self.hidden_dim))
        for layer in range(1, self.num_fc_layers-1):
            self.fcs.append(torch.nn.Linear(self.hidden_dim,self.hidden_dim))

        self.fcs.append(torch.nn.Linear(self.hidden_dim, self.n_classes))


    def forward(self,batch_data):
        graphEmbed = self.gnn(batch_data)
       # print(f"graphEmbed shape: {graphEmbed.shape}")
        x = F.relu(self.fcs[0](graphEmbed))
        for layer in range(1, self.num_fc_layers-1):
            x = F.relu(self.fcs[layer](x))
        x = self.fcs[-1](x)
        return x

    # Todo:
    # def reset_parameters(self):
    #     for lin in self.gnn.gnn_node.convs.lins:
    #         lin.reset_parameters()
    #     for gate in self.gates:
    #         gate.reset_parameters()
    #     for li in self.linear:
    #         li.reset_parameters()
    #     self.bn.reset_parameters()