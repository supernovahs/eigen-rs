# Eigen Layer Bls 

This crate contains the following utilities:

- New bls key pair generation 
- Get Public Key on G1 and G2
- Helper functions to convert Arkworks parameters to alloy compatible . Ex:
    - convert_to_g1_point : Converts G1Affine to Alloy compatible G1Point
    - convert_to_g2_point : Converts G2Affine to Alloy compatible G2Point
    - alloy_g1_point_to_g1_affine: Converts Alloy G1Point to G1Affine
- Signing a message using the keypair

## Example 
- [Registering an operator](https://github.com/Layr-Labs/eigensdk-rs/blob/main/examples/avsregistry-write/examples/register_operator_in_quorum_with_avs_registry_coordinator.rs) 