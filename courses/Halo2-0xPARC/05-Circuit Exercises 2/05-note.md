# Range Check
代码：
* [table.rs](/courses/Halo2-0xPARC/halo2-examples/src/range_check/example3/table.rs)
* [example3.rs](/courses/Halo2-0xPARC/halo2-examples/src/range_check/example3.rs)
* [decompose_range_check_spec.rs](/courses/Halo2-0xPARC/halo2-examples/src/range_check/decompose_range_check_spec.rs): 该代码思想是将 value 分为 K-bit 个块，再进行查找，目前只有代码框架，在下一讲中进行完善。

上一讲说到在 [example3_broken.rs](/courses/Halo2-0xPARC/halo2-examples/src/range_check/example3_broken.rs) 中下面的代码会导致报错：
```rust
// THIS IS BROKEN!!!!!!
// Hint: consider the case where q_lookup = 0. What are our input expressions to the lookup argument then?
vec![
    (q_lookup.clone() * num_bits, table.num_bits),
    (q_lookup.clone() * value, table.value),
]
```
原因是这里没有考虑到有的行不启用 `q_lookup` ，也就是置 `q_lookup = 0`，这时就要查找 `(0, 0)` 是否在 `(num_bits, table)` 里面了，实际上是不在的，在 [table.rs](/courses/Halo2-0xPARC/halo2-examples/src/range_check/example3_broken/table.rs) 代码中定义了查找表，查看表的 `load` 方法。
```rust
pub(super) fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
    layouter.assign_table(
        || "load range-check table", 
        |mut table| {
            let mut offset = 0;

            // Assign (num_bits = 1, value = 0)
            {
                table.assign_cell(
                    || "assign num_bits", 
                    self.num_bits, 
                    offset, 
                    || Value::known(F::one()),
                )?;
                table.assign_cell(
                    || "assign value", 
                    self.value, 
                    offset, 
                    || Value::known(F::zero()),
                )?;

                offset += 1;
            }

            for num_bits in 1..=NUM_BITS {
                for value in (1 << (num_bits - 1))..(1 << num_bits) {
                    table.assign_cell(
                        || "assign num_bits", 
                        self.num_bits, 
                        offset, 
                        || Value::known(F::from(num_bits as u64)),
                    )?;
                    table.assign_cell(
                        || "assign value", 
                        self.value, 
                        offset, 
                        || Value::known(F::from(value as u64)),
                    )?;
                    offset += 1;
                }
            }

            Ok(())
        }
    )
}  
```
可以看到 `num_bits` 是从 `1` 开始定义的，自然 `(0, 0)`肯定不在查找表里，因此就会报错。那我们这里要对那些没有启用查找表的行，定义为默认在表中的元素，这样在这些行依然能满足约束。下面定义的默认值为 `(default_num_bits, defalut_value) = (1, 0)`，选取其他在查找表中的任意一对值也是可以的。定义 `not_q_lookup` 为没有启用查找行的值，接着和 `q_lookup` 线性组合下就可以了，具体如下:
```rust
let not_q_lookup = Expression::Constant(F::one()) - q_lookup.clone();
let default_num_bits = Expression::Constant(F::one());
let default_value = Expression::Constant(F::zero());
let num_bits_input = q_lookup.clone() * num_bits + not_q_lookup.clone() * default_num_bits;
let value_input = q_lookup.clone() * value + not_q_lookup.clone() * default_value;

vec![
    (num_bits_input, table.num_bits),
    (value_input, table.value),
]
```