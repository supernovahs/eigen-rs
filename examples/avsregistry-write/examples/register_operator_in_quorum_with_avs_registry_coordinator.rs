//! register operator in quorum with avs registry coordinator
use alloy_primitives::U256;
use alloy_primitives::{Bytes, FixedBytes};
use ark_bn254::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{AffineRepr, CurveGroup, Group};
use ark_ff::{BigInt, BigInteger, PrimeField, UniformRand};
use ark_std::{ops::Mul, test_rng};
use eigen_client_avsregistry::writer::AvsRegistryChainWriter;
use eigen_client_elcontracts::reader::ELChainReader;
use eigen_crypto_bls::BlsKeypair;
use alloy_provider::Provider;
use eigen_testing_utils::m2_holesky_constants::{
    AVS_DIRECTORY_ADDRESS, BLS_APK_REGISTRY, DELEGATION_MANAGER_ADDRESS, OPERATOR_STATE_RETRIEVER,
    REGISTRY_COORDINATOR, SERVICE_MANAGER_ADDRESS, SLASHER_ADDRESS, STAKE_REGISTRY,
};
use eigen_utils::get_provider;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let holesky_provider = "https://ethereum-holesky-rpc.publicnode.com";
    let pvt_key = "160443ef7d1ada994b300f7d2bf88db16217db6f825708e4b23f69aa028d7c8c";
    let avs_registry_writer = AvsRegistryChainWriter::build_avs_registry_chain_writer(
        holesky_provider.to_string(),
        pvt_key.to_string(),
        REGISTRY_COORDINATOR,
        OPERATOR_STATE_RETRIEVER,
    )
    .await.expect("avs writer build fail ");
    // let mut rng = ark_std::test_rng();
    // let sk = Fr::rand(&mut rng);
    // let pubkey = G2Projective::from(G2Affine::generator())
    //     .mul(sk)
    //     .into_affine();

    // println!("private key {:?}", sk);
    // println!("pubkey {:?}",pubkey);

    let bls_pvt_key  = Fr::from(BigInt([9016117505638758543, 352751388875653018, 14946620785396285244, 211688466542070544]));

    let pubkey = G1Projective::generator()
    .mul(bls_pvt_key)
    .into_affine();

    let key_pair = BlsKeypair{private:bls_pvt_key,public:pubkey};

    let digest_hash :FixedBytes<32>= FixedBytes::from([0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02,0x02]) ;
    let provider = get_provider(&holesky_provider.to_string());
    let current_block_number = provider.get_block_number().await?;
    let sig_expiry : U256 = U256::from(current_block_number + 20);
    let quorum_nums = Bytes::from([0x01]);
    // println!("quorum nums : {:?}",quorum_nums);
    let tx_hash = avs_registry_writer.register_operator_in_quorum_with_avs_registry_coordinator(key_pair,digest_hash,sig_expiry,quorum_nums,"65.109.158.181:33078;31078".to_string()).await.unwrap();
    println!("tx hash :{:?}",tx_hash);
    Ok(())
}

pub fn deserialize_montgomery_elements(data: &[Fr], buffer: &mut Vec<u8>) {
    let mut temp_buffer: Vec<u8> = data
        .iter()
        .rev()
        .flat_map(|elem| elem.into_bigint().to_bytes_le())
        .collect();

    temp_buffer.reverse();
    buffer.extend(temp_buffer);
}

// #[tokio::main]
// async fn main() {}
