use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub type FurthestQueue<T> = BinaryHeap<Furthest<T>>;
pub type ClosestQueue<T> = BinaryHeap<Closest<T>>;

pub struct Closest<T> {
    pub node: T,
    pub dist: f32,
}

impl<T> PartialEq for Closest<T> {
    fn eq(&self, other: &Self) -> bool {
        return self.dist == other.dist;
    }
}

impl<T> Eq for Closest<T> {}

impl<T> PartialOrd for Closest<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return other.dist.partial_cmp(&self.dist);
    }
}

impl<T> Ord for Closest<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self
            .dist
            .partial_cmp(&other.dist)
            .unwrap_or(Ordering::Equal);
    }
}

pub struct Furthest<T> {
    pub node: T,
    pub dist: f32,
}

impl<T> PartialEq for Furthest<T> {
    fn eq(&self, other: &Self) -> bool {
        return self.dist == other.dist;
    }
}

impl<T> Eq for Furthest<T> {}

impl<T> PartialOrd for Furthest<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return self.dist.partial_cmp(&other.dist);
    }
}

impl<T> Ord for Furthest<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        return other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closest() {
        let mut q: ClosestQueue<u32> = BinaryHeap::new();

        let items = [(4, 7.2), (12, 9.3), (5, 3.1), (22, 4.7), (19, 7.2)];
        for &(node, dist) in items.iter() {
            q.push(Closest { node, dist });
        }

        let top = q.pop().unwrap();
        assert_eq!(top.node, 5);
        assert_eq!(top.dist, 3.1);

        let top = q.pop().unwrap();
        assert_eq!(top.node, 22);
        assert_eq!(top.dist, 4.7);

        let top = q.pop().unwrap();
        assert_eq!(top.node, 4);
        assert_eq!(top.dist, 7.2);

        let top = q.pop().unwrap();
        assert_eq!(top.node, 19);
        assert_eq!(top.dist, 7.2);
    }

    #[test]
    fn test_furthest() {
        let mut q: FurthestQueue<u32> = BinaryHeap::new();

        let items = [(4, 7.2), (12, 9.3), (5, 3.1), (22, 4.7), (19, 7.2)];
        for &(node, dist) in items.iter() {
            q.push(Furthest { node, dist });
        }

        let top = q.pop().unwrap();
        assert_eq!(top.node, 12);
        assert_eq!(top.dist, 9.3);

        let top = q.pop().unwrap();
        assert_eq!(top.node, 4);
        assert_eq!(top.dist, 7.2);

        let top = q.pop().unwrap();
        assert_eq!(top.node, 19);
        assert_eq!(top.dist, 7.2);

        let top = q.pop().unwrap();
        assert_eq!(top.node, 22);
        assert_eq!(top.dist, 4.7);
    }
}
