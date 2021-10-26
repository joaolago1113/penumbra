use ark_ff::PrimeField;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use std::convert::{TryFrom, TryInto};
use thiserror;

use crate::{ka, keys::Diversifier, Fq, Value};

// TODO: Should have a `leadByte` as in Sapling and Orchard note plaintexts?
// Do we need that in addition to the tx version?

/// A plaintext Penumbra note.
pub struct Note {
    // Value (32-byte asset ID plus 32-byte amount). 64 bytes.
    value: Value,

    // Commitment trapdoor. 32 bytes.
    note_blinding: Fq,

    // The diversifier of the destination address.
    diversifier: Diversifier,

    transmission_key: ka::Public,
    // The s-component of the transmission key of the destination address.
    transmission_key_s: Fq,
}

/// The domain separator used to generate note commitments.
static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid note commitment")]
    InvalidNoteCommitment,
    #[error("Invalid transmission key")]
    InvalidTransmissionKey,
}

impl Note {
    pub fn new(
        diversifier: Diversifier,
        transmission_key: &ka::Public,
        value: Value,
        note_blinding: Fq,
    ) -> Result<Self, Error> {
        Ok(Note {
            value: value,
            note_blinding,
            diversifier,
            transmission_key: *transmission_key,
            transmission_key_s: Fq::from_bytes(transmission_key.0)
                .map_err(|_| Error::InvalidTransmissionKey)?,
        })
    }

    pub fn diversified_generator(&self) -> decaf377::Element {
        self.diversifier.diversified_generator()
    }

    pub fn transmission_key(&self) -> ka::Public {
        self.transmission_key
    }

    pub fn diversifier(&self) -> Diversifier {
        self.diversifier
    }

    pub fn note_blinding(&self) -> Fq {
        self.note_blinding
    }

    pub fn value(&self) -> Value {
        self.value
    }

    pub fn commit(&self) -> Commitment {
        Commitment::new(
            self.note_blinding,
            self.value,
            self.diversified_generator(),
            self.transmission_key_s,
        )
    }
}

// Note commitment.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Commitment(pub Fq);

impl Commitment {
    pub fn new(
        note_blinding: Fq,
        value: Value,
        diversified_generator: decaf377::Element,
        transmission_key_s: Fq,
    ) -> Self {
        let commit = poseidon377::hash_5(
            &NOTECOMMIT_DOMAIN_SEP,
            (
                note_blinding,
                value.amount.into(),
                value.asset_id.0,
                diversified_generator.compress_to_field(),
                transmission_key_s,
            ),
        );

        Commitment(commit)
    }
}

impl Into<[u8; 32]> for Commitment {
    fn into(self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl TryFrom<[u8; 32]> for Commitment {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

impl TryFrom<&[u8]> for Commitment {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes: [u8; 32] = slice[..]
            .try_into()
            .map_err(|_| Error::InvalidNoteCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}
