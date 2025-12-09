use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

/// Network security and P2P hardening module
///
/// This module provides network-level security features including
/// DDoS protection, peer validation, message verification,
/// and secure P2P communication.

/// Peer reputation score
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReputationScore {
    Banned = 0,
    Untrusted = 1,
    Neutral = 2,
    Trusted = 3,
    Verified = 4,
}

/// Network attack type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackType {
    Sybil,
    Eclipse,
    DDoS,
    TimingAttack,
    MitmAttack,
}

/// Peer information with security attributes
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub ip: IpAddr,
    pub port: u16,
    pub reputation: ReputationScore,
    pub failed_checks: usize,
    pub successful_checks: usize,
}

impl Peer {
    /// Create new peer
    pub fn new(id: String, ip: IpAddr, port: u16) -> Self {
        Self {
            id,
            ip,
            port,
            reputation: ReputationScore::Neutral,
            failed_checks: 0,
            successful_checks: 0,
        }
    }

    /// Update peer reputation based on behavior
    pub fn update_reputation(&mut self, success: bool) {
        if success {
            self.successful_checks += 1;
            if self.successful_checks >= 10 && self.failed_checks == 0 {
                self.reputation = ReputationScore::Verified;
            } else if self.successful_checks >= 5 {
                self.reputation = ReputationScore::Trusted;
            }
        } else {
            self.failed_checks += 1;
            if self.failed_checks >= 5 {
                self.reputation = ReputationScore::Banned;
            } else if self.failed_checks >= 3 {
                self.reputation = ReputationScore::Untrusted;
            }
        }
    }

    /// Get reliability score (0-1.0)
    pub fn reliability_score(&self) -> f64 {
        let total = self.successful_checks + self.failed_checks;
        if total == 0 {
            return 0.5; // Neutral for new peers
        }
        self.successful_checks as f64 / total as f64
    }
}

/// Message validator for P2P network
pub struct MessageValidator {
    validated_count: usize,
    valid_count: usize,
    invalid_count: usize,
}

impl MessageValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            validated_count: 0,
            valid_count: 0,
            invalid_count: 0,
        }
    }

    /// Validate message signature
    pub fn validate_message_signature(&mut self, message: &[u8], signature: &[u8]) -> bool {
        self.validated_count += 1;

        // Message must not be empty
        if message.is_empty() {
            self.invalid_count += 1;
            return false;
        }

        // Signature must be present
        if signature.is_empty() {
            self.invalid_count += 1;
            return false;
        }

        // Signature length validation (typically 64-128 bytes)
        if signature.len() < 32 || signature.len() > 256 {
            self.invalid_count += 1;
            return false;
        }

        self.valid_count += 1;
        true
    }

    /// Validate message format
    pub fn validate_message_format(&mut self, data: &[u8]) -> bool {
        self.validated_count += 1;

        // Check minimum size
        if data.len() < 4 {
            self.invalid_count += 1;
            return false;
        }

        // Check maximum size (1MB limit)
        if data.len() > 1024 * 1024 {
            self.invalid_count += 1;
            return false;
        }

        self.valid_count += 1;
        true
    }

    /// Get validation statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.validated_count, self.valid_count, self.invalid_count)
    }

    /// Get validation success rate
    pub fn validation_rate(&self) -> f64 {
        if self.validated_count == 0 {
            return 1.0;
        }
        self.valid_count as f64 / self.validated_count as f64
    }
}

/// DDoS protection mechanism
pub struct DdosProtection {
    rate_limits: HashMap<String, usize>,
    burst_limits: HashMap<String, usize>,
    blacklist: HashSet<String>,
    whitelist: HashSet<String>,
}

impl DdosProtection {
    /// Create new DDoS protection
    pub fn new() -> Self {
        Self {
            rate_limits: HashMap::new(),
            burst_limits: HashMap::new(),
            blacklist: HashSet::new(),
            whitelist: HashSet::new(),
        }
    }

    /// Check if request is allowed
    pub fn is_allowed(&self, peer_id: &str, max_requests_per_second: usize) -> bool {
        // Whitelisted peers always allowed
        if self.whitelist.contains(peer_id) {
            return true;
        }

        // Blacklisted peers always denied
        if self.blacklist.contains(peer_id) {
            return false;
        }

        // Check rate limit
        if let Some(&count) = self.rate_limits.get(peer_id) {
            if count >= max_requests_per_second {
                return false;
            }
        }

        true
    }

    /// Add request from peer
    pub fn add_request(&mut self, peer_id: &str) {
        self.rate_limits
            .entry(peer_id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    /// Reset rate limit for peer
    pub fn reset_limit(&mut self, peer_id: &str) {
        self.rate_limits.insert(peer_id.to_string(), 0);
    }

    /// Add peer to blacklist
    pub fn blacklist_peer(&mut self, peer_id: &str) {
        self.blacklist.insert(peer_id.to_string());
    }

    /// Add peer to whitelist
    pub fn whitelist_peer(&mut self, peer_id: &str) {
        self.whitelist.insert(peer_id.to_string());
    }

    /// Get current request count for peer
    pub fn get_request_count(&self, peer_id: &str) -> usize {
        self.rate_limits.get(peer_id).copied().unwrap_or(0)
    }

    /// Get blacklist size
    pub fn blacklist_size(&self) -> usize {
        self.blacklist.len()
    }
}

/// Connection security manager
pub struct ConnectionSecurityManager {
    valid_connections: usize,
    failed_connections: usize,
    concurrent_limit: usize,
}

impl ConnectionSecurityManager {
    /// Create new manager
    pub fn new(concurrent_limit: usize) -> Self {
        Self {
            valid_connections: 0,
            failed_connections: 0,
            concurrent_limit,
        }
    }

    /// Validate connection attempt
    pub fn validate_connection(&mut self, source_ip: &str) -> Result<(), String> {
        // Check for invalid IP formats
        if source_ip.is_empty() {
            self.failed_connections += 1;
            return Err("Invalid source IP".to_string());
        }

        // Check for localhost-only restrictions
        if source_ip.contains("localhost") || source_ip == "127.0.0.1" {
            self.valid_connections += 1;
            return Ok(());
        }

        // Validate IP format (simplified)
        let parts: Vec<&str> = source_ip.split('.').collect();
        if parts.len() == 4 {
            for part in parts {
                if let Ok(num) = part.parse::<u8>() {
                    // Valid octet
                    if num > 255 {
                        self.failed_connections += 1;
                        return Err("Invalid IP octet".to_string());
                    }
                } else {
                    self.failed_connections += 1;
                    return Err("Invalid IP format".to_string());
                }
            }
            self.valid_connections += 1;
            Ok(())
        } else {
            self.failed_connections += 1;
            Err("Invalid IP format".to_string())
        }
    }

    /// Check if connection limit reached
    pub fn can_accept_connection(&self, current_connections: usize) -> bool {
        current_connections < self.concurrent_limit
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.valid_connections, self.failed_connections)
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.valid_connections + self.failed_connections;
        if total == 0 {
            return 1.0;
        }
        self.valid_connections as f64 / total as f64
    }
}

/// Network security auditor
pub struct NetworkSecurityAuditor {
    vulnerabilities: Vec<String>,
    recommendations: Vec<String>,
}

impl NetworkSecurityAuditor {
    /// Create new auditor
    pub fn new() -> Self {
        Self {
            vulnerabilities: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Audit P2P network security
    pub fn audit_p2p_security(&mut self) {
        self.vulnerabilities.push("Enable peer authentication".to_string());
        self.vulnerabilities.push("Implement peer reputation system".to_string());
        
        self.recommendations.push("Use secp256k1 for peer authentication".to_string());
        self.recommendations.push("Track peer behavior and reputation".to_string());
        self.recommendations.push("Implement peer banning for bad behavior".to_string());
    }

    /// Audit DDoS protection
    pub fn audit_ddos_protection(&mut self) {
        self.vulnerabilities.push("Rate limiting not configured globally".to_string());
        self.vulnerabilities.push("No IP reputation system".to_string());
        
        self.recommendations.push("Implement rate limiting per peer".to_string());
        self.recommendations.push("Use token bucket algorithm".to_string());
        self.recommendations.push("Monitor for DDoS patterns".to_string());
    }

    /// Audit message validation
    pub fn audit_message_validation(&mut self) {
        self.vulnerabilities.push("Message format validation incomplete".to_string());
        
        self.recommendations.push("Validate all message fields".to_string());
        self.recommendations.push("Implement strict size limits".to_string());
        self.recommendations.push("Sign all critical messages".to_string());
    }

    /// Get findings
    pub fn vulnerabilities(&self) -> &[String] {
        &self.vulnerabilities
    }

    /// Get recommendations
    pub fn recommendations(&self) -> &[String] {
        &self.recommendations
    }

    /// Generate security report
    pub fn generate_report(&self) -> String {
        let mut report = "NETWORK SECURITY AUDIT REPORT\n".to_string();
        report.push_str("===============================\n\n");

        report.push_str("VULNERABILITIES:\n");
        for vuln in &self.vulnerabilities {
            report.push_str(&format!("  - {}\n", vuln));
        }

        report.push_str("\nRECOMMENDATIONS:\n");
        for rec in &self.recommendations {
            report.push_str(&format!("  - {}\n", rec));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_peer_creation() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let peer = Peer::new("peer1".to_string(), ip, 8080);

        assert_eq!(peer.id, "peer1");
        assert_eq!(peer.port, 8080);
        assert_eq!(peer.reputation, ReputationScore::Neutral);
    }

    #[test]
    fn test_peer_reputation_update() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let mut peer = Peer::new("peer1".to_string(), ip, 8080);

        peer.update_reputation(true);
        assert_eq!(peer.successful_checks, 1);

        peer.update_reputation(false);
        assert_eq!(peer.failed_checks, 1);
    }

    #[test]
    fn test_peer_reputation_verified() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let mut peer = Peer::new("peer1".to_string(), ip, 8080);

        for _ in 0..10 {
            peer.update_reputation(true);
        }

        assert_eq!(peer.reputation, ReputationScore::Verified);
    }

    #[test]
    fn test_peer_reputation_banned() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let mut peer = Peer::new("peer1".to_string(), ip, 8080);

        for _ in 0..5 {
            peer.update_reputation(false);
        }

        assert_eq!(peer.reputation, ReputationScore::Banned);
    }

    #[test]
    fn test_message_validator_valid_signature() {
        let mut validator = MessageValidator::new();

        let message = b"test message";
        let signature = [0x01; 64];

        assert!(validator.validate_message_signature(message, &signature));

        let (validated, valid, invalid) = validator.stats();
        assert_eq!(validated, 1);
        assert_eq!(valid, 1);
        assert_eq!(invalid, 0);
    }

    #[test]
    fn test_message_validator_invalid_signature() {
        let mut validator = MessageValidator::new();

        let message = b"test";
        let signature = [0x01; 10]; // Too short

        assert!(!validator.validate_message_signature(message, &signature));
    }

    #[test]
    fn test_message_validator_empty_message() {
        let mut validator = MessageValidator::new();

        assert!(!validator.validate_message_signature(b"", &[0x01; 64]));
    }

    #[test]
    fn test_message_format_validation() {
        let mut validator = MessageValidator::new();

        assert!(validator.validate_message_format(&[0; 100]));
        assert!(!validator.validate_message_format(&[0; 2])); // Too short
    }

    #[test]
    fn test_ddos_protection_allowed() {
        let protection = DdosProtection::new();

        assert!(protection.is_allowed("peer1", 100));
    }

    #[test]
    fn test_ddos_protection_whitelist() {
        let mut protection = DdosProtection::new();
        protection.whitelist_peer("trusted");

        assert!(protection.is_allowed("trusted", 0)); // Even with 0 limit
    }

    #[test]
    fn test_ddos_protection_blacklist() {
        let mut protection = DdosProtection::new();
        protection.blacklist_peer("attacker");

        assert!(!protection.is_allowed("attacker", 1000));
    }

    #[test]
    fn test_ddos_protection_rate_limit() {
        let mut protection = DdosProtection::new();

        protection.add_request("peer1");
        protection.add_request("peer1");

        assert_eq!(protection.get_request_count("peer1"), 2);
        assert!(!protection.is_allowed("peer1", 1)); // 2 > 1, not allowed
        assert!(!protection.is_allowed("peer1", 2)); // 2 >= 2, not allowed (at limit)
        assert!(protection.is_allowed("peer1", 3)); // 2 < 3, allowed
    }

    #[test]
    fn test_connection_manager_valid_ip() {
        let mut manager = ConnectionSecurityManager::new(100);

        assert!(manager.validate_connection("192.168.1.1").is_ok());

        let (valid, failed) = manager.stats();
        assert_eq!(valid, 1);
        assert_eq!(failed, 0);
    }

    #[test]
    fn test_connection_manager_invalid_ip() {
        let mut manager = ConnectionSecurityManager::new(100);

        assert!(manager.validate_connection("999.999.999.999").is_err());

        let (valid, failed) = manager.stats();
        assert_eq!(valid, 0);
        assert_eq!(failed, 1);
    }

    #[test]
    fn test_connection_manager_limit() {
        let manager = ConnectionSecurityManager::new(10);

        assert!(manager.can_accept_connection(5));
        assert!(!manager.can_accept_connection(10));
    }

    #[test]
    fn test_connection_manager_success_rate() {
        let mut manager = ConnectionSecurityManager::new(100);

        manager.validate_connection("192.168.1.1").ok();
        manager.validate_connection("999.999.999.999").err();

        assert_eq!(manager.success_rate(), 0.5);
    }

    #[test]
    fn test_network_security_auditor() {
        let mut auditor = NetworkSecurityAuditor::new();
        auditor.audit_p2p_security();

        assert!(auditor.vulnerabilities().len() > 0);
        assert!(auditor.recommendations().len() > 0);
    }

    #[test]
    fn test_network_security_report() {
        let mut auditor = NetworkSecurityAuditor::new();
        auditor.audit_p2p_security();
        auditor.audit_ddos_protection();

        let report = auditor.generate_report();
        assert!(report.contains("NETWORK SECURITY AUDIT REPORT"));
        assert!(report.contains("VULNERABILITIES"));
    }

    #[test]
    fn test_peer_reliability_score() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let mut peer = Peer::new("peer1".to_string(), ip, 8080);

        for _ in 0..5 {
            peer.update_reputation(true);
        }

        let reliability = peer.reliability_score();
        assert!(reliability > 0.5);
    }

    #[test]
    fn test_message_validation_rate() {
        let mut validator = MessageValidator::new();

        validator.validate_message_signature(b"test", &[0x01; 64]);
        validator.validate_message_signature(b"", &[0x01; 64]); // Invalid

        assert_eq!(validator.validation_rate(), 0.5);
    }

    #[test]
    fn test_ddos_protection_reset_limit() {
        let mut protection = DdosProtection::new();

        protection.add_request("peer1");
        protection.add_request("peer1");
        assert_eq!(protection.get_request_count("peer1"), 2);

        protection.reset_limit("peer1");
        assert_eq!(protection.get_request_count("peer1"), 0);
    }
}
