use std::collections::HashMap;

/// Incentive programs and economic sustainability module
///
/// This module implements reward distribution, staking incentives,
/// and economic sustainability mechanisms.

/// Reward type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RewardType {
    BlockReward,
    StakingReward,
    GovernanceReward,
    DevelopmentGrant,
    BugBounty,
}

/// Staking info
#[derive(Debug, Clone)]
pub struct StakingInfo {
    pub staker: String,
    pub amount: u128,
    pub start_block: u64,
    pub lock_period: u64,
    pub is_active: bool,
}

impl StakingInfo {
    /// Create new staking position
    pub fn new(staker: String, amount: u128, start_block: u64, lock_period: u64) -> Self {
        Self {
            staker,
            amount,
            start_block,
            lock_period,
            is_active: true,
        }
    }

    /// Check if stake is locked
    pub fn is_locked(&self, current_block: u64) -> bool {
        current_block < self.start_block + self.lock_period
    }

    /// Unlock stake
    pub fn unlock(&mut self) {
        self.is_active = false;
    }

    /// Get stake age in blocks
    pub fn get_age(&self, current_block: u64) -> u64 {
        if current_block > self.start_block {
            current_block - self.start_block
        } else {
            0
        }
    }
}

/// Reward distribution engine
pub struct RewardDistributor {
    pending_rewards: HashMap<String, u128>,
    distributed_rewards: HashMap<String, u128>,
    reward_pool: u128,
}

impl RewardDistributor {
    /// Create new distributor
    pub fn new(initial_pool: u128) -> Self {
        Self {
            pending_rewards: HashMap::new(),
            distributed_rewards: HashMap::new(),
            reward_pool: initial_pool,
        }
    }

    /// Add to reward pool
    pub fn add_to_pool(&mut self, amount: u128) {
        self.reward_pool += amount;
    }

    /// Queue reward
    pub fn queue_reward(&mut self, recipient: String, amount: u128) -> Result<(), String> {
        if amount > self.reward_pool {
            return Err("Insufficient reward pool".to_string());
        }

        self.pending_rewards
            .entry(recipient)
            .and_modify(|e| *e += amount)
            .or_insert(amount);

        Ok(())
    }

    /// Distribute reward
    pub fn distribute_reward(&mut self, recipient: &str) -> Result<u128, String> {
        let amount = self
            .pending_rewards
            .remove(recipient)
            .ok_or("No pending reward".to_string())?;

        if amount > self.reward_pool {
            return Err("Insufficient pool for distribution".to_string());
        }

        self.reward_pool -= amount;
        self.distributed_rewards
            .entry(recipient.to_string())
            .and_modify(|e| *e += amount)
            .or_insert(amount);

        Ok(amount)
    }

    /// Get pending reward
    pub fn get_pending_reward(&self, recipient: &str) -> u128 {
        self.pending_rewards.get(recipient).copied().unwrap_or(0)
    }

    /// Get total distributed
    pub fn get_total_distributed(&self, recipient: &str) -> u128 {
        self.distributed_rewards.get(recipient).copied().unwrap_or(0)
    }

    /// Get reward pool
    pub fn get_reward_pool(&self) -> u128 {
        self.reward_pool
    }

    /// Total pending rewards
    pub fn total_pending(&self) -> u128 {
        self.pending_rewards.values().sum()
    }

    /// Recipients count
    pub fn recipients_count(&self) -> usize {
        self.distributed_rewards.len()
    }
}

/// Staking system
pub struct StakingSystem {
    stakes: HashMap<String, Vec<StakingInfo>>,
    total_staked: u128,
    annual_reward_rate: f64, // APY as decimal (0.05 = 5%)
}

impl StakingSystem {
    /// Create new staking system
    pub fn new(annual_reward_rate: f64) -> Self {
        Self {
            stakes: HashMap::new(),
            total_staked: 0,
            annual_reward_rate,
        }
    }

    /// Stake tokens
    pub fn stake(&mut self, staker: String, amount: u128, lock_period: u64, current_block: u64) {
        let stake = StakingInfo::new(staker.clone(), amount, current_block, lock_period);

        self.stakes.entry(staker).or_insert_with(Vec::new).push(stake);
        self.total_staked += amount;
    }

    /// Get staked amount for user
    pub fn get_staked_amount(&self, staker: &str) -> u128 {
        self.stakes
            .get(staker)
            .map(|stakes| stakes.iter().filter(|s| s.is_active).map(|s| s.amount).sum())
            .unwrap_or(0)
    }

    /// Calculate reward for stake
    pub fn calculate_reward(&self, amount: u128, blocks: u64) -> u128 {
        let blocks_per_year = 2_102_400u128; // ~365 days at 15 second blocks
        let reward = (amount as f64 * self.annual_reward_rate * blocks as f64 / blocks_per_year as f64)
            as u128;
        reward
    }

    /// Get total staked
    pub fn get_total_staked(&self) -> u128 {
        self.total_staked
    }

    /// Get stake count
    pub fn get_stake_count(&self) -> usize {
        self.stakes.values().map(|v| v.len()).sum()
    }

    /// Get active validators (stakers with active stakes)
    pub fn get_active_validators(&self) -> usize {
        self.stakes
            .values()
            .filter(|stakes| stakes.iter().any(|s| s.is_active))
            .count()
    }
}

/// Incentive program tracker
pub struct IncentiveProgram {
    name: String,
    description: String,
    budget: u128,
    distributed: u128,
    participants: HashMap<String, ParticipantInfo>,
}

#[derive(Debug, Clone)]
pub struct ParticipantInfo {
    pub address: String,
    pub earned: u128,
    pub contribution_score: f64,
}

impl IncentiveProgram {
    /// Create new program
    pub fn new(name: String, description: String, budget: u128) -> Self {
        Self {
            name,
            description,
            budget,
            distributed: 0,
            participants: HashMap::new(),
        }
    }

    /// Add participant
    pub fn add_participant(&mut self, address: String, contribution_score: f64) {
        self.participants.insert(
            address.clone(),
            ParticipantInfo {
                address,
                earned: 0,
                contribution_score,
            },
        );
    }

    /// Award participant
    pub fn award_participant(&mut self, address: &str, amount: u128) -> Result<(), String> {
        if self.distributed + amount > self.budget {
            return Err("Budget exceeded".to_string());
        }

        if let Some(participant) = self.participants.get_mut(address) {
            participant.earned += amount;
            self.distributed += amount;
            Ok(())
        } else {
            Err("Participant not found".to_string())
        }
    }

    /// Get remaining budget
    pub fn remaining_budget(&self) -> u128 {
        self.budget - self.distributed
    }

    /// Get participant count
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Get total earned by participants
    pub fn total_earned(&self) -> u128 {
        self.participants.values().map(|p| p.earned).sum()
    }

    /// Distribution percentage
    pub fn distribution_percentage(&self) -> f64 {
        if self.budget == 0 {
            return 0.0;
        }
        self.distributed as f64 / self.budget as f64 * 100.0
    }
}

/// Economic sustainability checker
pub struct EconomicSustainability {
    metrics: Vec<(String, f64)>,
}

impl EconomicSustainability {
    /// Create new checker
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    /// Add metric
    pub fn add_metric(&mut self, name: String, value: f64) {
        self.metrics.push((name, value));
    }

    /// Check inflation rate sustainability
    pub fn check_inflation_sustainability(&mut self, inflation_rate: f64) {
        let sustainable = inflation_rate <= 0.10; // Max 10% annually
        self.add_metric("inflation_sustainable".to_string(), if sustainable { 1.0 } else { 0.0 });
    }

    /// Check reward sustainability
    pub fn check_reward_sustainability(&mut self, rewards_per_block: u128, tx_fees: u128) {
        let sustainable = rewards_per_block <= 1000 && tx_fees >= 100;
        self.add_metric("reward_sustainable".to_string(), if sustainable { 1.0 } else { 0.0 });
    }

    /// Check validator participation
    pub fn check_validator_participation(&mut self, active_validators: usize, required_validators: usize) {
        let percentage = active_validators as f64 / required_validators as f64;
        let sustainable = percentage >= 0.67; // 2/3 for finality
        self.add_metric("validator_participation".to_string(), if sustainable { 1.0 } else { 0.0 });
    }

    /// Get sustainability score (0-1.0)
    pub fn sustainability_score(&self) -> f64 {
        if self.metrics.is_empty() {
            return 1.0;
        }

        self.metrics.iter().map(|(_, v)| v).sum::<f64>() / self.metrics.len() as f64
    }

    /// Is economically sustainable
    pub fn is_sustainable(&self) -> bool {
        self.sustainability_score() >= 0.67
    }

    /// Get all metrics
    pub fn all_metrics(&self) -> &[(String, f64)] {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_info_creation() {
        let stake = StakingInfo::new("user1".to_string(), 1000, 0, 100);
        assert_eq!(stake.staker, "user1");
        assert_eq!(stake.amount, 1000);
    }

    #[test]
    fn test_staking_info_locked() {
        let stake = StakingInfo::new("user1".to_string(), 1000, 0, 100);
        assert!(stake.is_locked(50)); // Block 50 < 0 + 100
        assert!(!stake.is_locked(150)); // Block 150 > 0 + 100
    }

    #[test]
    fn test_staking_info_age() {
        let stake = StakingInfo::new("user1".to_string(), 1000, 0, 100);
        assert_eq!(stake.get_age(50), 50);
        assert_eq!(stake.get_age(150), 150);
    }

    #[test]
    fn test_reward_distributor_creation() {
        let distributor = RewardDistributor::new(1_000_000);
        assert_eq!(distributor.get_reward_pool(), 1_000_000);
    }

    #[test]
    fn test_reward_distributor_add_pool() {
        let mut distributor = RewardDistributor::new(1_000_000);
        distributor.add_to_pool(500_000);

        assert_eq!(distributor.get_reward_pool(), 1_500_000);
    }

    #[test]
    fn test_reward_queue_and_distribute() {
        let mut distributor = RewardDistributor::new(1_000_000);
        distributor.queue_reward("user1".to_string(), 100_000).ok();

        let distributed = distributor.distribute_reward("user1");
        assert!(distributed.is_ok());
        assert_eq!(distributed.unwrap(), 100_000);
    }

    #[test]
    fn test_reward_insufficient_pool() {
        let mut distributor = RewardDistributor::new(100_000);
        let result = distributor.queue_reward("user1".to_string(), 200_000);

        assert!(result.is_err());
    }

    #[test]
    fn test_staking_system_creation() {
        let system = StakingSystem::new(0.05); // 5% APY
        assert_eq!(system.get_total_staked(), 0);
    }

    #[test]
    fn test_staking_system_stake() {
        let mut system = StakingSystem::new(0.05);
        system.stake("user1".to_string(), 1000, 100, 0);

        assert_eq!(system.get_staked_amount("user1"), 1000);
        assert_eq!(system.get_total_staked(), 1000);
    }

    #[test]
    fn test_staking_system_reward_calculation() {
        let system = StakingSystem::new(0.05); // 5% APY

        // One year worth of blocks
        let reward = system.calculate_reward(1000, 2_102_400);
        assert_eq!(reward, 50); // 5% of 1000
    }

    #[test]
    fn test_staking_system_multiple_stakes() {
        let mut system = StakingSystem::new(0.05);
        system.stake("user1".to_string(), 1000, 100, 0);
        system.stake("user1".to_string(), 2000, 100, 10);

        assert_eq!(system.get_total_staked(), 3000);
        assert_eq!(system.get_stake_count(), 2);
    }

    #[test]
    fn test_incentive_program_creation() {
        let program = IncentiveProgram::new(
            "Development".to_string(),
            "Developer grants".to_string(),
            1_000_000,
        );

        assert_eq!(program.budget, 1_000_000);
        assert_eq!(program.distributed, 0);
    }

    #[test]
    fn test_incentive_program_add_participant() {
        let mut program = IncentiveProgram::new(
            "Development".to_string(),
            "Developer grants".to_string(),
            1_000_000,
        );

        program.add_participant("dev1".to_string(), 0.8);
        assert_eq!(program.participant_count(), 1);
    }

    #[test]
    fn test_incentive_program_award() {
        let mut program = IncentiveProgram::new(
            "Development".to_string(),
            "Developer grants".to_string(),
            1_000_000,
        );

        program.add_participant("dev1".to_string(), 0.8);
        let result = program.award_participant("dev1", 50_000);

        assert!(result.is_ok());
        assert_eq!(program.distributed, 50_000);
    }

    #[test]
    fn test_incentive_program_budget_exceeded() {
        let mut program = IncentiveProgram::new(
            "Development".to_string(),
            "Developer grants".to_string(),
            100_000,
        );

        program.add_participant("dev1".to_string(), 0.8);
        let result = program.award_participant("dev1", 150_000);

        assert!(result.is_err());
    }

    #[test]
    fn test_incentive_program_distribution_percentage() {
        let mut program = IncentiveProgram::new(
            "Development".to_string(),
            "Developer grants".to_string(),
            1_000_000,
        );

        program.add_participant("dev1".to_string(), 0.8);
        program.award_participant("dev1", 500_000).ok();

        assert_eq!(program.distribution_percentage(), 50.0);
    }

    #[test]
    fn test_economic_sustainability_score() {
        let mut checker = EconomicSustainability::new();
        checker.check_inflation_sustainability(0.05);
        checker.check_reward_sustainability(500, 200);
        checker.check_validator_participation(70, 100);

        assert!(checker.is_sustainable());
        assert!(checker.sustainability_score() >= 0.67);
    }

    #[test]
    fn test_economic_sustainability_unsustainable() {
        let mut checker = EconomicSustainability::new();
        checker.check_inflation_sustainability(0.20); // Too high
        checker.check_reward_sustainability(5000, 10); // Bad rewards
        checker.check_validator_participation(30, 100); // Too few validators

        assert!(!checker.is_sustainable());
    }

    #[test]
    fn test_staking_unlock() {
        let mut stake = StakingInfo::new("user1".to_string(), 1000, 0, 100);
        assert!(stake.is_active);

        stake.unlock();
        assert!(!stake.is_active);
    }

    #[test]
    fn test_reward_distributor_pending() {
        let mut distributor = RewardDistributor::new(1_000_000);
        distributor.queue_reward("user1".to_string(), 100_000).ok();
        distributor.queue_reward("user2".to_string(), 50_000).ok();

        assert_eq!(distributor.total_pending(), 150_000);
    }

    #[test]
    fn test_staking_system_active_validators() {
        let mut system = StakingSystem::new(0.05);
        system.stake("val1".to_string(), 1000, 100, 0);
        system.stake("val2".to_string(), 1000, 100, 0);

        assert_eq!(system.get_active_validators(), 2);
    }
}
