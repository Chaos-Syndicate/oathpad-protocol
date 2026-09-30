use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::Mint;
use crate::{errors::OathError, state::*};

#[derive(Accounts)]
pub struct CreateOath<'info> {
    #[account(mut)] pub creator: Signer<'info>,
    #[account(seeds = [b"protocol"], bump, constraint = !protocol.paused @ OathError::Paused)]
    pub protocol: Account<'info, ProtocolConfig>,
    /// Re-checked here, not only at binding time: a creator removed from the allowlist keeps
    /// their binding but cannot add Oaths to it while the gate is closed.
    #[account(seeds = [b"launch_gate"], bump, constraint = launch_gate.allows(&creator.key()) @ OathError::LaunchesGated)]
    pub launch_gate: Account<'info, LaunchGate>,
    #[account(mut, seeds = [b"launch", launch.token_mint.as_ref()], bump, has_one = creator)]
    pub launch: Account<'info, LaunchBinding>,
    #[account(init, payer = creator, space = 8 + Oath::INIT_SPACE,
        seeds = [b"oath", launch.key().as_ref(), &[launch.oath_count]], bump)]
    pub oath: Account<'info, Oath>,
    /// CHECK: native SOL uses the system address; otherwise verified as an SPL mint below.
    pub asset_mint: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateOath>, committed_amount: u64, failure_action: FailureAction) -> Result<()> {
    require!(ctx.accounts.launch.status == LaunchStatus::Draft, OathError::LaunchLocked);
    require!(ctx.accounts.launch.oath_count < MAX_OATHS, OathError::TooManyOaths);
    require!(committed_amount > 0, OathError::ZeroAmount);
    let mint = ctx.accounts.asset_mint.key();
    if mint == Pubkey::default() {
        require!(!matches!(failure_action, FailureAction::Burn), OathError::UnsupportedSolBurn);
    } else {
        require_keys_eq!(*ctx.accounts.asset_mint.owner, anchor_spl::token::ID, OathError::WrongOwner);
        let data = ctx.accounts.asset_mint.try_borrow_data()?;
        let parsed = Mint::try_deserialize(&mut &data[..])?;
        require!(parsed.is_initialized, OathError::WrongMint);
        // An issuer must never be able to freeze escrow and veto settlement.
        require!(parsed.freeze_authority == COption::None, OathError::MintHasFreezeAuthority);
    }
    let oath_key = ctx.accounts.oath.key();
    let vault = Pubkey::find_program_address(&[b"vault", oath_key.as_ref()], ctx.program_id).0;
    if let FailureAction::FixedRecipient { recipient } = failure_action {
        require_keys_neq!(recipient, Pubkey::default(), OathError::WrongRecipient);
        require_keys_neq!(recipient, vault, OathError::WrongRecipient);
        require_keys_neq!(recipient, oath_key, OathError::WrongRecipient);
        require_keys_neq!(recipient, ctx.accounts.creator.key(), OathError::WrongRecipient);
    }
    ctx.accounts.oath.set_inner(Oath {
        launch: ctx.accounts.launch.key(), creator: ctx.accounts.creator.key(), asset_mint: mint,
        vault, committed_amount, milestone_count: 0,
        reviewer_set: Pubkey::find_program_address(&[b"reviewers", oath_key.as_ref()], ctx.program_id).0,
        failure_action, status: OathStatus::Draft,
    });
    ctx.accounts.launch.oath_count = ctx.accounts.launch.oath_count.checked_add(1).ok_or(OathError::ArithmeticOverflow)?;
    Ok(())
}
