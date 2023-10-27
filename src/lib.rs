#[macro_use]
mod internal_macros;

pub mod argument_tree;
pub mod diagnose;
pub mod generate_quantities;
pub mod laplace;
pub mod logprob;
pub mod method;
pub mod optimize;
pub mod pathfinder;
pub mod sample;
pub mod variational;

pub use crate::method::*;

// pub struct Model {
//     stan_file: String,
//     outdir: String,
// }
