pub trait AudioNodeEngine: Sync + Send {
    // XXX Create an AudioBuffer abstraction
    fn process(
        &self,
        data: &mut [u8],
        accumulator_ref: &mut f64,
        freq: u32,
        rate: u32,
        channels: u32,
        vol: f64,
    );
}
