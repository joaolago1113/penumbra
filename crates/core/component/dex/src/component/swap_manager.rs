use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sct::{component::SctManager as _, CommitmentSource};
use penumbra_tct as tct;
use tracing::instrument;

use crate::{state_key, swap::SwapPayload};

/// Manages the addition of new notes to the chain state.
#[async_trait]
pub trait SwapManager: StateWrite {
    #[instrument(skip(self, swap), fields(commitment = ?swap.commitment))]
    async fn add_swap_payload(&mut self, swap: SwapPayload, source: CommitmentSource) {
        tracing::debug!("adding swap payload");

        // 0. Record an ABCI event for transaction indexing.
        //self.record(event::state_payload(&payload));

        // 1. Insert it into the SCT, recording its source
        let position = self.add_sct_commitment(swap.commitment, source.clone())
            .await
            // TODO: why? can't we exceed the number of state commitments in a block?
            .expect("inserting into the state commitment tree should not fail because we should budget commitments per block (currently unimplemented)");

        // 3. Finally, record it to be inserted into the compact block:
        let mut payloads = self.pending_swap_payloads();
        payloads.push_back((position, swap, source));
        self.object_put(state_key::pending_payloads(), payloads);
    }

    fn pending_swap_payloads(&self) -> im::Vector<(tct::Position, SwapPayload, CommitmentSource)> {
        self.object_get(state_key::pending_payloads())
            .unwrap_or_default()
    }
}

impl<T: StateWrite + ?Sized> SwapManager for T {}
