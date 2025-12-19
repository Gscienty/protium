#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    #[default]
    FillAll,
    NonZero,
    EvenOdd,
}
