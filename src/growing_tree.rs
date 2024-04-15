use crate::{FenwickTree, FenwickTreeValue, TreeIndex};

pub struct GrowingFenwickTree<T> {
    data: Vec<T>,
}

impl<T: FenwickTreeValue> GrowingFenwickTree<T> {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![T::default(); size + 1],
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn resize(&mut self, idx: &TreeIndex) {
        let size_before_resize = self.size();

        self.data.resize(*idx.to_internal() + 1, T::default());

        if size_before_resize <= 1 {
            return;
        }

        let highest_index_before_resize = TreeIndex::Internal {
            val: size_before_resize - 1,
        };

        let first_new_index = TreeIndex::Internal {
            val: size_before_resize,
        };

        let aggregate_from = if highest_index_before_resize.is_power_of_2() {
            TreeIndex::Internal { val: 0 }
        } else {
            first_new_index
                .lsb_descending()
                .skip(1)
                .chain([TreeIndex::Internal { val: 0 }])
                .next()
                .unwrap()
        };

        let sum_from = self.query(&aggregate_from).unwrap_or_default();
        let sum_till = self.query(&highest_index_before_resize).unwrap();
        let value = sum_till.substract(sum_from);

        // TODO: why is it broken?
        // if value != T::default() {
        //      return;
        // }

        for data_position in highest_index_before_resize
            .lsb_ascending(self.size() - 1)
            .skip(1)
        {
            let data_position = data_position.to_internal();
            self[data_position].store_value(&value);
        }
    }
}

impl<T> std::ops::Index<TreeIndex> for GrowingFenwickTree<T> {
    type Output = T;

    fn index(&self, index: TreeIndex) -> &Self::Output {
        &self.data[*index.to_internal()]
    }
}

impl<T> std::ops::IndexMut<TreeIndex> for GrowingFenwickTree<T> {
    fn index_mut(&mut self, index: TreeIndex) -> &mut Self::Output {
        &mut self.data[*index.to_internal()]
    }
}

impl<T: FenwickTreeValue> FenwickTree for GrowingFenwickTree<T> {
    type Value = T;

    fn query(&self, idx: &TreeIndex) -> Result<T, String> {
        let mut idx = idx.to_external()?;

        if self.size() <= *idx.to_internal() {
            idx = TreeIndex::Internal {
                val: self.size() - 1,
            }
        }

        let mut res = Self::Value::default();

        for data_position in idx.lsb_descending() {
            let data_position = data_position.to_internal();
            res.store_value(&self[data_position]);
        }

        Ok(res)
    }

    fn update(&mut self, idx: &TreeIndex, value: Self::Value) -> Result<(), String> {
        let idx = idx.to_external()?;

        if *idx.to_internal() > self.size() - 1 {
            self.resize(&idx)
        }

        for data_position in idx.lsb_ascending(self.size() - 1) {
            let data_position = data_position.to_internal();
            self[data_position].store_value(&value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;
    use rand::Rng;

    use crate::growing_tree::GrowingFenwickTree;
    use crate::FenwickTree;

    #[test]
    fn test_no_upper_bound_error_is_raised() {
        let tree = GrowingFenwickTree::<i32>::new(0);
        assert_eq!(tree.query(&100.into()).unwrap(), 0);
        assert_eq!(tree.range_query(&10.into(), &100.into()).unwrap(), 0);
    }

    #[test]
    fn tree_grows_one_by_one() {
        let mut tree = GrowingFenwickTree::<i32>::new(1);
        tree.update(&3.into(), 1).unwrap();
        assert_eq!(tree.query(&3.into()).unwrap(), 1);

        tree.update(&0.into(), 1).unwrap();
        assert_eq!(tree.query(&3.into()).unwrap(), 2);
    }

    #[test]
    fn tree_suddenly_grows_much_bigger() {
        let mut tree = GrowingFenwickTree::<i32>::new(2);
        tree.update(&0.into(), 1).unwrap();
        assert_eq!(tree.query(&0.into()).unwrap(), 1);

        tree.update(&1.into(), 1).unwrap();
        assert_eq!(tree.query(&1.into()).unwrap(), 2);

        tree.update(&7.into(), 0).unwrap();
        assert_eq!(tree.query(&7.into()).unwrap(), 2);
    }

    #[test]
    fn simple_tree_generation_with_queries() {
        let mut tree = GrowingFenwickTree::<i32>::new(11);
        for i in 0..32 {
            if let Err(_) = tree.update(&i.into(), 1) {
                assert!(false)
            }
        }
        assert_eq!(tree.query(&3.into()).unwrap(), 4); // points at [0, 1, 2, 3, 4]
        assert_eq!(tree.query(&0.into()).unwrap(), 1);
        assert_eq!(tree.query(&31.into()).unwrap(), 32);
    }

    #[test]
    fn test_range_queries() {
        let mut tree = GrowingFenwickTree::<i32>::new(0);
        for i in 0..=29 {
            if let Err(_) = tree.update(&i.into(), 1) {
                assert!(false)
            }
        }

        match tree.range_query(&10.into(), &20.into()) {
            Ok(10) => assert!(true),
            _ => assert!(false),
        }
        match tree.range_query(&8.into(), &29.into()) {
            Ok(21) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn update_existent_value() {
        let mut tree = GrowingFenwickTree::<i32>::new(0);
        for _i in 0..32 {
            if let Err(_) = tree.update(&0.into(), 1) {
                assert!(false)
            }
        }
        let res = tree.query(&0.into()).unwrap();
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

        let mut tree = GrowingFenwickTree::<i32>::new(0);
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

        let mut tree = GrowingFenwickTree::<i32>::new(size);

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

        let mut tree = GrowingFenwickTree::<i32>::new(size);

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
