use crate::{FenwickTree, FenwickTreeValue, TreeIndex};

pub struct FixedSizeFenwickTree<T: FenwickTreeValue> {
    data: Vec<T>,
}

impl<T: FenwickTreeValue> FixedSizeFenwickTree<T> {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![T::default(); size + 1],
        }
    }

    fn size(&self) -> usize {
        self.data.len() - 1
    }
}

impl<T: FenwickTreeValue> std::ops::Index<TreeIndex> for FixedSizeFenwickTree<T> {
    type Output = T;

    fn index(&self, index: TreeIndex) -> &Self::Output {
        &self.data[*index.to_internal()]
    }
}

impl<T: FenwickTreeValue> std::ops::IndexMut<TreeIndex> for FixedSizeFenwickTree<T> {
    fn index_mut(&mut self, index: TreeIndex) -> &mut Self::Output {
        &mut self.data[*index.to_internal()]
    }
}

impl<T: FenwickTreeValue> FenwickTree for FixedSizeFenwickTree<T> {
    type Value = T;

    fn query(&self, idx: &TreeIndex) -> Result<T, String> {
        // TODO: need to discuss
        let idx = idx.to_external()?;

        if *idx >= self.size() {
            return Err("Index is out of bounds.".to_string());
        }

        let mut res = T::default();
        for data_position in idx.lsb_descending() {
            let data_position = data_position.to_internal();
            res.store_value(&self[data_position]);
        }

        Ok(res)
    }

    fn update(&mut self, idx: &TreeIndex, value: Self::Value) -> Result<(), String> {
        // TODO: need to discuss
        let idx = idx.to_external()?;

        if *idx > self.data.len() {
            return Err("Index is out of bounds".to_string());
        }

        for data_position in idx.lsb_ascending(self.size()) {
            let data_position = data_position.to_internal();
            self[data_position].store_value(&value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fixed_size_tree::FixedSizeFenwickTree;
    use crate::FenwickTree;
    use rand::seq::SliceRandom;
    use rand::Rng;
    extern crate test;

    #[test]
    fn edge_case() {
        let mut tree = FixedSizeFenwickTree::<i32>::new(4);
        tree.update(&3.into(), 1).unwrap();
        assert_eq!(tree.query(&3.into()).unwrap(), 1);
    }

    #[test]
    fn simple_tree_generation_with_queries() {
        let mut tree = FixedSizeFenwickTree::<i32>::new(32);
        for i in 0..32 {
            if let Err(_) = tree.update(&i.into(), 1) {
                assert!(false)
            }
        }
        assert_eq!(tree.query(&4.into()).unwrap(), 5); // points at [0, 1, 2, 3, 4]
        assert_eq!(tree.query(&0.into()).unwrap(), 1);
        assert_eq!(tree.query(&31.into()).unwrap(), 32);
    }

    // TODO: #[should_panic]?
    #[test]
    fn tree_indexing_overflow() {
        let tree = FixedSizeFenwickTree::<i32>::new(0);

        match tree.query(&1.into()) {
            Ok(_) => assert!(false),
            Err(message) => assert_eq!(message, "Index is out of bounds."),
        }
    }

    #[test]
    fn update_existent_value() {
        let mut tree = FixedSizeFenwickTree::<i32>::new(32);
        for _i in 0..32 {
            if let Err(_) = tree.update(&0.into(), 1) {
                assert!(false)
            }
        }
        let res = tree.query(&1.into()).unwrap();
        assert_eq!(res, 32);
    }

    #[test]
    fn random_100_point_data() {
        let size = 100;
        let mut input = vec![];
        let mut rng = rand::thread_rng();

        for _i in 0..size {
            input.push((rng.gen::<f32>() * 100.0) as i32);
        }

        let mut tree = FixedSizeFenwickTree::<i32>::new(size);
        for i in 0..size {
            if let Err(_) = tree.update(&i.into(), *input.get(i).unwrap()) {
                assert!(false)
            }
        }

        let mut sum = 0;
        for i in 0..size {
            sum += *input.get(i).unwrap();

            if let Ok(res) = tree.query(&i.into()) {
                assert_eq!(res, sum);
            } else {
                assert!(false)
            }
        }
    }

    #[test]
    fn random_100_point_data_with_random_update_order() {
        let size = 100;
        let mut input = vec![];
        let mut rng = rand::thread_rng();

        for _i in 0..size {
            input.push((rng.gen::<f32>() * 100.0) as i32);
        }

        let mut tree = FixedSizeFenwickTree::<i32>::new(size);

        let mut random_indexes: Vec<usize> = (0..size).collect();
        random_indexes.shuffle(&mut rng);
        for i in random_indexes {
            if let Err(_) = tree.update(&i.into(), *input.get(i).unwrap()) {
                assert!(false)
            }
        }

        let mut sum = 0;
        for i in 0..size {
            sum += *input.get(i).unwrap();
            if let Ok(res) = tree.query(&i.into()) {
                assert_eq!(res, sum);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn random_100_point_data_with_random_update_order_with_intermediate_asserts() {
        let size = 100;
        let mut input = vec![];
        let mut rng = rand::thread_rng();

        for _i in 0..size {
            input.push((rng.gen::<f32>() * 100.0) as i32);
        }

        let mut tree = FixedSizeFenwickTree::<i32>::new(size);

        let mut random_indexes: Vec<usize> = (0..size).collect();
        random_indexes.shuffle(&mut rng);
        for i in random_indexes {
            let sum_before_update = tree.query(&i.into()).unwrap();
            let value_to_update = *input.get(i).unwrap();
            if let Err(_) = tree.update(&i.into(), value_to_update) {
                assert!(false)
            }
            let sum_after_update = tree.query(&i.into()).unwrap();
            assert_eq!(sum_after_update - sum_before_update, value_to_update)
        }

        let mut sum = 0;
        for i in 0..size {
            sum += *input.get(i).unwrap();

            if let Ok(res) = tree.query(&i.into()) {
                assert_eq!(res, sum);
            } else {
                assert!(false)
            }
        }
    }

    use test::Bencher;

    fn bench_update(b: &mut Bencher, size: usize) {
        let mut input = vec![];
        let mut rng = rand::thread_rng();

        for _i in 0..size {
            input.push((rng.gen::<f32>() * 100.0) as i32);
        }

        let mut tree = FixedSizeFenwickTree::<i32>::new(size);

        let random_indexes: Vec<usize> = (0..size).collect();

        b.iter(|| {
            let i = *random_indexes.choose(&mut rng).unwrap();
            let value_to_update = *input.get(i).unwrap();
            tree.update(&i.into(), value_to_update).unwrap()
        });
    }

    fn bench_reads(b: &mut Bencher, size: usize) {
        let mut input = vec![];
        let mut rng = rand::thread_rng();

        for _i in 0..size {
            input.push((rng.gen::<f32>() * 100.0) as i32);
        }

        let mut tree = FixedSizeFenwickTree::<i32>::new(size);
        let random_indexes: Vec<usize> = (0..size).collect();

        for _i in 0..size {
            let i = *random_indexes.choose(&mut rng).unwrap();
            let value_to_update = *input.get(i).unwrap();
            tree.update(&i.into(), value_to_update).unwrap()
        }

        b.iter(|| {
            let i = *random_indexes.choose(&mut rng).unwrap();
            tree.query(&i.into()).unwrap();
        });
    }

    #[bench]
    fn bench_1000_writes(b: &mut Bencher) {
        bench_update(b, 1000);
    }

    #[bench]
    fn bench_10000_writes(b: &mut Bencher) {
        bench_update(b, 10000);
    }

    #[bench]
    fn bench_100000_writes(b: &mut Bencher) {
        bench_update(b, 100000);
    }

    #[bench]
    fn bench_10000000_writes(b: &mut Bencher) {
        bench_update(b, 10000000);
    }

    #[bench]
    fn bench_1000_reads(b: &mut Bencher) {
        bench_reads(b, 1000);
    }

    #[bench]
    fn bench_10000_reads(b: &mut Bencher) {
        bench_reads(b, 10000);
    }

    #[bench]
    fn bench_100000_reads(b: &mut Bencher) {
        bench_reads(b, 100000);
    }

    #[bench]
    fn bench_10000000_reads(b: &mut Bencher) {
        bench_reads(b, 10000000);
    }
}

// #[bench]
// mod benchmarks {}
