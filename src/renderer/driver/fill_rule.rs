#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FillRule {
    #[default]
    FillAll,
    NonZero,
    EvenOdd,
}
