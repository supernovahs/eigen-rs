#![doc(
    html_logo_url = "https://github.com/supernovahs/eigensdk-rs/assets/91280922/bd13caec-3c00-4afc-839a-b83d2890beb5",
    issue_tracker_base_url = "https://github.com/supernovahs/eigen-rs/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use ark_bn254::G1Projective;
use ark_bn254::G2Projective;
use ark_ff::BigInteger256;
// use eigen_crypto_bls::G1Point as AttestationG1Point;
use eigen_crypto_bn254::utils::biginteger256_to_u256;
use eigen_crypto_bn254::utils::u256_to_bigint256;
use eigen_utils::binding::BLSApkRegistry::{G1Point, G2Point};

// pub fn convert_bn254_to_ark(g1_point: G1Point) -> AttestationG1Point {
//     AttestationG1Point::new(u256_to_bigint256(g1_point.X), u256_to_bigint256(g1_point.Y))
// }

pub fn convert_to_bn254_g1_point(g1: G1Projective) -> G1Point {
    let x: BigInteger256 = g1.x.into();
    let y: BigInteger256 = g1.y.into();

    G1Point {
        X: biginteger256_to_u256(x),
        Y: biginteger256_to_u256(y),
    }
}

pub fn convert_to_bn254_g2_point(g2: G2Projective) -> G2Point {
    let x_0: BigInteger256 = g2.x.c0.into();
    let x_1: BigInteger256 = g2.x.c1.into();
    let y_0: BigInteger256 = g2.y.c0.into();
    let y_1: BigInteger256 = g2.y.c1.into();

    G2Point {
        X: [biginteger256_to_u256(x_0), biginteger256_to_u256(x_1)],
        Y: [biginteger256_to_u256(y_0), biginteger256_to_u256(y_1)],
    }
}
