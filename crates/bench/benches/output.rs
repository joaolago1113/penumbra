use std::str::FromStr;

use ark_ff::UniformRand;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::{Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;
use penumbra_asset::{balance, Balance, Value};
use penumbra_keys::{keys::Diversifier, Address};
use penumbra_proof_params::OUTPUT_PROOF_PROVING_KEY;
use penumbra_shielded_pool::{note, Note, OutputCircuit, OutputProof, Rseed};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(
    r: Fq,
    s: Fq,
    note: Note,
    v_blinding: Fr,
    balance_commitment: balance::Commitment,
    note_commitment: note::StateCommitment,
) {
    let _proof = OutputProof::prove(
        r,
        s,
        &OUTPUT_PROOF_PROVING_KEY,
        note,
        v_blinding,
        balance_commitment,
        note_commitment,
    )
    .expect("can generate proof");
}

fn output_proving_time(c: &mut Criterion) {
    let diversifier_bytes = [1u8; 16];
    let pk_d_bytes = decaf377::basepoint().vartime_compress().0;
    let clue_key_bytes = [1; 32];
    let diversifier = Diversifier(diversifier_bytes);
    let address = Address::from_components(
        diversifier,
        ka::Public(pk_d_bytes),
        fmd::ClueKey(clue_key_bytes),
    )
    .expect("generated 1 address");
    let value_to_send = Value::from_str("1upenumbra").expect("valid value");

    let note = Note::from_parts(address, value_to_send, Rseed([1u8; 32])).expect("can make a note");
    let v_blinding = Fr::from(1);
    let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);
    let note_commitment = note.commit();

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);

    c.bench_function("output proving", |b| {
        b.iter(|| {
            prove(
                r,
                s,
                note.clone(),
                v_blinding,
                balance_commitment,
                note_commitment,
            )
        })
    });

    // Also print out the number of constraints.
    let circuit = OutputCircuit::new(note, v_blinding, balance_commitment);

    let cs = ConstraintSystem::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);

    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints");
    cs.finalize();
    let num_constraints = cs.num_constraints();
    println!("Number of constraints: {}", num_constraints);
}

criterion_group!(benches, output_proving_time);
criterion_main!(benches);
