use std::collections::HashMap;

/// Community governance and voting system
///
/// This module implements on-chain governance with proposals,
/// voting mechanisms, and community participation.

/// Proposal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProposalType {
    ParameterChange,
    ProtocolUpgrade,
    FundAllocation,
    CommunitySplit,
    EmergencyPause,
}

/// Vote choice
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

/// Proposal status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled,
}

/// A governance proposal
#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub status: ProposalStatus,
    pub start_block: u64,
    pub end_block: u64,
    pub created_at: u64,
}

impl Proposal {
    /// Create new proposal
    pub fn new(
        id: u64,
        proposal_type: ProposalType,
        title: String,
        description: String,
        proposer: String,
        start_block: u64,
        end_block: u64,
    ) -> Self {
        Self {
            id,
            proposal_type,
            title,
            description,
            proposer,
            status: ProposalStatus::Pending,
            start_block,
            end_block,
            created_at: 0,
        }
    }

    /// Activate proposal
    pub fn activate(&mut self) {
        if self.status == ProposalStatus::Pending {
            self.status = ProposalStatus::Active;
        }
    }

    /// Mark as passed
    pub fn mark_passed(&mut self) {
        if self.status == ProposalStatus::Active {
            self.status = ProposalStatus::Passed;
        }
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        if self.status == ProposalStatus::Active {
            self.status = ProposalStatus::Failed;
        }
    }

    /// Execute proposal
    pub fn execute(&mut self) -> Result<(), String> {
        if self.status == ProposalStatus::Passed {
            self.status = ProposalStatus::Executed;
            Ok(())
        } else {
            Err("Can only execute passed proposals".to_string())
        }
    }

    /// Cancel proposal
    pub fn cancel(&mut self) {
        self.status = ProposalStatus::Cancelled;
    }
}

/// Vote record
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: u64,
    pub choice: VoteChoice,
    pub weight: u64, // Voting power
    pub timestamp: u64,
}

/// Voting system manager
pub struct VotingSystem {
    proposals: HashMap<u64, Proposal>,
    votes: Vec<Vote>,
    next_proposal_id: u64,
    voting_period: u64,
    quorum_percentage: u32,
}

impl VotingSystem {
    /// Create new voting system
    pub fn new(voting_period: u64, quorum_percentage: u32) -> Self {
        Self {
            proposals: HashMap::new(),
            votes: Vec::new(),
            next_proposal_id: 1,
            voting_period,
            quorum_percentage,
        }
    }

    /// Submit new proposal
    pub fn submit_proposal(
        &mut self,
        proposal_type: ProposalType,
        title: String,
        description: String,
        proposer: String,
        start_block: u64,
    ) -> u64 {
        let proposal = Proposal::new(
            self.next_proposal_id,
            proposal_type,
            title,
            description,
            proposer,
            start_block,
            start_block + self.voting_period,
        );

        let id = self.next_proposal_id;
        self.proposals.insert(id, proposal);
        self.next_proposal_id += 1;
        id
    }

    /// Cast vote
    pub fn cast_vote(
        &mut self,
        voter: String,
        proposal_id: u64,
        choice: VoteChoice,
        weight: u64,
    ) -> Result<(), String> {
        if !self.proposals.contains_key(&proposal_id) {
            return Err("Proposal not found".to_string());
        }

        let proposal = &self.proposals[&proposal_id];
        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".to_string());
        }

        // Check if voter already voted
        if self.votes.iter().any(|v| v.voter == voter && v.proposal_id == proposal_id) {
            return Err("Voter has already voted".to_string());
        }

        let vote = Vote {
            voter,
            proposal_id,
            choice,
            weight,
            timestamp: 0,
        };

        self.votes.push(vote);
        Ok(())
    }

    /// Get proposal
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }

    /// Get vote count for proposal
    pub fn get_vote_count(&self, proposal_id: u64) -> (u64, u64, u64) {
        let mut yes = 0;
        let mut no = 0;
        let mut abstain = 0;

        for vote in &self.votes {
            if vote.proposal_id == proposal_id {
                match vote.choice {
                    VoteChoice::Yes => yes += vote.weight,
                    VoteChoice::No => no += vote.weight,
                    VoteChoice::Abstain => abstain += vote.weight,
                }
            }
        }

        (yes, no, abstain)
    }

    /// Get total voting power used for proposal
    pub fn get_total_votes(&self, proposal_id: u64) -> u64 {
        let (yes, no, abstain) = self.get_vote_count(proposal_id);
        yes + no + abstain
    }

    /// Calculate approval percentage
    pub fn calculate_approval(&self, proposal_id: u64) -> f64 {
        let (yes, no, _abstain) = self.get_vote_count(proposal_id);
        let total = yes + no;

        if total == 0 {
            return 0.0;
        }

        yes as f64 / total as f64
    }

    /// Check if proposal has quorum
    pub fn has_quorum(&self, proposal_id: u64, total_voting_power: u64) -> bool {
        if total_voting_power == 0 {
            return false;
        }

        let votes = self.get_total_votes(proposal_id);
        let percentage = (votes as f64 / total_voting_power as f64) * 100.0;
        percentage >= self.quorum_percentage as f64
    }

    /// Finalize proposal
    pub fn finalize_proposal(&mut self, proposal_id: u64, total_voting_power: u64) -> Result<(), String> {
        // Get status check without mutable borrow
        {
            let proposal = self
                .proposals
                .get(&proposal_id)
                .ok_or("Proposal not found")?;

            if proposal.status != ProposalStatus::Active {
                return Err("Proposal is not active".to_string());
            }
        }

        // Calculate metrics without holding mutable borrow
        let approval = self.calculate_approval(proposal_id);
        let has_quorum = self.has_quorum(proposal_id, total_voting_power);

        // Now get mutable reference to update
        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        if has_quorum && approval > 0.5 {
            proposal.mark_passed();
        } else {
            proposal.mark_failed();
        }

        Ok(())
    }

    /// Execute proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or("Proposal not found")?;

        proposal.execute()
    }

    /// Get all proposals
    pub fn all_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    /// Get active proposals count
    pub fn active_proposals_count(&self) -> usize {
        self.proposals.values().filter(|p| p.status == ProposalStatus::Active).count()
    }

    /// Get total proposals count
    pub fn total_proposals(&self) -> usize {
        self.proposals.len()
    }
}

/// Community participation tracker
pub struct CommunityParticipation {
    user_voting_power: HashMap<String, u64>,
    participation_record: Vec<ParticipationRecord>,
}

#[derive(Debug, Clone)]
pub struct ParticipationRecord {
    pub user: String,
    pub action: String,
    pub timestamp: u64,
}

impl CommunityParticipation {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            user_voting_power: HashMap::new(),
            participation_record: Vec::new(),
        }
    }

    /// Add user with voting power
    pub fn add_user(&mut self, user: String, power: u64) {
        self.user_voting_power.insert(user, power);
    }

    /// Get user voting power
    pub fn get_voting_power(&self, user: &str) -> u64 {
        self.user_voting_power.get(user).copied().unwrap_or(0)
    }

    /// Record participation
    pub fn record_participation(&mut self, user: String, action: String) {
        self.participation_record.push(ParticipationRecord {
            user,
            action,
            timestamp: 0,
        });
    }

    /// Get participation score for user
    pub fn get_participation_score(&self, user: &str) -> usize {
        self.participation_record.iter().filter(|r| r.user == user).count()
    }

    /// Get total participants
    pub fn total_participants(&self) -> usize {
        self.user_voting_power.len()
    }

    /// Get total voting power
    pub fn total_voting_power(&self) -> u64 {
        self.user_voting_power.values().sum()
    }

    /// Get participation records
    pub fn participation_records(&self) -> &[ParticipationRecord] {
        &self.participation_record
    }
}

/// Governance configuration
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    pub voting_period: u64,
    pub quorum_percentage: u32,
    pub execution_delay: u64,
    pub proposal_threshold: u64,
}

impl GovernanceConfig {
    /// Create default config
    pub fn default() -> Self {
        Self {
            voting_period: 100,      // blocks
            quorum_percentage: 40,   // 40%
            execution_delay: 10,     // blocks
            proposal_threshold: 100, // minimum voting power
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Description".to_string(),
            "proposer".to_string(),
            0,
            100,
        );

        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.status, ProposalStatus::Pending);
    }

    #[test]
    fn test_proposal_activation() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
            100,
        );

        proposal.activate();
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_proposal_execution() {
        let mut proposal = Proposal::new(
            1,
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
            100,
        );

        assert!(proposal.execute().is_err()); // Not passed yet

        proposal.status = ProposalStatus::Passed;
        assert!(proposal.execute().is_ok());
        assert_eq!(proposal.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_voting_system_creation() {
        let system = VotingSystem::new(100, 40);
        assert_eq!(system.voting_period, 100);
        assert_eq!(system.quorum_percentage, 40);
    }

    #[test]
    fn test_submit_proposal() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        assert_eq!(id, 1);
        assert!(system.get_proposal(1).is_some());
    }

    #[test]
    fn test_cast_vote() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        let proposal = system.proposals.get_mut(&id).unwrap();
        proposal.activate();

        let result = system.cast_vote("voter1".to_string(), id, VoteChoice::Yes, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_vote() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id).unwrap().activate();

        system.cast_vote("voter1".to_string(), id, VoteChoice::Yes, 100).ok();
        let result = system.cast_vote("voter1".to_string(), id, VoteChoice::No, 100);

        assert!(result.is_err());
    }

    #[test]
    fn test_vote_count() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id).unwrap().activate();

        system.cast_vote("voter1".to_string(), id, VoteChoice::Yes, 60).ok();
        system.cast_vote("voter2".to_string(), id, VoteChoice::No, 40).ok();

        let (yes, no, abstain) = system.get_vote_count(id);
        assert_eq!(yes, 60);
        assert_eq!(no, 40);
        assert_eq!(abstain, 0);
    }

    #[test]
    fn test_approval_calculation() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id).unwrap().activate();

        system.cast_vote("voter1".to_string(), id, VoteChoice::Yes, 60).ok();
        system.cast_vote("voter2".to_string(), id, VoteChoice::No, 40).ok();

        let approval = system.calculate_approval(id);
        assert!(approval > 0.5);
    }

    #[test]
    fn test_quorum_check() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id).unwrap().activate();

        system.cast_vote("voter1".to_string(), id, VoteChoice::Yes, 50).ok();

        assert!(system.has_quorum(id, 100)); // 50% > 40%
        assert!(!system.has_quorum(id, 200)); // 25% < 40%
    }

    #[test]
    fn test_finalize_proposal_passed() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id).unwrap().activate();

        system.cast_vote("voter1".to_string(), id, VoteChoice::Yes, 60).ok();
        system.cast_vote("voter2".to_string(), id, VoteChoice::No, 30).ok();

        system.finalize_proposal(id, 100).ok();

        let proposal = system.get_proposal(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }

    #[test]
    fn test_community_participation() {
        let mut community = CommunityParticipation::new();

        community.add_user("user1".to_string(), 100);
        community.add_user("user2".to_string(), 200);

        assert_eq!(community.get_voting_power("user1"), 100);
        assert_eq!(community.get_voting_power("user2"), 200);
        assert_eq!(community.total_voting_power(), 300);
    }

    #[test]
    fn test_participation_record() {
        let mut community = CommunityParticipation::new();
        community.add_user("user1".to_string(), 100);

        community.record_participation("user1".to_string(), "voted".to_string());
        community.record_participation("user1".to_string(), "proposed".to_string());

        assert_eq!(community.get_participation_score("user1"), 2);
    }

    #[test]
    fn test_governance_config() {
        let config = GovernanceConfig::default();
        assert_eq!(config.voting_period, 100);
        assert_eq!(config.quorum_percentage, 40);
    }

    #[test]
    fn test_active_proposals_count() {
        let mut system = VotingSystem::new(100, 40);

        system.submit_proposal(
            ProposalType::ParameterChange,
            "Test1".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        let id2 = system.submit_proposal(
            ProposalType::ProtocolUpgrade,
            "Test2".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id2).unwrap().activate();

        assert_eq!(system.active_proposals_count(), 1);
    }

    #[test]
    fn test_multiple_votes_weighted() {
        let mut system = VotingSystem::new(100, 40);

        let id = system.submit_proposal(
            ProposalType::ParameterChange,
            "Test".to_string(),
            "Desc".to_string(),
            "proposer".to_string(),
            0,
        );

        system.proposals.get_mut(&id).unwrap().activate();

        for i in 0..10 {
            system.cast_vote(format!("voter{}", i), id, VoteChoice::Yes, 100).ok();
        }

        let total = system.get_total_votes(id);
        assert_eq!(total, 1000);
    }
}
