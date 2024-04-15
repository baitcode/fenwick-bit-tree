## Slighly over-engineered Fenwick Tree implmentation.

Allows efficient prefix sum calculation.

Created for trining purposes to test:
    1) rust typesystem, default trait implmentation, enums as a way for polymorphism
    2) memory management and consumption of value
    3) cargo tools, docs, tests, clippy and benchmarks, build and publish.

Code is free to do whatever you feel like.

Provides abstraction for Fenwick tree data structure and 2 implmentations:
 - [`prelude::FixedSizeFenwickTree`]
 - [`prelude::GrowingFenwickTree`]

Key space for a tree lies within [`usize`] range. Tree support any value that
implements [`FenwickTreeValue`] trait. [`FenwickTreeValue`] is automatically
implmented for all primitive numeric types that support [`std::ops::AddAssign`],
[`std::ops::Sub`], [`core::cmp::PartialEq`] and [`Copy`] traits.

### Installation

```bash
cargo install fenwick_bit_tree
```

### Test

```bash
cargo test
```

### Benchmarks

```bash
cargo bench --features benchmarks
```

### Basic usage:

```rust
use fenwick_bit_tree::prelude::*;

// Create the tree with capacity for 32 aggregated [`i32`] data points.
// One can use whole usize range to store datapoints for unicode timestamps
let mut tree = FixedSizeFenwickTree::<i32>::new(32);

// Add values

tree.update(&0.into(), 1);
tree.update(&0.into(), 4); // Will aggregate value at index 0 so it would be 5
tree.update(&10.into(), 10);
tree.update(&20.into(), 10);
tree.update(&30.into(), 10);

// Now you can query data.
// NOTE: FixedSizeFenwickTree will raise error when query goes out of bounds.
//       GrowingFenwickTree will automatically truncate the range torightmost index.

assert_eq!(tree.query(&4.into()).unwrap(), 5);
assert_eq!(tree.query(&15.into()).unwrap(), 15);
assert_eq!(tree.query(&31.into()).unwrap(), 35);

// Also allows making range queries

let val = tree.range_query(&2.into(), &16.into()).unwrap(); // Will return aggregated sum of all values between those keys.
assert_eq!(val, 10);
```

Current version: 1.0.0

License: MIT OR Apache-2.0
