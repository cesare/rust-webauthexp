use rand::{RngCore, SeedableRng, rngs::StdRng};

pub trait RandomStringGenerator<const N: usize> {
    fn generate(&self) -> String;
}

pub struct RandomString<const N: usize> {}

impl<const N: usize> RandomString<N> {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generator() -> Box<dyn RandomStringGenerator<N>> {
        Box::new(Self::new())
    }
}

impl<const N: usize> RandomStringGenerator<N> for RandomString<N> {
    fn generate(&self) -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; N] = [0; N];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE_NO_PAD)
     }
}
