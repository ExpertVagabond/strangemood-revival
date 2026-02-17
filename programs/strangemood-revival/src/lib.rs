use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("Av997JVrRJXPTrjbMnkPmMzbgwWWsHxuGRjecqVWUMFi");

pub fn mint_to_and_freeze<'a>(
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump: u8,
    amount: u64,
) -> Result<()> {
    mint_to(
        token_program.clone(),
        mint.clone(),
        to.clone(),
        authority.clone(),
        bump,
        amount,
    )?;
    freeze_account(token_program, mint, to, authority, bump)
}

pub fn mint_to<'a>(
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_program = token_program;
    let cloned_mint = mint.key.clone();
    let cpi_accounts = anchor_spl::token::MintTo {
        mint,
        to,
        authority,
    };
    let seeds = &[b"mint", cloned_mint.as_ref(), &[bump]];
    let signers = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    anchor_spl::token::mint_to(cpi_ctx, amount)
}

pub fn freeze_account<'a>(
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    account: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump: u8,
) -> Result<()> {
    let cpi_program = token_program;
    let cloned_mint = mint.key.clone();
    let cpi_accounts = anchor_spl::token::FreezeAccount {
        mint,
        account,
        authority,
    };
    let seeds = &[b"mint", cloned_mint.as_ref(), &[bump]];
    let signers = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    anchor_spl::token::freeze_account(cpi_ctx)
}

pub fn token_escrow_transfer<'a>(
    token_program: AccountInfo<'a>,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    bump: u8,
) -> Result<()> {
    let cpi_program = token_program;
    let key = from.key.clone();
    let cpi_accounts = anchor_spl::token::Transfer {
        from,
        to,
        authority,
    };
    let seeds = &[b"escrow", key.as_ref(), &[bump]];
    let signers = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    anchor_spl::token::transfer(cpi_ctx, amount)
}

pub fn token_transfer<'a>(
    token_program: AccountInfo<'a>,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
) -> Result<()> {
    let cpi_program = token_program;
    let cpi_accounts = anchor_spl::token::Transfer {
        from,
        to,
        authority,
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_spl::token::transfer(cpi_ctx, amount)
}

pub fn burn<'a>(
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    account: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_program = token_program;
    let cloned_mint = mint.key.clone();
    let cpi_accounts = anchor_spl::token::Burn {
        mint,
        from: account,
        authority,
    };
    let seeds = &[b"mint", cloned_mint.as_ref(), &[bump]];
    let signers = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    anchor_spl::token::burn(cpi_ctx, amount)
}

pub fn sync_native<'a>(token_program: &AccountInfo<'a>, account: AccountInfo<'a>) -> Result<()> {
    let ix = spl_token::instruction::sync_native(&token_program.key(), &account.key())?;
    anchor_lang::solana_program::program::invoke(&ix, &[account.clone()])?;
    Ok(())
}

pub fn system_transfer<'a>(
    system_program: &AccountInfo<'a>,
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(&from.key(), &to.key(), lamports);
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[from.clone(), to.clone(), system_program.clone()],
    )?;
    Ok(())
}

pub fn erase_data<'a>(account: &AccountInfo<'a>) {
    let mut data = account.data.borrow_mut();
    data.fill(0);
}

pub fn move_lamports<'a>(
    source_account_info: &AccountInfo<'a>,
    dest_account_info: &AccountInfo<'a>,
    amount: u64,
) {
    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(amount)
        .unwrap();
    **source_account_info.lamports.borrow_mut() = source_account_info
        .lamports()
        .checked_sub(amount)
        .unwrap();
}

pub fn close_native_account<'a>(
    source_account_info: &AccountInfo<'a>,
    dest_account_info: &AccountInfo<'a>,
) {
    let dest_starting_lamports = dest_account_info.lamports();
    **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;
    erase_data(source_account_info);
}

pub fn close_token_escrow_account<'a>(
    token_program: AccountInfo<'a>,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump: u8,
) -> Result<()> {
    let cpi_program = token_program;
    let key = from.key.clone();
    let cpi_accounts = anchor_spl::token::CloseAccount {
        authority,
        account: from,
        destination: to,
    };
    let seeds = &[b"escrow", key.as_ref(), &[bump]];
    let signers = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    anchor_spl::token::close_account(cpi_ctx)
}

#[program]
pub mod strangemood_revival {
    use super::*;

    pub fn init_listing(
        ctx: Context<InitListing>,
        _mint_bump: u8,
        _listing_bump: u8,
        _decimals: u8,
        price: u64,
        refundable: bool,
        consumable: bool,
        available: bool,
        uri: String,
    ) -> Result<()> {
        let charter = ctx.accounts.charter.clone().into_inner();

        let payment_deposit = ctx.accounts.payment_deposit.clone().into_inner();
        if payment_deposit.mint != ctx.accounts.charter_treasury.clone().into_inner().mint {
            return Err(StrangemoodError::MintNotSupported.into());
        }

        let vote_deposit = ctx.accounts.vote_deposit.clone().into_inner();
        if vote_deposit.mint != charter.mint {
            return Err(StrangemoodError::MintNotSupported.into());
        }

        let listing = &mut ctx.accounts.listing;
        listing.is_initialized = true;
        listing.price = price;
        listing.mint = ctx.accounts.mint.key();
        listing.authority = *ctx.accounts.user.key;
        listing.payment_deposit = ctx.accounts.payment_deposit.key();
        listing.vote_deposit = ctx.accounts.vote_deposit.key();
        listing.charter = ctx.accounts.charter.key();
        listing.uri = uri;
        listing.is_refundable = refundable;
        listing.is_consumable = consumable;
        listing.is_available = available;

        Ok(())
    }

    pub fn purchase(
        ctx: Context<Purchase>,
        receipt_nonce: u128,
        _receipt_bump: u8,
        listing_mint_bump: u8,
        _escrow_authority_bump: u8,
        amount: u64,
    ) -> Result<()> {
        msg!("Purchasing");
        let listing = ctx.accounts.listing.clone().into_inner();

        if !listing.is_available {
            return Err(StrangemoodError::ListingUnavailable.into());
        }
        if listing.mint != ctx.accounts.listing_mint.key() {
            return Err(StrangemoodError::UnexpectedListingMint.into());
        }

        token_transfer(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.purchase_token_account.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.user.to_account_info(),
            amount * listing.price,
        )?;

        msg!("Transferred tokens");

        if listing.is_refundable {
            mint_to_and_freeze(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.listing_mint.to_account_info(),
                ctx.accounts.listing_token_account.to_account_info(),
                ctx.accounts.listing_mint_authority.to_account_info(),
                listing_mint_bump,
                amount,
            )?;
        }

        let receipt = &mut ctx.accounts.receipt;
        receipt.is_initialized = true;
        receipt.is_refundable = listing.is_refundable;
        receipt.listing = ctx.accounts.listing.key();
        receipt.purchaser = ctx.accounts.user.key();
        receipt.quantity = amount;
        receipt.listing_token_account = ctx.accounts.listing_token_account.key();
        receipt.cashier = ctx.accounts.cashier.key();
        receipt.nonce = receipt_nonce;
        receipt.price = listing.price;
        receipt.escrow = ctx.accounts.escrow.key();
        receipt.is_cashable = !listing.is_refundable;

        Ok(())
    }

    pub fn cash(
        ctx: Context<Cash>,
        listing_mint_bump: u8,
        charter_mint_bump: u8,
        escrow_authority_bump: u8,
    ) -> Result<()> {
        let listing = ctx.accounts.listing.clone().into_inner();
        let charter = ctx.accounts.charter.clone().into_inner();
        let receipt = ctx.accounts.receipt.clone().into_inner();

        if !receipt.is_cashable {
            return Err(StrangemoodError::ReceiptNotCashable.into());
        }
        if receipt.cashier != ctx.accounts.cashier.key() {
            return Err(StrangemoodError::OnlyCashableByTheCashier.into());
        }
        if listing.mint != ctx.accounts.listing_mint.key() {
            return Err(StrangemoodError::UnexpectedListingMint.into());
        }
        if ctx.accounts.listing_token_account.key() != receipt.listing_token_account {
            return Err(StrangemoodError::UnexpectedListingTokenAccount.into());
        }
        if listing.vote_deposit != ctx.accounts.listings_vote_deposit.key()
            || listing.payment_deposit != ctx.accounts.listings_payment_deposit.key()
        {
            return Err(StrangemoodError::DepositIsNotFoundInListing.into());
        }
        if listing.charter != ctx.accounts.charter.key() {
            return Err(StrangemoodError::UnauthorizedCharter.into());
        }
        if charter.mint != ctx.accounts.charter_mint.key() {
            return Err(StrangemoodError::MintIsNotFoundInCharter.into());
        }
        if ctx.accounts.charter_vote_deposit.key() != charter.vote_deposit {
            return Err(StrangemoodError::DepositIsNotFoundInCharter.into());
        }

        if !receipt.is_refundable {
            mint_to_and_freeze(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.listing_mint.to_account_info(),
                ctx.accounts.listing_token_account.to_account_info(),
                ctx.accounts.listing_mint_authority.to_account_info(),
                listing_mint_bump,
                receipt.quantity,
            )?;
        }

        let lamports: u64 = receipt.price;
        let deposit_rate = 1.0 - charter.payment_contribution_rate();
        let deposit_amount = (deposit_rate * lamports as f64) as u64;
        let contribution_amount = lamports - deposit_amount;

        token_escrow_transfer(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.listings_payment_deposit.to_account_info(),
            ctx.accounts.escrow_authority.to_account_info(),
            deposit_amount,
            escrow_authority_bump,
        )?;

        token_escrow_transfer(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.charter_treasury_deposit.to_account_info(),
            ctx.accounts.escrow_authority.to_account_info(),
            contribution_amount,
            escrow_authority_bump,
        )?;

        move_lamports(
            &ctx.accounts.receipt.to_account_info(),
            &ctx.accounts.charter_treasury_deposit.to_account_info(),
            contribution_amount,
        );

        let treasury = ctx.accounts.charter_treasury.clone().into_inner();
        let votes = contribution_amount as f64
            * charter.expansion_rate(
                treasury.expansion_scalar_amount,
                treasury.expansion_scalar_decimals,
            );
        let deposit_rate = 1.0 - charter.vote_contribution_rate();
        let deposit_amount = (deposit_rate * votes as f64) as u64;
        let contribution_amount = (votes as u64) - deposit_amount;

        mint_to(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.charter_mint.to_account_info(),
            ctx.accounts.listings_vote_deposit.to_account_info(),
            ctx.accounts.charter_mint_authority.to_account_info(),
            charter_mint_bump,
            deposit_amount,
        )?;

        mint_to(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.charter_mint.to_account_info(),
            ctx.accounts.charter_vote_deposit.to_account_info(),
            ctx.accounts.charter_mint_authority.to_account_info(),
            charter_mint_bump,
            contribution_amount,
        )?;

        close_token_escrow_account(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.cashier.to_account_info(),
            ctx.accounts.escrow_authority.to_account_info(),
            escrow_authority_bump,
        )?;

        close_native_account(
            &ctx.accounts.receipt.to_account_info(),
            &ctx.accounts.cashier.to_account_info(),
        );

        Ok(())
    }

    pub fn cancel(
        ctx: Context<Cancel>,
        _listing_bump: u8,
        listing_mint_bump: u8,
        escrow_authority_bump: u8,
    ) -> Result<()> {
        let receipt = ctx.accounts.receipt.clone().into_inner();

        if receipt.is_refundable {
            burn(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.listing_mint.to_account_info(),
                ctx.accounts.listing_token_account.to_account_info(),
                ctx.accounts.listing_mint_authority.to_account_info(),
                listing_mint_bump,
                receipt.quantity,
            )?;
        }

        close_native_account(
            &ctx.accounts.receipt.to_account_info(),
            &ctx.accounts.purchaser.to_account_info(),
        );

        token_escrow_transfer(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.return_deposit.to_account_info(),
            ctx.accounts.escrow_authority.to_account_info(),
            ctx.accounts.escrow.amount,
            escrow_authority_bump,
        )?;

        close_token_escrow_account(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.purchaser.to_account_info(),
            ctx.accounts.escrow_authority.to_account_info(),
            escrow_authority_bump,
        )?;

        Ok(())
    }

    pub fn consume(
        ctx: Context<Consume>,
        _receipt_bump: u8,
        listing_mint_bump: u8,
        amount: u64,
    ) -> Result<()> {
        let listing = ctx.accounts.listing.clone().into_inner();

        if ctx.accounts.authority.key() != listing.authority {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }

        if !listing.is_consumable {
            return Err(StrangemoodError::ListingIsNotConsumable.into());
        }

        burn(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.listing_token_account.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            listing_mint_bump,
            amount,
        )?;

        Ok(())
    }

    pub fn set_receipt_cashable(ctx: Context<SetReceiptCashable>) -> Result<()> {
        if ctx.accounts.authority.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }

        let receipt = &mut ctx.accounts.receipt;
        receipt.is_cashable = true;

        Ok(())
    }

    pub fn init_charter(
        ctx: Context<InitCharter>,
        _charter_bump: u8,
        expansion_rate_amount: u64,
        expansion_rate_decimals: u8,
        sol_contribution_rate_amount: u64,
        sol_contribution_rate_decimals: u8,
        vote_contribution_rate_amount: u64,
        vote_contribution_rate_decimals: u8,
        uri: String,
    ) -> Result<()> {
        let charter = &mut ctx.accounts.charter;
        charter.authority = ctx.accounts.authority.key();
        charter.expansion_rate_amount = expansion_rate_amount;
        charter.expansion_rate_decimals = expansion_rate_decimals;
        charter.payment_contribution_rate_amount = sol_contribution_rate_amount;
        charter.payment_contribution_rate_decimals = sol_contribution_rate_decimals;
        charter.vote_contribution_rate_amount = vote_contribution_rate_amount;
        charter.vote_contribution_rate_decimals = vote_contribution_rate_decimals;
        charter.vote_deposit = ctx.accounts.vote_deposit.key();
        charter.mint = ctx.accounts.mint.key();
        charter.uri = uri;
        charter.is_initialized = true;

        Ok(())
    }

    pub fn set_listing_price(ctx: Context<SetListing>, price: u64) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.listing.price = price;
        Ok(())
    }

    pub fn set_listing_uri(ctx: Context<SetListing>, uri: String) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.listing.uri = uri;
        Ok(())
    }

    pub fn set_listing_availability(ctx: Context<SetListing>, is_available: bool) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.listing.is_available = is_available;
        Ok(())
    }

    pub fn set_listing_deposits(ctx: Context<SetListingDeposit>) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.listing.vote_deposit = ctx.accounts.vote_deposit.key();
        ctx.accounts.listing.payment_deposit = ctx.accounts.payment_deposit.key();
        Ok(())
    }

    pub fn set_listing_authority(ctx: Context<SetListingAuthority>) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.listing.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn set_listing_charter(ctx: Context<SetListingCharter>) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.listing.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.listing.charter = ctx.accounts.charter.key();
        Ok(())
    }

    pub fn set_charter_expansion_rate(
        ctx: Context<SetCharter>,
        expansion_rate_amount: u64,
        expansion_rate_decimals: u8,
    ) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.charter.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.charter.expansion_rate_amount = expansion_rate_amount;
        ctx.accounts.charter.expansion_rate_decimals = expansion_rate_decimals;
        Ok(())
    }

    pub fn set_charter_contribution_rate(
        ctx: Context<SetCharter>,
        sol_contribution_rate_amount: u64,
        sol_contribution_rate_decimals: u8,
        vote_contribution_rate_amount: u64,
        vote_contribution_rate_decimals: u8,
    ) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.charter.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.charter.payment_contribution_rate_amount = sol_contribution_rate_amount;
        ctx.accounts.charter.payment_contribution_rate_decimals = sol_contribution_rate_decimals;
        ctx.accounts.charter.vote_contribution_rate_amount = vote_contribution_rate_amount;
        ctx.accounts.charter.vote_contribution_rate_decimals = vote_contribution_rate_decimals;
        Ok(())
    }

    pub fn set_charter_authority(ctx: Context<SetCharterAuthority>) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.charter.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.charter.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn set_charter_vote_deposit(ctx: Context<SetCharterVoteDeposit>) -> Result<()> {
        if ctx.accounts.user.key() != ctx.accounts.charter.authority.key() {
            return Err(StrangemoodError::UnauthorizedAuthority.into());
        }
        ctx.accounts.charter.vote_deposit = ctx.accounts.vote_deposit.key();
        Ok(())
    }

    pub fn init_charter_treasury(
        ctx: Context<InitCharterTreasury>,
        _treasury_bump: u8,
        expansion_scalar_amount: u64,
        expansion_scalar_decimals: u8,
    ) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.charter = ctx.accounts.charter.key();
        treasury.deposit = ctx.accounts.deposit.key();
        treasury.mint = ctx.accounts.mint.key();
        treasury.expansion_scalar_amount = expansion_scalar_amount;
        treasury.expansion_scalar_decimals = expansion_scalar_decimals;
        treasury.is_initialized = true;

        Ok(())
    }

    pub fn set_charter_treasury_expansion_scalar(
        ctx: Context<SetCharterTreasuryExpansionScalar>,
        expansion_scalar_amount: u64,
        expansion_scalar_decimals: u8,
    ) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.expansion_scalar_amount = expansion_scalar_amount;
        treasury.expansion_scalar_decimals = expansion_scalar_decimals;

        Ok(())
    }

    pub fn set_charter_treasury_deposit(ctx: Context<SetCharterTreasuryDeposit>) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.deposit = ctx.accounts.deposit.key();

        Ok(())
    }
}

// ─── Account Structs ────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(receipt_nonce: u128, receipt_bump: u8, listing_mint_bump: u8, escrow_authority_bump: u8)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub purchase_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = listing_payment_deposit.key() == listing.clone().into_inner().payment_deposit.key(),
        constraint = listing_mint.key() == listing.clone().into_inner().mint.key(),
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(
        constraint = listing_payment_deposit.mint == listing_payment_deposit_mint.key()
    )]
    pub listing_payment_deposit: Box<Account<'info, TokenAccount>>,

    pub listing_payment_deposit_mint: Account<'info, Mint>,

    /// CHECK: cashier is stored in receipt, validated during cash instruction
    pub cashier: AccountInfo<'info>,

    #[account(mut)]
    pub listing_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub listing_mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"mint", listing_mint.key().as_ref()],
        bump = listing_mint_bump,
    )]
    pub listing_mint_authority: AccountInfo<'info>,

    #[account(
        init,
        seeds = [b"receipt" as &[u8], &receipt_nonce.to_le_bytes()],
        bump,
        payer = user,
        space = 8 + 1 + 1 + 1 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 16
    )]
    pub receipt: Box<Account<'info, Receipt>>,

    #[account(
        init,
        payer = user,
        token::mint = listing_payment_deposit_mint,
        token::authority = escrow_authority,
    )]
    pub escrow: Account<'info, TokenAccount>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"escrow", escrow.key().as_ref()],
        bump = escrow_authority_bump,
    )]
    pub escrow_authority: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(listing_mint_bump: u8, charter_mint_bump: u8, escrow_authority_bump: u8)]
pub struct Cash<'info> {
    pub cashier: Signer<'info>,

    #[account(mut, has_one = listing, has_one = listing_token_account, has_one = cashier, has_one = escrow)]
    pub receipt: Account<'info, Receipt>,

    #[account(mut)]
    pub escrow: Account<'info, TokenAccount>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"escrow", escrow.key().as_ref()],
        bump = escrow_authority_bump,
    )]
    pub escrow_authority: AccountInfo<'info>,

    #[account(mut)]
    pub listing_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub listings_payment_deposit: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub listings_vote_deposit: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = charter.key() == listing.clone().into_inner().charter.key(),
        constraint = listing_mint.key() == listing.clone().into_inner().mint.key(),
        constraint = listings_payment_deposit.key() == listing.clone().into_inner().payment_deposit.key(),
        constraint = listings_vote_deposit.key() == listing.clone().into_inner().vote_deposit.key(),
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(mut)]
    pub listing_mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"mint", listing_mint.key().as_ref()],
        bump = listing_mint_bump,
    )]
    pub listing_mint_authority: AccountInfo<'info>,

    #[account(
        has_one = charter,
        constraint = charter_treasury_deposit.key() == charter_treasury.clone().into_inner().deposit.key(),
        constraint = charter_treasury.mint == listings_payment_deposit.mint,
    )]
    pub charter_treasury: Box<Account<'info, CharterTreasury>>,

    #[account(mut)]
    pub charter_treasury_deposit: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub charter_vote_deposit: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub charter_mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"mint", charter_mint.key().as_ref()],
        bump = charter_mint_bump,
    )]
    pub charter_mint_authority: AccountInfo<'info>,

    #[account(
        constraint = charter.clone().into_inner().mint == charter_mint.key()
    )]
    pub charter: Box<Account<'info, Charter>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(listing_bump: u8, listing_mint_authority_bump: u8)]
pub struct Cancel<'info> {
    pub purchaser: Signer<'info>,

    #[account(mut)]
    pub return_deposit: Account<'info, TokenAccount>,

    #[account(mut, has_one = listing, has_one = listing_token_account, has_one = purchaser, has_one = escrow)]
    pub receipt: Account<'info, Receipt>,

    #[account(mut)]
    pub escrow: Account<'info, TokenAccount>,

    /// CHECK: PDA used as escrow authority
    pub escrow_authority: AccountInfo<'info>,

    #[account(mut)]
    pub listing_token_account: Box<Account<'info, TokenAccount>>,

    #[account(seeds = [b"listing", listing_mint.key().as_ref()], bump = listing_bump)]
    pub listing: Box<Account<'info, Listing>>,

    pub listing_mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"mint", listing_mint.key().as_ref()],
        bump = listing_mint_authority_bump,
    )]
    pub listing_mint_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(listing_bump: u8, listing_mint_authority_bump: u8)]
pub struct Consume<'info> {
    #[account(seeds = [b"listing", mint.key().as_ref()], bump = listing_bump, has_one = authority, has_one = mint)]
    pub listing: Box<Account<'info, Listing>>,

    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"mint", mint.key().as_ref()],
        bump = listing_mint_authority_bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub listing_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetReceiptCashable<'info> {
    #[account(has_one = authority)]
    pub listing: Box<Account<'info, Listing>>,

    #[account(mut, has_one = listing)]
    pub receipt: Account<'info, Receipt>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(mint_bump: u8, listing_bump: u8, listing_mint_decimals: u8)]
pub struct InitListing<'info> {
    #[account(
        init,
        seeds = [b"listing", mint.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 235 + 128
    )]
    pub listing: Box<Account<'info, Listing>>,

    /// CHECK: PDA seed validation via seeds constraint
    #[account(
        seeds = [b"mint", mint.key().as_ref()],
        bump = mint_bump,
    )]
    pub mint_authority_pda: AccountInfo<'info>,

    #[account(
        init,
        mint::decimals = listing_mint_decimals,
        mint::authority = mint_authority_pda,
        mint::freeze_authority = mint_authority_pda,
        payer = user
    )]
    pub mint: Box<Account<'info, Mint>>,

    pub payment_deposit: Box<Account<'info, TokenAccount>>,
    pub vote_deposit: Box<Account<'info, TokenAccount>>,

    pub charter: Box<Account<'info, Charter>>,

    #[account(has_one = charter)]
    pub charter_treasury: Box<Account<'info, CharterTreasury>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetListing<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetListingDeposit<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    pub payment_deposit: Account<'info, TokenAccount>,
    pub vote_deposit: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetListingAuthority<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    /// CHECK: new authority, no validation needed
    pub authority: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetListingCharter<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,

    pub charter: Account<'info, Charter>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(charter_bump: u8)]
pub struct InitCharter<'info> {
    #[account(
        init,
        seeds = [b"charter", mint.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 8 + 1 + 8 + 1 + 8 + 1 + 32 + 32 + 32 + 128 + 256
    )]
    pub charter: Account<'info, Charter>,

    pub mint: Account<'info, Mint>,

    /// CHECK: authority stored in charter, validated in set instructions
    pub authority: AccountInfo<'info>,

    pub vote_deposit: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCharter<'info> {
    #[account(mut)]
    pub charter: Account<'info, Charter>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCharterVoteDeposit<'info> {
    #[account(mut)]
    pub charter: Account<'info, Charter>,

    pub vote_deposit: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCharterAuthority<'info> {
    #[account(mut)]
    pub charter: Account<'info, Charter>,

    /// CHECK: new authority, no validation needed
    pub authority: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(treasury_bump: u8)]
pub struct InitCharterTreasury<'info> {
    #[account(
        init,
        seeds = [b"treasury", charter.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + 1 + 32 + 32 + 8 + 1 + 128
    )]
    pub treasury: Account<'info, CharterTreasury>,

    #[account(has_one = authority)]
    pub charter: Account<'info, Charter>,

    #[account(has_one = mint)]
    pub deposit: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCharterTreasuryExpansionScalar<'info> {
    #[account(mut, has_one = charter)]
    pub treasury: Account<'info, CharterTreasury>,

    #[account(has_one = authority)]
    pub charter: Account<'info, Charter>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetCharterTreasuryDeposit<'info> {
    #[account(mut, has_one = charter)]
    pub treasury: Account<'info, CharterTreasury>,

    #[account(has_one = authority)]
    pub charter: Account<'info, Charter>,

    #[account(has_one = mint)]
    pub deposit: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ─── State Accounts ─────────────────────────────────────────────────────────

#[account]
pub struct Receipt {
    pub is_initialized: bool,
    pub is_refundable: bool,
    pub is_cashable: bool,
    pub listing: Pubkey,
    pub listing_token_account: Pubkey,
    pub purchaser: Pubkey,
    pub cashier: Pubkey,
    pub escrow: Pubkey,
    pub quantity: u64,
    pub price: u64,
    pub nonce: u128,
}

#[account]
pub struct Listing {
    pub is_initialized: bool,
    pub is_available: bool,
    pub charter: Pubkey,
    pub authority: Pubkey,
    pub payment_deposit: Pubkey,
    pub vote_deposit: Pubkey,
    pub price: u64,
    pub mint: Pubkey,
    pub uri: String,
    pub is_refundable: bool,
    pub is_consumable: bool,
}

#[account]
pub struct Charter {
    pub is_initialized: bool,
    pub expansion_rate_amount: u64,
    pub expansion_rate_decimals: u8,
    pub payment_contribution_rate_amount: u64,
    pub payment_contribution_rate_decimals: u8,
    pub vote_contribution_rate_amount: u64,
    pub vote_contribution_rate_decimals: u8,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub vote_deposit: Pubkey,
    pub uri: String,
}

#[account]
pub struct CharterTreasury {
    pub is_initialized: bool,
    pub charter: Pubkey,
    pub deposit: Pubkey,
    pub mint: Pubkey,
    pub expansion_scalar_amount: u64,
    pub expansion_scalar_decimals: u8,
}

pub(crate) fn amount_as_float(amount: u64, decimals: u8) -> f64 {
    amount as f64 / i32::pow(10, decimals.into()) as f64
}

impl Charter {
    pub fn expansion_rate(&self, scalar_amount: u64, scalar_decimals: u8) -> f64 {
        amount_as_float(self.expansion_rate_amount, self.expansion_rate_decimals)
            * amount_as_float(scalar_amount, scalar_decimals)
    }
    pub fn payment_contribution_rate(&self) -> f64 {
        amount_as_float(
            self.payment_contribution_rate_amount,
            self.payment_contribution_rate_decimals,
        )
    }
    pub fn vote_contribution_rate(&self) -> f64 {
        amount_as_float(
            self.vote_contribution_rate_amount,
            self.vote_contribution_rate_decimals,
        )
    }
}

#[error_code]
pub enum StrangemoodError {
    #[msg("MintNotSupported")]
    MintNotSupported,

    #[msg("Unauthorized Charter")]
    UnauthorizedCharter,

    #[msg("Deposit not found in listing")]
    DepositIsNotFoundInListing,

    #[msg("Unexpected Listing Token Account")]
    UnexpectedListingTokenAccount,

    #[msg("Deposit not found in charter")]
    DepositIsNotFoundInCharter,

    #[msg("Mint is not found in charter")]
    MintIsNotFoundInCharter,

    #[msg("Provided Authority Account Does Not Have Access")]
    UnauthorizedAuthority,

    #[msg("Receipt is not currently cashable")]
    ReceiptNotCashable,

    #[msg("Only Cashable by the Cashier")]
    OnlyCashableByTheCashier,

    #[msg("Listing is Unavailable")]
    ListingUnavailable,

    #[msg("Mint did not match Listing")]
    UnexpectedListingMint,

    #[msg("Listing is not consumable")]
    ListingIsNotConsumable,
}
