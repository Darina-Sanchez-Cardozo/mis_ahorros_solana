use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::solana_program::clock::Clock;

declare_id!("CR7XqeU2TqY4p8Px9jgM3QkgE254i7Prd6x4HWT4gTR7"); // Lo obtendrás al hacer 'anchor build'


#[program]
pub mod safe_deposit {
    use super::*;

    // Función para crear la bóveda y depositar
    pub fn initialize_vault(ctx: Context<Initialize>, amount: u64, duration: i64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?; // Obtenemos el tiempo real de la blockchain

        vault.owner = *ctx.accounts.user.key;
        vault.amount = amount;
        vault.unlock_time = clock.unix_timestamp + duration; // Tiempo actual + segundos elegidos

        // Transferencia de SOL del usuario al PDA (Bóveda)
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault.to_account_info(),
            ],
        )?;

        msg!("Bóveda creada. Desbloqueo en: {} segundos", vault.unlock_time);
        Ok(())
    }

    // Función para retirar
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let clock = Clock::get()?;

        // Verificación de seguridad: ¿Ya pasó el tiempo?
        require!(
            clock.unix_timestamp >= vault.unlock_time,
            ErrorCode::StillLocked
        );

        // Si pasa la validación, cerramos la cuenta y enviamos los fondos al dueño
        // Anchor hace esto automáticamente al usar 'close = user' en el contexto
        msg!("¡Tiempo cumplido! Fondos liberados.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32 + 8 + 8, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = owner, close = owner, seeds = [b"vault", owner.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub amount: u64,
    pub unlock_time: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("El tiempo de bloqueo aún no ha terminado.")]
    StillLocked,
}