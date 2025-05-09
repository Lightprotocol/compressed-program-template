use anchor_lang::prelude::*;
use light_account_checks::discriminator::Discriminator;

use light_sdk_macros::{LightDiscriminator, LightHasher};

#[derive(
    Clone,
    Debug,
    Default,
    anchor_lang::AnchorDeserialize,
    anchor_lang::AnchorSerialize,
    LightDiscriminator,
    LightHasher,
)]
pub struct CounterCompressedAccount {
    #[hash]
    pub owner: Pubkey,
    pub counter: u64,
}