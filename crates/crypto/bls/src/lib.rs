#![doc(
    html_logo_url = "https://github.com/Layr-Labs/eigensdk-rs/assets/91280922/bd13caec-3c00-4afc-839a-b83d2890beb5",
    issue_tracker_base_url = "https://github.com/Layr-Labs/eigensdk-rs/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use alloy_primitives::{B256, U256};
use ark_std::str::FromStr;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
pub mod error;

use crate::error::BlsError;
use ark_bn254::{Fq, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{
    fields::{Field, PrimeField},
    BigInt, BigInteger256, One,
};
use eigen_utils::binding::RegistryCoordinator::{self};
use RegistryCoordinator::{G1Point, G2Point};
pub type PrivateKey = Fr;
pub type PublicKey = G1Affine;
pub type BlsSignature = G1Affine;
pub type OperatorId = B256;

#[derive(Debug, Clone)]
pub struct BlsG1Point {
    g1: G1Affine,
}

impl BlsG1Point {
    pub fn new(g1: G1Affine) -> Self {
        Self { g1 }
    }

    pub fn g1(&self) -> G1Affine {
        self.g1
    }
}

#[derive(Debug, Clone)]
pub struct BlsG2Point {
    g2: G2Affine,
}

impl BlsG2Point {
    pub fn new(g2: G2Affine) -> Self {
        Self { g2 }
    }

    pub fn g2(&self) -> G2Affine {
        self.g2
    }
}

/// Bls key pair with public key on G1
pub struct BlsKeyPair {
    /// Private Key
    priv_key: Fr,
    /// Public Key on G1
    pub_key: BlsG1Point,
}

impl BlsKeyPair {
    /// Input [`Fr`] as a [`String`]
    pub fn new(fr: String) -> Result<Self, BlsError> {
        let sk_result = Fr::from_str(&fr);
        match sk_result {
            Ok(sk) => {
                let pk = G1Projective::from(G1Affine::generator()) * sk;
                Ok(Self {
                    priv_key: sk,
                    pub_key: BlsG1Point::new(pk.into_affine()),
                })
            }
            Err(_) => Err(BlsError::InvalidBlsPrivateKey),
        }
    }

    /// Get public key on G1
    pub fn public_key(&self) -> BlsG1Point {
        self.pub_key.clone()
    }

    pub fn sign_hashed_to_curve_message(&self, g1_hashed_msg: G1Affine) -> Signature {
        let sk_int: BigInteger256 = self.priv_key.into();
        let r = g1_hashed_msg.mul_bigint(sk_int);
        Signature::new(r.into_affine())
    }

    /// TODO(supernova): Wait for hash_to_curve to be tested in
    /// https://github.com/Layr-Labs/rust-bls-bn254/blob/main/src/lib.rs
    // pub fn sign_message(&self, message: &[u8]) -> Signature {
    //     let g1 = hash_to_curve(message);
    //     println!("hash to curve {:?}", g1);
    //     let sk_int: BigInteger256 = self.priv_key.into();
    //     let r = g1.mul_bigint(sk_int);
    //     Signature::new(r.into_affine())
    // }

    /// Get public key on G2
    pub fn public_key_g2(&self) -> BlsG2Point {
        let pk = G2Projective::from(G2Affine::generator()) * self.priv_key;
        BlsG2Point::new(pk.into_affine())
    }
}

/// Convert [`G1Point`] to [`G1Affine`]
pub fn alloy_g1_point_to_g1_affine(g1_point: G1Point) -> G1Affine {
    let x_point = g1_point.X.into_limbs();
    let x = Fq::new(BigInteger256::new(x_point));
    let y_point = g1_point.Y.into_limbs();
    let y = Fq::new(BigInteger256::new(y_point));
    G1Affine::new(x, y)
}

/// Convert [`G1Affine`] to  Alloy [`G1Point`]
pub fn convert_to_g1_point(g1: G1Affine) -> Result<G1Point, BlsError> {
    let x_point_result = g1.x();
    let y_point_result = g1.y();

    if let (Some(x_point), Some(y_point)) = (x_point_result, y_point_result) {
        let x = BigInt::new(x_point.into_bigint().0);
        let y = BigInt::new(y_point.into_bigint().0);

        let x_u256 = U256::from_limbs(x.0);
        let y_u256 = U256::from_limbs(y.0);

        Ok(G1Point {
            X: x_u256,
            Y: y_u256,
        })
    } else {
        Err(BlsError::InvalidG1Affine)
    }
}

/// Convert [`G2Affine`] to [`G2Point`]
pub fn convert_to_g2_point(g2: G2Affine) -> Result<G2Point, BlsError> {
    let x_point_result = g2.x();
    // let x_point_c1 = g2.x().unwrap().c1;

    let y_point_result = g2.y();
    // let y_point_c1 = g2.y().unwrap().c1;

    if let (Some(x_point), Some(y_point)) = (x_point_result, y_point_result) {
        let x_point_c0 = x_point.c0;
        let x_point_c1 = x_point.c1;
        let y_point_c0 = y_point.c0;
        let y_point_c1 = y_point.c1;

        let x_0 = BigInt::new(x_point_c0.into_bigint().0);
        let x_1 = BigInt::new(x_point_c1.into_bigint().0);
        let y_0 = BigInt::new(y_point_c0.into_bigint().0);
        let y_1 = BigInt::new(y_point_c1.into_bigint().0);

        let x_u256_0 = U256::from_limbs(x_0.0);
        let x_u256_1 = U256::from_limbs(x_1.0);
        let y_u256_0 = U256::from_limbs(y_0.0);
        let y_u256_1 = U256::from_limbs(y_1.0);

        Ok(G2Point {
            X: [x_u256_1, x_u256_0],
            Y: [y_u256_1, y_u256_0],
        })
    } else {
        Err(BlsError::InvalidG2Affine)
    }
}

#[derive(Debug, Clone)]
pub struct Signature {
    g1_point: BlsG1Point,
}

impl Signature {
    pub fn new(g1: G1Affine) -> Self {
        Self {
            g1_point: BlsG1Point::new(g1),
        }
    }

    pub fn g1_point(&self) -> BlsG1Point {
        self.g1_point.clone()
    }
}

/// https://github.com/Layr-Labs/rust-bls-bn254/blob/main/src/lib.rs
pub fn hash_to_curve(digest: &[u8]) -> G1Affine {
    let one = Fq::one();
    let three = Fq::from(3u64);

    let mut hasher = Sha256::new();
    hasher.update(digest);
    let hashed_result = hasher.finalize();

    // Convert digest to a big integer and then to a field element
    let mut x = {
        let big_int = BigUint::from_bytes_be(&hashed_result);
        let mut bytes = [0u8; 32];
        big_int
            .to_bytes_be()
            .iter()
            .rev()
            .enumerate()
            .for_each(|(i, &b)| bytes[i] = b);
        Fq::from_le_bytes_mod_order(&bytes)
    };

    loop {
        // y = x^3 + 3
        let mut y = x;
        y.square_in_place();
        y *= x;
        y += three;

        // Check if y is a quadratic residue (i.e., has a square root in the field)
        if let Some(y) = y.sqrt() {
            return G1Projective::new(x, y, Fq::one()).into_affine();
        } else {
            // x = x + 1
            x += one;
        }
    }
}

#[cfg(test)]
mod tests {

    use ark_bn254::Fq2;

    use super::*;

    #[test]
    fn test_convert_to_g1_point() {
        let x_point = Fq::from_str(
            "17709620697113958145616918533531128159269167719799793368595970620022661612059",
        )
        .unwrap();
        let y_point = Fq::from_str(
            "9890439522434691655532127414660267222813910180198976870423582442696952349816",
        )
        .unwrap();
        let g1_affine = G1Affine::new(x_point, y_point);

        let alloy_g1_point = convert_to_g1_point(g1_affine).unwrap();
        assert_eq!(
            alloy_g1_point.X,
            U256::from_str(
                "17709620697113958145616918533531128159269167719799793368595970620022661612059"
            )
            .unwrap()
        );
        assert_eq!(
            alloy_g1_point.Y,
            U256::from_str(
                "9890439522434691655532127414660267222813910180198976870423582442696952349816"
            )
            .unwrap()
        );
    }

    #[test]
    fn test_alloy_g1_point_to_g1_affine() {
        let alloy_g1_point = G1Point {
            X: U256::from_str(
                "17709620697113958145616918533531128159269167719799793368595970620022661612059",
            )
            .unwrap(),
            Y: U256::from_str(
                "9890439522434691655532127414660267222813910180198976870423582442696952349816",
            )
            .unwrap(),
        };

        let g1_affine = alloy_g1_point_to_g1_affine(alloy_g1_point);
        assert_eq!(
            U256::from_limbs(g1_affine.x().unwrap().into_bigint().0),
            U256::from_str(
                "17709620697113958145616918533531128159269167719799793368595970620022661612059"
            )
            .unwrap()
        );
        assert_eq!(
            U256::from_limbs(g1_affine.y().unwrap().into_bigint().0),
            U256::from_str(
                "9890439522434691655532127414660267222813910180198976870423582442696952349816"
            )
            .unwrap()
        );
    }

    #[test]
    fn test_convert_to_g2_point() {
        let x_point_c0 = Fq::from_str(
            "6834287759893774453556191528501556195232162436167606874229072410417955767882",
        )
        .unwrap();
        let x_point_c1 = Fq::from_str(
            "15529400123788596166111036611862227541174221446291015207340396747864347375335",
        )
        .unwrap();

        let y_point_c0 = Fq::from_str(
            "7616309349481520605447660298084926776417001188005125143383153219707218450524",
        )
        .unwrap();
        let y_point_c1 = Fq::from_str(
            "19775028091101520702581412350510183088819198056772055625089714355379667714558",
        )
        .unwrap();

        let x_point = Fq2::new(x_point_c0, x_point_c1);
        let y_point = Fq2::new(y_point_c0, y_point_c1);

        let g2_affine = G2Affine::new(x_point, y_point);

        let alloy_g2_point = convert_to_g2_point(g2_affine).unwrap();
        assert_eq!(
            alloy_g2_point.X[0],
            U256::from_str(
                "15529400123788596166111036611862227541174221446291015207340396747864347375335"
            )
            .unwrap()
        );
        assert_eq!(
            alloy_g2_point.X[1],
            U256::from_str(
                "6834287759893774453556191528501556195232162436167606874229072410417955767882"
            )
            .unwrap()
        );
        assert_eq!(
            alloy_g2_point.Y[0],
            U256::from_str(
                "19775028091101520702581412350510183088819198056772055625089714355379667714558"
            )
            .unwrap()
        );
        assert_eq!(
            alloy_g2_point.Y[1],
            U256::from_str(
                "7616309349481520605447660298084926776417001188005125143383153219707218450524"
            )
            .unwrap()
        );
    }

    #[test]
    fn test_bls_key_pair() {
        let bls_priv_key =
            "12248929636257230549931416853095037629726205319386239410403476017439825112537";
        let bls_key_pair = BlsKeyPair::new(bls_priv_key.to_string()).unwrap();

        assert_eq!(
            U256::from_limbs(bls_key_pair.public_key().g1().x().unwrap().into_bigint().0),
            U256::from_str(
                "277950648056014144722774518899051149098728246263316284984520891067822832300"
            )
            .unwrap()
        );
        assert_eq!(
            U256::from_limbs(bls_key_pair.public_key().g1().y().unwrap().into_bigint().0),
            U256::from_str(
                "16927236637669640540790285431111034664564710839671197540688155537113438534238"
            )
            .unwrap()
        );
    }
}
