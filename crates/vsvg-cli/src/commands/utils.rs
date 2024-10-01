use bpaf::Bpaf;
use vsvg::Length;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
pub(crate) struct Pivot {
    /// Pivot point
    #[bpaf(short, long)]
    #[allow(dead_code)]
    pivot: (),

    #[bpaf(any::<_>("X", Some))]
    pub(crate) x: Length,

    #[bpaf(any::<_>("Y", Some))]
    pub(crate) y: Length,
}
