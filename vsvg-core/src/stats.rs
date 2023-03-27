use crate::{DocumentImpl, LayerID, LayerImpl, PathType};
use std::collections::HashMap;

#[derive(Debug)]
pub struct LayerStats {
    pub num_paths: usize,
    pub pen_up_length: f64,
}

impl<T: PathType> LayerImpl<T> {
    #[must_use]
    pub fn stats(&self) -> LayerStats {
        LayerStats {
            num_paths: self.paths.len(),
            pen_up_length: self
                .paths
                .windows(2)
                .map(|w| {
                    if let Some(ref start) = w[0].data.end() {
                        if let Some(ref end) = w[1].data.start() {
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

impl<T: PathType> DocumentImpl<T> {
    #[must_use]
    pub fn stats(&self) -> HashMap<LayerID, LayerStats> {
        self.layers
            .iter()
            .map(|(id, layer)| (*id, layer.stats()))
            .collect()
    }
}
