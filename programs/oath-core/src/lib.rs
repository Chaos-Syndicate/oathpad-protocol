use anchor_lang::prelude::*;

pub mod dbc;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
use state::FailureAction;

declare_id!("8GDUGYTnykMWmL5JX5m4nGf2zogq9jJ1QtCJYqpuXVQS");

#[program]
pub mod oath_core {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>, treasury: Pubkey) -> Result<()> {
        instructions::initialize_protocol::handler(ctx, treasury)
    }
    /// Launch gate (admin only; creation-time scope only, see instructions/launch_gate.rs).
    pub fn initialize_launch_gate(ctx: Context<InitializeLaunchGate>) -> Result<()> {
        instructions::launch_gate::initialize(ctx)
    }
    pub fn set_launches_permissionless(ctx: Context<ManageLaunchGate>, permissionless: bool) -> Result<()> {
        instructions::launch_gate::set_permissionless(ctx, permissionless)
    }
    pub fn add_allowed_creator(ctx: Context<ManageLaunchGate>, creator: Pubkey) -> Result<()> {
        instructions::launch_gate::add_creator(ctx, creator)
    }
    pub fn remove_allowed_creator(ctx: Context<ManageLaunchGate>, creator: Pubkey) -> Result<()> {
        instructions::launch_gate::remove_creator(ctx, creator)
    }
    /// `dbc_pool` is now an account (verified when DBC-owned); only `dbc_config` stays an argument.
    pub fn create_launch_binding(ctx: Context<CreateLaunchBinding>, dbc_config: Pubkey) -> Result<()> {
        instructions::create_launch_binding::handler(ctx, dbc_config)
    }
    pub fn create_oath(ctx: Context<CreateOath>, committed_amount: u64, failure_action: FailureAction) -> Result<()> {
        instructions::create_oath::handler(ctx, committed_amount, failure_action)
    }
    pub fn create_reviewer_set(ctx: Context<CreateReviewerSet>, reviewers: Vec<Pubkey>, threshold: u8) -> Result<()> {
        instructions::create_reviewer_set::handler(ctx, reviewers, threshold)
    }
    pub fn add_milestone(ctx: Context<AddMilestone>, allocation_amount: u64, deadline: i64) -> Result<()> {
        instructions::add_milestone::handler(ctx, allocation_amount, deadline)
    }
    pub fn fund_oath_spl(ctx: Context<FundOathSpl>, amount: u64) -> Result<()> {
        instructions::fund_oath_spl::handler(ctx, amount)
    }
    pub fn fund_oath_sol(ctx: Context<FundOathSol>, amount: u64) -> Result<()> {
        instructions::fund_oath_sol::handler(ctx, amount)
    }
    pub fn activate_oath(ctx: Context<ActivateOath>) -> Result<()> {
        instructions::activate_oath::handler(ctx)
    }
    pub fn cancel_draft_oath(ctx: Context<CancelDraftOath>) -> Result<()> {
        instructions::cancel_draft_oath::handler(ctx)
    }
    pub fn submit_evidence(ctx: Context<SubmitEvidence>, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::submit_evidence::handler(ctx, evidence_hash)
    }
    pub fn approve_milestone(ctx: Context<ApproveMilestone>) -> Result<()> {
        instructions::approve_milestone::handler(ctx)
    }
    pub fn finalize_milestone(ctx: Context<FinalizeMilestone>) -> Result<()> {
        instructions::finalize_milestone::handler(ctx)
    }
    pub fn claim_tranche(ctx: Context<Settlement>) -> Result<()> {
        instructions::claim_tranche::handler(ctx)
    }
    pub fn resolve_expired(ctx: Context<ResolveExpired>) -> Result<()> {
        instructions::resolve_expired::handler(ctx)
    }
    pub fn execute_failure(ctx: Context<Settlement>) -> Result<()> {
        instructions::execute_failure::handler(ctx)
    }
    pub fn close_completed_oath(ctx: Context<CloseCompletedOath>) -> Result<()> {
        instructions::close_completed_oath::handler(ctx)
    }
}
