use ark_bls12_381::{Bls12_381, Fr as F};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, PreparedVerifyingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::{prelude::*, fields::fp::FpVar};
use ark_snark::SNARK;
use ark_std::rand::{RngCore, CryptoRng};
use anyhow::Result;

pub struct MyCircuit {
    pub a: Option<F>,
    pub b: Option<F>,
    pub c: Option<F>,
}

impl ConstraintSynthesizer<F> for MyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let a_var = FpVar::new_witness(cs.clone(), || self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b_var = FpVar::new_witness(cs.clone(), || self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c_var = FpVar::new_input(cs.clone(), || self.c.ok_or(SynthesisError::AssignmentMissing))?;

        let ab = &a_var * &b_var;
        ab.enforce_equal(&c_var)?;

        Ok(())
    }
}

pub fn setup_groth16<R: RngCore + CryptoRng>(rng: &mut R) -> Result<(ProvingKey<Bls12_381>, VerifyingKey<Bls12_381>)> {
    let circuit = MyCircuit {
        a: None,
        b: None,
        c: None,
    };

    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, rng)?;
    Ok((pk, vk))
}

pub fn prove_groth16<R: RngCore + CryptoRng>(
    pk: &ProvingKey<Bls12_381>,
    a: F,
    b: F,
    c: F,
    rng: &mut R,
) -> Result<Proof<Bls12_381>> {
    let circuit = MyCircuit {
        a: Some(a),
        b: Some(b),
        c: Some(c),
    };

    let proof = Groth16::<Bls12_381>::prove(pk, circuit, rng)?;
    Ok(proof)
}

pub fn verify_groth16(
    vk: &VerifyingKey<Bls12_381>,
    public_inputs: &[F],
    proof: &Proof<Bls12_381>,
) -> Result<bool> {
    let pvk = PreparedVerifyingKey::from(vk.clone());
    let result = Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, public_inputs, proof)?;
    Ok(result)
}

pub fn generate_and_verify_proof(a: i32, b: i32) -> Result<()> {
    use ark_std::rand::thread_rng;
    
    let mut rng = thread_rng();
    
    // Convert to field elements
    let a_field = F::from(a as u64);
    let b_field = F::from(b as u64);
    let c_field = a_field * b_field;
    
    println!("Setting up Groth16 parameters...");
    let (pk, vk) = setup_groth16(&mut rng)?;
    
    println!("Generating proof for {} * {} = {}...", a, b, a as i64 * b as i64);
    let proof = prove_groth16(&pk, a_field, b_field, c_field, &mut rng)?;
    
    println!("Verifying proof...");
    let public_inputs = vec![c_field];
    let is_valid = verify_groth16(&vk, &public_inputs, &proof)?;
    
    if is_valid {
        println!(" Proof verified successfully!");
    } else {
        println!(" Proof verification failed!");
    }
    
    Ok(())
}