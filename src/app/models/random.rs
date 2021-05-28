use rand::{RngCore, SeedableRng, rngs::StdRng};

pub trait RandomStringGenerator {
    fn generate(&self, size: usize) -> String;
}

pub struct RandomString {}

impl RandomString {
    pub fn new() -> Self {
        Self {}
    }
}

impl RandomStringGenerator for RandomString {
     fn generate(&self, size: usize) -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs = vec![0; size];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE_NO_PAD)
     }
}
