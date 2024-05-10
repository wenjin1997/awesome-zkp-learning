# 2 Halo2 API & Building a Basic Fibonacci Circuit
## 构建电路
构建电路分为三步：
1. 定义 `Config` 结构，包含电路中要用到的列。
2. 定义 `Chip` 结构，配置电路中的约束，以及提供 assignment 函数。
3. 定义一个 `Circuit` 结构，该结构实现了 `Circuit` trait，为电路实例化。
### Constraint system
```rust
fn selector(&mut self) -> Selector
fn complex_selector(&mut self) -> Selector
```
`selector`与`complex_selector`的不同：
* `selector`是简单的一个选择器函数，可以选择一些自定义门或自定义约束，而`complex_selector`可以使用它来构造一个查找参数。
* 这样进行区分的一个出发点是，简单的selector可以用来进行后端的组合优化，而complex_selector不会去进行优化。

```rust
fn enable_equality<C: Into<<Column<Any>>>>(&mut self, column: C)
```
* 该函数会检查包含的列中的permutation是否正确。

### Circuit trait
下面就是 `Circuit` trait 的定义，在自己的电路中就需要实现该 trait。
```rust
pub trait Circuit<F: Field> {
    type Config: Clone;
    type FloorPlanner: FloorPlanner;

    /// Returns a copy of this circuit with no witness values
    fn without_witness(&self) -> Self;

    /// The circuit is given an opportunity to describe the exact gate
    /// arrangement, column arrangement, etc.
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config;

    /// Given the provided `cs`, synthesize the circuit.
    fn synthesize(&self, config: Self:Config, layouter: impl Layouter<F>) -> Result<(), Error>;
}
```
* 在`configure`函数中可以配置多个`Chip`。

为什么要有selector列？可以不添加吗？
在Fibonacci例子中，需要 $2^k$ 行，可能有的行不会有加法约束，这个时候就需要 selector 列，将 selector 置为 0。

```rust
meta.enable_equality(col_a);
meta.enable_equality(col_b);
meta.enable_equality(col_c);
```
调用 `enable_equality()` 函数表明该列参与了 permutation argument。

## Rust知识补充
### `PhantomData` 数据
在[example1.rs](/courses/Halo2-0xPARC/02-Halo2%20API/fibonacci/src/example1.rs)中，结构 `FibonacciChip` 结构如下：
```rust
#[derive(Debug, Clone)]
struct FibonacciChip<F: FieldExt> {
    config: FibonacciConfig,
    _marker: PhantomData<F>,
}
```
参考[Rust秘典](https://nomicon.purewhite.io/phantom-data.html)，`PhantomData`是Rust中的幽灵数据。

> 在结构定义中，不受约束的生命周期和类型是禁止的，因此我们必须在主体中以某种方式引用这些类型，正确地做到这一点对于正确的变异性和丢弃检查是必要的。
> 我们使用PhantomData来做这个，它是一个特殊的标记类型。PhantomData不消耗空间，但为了静态分析的目的，模拟了一个给定类型的字段。这被认为比明确告诉类型系统你想要的变量类型更不容易出错，同时也提供了其他有用的东西，例如 auto traits 和 drop check 需要的信息。

### cargo
如果要运行带有`features`的test，可以运行`cargo test --all-features` .

运行`cargo test --help`：
```shell
jinjin@MacBook-Air halo2-examples % cargo test --help
Execute all unit and integration tests and build examples of a local package

Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

Arguments:
  [TESTNAME]  If specified, only run tests containing this string in their names
  [ARGS]...   Arguments for the test binary

Options:
      --doc                     Test only this library's documentation
      --no-run                  Compile, but don't run tests
      --no-fail-fast            Run all tests regardless of failure
      --ignore-rust-version     Ignore `rust-version` specification in packages
      --future-incompat-report  Outputs a future incompatibility report at the end of the build
      --message-format <FMT>    Error format
  -q, --quiet                   Display one character per test instead of one line
  -v, --verbose...              Use verbose output (-vv very verbose/build.rs output)
      --color <WHEN>            Coloring: auto, always, never
      --config <KEY=VALUE>      Override a configuration value
  -Z <FLAG>                     Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for
                                details
  -h, --help                    Print help

Package Selection:
  -p, --package [<SPEC>]  Package to run tests for
      --workspace         Test all packages in the workspace
      --exclude <SPEC>    Exclude packages from the test
      --all               Alias for --workspace (deprecated)

Target Selection:
      --lib               Test only this package's library unit tests
      --bins              Test all binaries
      --bin [<NAME>]      Test only the specified binary
      --examples          Test all examples
      --example [<NAME>]  Test only the specified example
      --tests             Test all test targets
      --test [<NAME>]     Test only the specified test target
      --benches           Test all bench targets
      --bench [<NAME>]    Test only the specified bench target
      --all-targets       Test all targets (does not include doctests)

Feature Selection:
  -F, --features <FEATURES>  Space or comma separated list of features to activate
      --all-features         Activate all available features
      --no-default-features  Do not activate the `default` feature

Compilation Options:
  -j, --jobs <N>                Number of parallel jobs, defaults to # of CPUs.
  -r, --release                 Build artifacts in release mode, with optimizations
      --profile <PROFILE-NAME>  Build artifacts with the specified profile
      --target [<TRIPLE>]       Build for the target triple
      --target-dir <DIRECTORY>  Directory for all generated artifacts
      --unit-graph              Output build graph in JSON (unstable)
      --timings[=<FMTS>]        Timing output formats (unstable) (comma separated): html, json

Manifest Options:
      --manifest-path <PATH>  Path to Cargo.toml
      --frozen                Require Cargo.lock and cache are up to date
      --locked                Require Cargo.lock is up to date
      --offline               Run without accessing the network
```