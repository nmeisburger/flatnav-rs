pub trait Distance<T> {
    fn call(&self, a: &[T], b: &[T]) -> f32;
}

pub struct EuclideanDistance;

impl Distance<f32> for EuclideanDistance {
    fn call(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}
