//! Read-only view of a Meteora Dynamic Bonding Curve `VirtualPool` account.
//!
//! Oath Core never calls the DBC program and never vendors its source
//! (docs/THREAT-MODEL.md section 2). It only reads the public account layout of a
//! pool the way any RPC client does, to prove that the signer of
//! `create_launch_binding` is the creator of the pool for the launch mint.
//!
//! Layout (from the DBC IDL shipped in @meteora-ag/dynamic-bonding-curve-sdk 1.5.13,
//! type `VirtualPool { pool_state: PoolState }`, `serialization: bytemuck`, `repr(C)`),
//! cross-checked byte for byte against the real devnet pool
//! `JEJrGj2t4zDvmHTR9Dcwpynpd11yMTAzdvJuwbSwXWb8` (docs/B2-PROGRAM-FIX-REPORT.md):
//!
//! ```text
//! offset  size  field
//!      0     8  account discriminator = sha256("account:VirtualPool")[..8]
//!      8    64  volatility_tracker { last_update_timestamp u64, padding [u8;8],
//!                                    sqrt_price_reference u128, volatility_accumulator u128,
//!                                    volatility_reference u128 }
//!     72    32  config      (PoolConfig account the pool was created from)
//!    104    32  creator     (wallet that created the pool)
//!    136    32  base_mint   (the launch token mint)
//!    168   ...  vaults, reserves, fees, price, flags (not read here)
//! ```
use anchor_lang::prelude::*;
use crate::errors::OathError;

/// Meteora Dynamic Bonding Curve program. One canonical ID on devnet and mainnet-beta
/// (docs/UPSTREAM-RESEARCH.md section 3).
pub const METEORA_DBC_PROGRAM_ID: Pubkey = anchor_lang::solana_program::pubkey!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");

/// `sha256("account:VirtualPool")[..8]`, as published in the DBC IDL (`accounts[].discriminator`).
pub const VIRTUAL_POOL_DISCRIMINATOR: [u8; 8] = [213, 224, 5, 209, 98, 69, 119, 92];

pub const VIRTUAL_POOL_CONFIG_OFFSET: usize = 72;
pub const VIRTUAL_POOL_CREATOR_OFFSET: usize = 104;
pub const VIRTUAL_POOL_BASE_MINT_OFFSET: usize = 136;
/// The shortest buffer that still contains every field read here. Real pools are 424 bytes.
pub const VIRTUAL_POOL_MIN_LEN: usize = VIRTUAL_POOL_BASE_MINT_OFFSET + 32;

/// The three facts a launch binding needs from a pool.
pub struct VirtualPoolFacts {
    pub config: Pubkey,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
}

/// True when the account is owned by the DBC program (whatever its contents).
pub fn is_dbc_owned(info: &AccountInfo) -> bool {
    *info.owner == METEORA_DBC_PROGRAM_ID
}

/// Parse a DBC-owned account as a `VirtualPool`. Every failure is `NotDbcPool`.
pub fn read_virtual_pool(info: &AccountInfo) -> Result<VirtualPoolFacts> {
    require_keys_eq!(*info.owner, METEORA_DBC_PROGRAM_ID, OathError::NotDbcPool);
    let data = info.try_borrow_data()?;
    require!(data.len() >= VIRTUAL_POOL_MIN_LEN, OathError::NotDbcPool);
    require!(data[..8] == VIRTUAL_POOL_DISCRIMINATOR, OathError::NotDbcPool);
    let key = |offset: usize| -> Result<Pubkey> {
        let bytes: [u8; 32] = data[offset..offset + 32].try_into().map_err(|_| error!(OathError::NotDbcPool))?;
        Ok(Pubkey::new_from_array(bytes))
    };
    Ok(VirtualPoolFacts {
        config: key(VIRTUAL_POOL_CONFIG_OFFSET)?,
        creator: key(VIRTUAL_POOL_CREATOR_OFFSET)?,
        base_mint: key(VIRTUAL_POOL_BASE_MINT_OFFSET)?,
    })
}
