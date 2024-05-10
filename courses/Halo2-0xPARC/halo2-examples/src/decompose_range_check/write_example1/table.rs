use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::*,
    plonk::{ConstraintSystem, Error, TableColumn},
};
use std::marker::PhantomData;


#[derive(Clone, Debug)]
pub(super) struct RangeTableConfig<F: FieldExt, const NUM_BITS: usize, const RANGE: usize> {
    num_bits: TableColumn,
    value: TableColumn,
    _marker: PhantomData<F>,
}

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
                {
                    table.assign_cell(
                        || "assign num_bits", 
                        , offset, to)
                }
            }
        )
    }
}