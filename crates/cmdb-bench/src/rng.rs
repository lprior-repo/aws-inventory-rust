//! Small deterministic PRNG. Same seed and call sequence always produce the same values.

const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const INCREMENT: u64 = 1_442_695_040_888_963_407;

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        self.state
    }

    /// Uniform value in `0..bound`; `bound == 0` yields `0`.
    ///
    /// Multiply-high keeps the high-entropy bits of the LCG state; taking a
    /// remainder of the raw state would inherit the low bits' short period.
    pub fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        let product = u128::from(self.next_u64()).wrapping_mul(u128::from(bound));
        u64::try_from(product >> 64).unwrap_or(0)
    }
}
