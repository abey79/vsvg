use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Platforms
        puffin: { all(not(target_arch = "wasm32"), feature = "puffin") },
    }
}
