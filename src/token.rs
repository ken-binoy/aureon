use crate::state::State;

pub const MAX_SUPPLY: u64 = 369_000_000;

pub fn mint_initial_supply(state: &mut State, validators: &[String], amount_per: u64) {
    for addr in validators {
        if state.total_supply + amount_per <= MAX_SUPPLY {
            state.mint(addr, amount_per);
        }
    }
}