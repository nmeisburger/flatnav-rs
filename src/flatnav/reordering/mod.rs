mod gorder;

pub use gorder::GOrder;

pub trait Reordering {
    fn reorder(&self, out_nodes: &Vec<Vec<usize>>) -> Vec<usize>;
}
