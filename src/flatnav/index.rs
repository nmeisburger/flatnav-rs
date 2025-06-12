use super::distance::Distance;
use super::priority_queue::{Closest, ClosestQueue, Furthest, FurthestQueue};
use super::storage::InMemStorage;
use std::collections::{BinaryHeap, HashSet};

type LabelT = u64;

pub struct Index<NbrT, DataT, DistanceFn>
where
    NbrT: num_traits::PrimInt
        + num_traits::AsPrimitive<usize>
        + num_traits::FromPrimitive
        + std::hash::Hash
        + num_traits::Bounded,
    DataT: num_traits::Num + Copy,
    DistanceFn: Distance<DataT>,
{
    graph: InMemStorage<NbrT, DataT>,
    distance_fn: DistanceFn,
}

impl<NbrT, DataT, DistanceFn> Index<NbrT, DataT, DistanceFn>
where
    NbrT: num_traits::PrimInt
        + num_traits::AsPrimitive<usize>
        + num_traits::FromPrimitive
        + std::hash::Hash
        + num_traits::Bounded,
    DataT: num_traits::Num + Copy,
    DistanceFn: Distance<DataT>,
{
    pub fn new(max_nbrs: usize, data_dim: usize, capacity: usize, distance_fn: DistanceFn) -> Self {
        let graph = InMemStorage::<NbrT, DataT>::new(max_nbrs, data_dim, capacity);
        Self { graph, distance_fn }
    }

    fn dist(&self, a: &[DataT], b: &[DataT]) -> f32 {
        return self.distance_fn.call(a, b);
    }

    pub fn beam_search(
        &self,
        query: &[DataT],
        entry: NbrT,
        ef_search: usize,
    ) -> ClosestQueue<NbrT> {
        let mut visited: HashSet<NbrT> = HashSet::new();
        let mut candidates: ClosestQueue<NbrT> = BinaryHeap::new();
        let mut worklist: FurthestQueue<NbrT> = BinaryHeap::with_capacity(ef_search + 1);

        let dist = self.dist(query, self.graph.data(entry.as_()));
        visited.insert(entry);
        candidates.push(Closest { node: entry, dist });
        worklist.push(Furthest { node: entry, dist });

        while !candidates.is_empty() {
            let best_candidate = candidates.pop().unwrap();
            let worst_nbr = worklist.peek().unwrap();

            if best_candidate.dist > worst_nbr.dist {
                break;
            }

            for &nbr in self.graph.nbrs(best_candidate.node.as_()) {
                if nbr != NbrT::max_value() && !visited.contains(&nbr) {
                    visited.insert(nbr);
                    let dist = self.dist(query, self.graph.data(nbr.as_()));
                    candidates.push(Closest { node: nbr, dist });
                    worklist.push(Furthest { node: nbr, dist });
                    if worklist.len() > ef_search {
                        worklist.pop();
                    }
                }
            }
        }

        let mut queue: ClosestQueue<NbrT> = BinaryHeap::with_capacity(worklist.len());
        worklist.drain().for_each(|x| {
            queue.push(Closest {
                node: x.node,
                dist: x.dist,
            })
        });
        return queue;
    }

    fn select_neighbors(&self, mut candidates: ClosestQueue<NbrT>) -> Vec<NbrT> {
        if candidates.len() < self.graph.max_nbrs() {
            let mut selected: Vec<NbrT> = candidates.drain().map(|x| x.node).collect();
            selected.resize(self.graph.max_nbrs(), NbrT::max_value());
            return selected;
        }

        let mut selected: Vec<NbrT> = Vec::with_capacity(self.graph.max_nbrs());

        while candidates.len() > 0 && selected.len() < self.graph.max_nbrs() {
            let best_candidate = candidates.pop().unwrap();

            let mut keep = true;

            for &choosen_nbr in &selected {
                let dist = self.dist(
                    self.graph.data(best_candidate.node.as_()),
                    self.graph.data(choosen_nbr.as_()),
                );
                if dist < best_candidate.dist {
                    keep = false;
                }
            }

            if keep {
                selected.push(best_candidate.node);
            }
        }

        selected.resize(self.graph.max_nbrs(), NbrT::max_value());
        return selected;
    }

    fn connect_neighbors(&mut self, neighbors: Vec<NbrT>, new_node: NbrT, new_data: &[DataT]) {
        for nbr in neighbors {
            if nbr == NbrT::max_value() {
                continue;
            }

            let mut added = false;

            for nbr_nbr in self.graph.nbrs_mut(nbr.as_()) {
                if *nbr_nbr == NbrT::max_value() {
                    *nbr_nbr = new_node;
                    added = true;
                    break;
                }
            }

            if !added {
                let mut candidates: ClosestQueue<NbrT> =
                    BinaryHeap::with_capacity(self.graph.max_nbrs() + 1);

                let nbr_data = self.graph.data(nbr.as_());

                candidates.push(Closest {
                    node: new_node,
                    dist: self.dist(new_data, nbr_data),
                });

                for &nbr_nbr in self.graph.nbrs(nbr.as_()) {
                    candidates.push(Closest {
                        node: nbr_nbr,
                        dist: self.dist(self.graph.data(nbr_nbr.as_()), nbr_data),
                    });
                }

                let selected: Vec<NbrT> = self.select_neighbors(candidates);
                self.graph.nbrs_mut(nbr.as_()).copy_from_slice(&selected);
            }
        }
    }

    fn search_initialization(&self, query: &[DataT], num_initializations: usize) -> usize {
        let mut step_size = self.graph.len() / num_initializations;
        if step_size < 1 {
            step_size = 1;
        }

        let mut min_dist = f32::MAX;
        let mut entry_node = 0;

        for node in (0..self.graph.len()).step_by(step_size) {
            let dist = self.dist(query, self.graph.data(node));
            if dist < min_dist {
                min_dist = dist;
                entry_node = node;
            }
        }

        return entry_node;
    }

    pub fn insert(&mut self, label: LabelT, data: &[DataT], ef_construction: usize) {
        if self.graph.len() == 0 {
            let mut neighbors = Vec::with_capacity(self.graph.max_nbrs());
            neighbors.resize(self.graph.max_nbrs(), NbrT::max_value());
            self.graph.add_node(label, &neighbors, data);
            return;
        }

        let entry = self.search_initialization(data, 100);

        let candidates = self.beam_search(data, NbrT::from(entry).unwrap(), ef_construction);
        let neighbors = self.select_neighbors(candidates);

        let new_node = self.graph.add_node(label, &neighbors, data);

        self.connect_neighbors(neighbors, NbrT::from_usize(new_node).unwrap(), data);
    }

    pub fn query(&self, query: &[DataT], ef_search: usize, topk: usize) -> Vec<(LabelT, f32)> {
        if self.graph.len() == 0 {
            return Vec::new();
        }

        let entry = self.search_initialization(query, 100);

        let mut results = self.beam_search(query, NbrT::from(entry).unwrap(), ef_search);

        let mut output = Vec::with_capacity(topk);
        while output.len() < topk && !results.is_empty() {
            let best = results.pop().unwrap();
            output.push((self.graph.label(best.node.as_()), best.dist));
        }

        return output;
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn data_dim(&self) -> usize {
        self.graph.data_dim()
    }
}

#[cfg(test)]
mod tests {
    use super::super::distance::EuclideanDistance;
    use super::*;
    use rand::Rng;

    #[test]
    fn test_index() {
        const MAX_NBRS: usize = 10;
        const DATA_DIM: usize = 64;
        const N_VECS: usize = 1000;

        let mut rng = rand::rng();
        let data_dist = rand::distr::Uniform::new(-1.0, 1.0).unwrap();
        let noise_dist = rand::distr::Uniform::new(-0.01, 0.01).unwrap();

        let dataset: Vec<(Vec<f32>, Vec<f32>)> = (0..N_VECS)
            .map(|_| {
                let vec: Vec<_> = (0..DATA_DIM).map(|_| rng.sample(data_dist)).collect();
                let query = vec.iter().map(|x| x + rng.sample(noise_dist)).collect();
                (vec, query)
            })
            .collect();

        let mut index = Index::<u32, f32, EuclideanDistance>::new(
            MAX_NBRS,
            DATA_DIM,
            N_VECS,
            EuclideanDistance,
        );

        for (idx, (data, _)) in dataset.iter().enumerate() {
            index.insert(idx as u64, data, 16);
        }

        for (idx, (_, query)) in dataset.iter().take(4).enumerate() {
            let results = index.query(query, 16, 5);
            assert!(!results.is_empty(), "Query {} returned no results", idx);
            assert_eq!(results[0].0, idx as u64);
        }
    }
}
