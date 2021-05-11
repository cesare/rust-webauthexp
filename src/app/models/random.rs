use rand::{RngCore, SeedableRng, rngs::StdRng};

pub struct RandomString<const N: usize> {}

impl<const N: usize> RandomString<N> {
    pub fn generate() -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; N] = [0; N];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE_NO_PAD)
    }
}
