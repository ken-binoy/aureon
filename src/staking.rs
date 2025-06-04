use crate::state::State;

pub fn calculate_reward(block_height: u64) -> u64 {
    // Base reward = 100, halves every ~500,000 blocks (adjust as needed)
    let mut reward = 100;
    let mut height = block_height;
    while height >= 500_000 {
        reward = reward / 2;
        height -= 500_000;
    }
    reward
}

pub fn apply_reward(state: &mut State, validator: &str, block_height: u64) {
    let reward = calculate_reward(block_height);
    if state.total_supply + reward <= crate::token::MAX_SUPPLY {
        state.mint(validator, reward);
    }
}