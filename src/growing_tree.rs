use crate::{FenwickTree, FenwickTreeValue, TreeError, TreeIndex};

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

    fn resize(&mut self, idx: &TreeIndex) -> Result<(), TreeError> {
        let size_before_resize = self.size();

        // TODO: resize should grow to the closest including power of 2
        self.data.resize(*idx.to_internal() + 1, T::default());

        if size_before_resize <= 1 {
            return Ok(());
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

        let sum_from = aggregate_from
            .to_external()
            .map_or(Ok(T::default()), |idx| self.query(*idx))?;

        let sum_till = highest_index_before_resize
            .to_external()
            .map_or(Ok(T::default()), |idx| self.query(*idx))?;

        let value = sum_till.substract(sum_from);

        for data_position in highest_index_before_resize
            .lsb_ascending(self.size() - 1)
            .skip(1)
        {
            let data_position = data_position.to_internal();
            self[data_position].store_value(&value);
        }

        Ok(())
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

    fn query(&self, idx: usize) -> Result<T, TreeError> {
        let mut idx: TreeIndex = idx.into();

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

    fn update(&mut self, idx: usize, value: Self::Value) -> Result<(), TreeError> {
        let idx: TreeIndex = idx.into();

        if *idx.to_internal() > self.size() - 1 {
            self.resize(&idx)?
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
    fn empty_tree_query() {
        let tree = GrowingFenwickTree::<i32>::new(0);
        assert!(tree.query(0).is_ok_and(|val| val == 0));
        assert!(tree.query(1).is_ok_and(|val| val == 0));
    }

    #[test]
    fn one_element_tree_query() {
        let tree = GrowingFenwickTree::<i32>::new(1);
        assert!(tree.query(0).is_ok_and(|val| val == 0));
        assert!(tree.query(1).is_ok_and(|val| val == 0));
    }

    #[test]
    fn test_no_upper_bound_error_is_raised() {
        let tree = GrowingFenwickTree::<i32>::new(0);
        assert_eq!(tree.query(100).unwrap(), 0);
        assert_eq!(tree.range_query(10, 100).unwrap(), 0);
    }

    #[test]
    fn tree_grows_one_by_one() {
        let mut tree = GrowingFenwickTree::<i32>::new(1);
        tree.update(3, 1).unwrap();
        assert_eq!(tree.query(3).unwrap(), 1);

        tree.update(0, 1).unwrap();
        assert_eq!(tree.query(3).unwrap(), 2);
    }

    #[test]
    fn tree_suddenly_grows_much_bigger() {
        let mut tree = GrowingFenwickTree::<i32>::new(2);
        tree.update(0, 1).unwrap();
        assert_eq!(tree.query(0).unwrap(), 1);

        tree.update(1, 1).unwrap();
        assert_eq!(tree.query(1).unwrap(), 2);

        tree.update(7, 0).unwrap();
        assert_eq!(tree.query(7).unwrap(), 2);
    }

    #[test]
    fn simple_tree_generation_with_queries() {
        let mut tree = GrowingFenwickTree::<i32>::new(11);
        for i in 0..32 {
            if let Err(_) = tree.update(i, 1) {
                assert!(false)
            }
        }
        assert_eq!(tree.query(3).unwrap(), 4); // points at [0, 1, 2, 3, 4]
        assert_eq!(tree.query(0).unwrap(), 1);
        assert_eq!(tree.query(31).unwrap(), 32);
    }

    #[test]
    fn test_range_queries() {
        let mut tree = GrowingFenwickTree::<i32>::new(0);
        for i in 0..=29 {
            if let Err(_) = tree.update(i, 1) {
                assert!(false)
            }
        }

        match tree.range_query(10, 20) {
            Ok(10) => assert!(true),
            _ => assert!(false),
        }
        match tree.range_query(8, 29) {
            Ok(21) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn update_existent_value() {
        let mut tree = GrowingFenwickTree::<i32>::new(0);
        for _i in 0..32 {
            if let Err(_) = tree.update(0, 1) {
                assert!(false)
            }
        }
        let res = tree.query(0).unwrap();
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
            if let Err(_) = tree.update(i, *input.get(i).unwrap()) {
                assert!(false)
            }
        }

        let mut sum = 0;
        for i in 0..size {
            sum += *input.get(i).unwrap();

            if let Ok(res) = tree.query(i) {
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
            if let Err(_) = tree.update(i, *input.get(i).unwrap()) {
                assert!(false)
            }
        }

        let mut sum = 0;
        for i in 0..size {
            sum += *input.get(i).unwrap();
            if let Ok(res) = tree.query(i) {
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
            let sum_before_update = tree.query(i).unwrap();
            let value_to_update = *input.get(i).unwrap();
            if let Err(_) = tree.update(i, value_to_update) {
                assert!(false)
            }
            let sum_after_update = tree.query(i).unwrap();
            assert_eq!(sum_after_update - sum_before_update, value_to_update)
        }

        let mut sum = 0;
        for i in 0..size {
            sum += *input.get(i).unwrap();

            if let Ok(res) = tree.query(i) {
                assert_eq!(res, sum);
            } else {
                assert!(false)
            }
        }
    }
}
