# 范围查找: Decompose

对于一个比较大的值 `value` 要进行范围查找，可以先将 `value` 写成二进制，再对其进行分块，每个块为K-bit（或者不超过），对每一个块进行 lookup 查找，从而实现 `value` 的范围查找。

假设 `value` 总共是 N-bits, 将之分为 `C` 个块，每个块为 K-bits, 这里先假设 `N` 能够整除 `K`, 即 `C = N / K`. 那么

```rust
/// Given an element `value`, we use a running sum to break it into K-bit chunks.
/// Assume for now that N | K, and define C = N / K.
/// 
///     value = [b_0, b_1, ..., b_{N-1}]    (little-endian)
///           = c_0 + 2^K * c_1 + 2^{2K} * c_2 + ... + 2^{(C-1)K} * c_{C-1}
/// 
/// Initialise the running sum at
///                                 value = z_0.
/// 
/// Consequent terms of the running sum are z_{i+1} = (z_i - c_i) * 2^{-K}:
///
///                           z_1 = (z_0 - c_0) * 2^{-K}
///                           z_2 = (z_1 - c_1) * 2^{-K}
///                              ...
///                       z_{C-1} = c_{C-1}
///                           z_C = (z_{C-1} - c_{C-1}) * 2^{-K}
///                               = 0
```

下面推导下 `z_C = 0`. 根据 $z_{i+1} = (z_i - c_i) \times 2^{-K}$ , 递推可知

$$
\begin{aligned}
z_1 & = (z_0 - c_0) \times 2^{-K}\\
z_2 & = (z_1 - c_1) \times 2^{-K} \\
& = z_1 \times 2^{-K} - c_1 \times 2^{-K} \\
& = z_0 \times 2^{-2K} - c_0 \times 2^{-2K} - c_1 \times 2^{-K} \\
z_3 & = (z_2 - c_2) \times 2^{-K} \\
& = z_2 \times 2^{-K} - c_2 \times 2^{-K} \\
& = z_0 \times 2^{-3K} - c_0 \times 2^{-3K} - c_1 \times 2^{-2K} - c_2 \times 2^{-K} \\
\cdots & \\
z_{C-1} & = z_0 \times 2^{-(C-1)K} - c_0 \times 2^{-(C-1)K} - c_1 \times 2^{-(C-1)K+K} - c_2 \times 2^{-(C-1)K+2K} - \\
& \quad \cdots - c_{C-2} \times 2^{-(C-1)K+(C-2)K}
\end{aligned}
$$

再根据 $z_0 = value$ 的表达式

$$
z_0 = value = c_0 + c_1 \times 2^{K} + c_2 \times 2^{2K} + \cdots + c_{C-2} \times 2^{(C-2)K} + c_{C-1} \times 2^{(C-1)K}
$$

则

$$
z_0 \times 2^{-(C-1)K} =  c_0 \times 2^{-(C-1)K} + c_1 \times 2^{-(C-1)K+K} + c_2 \times 2^{-(C-1)K+2K} \cdots + c_{C-2} \times 2^{-(C-1)K+(C-2)K} + c_{C-1}
$$

在前面 $z_{C-1}$ 的等式中，刚好上述 $z_0 \times 2^{-(C-1)K}$ 展开的前面 $C-1$ 项都可以抵消，只留下最后一项没有减掉，剩下 $c_{C-1}$ , 也就是 $z_{C-1} = c_{C-1}$, 因此根据递推式 $z_{i+1} = (z_i - c_i) \times 2^{-K}$ 得到 $z$ 的最后一项 $z_C = (z_{C-1} - c_{C-1}) \times 2^{-K} = 0$.

## Example1: 整除情况

第一个例子，假设 $N / K = C$, 也就是 `value` 的二进制表示的长度能够整除 $K$ 。

代码见：

* [example1/table.rs](/courses/Halo2-0xPARC/halo2-examples/src/decompose_range_check/example1/table.rs)
* [example1.rs](/courses/Halo2-0xPARC/halo2-examples/src/decompose_range_check/example1.rs)

### 查找表

这里查找表的定义与前面一讲 example3 的查找表一样，实际上就是复制过来的。

```rust
/// A lookup table of values up to RANGE
/// e.g. RANGE = 256, values = [0..255]
/// This table is tagged by an index `k`, where `k` is the number of bits of the element in the `value` column.
#[derive(Debug, Clone)]
pub(super) struct RangeTableConfig<F: FieldExt, const NUM_BITS: usize, const RANGE: usize> {
    pub(super) num_bits: TableColumn,
    pub(super) value: TableColumn,
    _marker: PhantomData<F>,
}
```

同样地，需要实现 `configure` 和 `load` 这两个方法。

```rust
impl<F: FieldExt, const NUM_BITS: usize, const RANGE: usize> RangeTableConfig<F, NUM_BITS, RANGE> {
    pub(super) fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        assert_eq!(1 << NUM_BITS, RANGE);

        let num_bits = meta.lookup_table_column();
        let value = meta.lookup_table_column();

        Self {
            num_bits,
            value,
            _marker: PhantomData,
        }
    }

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
}
```

### DecomposeConfig

在 [example1.rs](/courses/Halo2-0xPARC/halo2-examples/src/decompose_range_check/example1.rs) 中来定义主要的结构 `DecomposeConfig`。

先设定好我们的电路表：

```rust
/// One configuration for this gadget could look like:
///
///     | running_sum |  q_decompose  |  table_value  |
///     -----------------------------------------------
///     |     z_0     |       1       |       0       |
///     |     z_1     |       1       |       1       |
///     |     ...     |      ...      |      ...      |
///     |   z_{C-1}   |       1       |      ...      |
///     |     z_C     |       0       |      ...      |
```

因此结构 `DecomposeConfig` 的定义就很清晰了。

```rust
#[derive(Debug, Clone)]
struct DecomposeConfig<
    F: FieldExt + PrimeFieldBits, 
    const LOOKUP_NUM_BITS: usize, 
    const LOOKUP_RANGE: usize
> {
    // You'll need an advice column to witness your running sum;
    running_sum: Column<Advice>,
    // A selector to constrain the running sum;
    q_decompose: Selector,
    // And of course, the K-bit lookup table
    table: RangeTableConfig<F, LOOKUP_NUM_BITS, LOOKUP_RANGE>,
    _marker: PhantomData<F>,
}
```

上述结构中的 `LOOKUP_NUM_BITS` 就是前面所说的分块后每个块的比特长度 $K$ , `LOOKUP_RANGE` 和 $K$ 是相关的，例如 `K = LOOKUP_NUM_BITS = 8`, 则 `LOOKUP_RANGE = 2^8 - 1 = 255`。

这里 `F` 多了一个 `PrimeFieldBits` , 该 trait 在 `ff` 库中，能够对素数域中的元素进行二进制的表达。

```rust
/// This represents the bits of an element of a prime field.
#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
pub trait PrimeFieldBits: PrimeField {
    /// The backing store for a bit representation of a prime field element.
    type ReprBits: BitViewSized + Send + Sync;

    /// Converts an element of the prime field into a little-endian sequence of bits.
    fn to_le_bits(&self) -> FieldBits<Self::ReprBits>;

    /// Returns the bits of the field characteristic (the modulus) in little-endian order.
    fn char_le_bits() -> FieldBits<Self::ReprBits>;
}
```

接着为 `DecomposeConfig` 实现基本的方法。首先是配置方法 `configure`。

```rust
fn configure(meta: &mut ConstraintSystem<F>, running_sum: Column<Advice>) -> Self {
    // Create the needed columns and internal configs.
    let q_decompose = meta.complex_selector();  // 查找表的selector, 这里必须是 complex_selector
    let table = RangeTableConfig::configure(meta);

    meta.enable_equality(running_sum);  // 运行 running_sum 这一列进行 permutation 

    
    // Range-constrain each K-bit chunck `c_i = z_i - z_{i+1} * 2^K` derived from the running sum.
    meta.lookup(|meta| {
        let q_decompose = meta.query_selector(q_decompose);

        // z_i
        let z_cur = meta.query_advice(running_sum, Rotation::cur());
        // z_{i + 1}
        let z_next = meta.query_advice(running_sum, Rotation::next());
        // c_i = z_i - z_{i + 1} * 2^K
        let chunk = z_cur - z_next * F::from(1u64 << LOOKUP_NUM_BITS);

        // Lookup default value 0 when q_decompose = 0
        let not_q_decompose = Expression::Constant(F::one()) - q_decompose.clone();
        // 虽然默认情况下查找的是 0 , 但是为了防止后续的变更，
        // 这里再单独定义下不开启 `q_decompose = 0` 的那些行的默认值。
        let default_chunk = Expression::Constant(F::zero());

        vec![(
            q_decompose * chunk + not_q_decompose * default_chunk,
            table.value,
        )]
    });

    Self {
        running_sum,
        q_decompose,
        table,
        _marker: PhantomData,
    }
}
```

* 这里 `z_C = 0` constant_constrain 要进行Copy Constrain.

第二个要实现的是 `assign` 方法。

```rust
fn assign(
    &self,
    mut layouter: impl Layouter<F>,
    value: AssignedCell<Assigned<F>, F>,
    num_bits: usize,
) -> Result<(), Error> {
    // `num_bits` must be a multiple of K
    // `num_bits` 表示 `value` 的二进制表达的长度，也就是 N
    // 这里确保 N / K , N 能够整除 K
    assert_eq!(num_bits % LOOKUP_NUM_BITS, 0);

    layouter.assign_region(
        || "Decompose value", 
        |mut region| {
            let mut offset = 0;
            
            // 0. Copy in the witnessed `value` 
            let mut z = value.copy_advice(
                || "Copy in value for decomposition", 
                &mut region, 
                self.running_sum, 
                offset
            )?;

            // Increase offset after copying `value`
            offset += 1;

            // 1. Compute the interstitial running sum values {z_1, ..., z_C}}
            let running_sum: Vec<_> = value
                .value()
                .map(|&v| compute_running_sum::<_, LOOKUP_NUM_BITS>(v, num_bits))
                .transpose_vec(num_bits / LOOKUP_NUM_BITS);  // Transposes a `Value<impl IntoIterator<Item = V>>` into a `Vec<Value<V>>`.

            // 2. Assign the running sum values
            for z_i in running_sum.into_iter() {
                z = region.assign_advice(
                    || format!("assign z_{:?}", offset), 
                    self.running_sum, 
                    offset, 
                    || z_i,
                )?;
                offset += 1;
            }

            // 3. Make sure to enable the relevant selector on each row of the running sum
            //    (but not on the row where z_C is witnessed)
            for offset in 0..(num_bits / LOOKUP_NUM_BITS) {
                self.q_decompose.enable(&mut region, offset)?;
            }

            // 4. Constrain the final running sum `z_C` to be 0.
            region.constrain_constant(z.cell(), F::zero())
        },
    )
}
```

上面用到了 `compute_running_sum` 函数，这是一个自己定义的函数，用来计算 `z_i` 这一列中的 `(z_1, ..., z_C)`。

```rust
// Function to compute the interstitial running sum values {z_1, ... , z_C}
fn compute_running_sum<F: FieldExt + PrimeFieldBits, const LOOKUP_NUM_BITS: usize>(
    value: Assigned<F>,
    num_bits: usize
) -> Vec<Assigned<F>> {
    let mut running_sum = vec![];
    let mut z = value;

    // Get the little-endian bit representation of `value`
    let value: Vec<_> = value
        .evaluate()
        .to_le_bits()
        .iter()
        .by_vals()
        .take(num_bits)
        .collect();
    // `value.chunks(LOOKUP_NUM_BITS)` 调用的是 `Vec` 的 `chunk()` 函数，
    // 会将 `Vec` 中的元素分成参数 `LOOKUP_NUM_BITS` 块大小
    for chunck in value.chunks(LOOKUP_NUM_BITS) {
        let chunck = Assigned::from(F::from(lebs2ip(chunck)));
        // z_{i + 1} = (z_i - c_i) * 2^{-K}
        // Assigned 的 invert() 函数对于非 0 数会取倒数
        z = (z - chunck) * Assigned::from(F::from(1u64 << LOOKUP_NUM_BITS)).invert();
        running_sum.push(z);
    }

    assert_eq!(running_sum.len(), num_bits / LOOKUP_NUM_BITS);
    running_sum
}

fn lebs2ip(bits: &[bool]) -> u64 {
    assert!(bits.len() <= 64);
    // 将 bits 个位数进行累加，从 `acc + if *b { 1 << i } else { 0 }` 可以看出
    // 例如 bits 为 110
    // 则 acc = 1 * 2^2 + 1 * 2^1 + 0 = 5，其正是 110 二进制表示的数字 5.
    bits.iter()
        .enumerate()
        .fold(0u64, |acc, (i, b)| acc + if *b { 1 << i } else { 0 })
}
```

至此，`DecomposeConfig` 就定义好了，下面写一个例子进行调试。

### Test

先定义一个自己的电路 `MyCircuit` , 接着为其实现 `Circuit` trait.

```rust
struct MyCircuit<F: FieldExt, const NUM_BITS: usize, const RANGE: usize> {
    value: Value<Assigned<F>>,
    num_bits: usize,
}

impl<F: FieldExt + PrimeFieldBits, const NUM_BITS: usize, const RANGE: usize> Circuit<F> 
    for MyCircuit<F, NUM_BITS, RANGE> 
{
    type Config = DecomposeConfig<F, NUM_BITS, RANGE>;
    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self {
            value: Value::unknown(),
            num_bits: self.num_bits,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // Fixed column for constants
        let constants = meta.fixed_column();
        meta.enable_constant(constants);

        let value = meta.advice_column();
        DecomposeConfig::configure(meta, value)
    }

    fn synthesize(
        &self, 
        config: Self::Config, 
        mut layouter: impl Layouter<F>
    ) -> Result<(), Error> {
        config.table.load(&mut layouter)?;

        // Witness the value somewhere
        let value = layouter.assign_region(
            || "Witness value", 
            |mut region| {
                region.assign_advice(|| "Witness value", config.running_sum, 0, || self.value)
            },
        )?;

        config.assign(
            layouter.namespace(|| "Decompose value"), 
            value, 
            self.num_bits,
        )?;

        Ok(())
    }  
}
```

* `synthesize` 方法告诉我们怎样对value进行操作

编写 test ，来进行测试，这里选取的 `K = NUM_BITS = 8`, `N = num_bits = 64`, 符合 `N % K == 0`, `value` 是通过随机产生的一个 `u64` 的值。

```rust
#[test]
fn test_decompose_1() {
    let k = 9;
    const NUM_BITS: usize = 8;
    const RANGE: usize = 256;   // 8-bit value

    // Random u64 value
    let value: u64 = rand::random();
    let value = Value::known(Assigned::from(Fp::from(value)));

    let circuit = MyCircuit::<Fp, NUM_BITS, RANGE> {
        value,
        num_bits: 64,
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}
```

运行 `cargo test --release test_decompose_1` 能看到测试通过。

### 电路布局

使用 `dev-graph` features 和 `plotters` 库可以将上述的电路 `MyCircuit` 的布局图直观的画出来。

```rust
#[cfg(feature = "dev-graph")]
#[test]
fn print_decompose_1() {
    use plotters::prelude::*;

    let root = BitMapBackend::new("decompose-layout.png", (1024, 3096)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Decompose Range Check Layout", ("sans-serif", 60))
        .unwrap();

    let circuit = MyCircuit::<Fp, 8, 256> {
        value: Value::unknown(),
        num_bits: 64,
    };
    halo2_proofs::dev::CircuitLayout::default()
        .render(9, &circuit, &root)
        .unwrap();
}
```

运行命令 `cargo test --release --all-features print_decompose_1` 可以得到电路的布局图：
<!-- <img src="/courses/Halo2-0xPARC/halo2-examples/decompose-layout.png" width="50%" height="50%"> -->

![image](/courses/Halo2-0xPARC/halo2-examples/decompose-layout.png)

* advice columns(witness) 是<font color=#F7CEDE>粉色</font>的，见图中 Witness value这一列。
* <font color=#CED6A8>浅绿色</font>的 cells 说明在电路定义时用到了，是 Region 的一部分。
* <font color=#BAC198>深绿色</font>的 cells 说明被赋值了。
* <font color=#CCCCFB>浅紫色</font>：selector, 对应代码中的 `q_decompose`.
* <font color=#B8B8FA>深紫色</font>：constant value, 在此电路中有 constant value 使得 `z_c = 0`，因此图中最后一列表示 constant value.
* 图中的 `Decompose value` 就是 `z_1, ..., z_c`

注意到，这次对 `MyCircuit` 实现 `Circuit` trait 的 `without_witnesses` 函数时，没有用 `default` 方法，也没有给 `MyCircuit` 默认方法。

```rust
fn without_witnesses(&self) -> Self {
    Self {
        value: Value::unknown(),
        num_bits: self.num_bits,
    }
}
```

如果给 `MyCircuit` 加上 `#[derive(Default)]`, 同时

```rust
fn without_witnesses(&self) -> Self {
    Self::default()
}
```

此时运行会报错：

```shell
running 1 test
test decompose_range_check::example1::tests::test_decompose_1 ... FAILED

failures:

---- decompose_range_check::example1::tests::test_decompose_1 stdout ----
Equality constraint not satisfied by cell (Column { column_type: Advice, index: 0 }, in Region 2 ('Decompose value') at offset 0)

Equality constraint not satisfied by cell (Column { column_type: Advice, index: 0 }, in Region 1 ('Witness value') at offset 0)

thread 'decompose_range_check::example1::tests::test_decompose_1' panicked at /Users/jinjin/.cargo/git/checkouts/halo2-d6fd4df1666d8b25/a898d65/halo2_proofs/src/dev.rs:873:13:
circuit was not satisfied
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    decompose_range_check::example1::tests::test_decompose_1

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

此时打印出电路图，得到：

![image](/courses/Halo2-0xPARC/halo2-examples/decompose-layout-fail.png)

可以看到左上角的 `Decompose Value` 和 `Witness value` 重复了，被重复赋值了。主要到目前用的是双通道布局。

```rust
type FloorPlanner = V1;
```
在 V1 中，`without_witness` 函数需要创建第一个电路，来传递给layouter，开始会先传递一个空的电路，此时 `num_bits = 0`， 而之后电路传进去的 `num_bits = 64`, 这里 `num_bits` 是一个变量，因此重复赋值了，导致了错误。

如果将 `FloorPlanner` 改为 `SimpleFloorPlanner` 会通过测试。

```rust
type FloorPlanner = SimpleFloorPlanner;
```

因为 `SimpleFloorPlanner` 单通道布局初始时不会先传递一个空电路。其电路布局如下图所示：

![image](/courses/Halo2-0xPARC/halo2-examples/decompose-layout-SimpleFloorPlanner.png)

如果电路中字段含有其他信息，不要用derive(defalut)
halo2不好检查，因为不清楚那些field是without_witness，最好自己明确清楚。

### 生成证明

如果想生成证明，在 [halo2_proofs/tests/plonk_api.rs](https://github.com/zcash/halo2/blob/main/halo2_proofs/tests/plonk_api.rs) 中有对应的例子。

```rust
// Initialize the proving key
let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

let pubinputs = vec![instance];

// Check this circuit is satisfied.
let prover = match MockProver::run(K, &circuit, vec![pubinputs.clone()]) {
    Ok(prover) => prover,
    Err(e) => panic!("{:?}", e),
};
assert_eq!(prover.verify(), Ok(()));

if std::env::var_os("HALO2_PLONK_TEST_GENERATE_NEW_PROOF").is_some() {
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    // Create a proof
    create_proof(
        &params,
        &pk,
        &[circuit.clone(), circuit.clone()],
        &[&[&[instance]], &[&[instance]]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation should not fail");
    let proof: Vec<u8> = transcript.finalize();

    std::fs::write("./tests/plonk_api_proof.bin", &proof[..])
        .expect("should succeed to write new proof");
}

{
    // Check that a hardcoded proof is satisfied
    let proof =
        std::fs::read("./tests/plonk_api_proof.bin").expect("should succeed to read proof");
    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    assert!(verify_proof(
        &params,
        pk.get_vk(),
        strategy,
        &[&[&pubinputs[..]], &[&pubinputs[..]]],
        &mut transcript,
    )
    .is_ok());
}
```

* OsRng 是和 transcript 相关的随机数

## 非整除情况

如果 `N % K != 0`, 假设最后剩下 `l` bits (`l < K`). 可以对最后的 `l` bits 进行单独表达式查询，但是如果 `l` 很接近 `K` 的话，次数还是比较高的，这样速度就比较慢了，加速的一个方法是对每一个可能的 `l` hard code 一个电路门。
