# Flatnav-rs 

Rust implementation of HNSW inspired by https://github.com/brc7/flatnav.

## Installation 
1. Clone the repo:
   ```bash
   https://github.com/nmeisburger/flatnav-rs
   ```
2. Navigate into the directory:
   ```bash
   cd flatnav-rs
   ```
3. Install python library:
   ```bash
   pip3 install .
   ```

## Example Usage
```python
import flatnav

index = flatnav.IndexEuclideanF32(max_nbrs=32, data_dim=784, capacity=60000)

for i, sample in tqdm.tqdm(enumerate(train), total=len(train)):
        index.insert(label=i, data=sample, ef_construction=64)

for query in tqdm.tqdm(test, total=len(test)):
        results = index.query(query=query, ef_search=64, topk=10)
```
