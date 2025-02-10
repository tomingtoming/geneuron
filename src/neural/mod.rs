use nalgebra::{DMatrix, DVector};
use rand::prelude::*;

// Neural network trait for different implementations
pub trait Neural {
    fn process(&self, inputs: &[f32]) -> Vec<f32>;
    fn mutate(&mut self, mutation_rate: f32);
    fn extract_genome(&self) -> Vec<f32>;
    fn apply_genome(&mut self, genome: &[f32]) -> usize;
    fn clone_box(&self) -> Box<dyn Neural>;
}

// Simple feedforward neural network implementation
pub struct FeedForwardNetwork {
    weights: DMatrix<f32>,
    bias: DVector<f32>,
}

impl FeedForwardNetwork {
    pub fn new(inputs: usize, outputs: usize) -> Self {
        let mut rng = thread_rng();
        FeedForwardNetwork {
            weights: DMatrix::from_fn(inputs, outputs, |_, _| rng.gen_range(-1.0..1.0)),
            bias: DVector::from_fn(outputs, |_, _| rng.gen_range(-1.0..1.0)),
        }
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }
}

impl Neural for FeedForwardNetwork {
    fn process(&self, inputs: &[f32]) -> Vec<f32> {
        let input_matrix = DMatrix::from_row_slice(1, inputs.len(), inputs);
        let output = input_matrix * &self.weights + self.bias.transpose();
        output.map(Self::sigmoid).row(0).iter().cloned().collect()
    }

    fn mutate(&mut self, mutation_rate: f32) {
        let mut rng = thread_rng();

        for weight in self.weights.iter_mut() {
            if rng.gen::<f32>() < mutation_rate {
                *weight += rng.gen_range(-0.5..0.5);
            }
        }

        for bias in self.bias.iter_mut() {
            if rng.gen::<f32>() < mutation_rate {
                *bias += rng.gen_range(-0.5..0.5);
            }
        }
    }

    fn extract_genome(&self) -> Vec<f32> {
        let mut genome = Vec::new();
        genome.extend(self.weights.iter());
        genome.extend(self.bias.iter());
        genome
    }

    fn apply_genome(&mut self, genome: &[f32]) -> usize {
        let mut idx = 0;

        for weight in self.weights.iter_mut() {
            if idx < genome.len() {
                *weight = genome[idx];
                idx += 1;
            }
        }

        for bias in self.bias.iter_mut() {
            if idx < genome.len() {
                *bias = genome[idx];
                idx += 1;
            }
        }

        idx
    }

    fn clone_box(&self) -> Box<dyn Neural> {
        Box::new(FeedForwardNetwork {
            weights: self.weights.clone(),
            bias: self.bias.clone(),
        })
    }
}

impl Clone for Box<dyn Neural> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
