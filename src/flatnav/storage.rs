use std::marker::PhantomData;

pub type LabelT = u64;

// Layout of the data buffer:
// [Label 1][Neighbors 1][Data 1][Label 2][Neighbors 2][Data 2]...
pub struct InMemStorage<NbrT, DataT>
where
    NbrT: Copy,
    DataT: Copy,
{
    n_nodes: usize,
    nbr_offset: usize,
    max_nbrs: usize,
    data_offset: usize,
    data_dim: usize,
    node_size: usize,
    data: Vec<u8>,
    _a: PhantomData<NbrT>,
    _b: PhantomData<DataT>,
}

impl<NbrT, DataT> InMemStorage<NbrT, DataT>
where
    NbrT: Copy,
    DataT: Copy,
{
    pub fn new(max_nbrs: usize, data_dim: usize, capacity: usize) -> Self {
        let node_size = Self::node_size(max_nbrs, data_dim);
        Self {
            n_nodes: 0,
            nbr_offset: Self::nbr_offset(),
            max_nbrs: max_nbrs,
            data_offset: Self::data_offset(max_nbrs),
            data_dim: data_dim,
            node_size: node_size,
            data: Vec::with_capacity(node_size * capacity),
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    fn nbr_offset() -> usize {
        return pad_to(std::mem::size_of::<LabelT>(), std::mem::align_of::<NbrT>());
    }

    fn data_offset(max_nbrs: usize) -> usize {
        return pad_to(
            Self::nbr_offset() + max_nbrs * std::mem::size_of::<NbrT>(),
            std::mem::align_of::<DataT>(),
        );
    }

    fn node_size(max_nbrs: usize, data_len: usize) -> usize {
        return pad_to(
            Self::data_offset(max_nbrs) + data_len * std::mem::size_of::<DataT>(),
            std::mem::align_of::<LabelT>(),
        );
    }

    pub fn label(&self, node: usize) -> LabelT {
        debug_assert!(node < self.n_nodes);
        let offset = node * self.node_size;
        let label = unsafe { std::ptr::read(self.data.as_ptr().add(offset) as *const LabelT) };
        return label;
    }

    fn set_label(&mut self, node: usize, label: LabelT) {
        debug_assert!(node < self.n_nodes);
        let offset = node * self.node_size;

        unsafe { std::ptr::write(self.data.as_mut_ptr().add(offset) as *mut LabelT, label) };
    }

    pub fn nbrs(&self, node: usize) -> &[NbrT] {
        debug_assert!(node < self.n_nodes);
        let offset = self.nbr_offset + node * self.node_size;
        let nbrs = unsafe {
            std::slice::from_raw_parts(self.data.as_ptr().add(offset) as *const NbrT, self.max_nbrs)
        };
        return nbrs;
    }

    pub fn nbrs_mut(&mut self, node: usize) -> &mut [NbrT] {
        debug_assert!(node < self.n_nodes);
        let offset = self.nbr_offset + node * self.node_size;
        let nbrs = unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(offset) as *mut NbrT,
                self.max_nbrs,
            )
        };
        return nbrs;
    }

    pub fn data(&self, node: usize) -> &[DataT] {
        debug_assert!(node < self.n_nodes);
        let offset = self.data_offset + node * self.node_size;
        let data = unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr().add(offset) as *const DataT,
                self.data_dim,
            )
        };
        return data;
    }

    fn data_mut(&mut self, node: usize) -> &mut [DataT] {
        debug_assert!(node < self.n_nodes);
        let offset = self.data_offset + node * self.node_size;
        let data = unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(offset) as *mut DataT,
                self.data_dim,
            )
        };
        return data;
    }

    pub fn add_node(&mut self, label: LabelT, nbrs: &[NbrT], data: &[DataT]) -> usize {
        if data.len() != self.data_dim {
            panic!(
                "data dim mismatch: expected {}, got {}",
                self.data_dim,
                data.len()
            );
        }

        if nbrs.len() != self.max_nbrs {
            panic!(
                "neighbor count mismatch: expected {}, got {}",
                self.max_nbrs,
                nbrs.len()
            );
        }

        let new_id = self.n_nodes;
        self.n_nodes = self.n_nodes + 1;

        if self.data.len() <= self.n_nodes * self.node_size {
            self.data.resize(self.n_nodes * self.node_size, 0);
        }

        self.set_label(new_id, label);
        self.nbrs_mut(new_id).copy_from_slice(nbrs);
        self.data_mut(new_id).copy_from_slice(data);

        return new_id;
    }

    pub fn len(&self) -> usize {
        return self.n_nodes;
    }

    pub fn max_nbrs(&self) -> usize {
        return self.max_nbrs;
    }

    pub fn data_dim(&self) -> usize {
        return self.data_dim;
    }
}

fn pad_to(n: usize, align: usize) -> usize {
    let rem = n % align;
    if rem == 0 {
        return n;
    }
    return n + align - rem;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_basic() {
        let mut storage = InMemStorage::<u32, u8>::new(4, 3, 2);

        assert_eq!(storage.len(), 0);

        let id = storage.add_node(10, &[1, 2, 3, 4], &[10, 20, 30]);
        assert_eq!(id, 0);
        let id = storage.add_node(20, &[5, 6, 7, 8], &[40, 50, 60]);
        assert_eq!(id, 1);
        let id = storage.add_node(30, &[9, 10, 11, 12], &[70, 80, 90]);
        assert_eq!(id, 2);

        assert_eq!(storage.len(), 3);

        assert_eq!(storage.label(0), 10);
        assert_eq!(storage.nbrs(0), &[1, 2, 3, 4]);
        assert_eq!(storage.data(0), &[10, 20, 30]);

        assert_eq!(storage.label(1), 20);
        assert_eq!(storage.nbrs(1), &[5, 6, 7, 8]);
        assert_eq!(storage.data(1), &[40, 50, 60]);

        assert_eq!(storage.label(2), 30);
        assert_eq!(storage.nbrs(2), &[9, 10, 11, 12]);
        assert_eq!(storage.data(2), &[70, 80, 90]);
    }

    #[test]
    fn test_storage_large() {
        const MAX_NBRS: usize = 3;
        const DATA_DIM: usize = 5;
        let mut storage = InMemStorage::<u8, i8>::new(MAX_NBRS, DATA_DIM, 100);

        for i in 0..200 {
            let label = i * 11;
            let nbrs: Vec<u8> = (i..i + MAX_NBRS).map(|x| x as u8).collect();
            let data: Vec<i8> = (i..i + DATA_DIM)
                .map(|x| ((x as i32) - 127) as i8)
                .collect();
            storage.add_node(label as u64, &nbrs, &data);
        }

        for i in 0..200 {
            let label = i * 11;
            let nbrs: Vec<u8> = (i..i + MAX_NBRS).map(|x| x as u8).collect();
            let data: Vec<i8> = (i..i + DATA_DIM)
                .map(|x| ((x as i32) - 127) as i8)
                .collect();

            assert_eq!(storage.label(i), label as u64);
            assert_eq!(storage.nbrs(i), &nbrs);
            assert_eq!(storage.data(i), &data);
        }
    }
}
