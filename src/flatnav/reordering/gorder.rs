use super::Reordering;
use std::collections::{BTreeMap, HashMap, HashSet};

struct PriorityQueue {
    node_to_score: HashMap<usize, i32>,
    scores: BTreeMap<i32, HashSet<usize>>,
}

impl PriorityQueue {
    fn new(n_nodes: usize) -> Self {
        let mut scores = BTreeMap::new();
        scores.insert(0, (0..n_nodes).collect());
        Self {
            node_to_score: (0..n_nodes).map(|x| (x, 0)).collect(),
            scores: scores,
        }
    }

    fn update(&mut self, node: usize, delta: i32) {
        let score = self.node_to_score.get(&node);
        if score.is_none() {
            return;
        }

        let score = *score.unwrap();
        let new_score = score + delta;
        self.node_to_score.insert(node, new_score);

        if let Some(nodes) = self.scores.get_mut(&score) {
            if nodes.len() == 1 {
                self.scores.remove(&score);
            } else {
                nodes.remove(&node);
            }
        } else {
            panic!("scores does not contain entry for old score")
        }

        if !self.scores.contains_key(&new_score) {
            self.scores.insert(new_score, HashSet::new());
        }

        if let Some(nodes) = self.scores.get_mut(&new_score) {
            nodes.insert(node);
        } else {
            panic!("scores does not contain entry for new score")
        }
    }

    fn pop(&mut self) -> usize {
        assert!(
            self.scores.len() > 0,
            "queue must not be empty when calling pop"
        );

        let mut last = self.scores.last_entry().unwrap();

        let node = *last.get().iter().next().unwrap();

        if last.get().len() == 1 {
            last.remove();
        } else {
            last.get_mut().remove(&node);
        }

        self.node_to_score.remove(&node);

        return node;
    }
}

pub struct GOrder {
    w: usize,
}

impl GOrder {
    pub fn new(w: usize) -> Self {
        Self { w: w }
    }
}

impl Reordering for GOrder {
    fn reorder(&self, out_nodes: &Vec<Vec<usize>>) -> Vec<usize> {
        let mut in_nodes = Vec::with_capacity(out_nodes.len());
        in_nodes.resize(out_nodes.len(), Vec::new());
        for (node, nbrs) in out_nodes.iter().enumerate() {
            for nbr in nbrs {
                in_nodes[*nbr].push(node);
            }
        }

        let mut perm = Vec::with_capacity(out_nodes.len());
        let mut queue = PriorityQueue::new(out_nodes.len());
        queue.update(0, 1);

        for i in 0..out_nodes.len() {
            let next = queue.pop();
            perm.push(next);

            for &u in &out_nodes[next] {
                queue.update(u, 1);
            }

            for &u in &in_nodes[next] {
                queue.update(u, 1);

                for &v in &out_nodes[u] {
                    queue.update(v, 1);
                }
            }

            if i >= self.w {
                let last = perm[i - self.w];

                for &u in &out_nodes[last] {
                    queue.update(u, -1);
                }

                for &u in &in_nodes[last] {
                    queue.update(u, -1);

                    for &v in &out_nodes[u] {
                        queue.update(v, -1);
                    }
                }
            }
        }

        // perm[i] = n means that node n goes to position i. Inverting it means
        // that perm_inv[n] = i means that node n goes to position i which allows
        // for easy lookup for mapping nodes
        let mut perm_inv = vec![0; out_nodes.len()];
        for (i, &node) in perm.iter().enumerate() {
            perm_inv[node] = i;
        }

        return perm_inv;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    impl PriorityQueue {
        fn nodes(&self) -> Vec<usize> {
            self.scores.iter().fold(Vec::new(), |mut vec, (_, nodes)| {
                vec.extend(nodes.iter());
                vec
            })
        }
    }

    #[test]
    fn test_priority_queue_basic() {
        let mut queue = PriorityQueue::new(5);
        (0..5).for_each(|i| queue.update(i, i as i32));

        assert_eq!(queue.nodes(), vec![0, 1, 2, 3, 4]);

        queue.update(2, 4);
        assert_eq!(queue.nodes(), vec![0, 1, 3, 4, 2]);

        queue.update(3, 2);
        assert_eq!(queue.nodes(), vec![0, 1, 4, 3, 2]);

        queue.update(0, 3);
        assert_eq!(queue.nodes(), vec![1, 0, 4, 3, 2]);

        queue.update(0, -3);
        assert_eq!(queue.nodes(), vec![0, 1, 4, 3, 2]);

        assert_eq!(queue.pop(), 2);
        assert_eq!(queue.nodes(), vec![0, 1, 4, 3]);

        assert_eq!(queue.pop(), 3);
        assert_eq!(queue.nodes(), vec![0, 1, 4]);
    }

    #[test]
    fn test_priority_queue_duplicates() {
        let mut queue = PriorityQueue::new(100);

        for i in 0..10 {
            for n in 0..100 {
                if (n % 10) >= i {
                    queue.update(n, 1);
                }
            }
        }

        let mut popped = HashSet::new();
        for i in (0..10).rev() {
            for _ in 0..10 {
                let next = queue.pop();
                assert_eq!(next % 10, i);
                popped.insert(next);
            }
        }

        assert_eq!(popped.len(), 100);

        for i in 0..100 {
            assert!(popped.contains(&i));
        }
    }

    #[test]
    fn test_gorder() {
        let out_nodes = vec![vec![1, 2], vec![0], vec![4], vec![1, 2], vec![]];

        let gorder = GOrder::new(2);

        let perm = gorder.reorder(&out_nodes);

        assert_eq!(perm, vec![0, 1, 2, 3, 4]);
    }
}
