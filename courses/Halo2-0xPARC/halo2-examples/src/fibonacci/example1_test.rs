use std::{backtrace, intrinsics::offset, marker::PhantomData, os::unix::ucred::impl_mac};

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::*,
    plonk::*, poly::Rotation,
};

#[derive(Debug, Clone)]
struct ACell<F: FieldExt>(AssignedCell<F, F>);

#[derive(Debug, Clone)]
struct FiboConfig {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
}

struct FiboChip<F: FieldExt> {
    config: FiboConfig,
    _maker: PhantomData<F>,
}

impl<F: FieldExt> FiboChip<F> {
    fn construct(config: FiboConfig) -> Self {
        Self { config, _maker: PhantomData }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
    ) -> FiboConfig {
        let col_a = advice[0];
        let col_b = advice[1];
        let col_c = advice[2];
        let selector = meta.selector();

        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_c);

        meta.create_gate("add", |meta| {
            ///
            /// col_a | col_b | col_c | selector
            ///   a       b       c         s
            /// 
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            vec![s * (a + b - c)]
        });

        FiboConfig { 
            advice: [col_a, col_b, col_c],
            selector,
        }
    }

    fn assign_first_row(&self, mut layouter: impl Layouter<F>, a: Option<F>, b: Option<F>) 
        -> Result<(ACell<F>, ACell<F>, ACell<F>), Error>{
        layouter.assign_region(
            || "first row", 
            |mut region| {
                self.config.selector.enable(&mut region, 0); 

                let a_cell = region.assign_advice(
                    || "a", 
                    self.config.advice[0], 
                    0, 
                    || a.ok_or(Error::Synthesis),
                ).map(ACell)?;

                let b_cell = region.assign_advice(
                    || "b", 
                    self.config.advice[1], 
                    0, 
                    || b.ok_or(Error::Synthesis),
                ).map(ACell)?;

                let c_val = a.and_then(|a| b.map(|b| a + b));      

                let c_cell = region.assign_advice(
                    || "c", 
                    self.config.advice[2], 
                    0, 
                    || c_val.ok_or(Error::Synthesis),
                ).map(ACell)?;    

                Ok((a_cell, b_cell, c_cell))  
        })
    }

    fn assign_row(&self, mut layouter: impl Layouter<F>, prev_b: ACell<F>, prev_c: ACell<F>)
        -> Result<ACell<F, Error> {
            layouter.assign_region(
                || "next row",
                |region| {
                    self.config.selector.enable(&mut region, 0);
                    
                    prev_b.0.copy_advice(|| "a", &mut region, self.config.advice[0], 0)?;
                    prev_c.0.copy_advice(|| "b", &mut region, self.config.advice[1], 0)?;

                    let c_val = prev_b.0.value().and_then(
                        |b| {
                            prev_c.map(|c| *b + *c)
                        }
                    );
                    
                    let c_cell = region.assign_advice(
                        || "c",
                        self. config.advice[2],
                        0,
                        || c_val.ok_or(Error::Synthesis),
                    ).map(ACell)?;

                    Ok(c_cell)
                }
            )
        }
    
}

#[derive(Default)]
struct MyCircuit<F> {
    pub a: Option<F>,
    pub b: Option<F>,
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F>  {
    type Config = FiboConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        FiboChip::configure(meta, [col_a, col_b, col_c])
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = FiboChip::construct(config);

        let (_, mut prev_b, mut prev_c) = chip.assign_first_row(
            layouter.namespace(|| "first row"),
            self.a, self.b,
        );

        for _i in 3..10 {
            let c_cell = chip.assign_row(
                layouter.namespace(|| "next row"),
                &prev_b,
                &prev_c,
            );
            prev_b = prev_c;
            prev_c = c_cell;
        }

        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
