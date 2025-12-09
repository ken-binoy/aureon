use std::collections::HashMap;

/// Security assessment and vulnerability detection module
/// 
/// This module provides security testing, vulnerability scanning,
/// and security audit capabilities for the Aureon blockchain.

/// Vulnerability severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Vulnerability information
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub component: String,
    pub remediation: String,
    pub cve_id: Option<String>,
}

/// Security assessment results
pub struct SecurityAssessment {
    pub component: String,
    pub timestamp: u64,
    pub vulnerabilities: Vec<Vulnerability>,
    pub risk_score: f64,
}

impl SecurityAssessment {
    /// Create new assessment
    pub fn new(component: String) -> Self {
        Self {
            component,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            vulnerabilities: Vec::new(),
            risk_score: 0.0,
        }
    }

    /// Add vulnerability finding
    pub fn add_vulnerability(&mut self, vuln: Vulnerability) {
        self.vulnerabilities.push(vuln);
        self.recalculate_risk_score();
    }

    /// Recalculate overall risk score
    fn recalculate_risk_score(&mut self) {
        if self.vulnerabilities.is_empty() {
            self.risk_score = 0.0;
            return;
        }

        let severity_weights = [
            (Severity::Info, 0.1),
            (Severity::Low, 0.3),
            (Severity::Medium, 0.6),
            (Severity::High, 0.85),
            (Severity::Critical, 1.0),
        ];

        let total_weight: f64 = self.vulnerabilities.iter()
            .map(|v| {
                severity_weights.iter()
                    .find(|(s, _)| *s == v.severity)
                    .map(|(_, w)| w)
                    .copied()
                    .unwrap_or(0.0)
            })
            .sum();

        self.risk_score = total_weight / self.vulnerabilities.len() as f64;
    }

    /// Get critical vulnerabilities
    pub fn critical_vulnerabilities(&self) -> Vec<&Vulnerability> {
        self.vulnerabilities.iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect()
    }

    /// Get high-risk vulnerabilities
    pub fn high_risk_vulnerabilities(&self) -> Vec<&Vulnerability> {
        self.vulnerabilities.iter()
            .filter(|v| v.severity >= Severity::High)
            .collect()
    }

    /// Generate security report
    pub fn generate_report(&self) -> String {
        let mut report = format!(
            "SECURITY ASSESSMENT REPORT\n\
             ===========================\n\
             Component: {}\n\
             Timestamp: {}\n\
             Risk Score: {:.2}%\n\n",
            self.component,
            self.timestamp,
            self.risk_score * 100.0
        );

        report.push_str(&format!("Total Issues: {}\n", self.vulnerabilities.len()));

        let by_severity = self.count_by_severity();
        for (sev, count) in &by_severity {
            if *count > 0 {
                report.push_str(&format!("  {:?}: {}\n", sev, count));
            }
        }

        if !self.vulnerabilities.is_empty() {
            report.push_str("\nDetailed Findings:\n");
            for (i, vuln) in self.vulnerabilities.iter().enumerate() {
                report.push_str(&format!(
                    "\n{}. [{}] {} ({})\n",
                    i + 1,
                    vuln.id,
                    vuln.title,
                    match vuln.severity {
                        Severity::Info => "ℹ️  Info",
                        Severity::Low => "⚠️  Low",
                        Severity::Medium => "⚠️  Medium",
                        Severity::High => "🔴 High",
                        Severity::Critical => "🔴 CRITICAL",
                    }
                ));
                report.push_str(&format!("   {}\n", vuln.description));
                report.push_str(&format!("   Remediation: {}\n", vuln.remediation));
            }
        }

        report
    }

    fn count_by_severity(&self) -> HashMap<Severity, usize> {
        let mut counts = HashMap::new();
        for vuln in &self.vulnerabilities {
            *counts.entry(vuln.severity).or_insert(0) += 1;
        }
        counts
    }
}

/// Threat model analyzer
pub struct ThreatModelAnalyzer {
    threats: Vec<String>,
}

impl ThreatModelAnalyzer {
    /// Create new threat model analyzer
    pub fn new() -> Self {
        Self {
            threats: Vec::new(),
        }
    }

    /// Add threat to model
    pub fn add_threat(&mut self, threat: String) {
        self.threats.push(threat);
    }

    /// Analyze consensus threats
    pub fn analyze_consensus_threats(&mut self) {
        self.threats.push("51% attack via stake concentration".to_string());
        self.threats.push("Validator key compromise".to_string());
        self.threats.push("State fork from chain reorganization".to_string());
        self.threats.push("Nonce overflow attack".to_string());
    }

    /// Analyze contract threats
    pub fn analyze_contract_threats(&mut self) {
        self.threats.push("Reentrancy (WASM execution)".to_string());
        self.threats.push("Integer overflow in calculations".to_string());
        self.threats.push("Memory exhaustion attack".to_string());
        self.threats.push("Unbounded loop in contract".to_string());
    }

    /// Analyze network threats
    pub fn analyze_network_threats(&mut self) {
        self.threats.push("Sybil attack (multiple node identities)".to_string());
        self.threats.push("Man-in-the-middle (no TLS)".to_string());
        self.threats.push("Peer eclipse attack".to_string());
        self.threats.push("Transaction flooding (DDoS)".to_string());
    }

    /// Get all identified threats
    pub fn threats(&self) -> &[String] {
        &self.threats
    }

    /// Count threats
    pub fn threat_count(&self) -> usize {
        self.threats.len()
    }
}

/// Input validation checker
pub struct InputValidator {
    checks_passed: usize,
    checks_failed: usize,
}

impl InputValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            checks_passed: 0,
            checks_failed: 0,
        }
    }

    /// Validate address format
    pub fn validate_address(&mut self, address: &str) -> Result<(), String> {
        if address.is_empty() {
            self.checks_failed += 1;
            return Err("Address cannot be empty".to_string());
        }

        if !address.starts_with("0x") && !address.starts_with("0x") {
            self.checks_failed += 1;
            return Err("Address must start with 0x".to_string());
        }

        if address.len() != 42 {
            self.checks_failed += 1;
            return Err("Address must be 42 characters".to_string());
        }

        self.checks_passed += 1;
        Ok(())
    }

    /// Validate amount
    pub fn validate_amount(&mut self, amount: f64) -> Result<(), String> {
        if amount < 0.0 {
            self.checks_failed += 1;
            return Err("Amount cannot be negative".to_string());
        }

        if amount > 1e18 {
            self.checks_failed += 1;
            return Err("Amount exceeds maximum".to_string());
        }

        self.checks_passed += 1;
        Ok(())
    }

    /// Validate nonce
    pub fn validate_nonce(&mut self, nonce: u64) -> Result<(), String> {
        if nonce > u32::MAX as u64 {
            self.checks_failed += 1;
            return Err("Nonce overflow risk".to_string());
        }

        self.checks_passed += 1;
        Ok(())
    }

    /// Get validation statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.checks_passed, self.checks_failed)
    }

    /// Get pass rate
    pub fn pass_rate(&self) -> f64 {
        let total = self.checks_passed + self.checks_failed;
        if total == 0 {
            return 1.0;
        }
        self.checks_passed as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_creation() {
        let vuln = Vulnerability {
            id: "AUREON-001".to_string(),
            title: "Missing input validation".to_string(),
            description: "Function doesn't validate inputs".to_string(),
            severity: Severity::High,
            component: "state".to_string(),
            remediation: "Add input validation checks".to_string(),
            cve_id: None,
        };

        assert_eq!(vuln.id, "AUREON-001");
        assert_eq!(vuln.severity, Severity::High);
    }

    #[test]
    fn test_assessment_creation() {
        let assessment = SecurityAssessment::new("consensus".to_string());
        assert_eq!(assessment.component, "consensus");
        assert_eq!(assessment.vulnerabilities.len(), 0);
        assert_eq!(assessment.risk_score, 0.0);
    }

    #[test]
    fn test_add_vulnerability() {
        let mut assessment = SecurityAssessment::new("contracts".to_string());

        let vuln = Vulnerability {
            id: "AUREON-002".to_string(),
            title: "Reentrancy vulnerability".to_string(),
            description: "Contract can be reentered".to_string(),
            severity: Severity::Critical,
            component: "contracts".to_string(),
            remediation: "Add reentrancy guard".to_string(),
            cve_id: None,
        };

        assessment.add_vulnerability(vuln);
        assert_eq!(assessment.vulnerabilities.len(), 1);
        assert!(assessment.risk_score > 0.8);
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut assessment = SecurityAssessment::new("test".to_string());

        assessment.add_vulnerability(Vulnerability {
            id: "V1".to_string(),
            title: "Info".to_string(),
            description: "".to_string(),
            severity: Severity::Info,
            component: "test".to_string(),
            remediation: "".to_string(),
            cve_id: None,
        });

        assessment.add_vulnerability(Vulnerability {
            id: "V2".to_string(),
            title: "Critical".to_string(),
            description: "".to_string(),
            severity: Severity::Critical,
            component: "test".to_string(),
            remediation: "".to_string(),
            cve_id: None,
        });

        let expected_score = (0.1 + 1.0) / 2.0;
        assert!((assessment.risk_score - expected_score).abs() < 0.01);
    }

    #[test]
    fn test_critical_vulnerabilities() {
        let mut assessment = SecurityAssessment::new("test".to_string());

        for severity in &[Severity::Low, Severity::Critical, Severity::High] {
            assessment.add_vulnerability(Vulnerability {
                id: format!("{:?}", severity),
                title: "Test".to_string(),
                description: "".to_string(),
                severity: *severity,
                component: "test".to_string(),
                remediation: "".to_string(),
                cve_id: None,
            });
        }

        let critical = assessment.critical_vulnerabilities();
        assert_eq!(critical.len(), 1);
    }

    #[test]
    fn test_threat_model_analyzer() {
        let mut analyzer = ThreatModelAnalyzer::new();
        assert_eq!(analyzer.threat_count(), 0);

        analyzer.analyze_consensus_threats();
        assert_eq!(analyzer.threat_count(), 4);

        analyzer.analyze_contract_threats();
        assert_eq!(analyzer.threat_count(), 8);

        analyzer.analyze_network_threats();
        assert_eq!(analyzer.threat_count(), 12);

        assert!(!analyzer.threats().is_empty());
    }

    #[test]
    fn test_input_validator_address() {
        let mut validator = InputValidator::new();

        // Valid address
        assert!(validator.validate_address("0x1234567890123456789012345678901234567890").is_ok());
        assert_eq!(validator.stats(), (1, 0));

        // Invalid addresses
        assert!(validator.validate_address("").is_err());
        assert!(validator.validate_address("invalid").is_err());
        assert_eq!(validator.stats(), (1, 2));
    }

    #[test]
    fn test_input_validator_amount() {
        let mut validator = InputValidator::new();

        assert!(validator.validate_amount(100.0).is_ok());
        assert!(validator.validate_amount(-1.0).is_err());
        assert!(validator.validate_amount(1e20).is_err());
    }

    #[test]
    fn test_input_validator_nonce() {
        let mut validator = InputValidator::new();

        assert!(validator.validate_nonce(0).is_ok());
        assert!(validator.validate_nonce(1000).is_ok());
        assert!(validator.validate_nonce(u32::MAX as u64 + 1).is_err());
    }

    #[test]
    fn test_validator_pass_rate() {
        let mut validator = InputValidator::new();

        validator.validate_nonce(100).ok();
        validator.validate_nonce(u32::MAX as u64 + 1).ok();

        assert_eq!(validator.pass_rate(), 0.5);
    }

    #[test]
    fn test_security_report_generation() {
        let mut assessment = SecurityAssessment::new("security_test".to_string());

        assessment.add_vulnerability(Vulnerability {
            id: "TEST-001".to_string(),
            title: "Test vulnerability".to_string(),
            description: "This is a test".to_string(),
            severity: Severity::High,
            component: "test".to_string(),
            remediation: "Fix it".to_string(),
            cve_id: None,
        });

        let report = assessment.generate_report();
        assert!(report.contains("SECURITY ASSESSMENT REPORT"));
        assert!(report.contains("security_test"));
        assert!(report.contains("TEST-001"));
        assert!(report.contains("High"));
    }

    #[test]
    fn test_multiple_vulnerabilities_report() {
        let mut assessment = SecurityAssessment::new("multi".to_string());

        for i in 0..5 {
            assessment.add_vulnerability(Vulnerability {
                id: format!("V{}", i),
                title: format!("Vuln {}", i),
                description: "".to_string(),
                severity: if i % 2 == 0 { Severity::High } else { Severity::Low },
                component: "multi".to_string(),
                remediation: "Fix".to_string(),
                cve_id: None,
            });
        }

        let report = assessment.generate_report();
        assert!(report.contains("Total Issues: 5"));
        assert!(report.contains("Detailed Findings:"));
    }
}
