use super::constants::Constants;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Activation {
    Sigmoid,
    ReLu,
    LeakyRelu,
    Tanh,
    None,
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub fn relu(x: f32) -> f32 {
    x.max(0.0)
}
pub fn leaky_relu(x: f32) -> f32 {
    x.max(0.1 * x)
}

pub fn tanh(x: f32) -> f32 {
    x.tanh()
}

pub fn which_activation(constants: &Constants) -> Activation {
    match constants.activation.as_str() {
        "sigmoid" => Activation::Sigmoid,
        "relu" => Activation::ReLu,
        "leaky_relu" => Activation::LeakyRelu,
        "tanh" => Activation::Tanh,
        _ => Activation::None,
    }
}

impl Activation {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            Activation::Sigmoid => sigmoid(x),
            Activation::ReLu => relu(x),
            Activation::LeakyRelu => leaky_relu(x),
            Activation::Tanh => tanh(x),
            Activation::None => x,
        }
    }
}
