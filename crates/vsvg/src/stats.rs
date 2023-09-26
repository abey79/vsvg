use crate::{LayerTrait, PathDataTrait, PathTrait};

#[derive(Debug)]
pub struct LayerStats {
    pub num_paths: usize,
    pub pen_up_length: f64,
}

impl LayerStats {
    pub fn from_layer<L: LayerTrait<P, D>, P: PathTrait<D>, D: PathDataTrait>(layer: &L) -> Self {
        LayerStats {
            num_paths: layer.paths().len(),
            pen_up_length: layer
                .paths()
                .windows(2)
                .map(|w| {
                    if let Some(ref start) = w[0].end() {
                        if let Some(ref end) = w[1].start() {
                            start.distance(end)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                })
                .sum(),
        }
    }
}
