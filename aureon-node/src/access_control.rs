use std::collections::{HashMap, HashSet};

/// Access control and authorization module
///
/// This module provides role-based access control (RBAC),
/// permission management, and authorization enforcement.

/// User role type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Operator,
    Node,
    Validator,
    User,
    Guest,
}

/// Permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // Admin permissions
    ManageUsers,
    ManageRoles,
    ModifyConfig,
    ViewLogs,

    // Operator permissions
    StartNode,
    StopNode,
    RestartNode,
    ViewMetrics,

    // Node permissions
    ProposeBlock,
    ValidateBlock,
    SyncState,

    // Validator permissions
    Sign,
    Stake,
    Unstake,
    Vote,

    // User permissions
    CreateTransaction,
    QueryState,
    ViewBlocks,

    // Guest permissions
    ReadOnly,
}

/// Role-permission mapping
#[derive(Debug, Clone)]
pub struct RolePermissions {
    role: Role,
    permissions: HashSet<Permission>,
}

impl RolePermissions {
    /// Create role with permissions
    pub fn new(role: Role) -> Self {
        let mut permissions = HashSet::new();

        match role {
            Role::Admin => {
                permissions.insert(Permission::ManageUsers);
                permissions.insert(Permission::ManageRoles);
                permissions.insert(Permission::ModifyConfig);
                permissions.insert(Permission::ViewLogs);
                permissions.insert(Permission::StartNode);
                permissions.insert(Permission::StopNode);
                permissions.insert(Permission::ViewMetrics);
            }
            Role::Operator => {
                permissions.insert(Permission::StartNode);
                permissions.insert(Permission::StopNode);
                permissions.insert(Permission::RestartNode);
                permissions.insert(Permission::ViewMetrics);
                permissions.insert(Permission::ViewLogs);
            }
            Role::Node => {
                permissions.insert(Permission::ProposeBlock);
                permissions.insert(Permission::ValidateBlock);
                permissions.insert(Permission::SyncState);
            }
            Role::Validator => {
                permissions.insert(Permission::Sign);
                permissions.insert(Permission::Stake);
                permissions.insert(Permission::Vote);
                permissions.insert(Permission::ValidateBlock);
            }
            Role::User => {
                permissions.insert(Permission::CreateTransaction);
                permissions.insert(Permission::QueryState);
                permissions.insert(Permission::ViewBlocks);
            }
            Role::Guest => {
                permissions.insert(Permission::ReadOnly);
                permissions.insert(Permission::ViewBlocks);
            }
        }

        Self { role, permissions }
    }

    /// Check if role has permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Get all permissions
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    /// Add permission to role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove permission from role
    pub fn remove_permission(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
    }

    /// Get permission count
    pub fn permission_count(&self) -> usize {
        self.permissions.len()
    }
}

/// User with role and permissions
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub role: Role,
    pub is_active: bool,
    pub created_at: u64,
    pub last_login: Option<u64>,
}

impl User {
    /// Create new user
    pub fn new(id: String, role: Role) -> Self {
        Self {
            id,
            role,
            is_active: true,
            created_at: 0,
            last_login: None,
        }
    }

    /// Update last login
    pub fn update_login(&mut self, timestamp: u64) {
        self.last_login = Some(timestamp);
    }

    /// Deactivate user
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Reactivate user
    pub fn activate(&mut self) {
        self.is_active = true;
    }
}

/// Access control manager
pub struct AccessControlManager {
    users: HashMap<String, User>,
    role_permissions: HashMap<Role, RolePermissions>,
    access_log: Vec<AccessLogEntry>,
}

#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: u64,
    pub allowed: bool,
}

impl AccessControlManager {
    /// Create new ACL manager
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();
        for role in &[
            Role::Admin,
            Role::Operator,
            Role::Node,
            Role::Validator,
            Role::User,
            Role::Guest,
        ] {
            role_permissions.insert(*role, RolePermissions::new(*role));
        }

        Self {
            users: HashMap::new(),
            role_permissions,
            access_log: Vec::new(),
        }
    }

    /// Add user
    pub fn add_user(&mut self, user: User) -> Result<(), String> {
        if self.users.contains_key(&user.id) {
            return Err(format!("User {} already exists", user.id));
        }

        self.users.insert(user.id.clone(), user);
        Ok(())
    }

    /// Remove user
    pub fn remove_user(&mut self, user_id: &str) -> Result<(), String> {
        if self.users.remove(user_id).is_some() {
            Ok(())
        } else {
            Err(format!("User {} not found", user_id))
        }
    }

    /// Get user
    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }

    /// Check if user can perform action
    pub fn check_permission(
        &mut self,
        user_id: &str,
        permission: Permission,
    ) -> Result<bool, String> {
        let user = self
            .users
            .get(user_id)
            .ok_or(format!("User {} not found", user_id))?;

        if !user.is_active {
            self.log_access(user_id, "check_permission", &format!("{:?}", permission), false);
            return Ok(false);
        }

        let role_perms = &self.role_permissions[&user.role];
        let allowed = role_perms.has_permission(permission);

        self.log_access(user_id, "check_permission", &format!("{:?}", permission), allowed);

        Ok(allowed)
    }

    /// Grant permission to role
    pub fn grant_permission(&mut self, role: Role, permission: Permission) {
        if let Some(role_perms) = self.role_permissions.get_mut(&role) {
            role_perms.add_permission(permission);
        }
    }

    /// Revoke permission from role
    pub fn revoke_permission(&mut self, role: Role, permission: Permission) {
        if let Some(role_perms) = self.role_permissions.get_mut(&role) {
            role_perms.remove_permission(permission);
        }
    }

    /// Change user role
    pub fn change_user_role(&mut self, user_id: &str, new_role: Role) -> Result<(), String> {
        if let Some(user) = self.users.get_mut(user_id) {
            user.role = new_role;
            Ok(())
        } else {
            Err(format!("User {} not found", user_id))
        }
    }

    /// Log access attempt
    fn log_access(&mut self, user_id: &str, action: &str, resource: &str, allowed: bool) {
        self.access_log.push(AccessLogEntry {
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp: 0, // Would be actual timestamp
            allowed,
        });
    }

    /// Get access log
    pub fn access_log(&self) -> &[AccessLogEntry] {
        &self.access_log
    }

    /// Get denied access count
    pub fn denied_access_count(&self) -> usize {
        self.access_log.iter().filter(|e| !e.allowed).count()
    }

    /// Get allowed access count
    pub fn allowed_access_count(&self) -> usize {
        self.access_log.iter().filter(|e| e.allowed).count()
    }

    /// Get user count by role
    pub fn users_by_role(&self, role: Role) -> usize {
        self.users.values().filter(|u| u.role == role).count()
    }

    /// Get total user count
    pub fn total_users(&self) -> usize {
        self.users.len()
    }

    /// Get active user count
    pub fn active_users(&self) -> usize {
        self.users.values().filter(|u| u.is_active).count()
    }

    /// Generate access report
    pub fn generate_report(&self) -> String {
        let mut report = "ACCESS CONTROL REPORT\n".to_string();
        report.push_str("====================\n\n");

        report.push_str("USERS BY ROLE:\n");
        for role in &[
            Role::Admin,
            Role::Operator,
            Role::Node,
            Role::Validator,
            Role::User,
            Role::Guest,
        ] {
            let count = self.users_by_role(*role);
            report.push_str(&format!("  {:?}: {}\n", role, count));
        }

        report.push_str("\nACCESS LOG SUMMARY:\n");
        report.push_str(&format!("  Total Allowed: {}\n", self.allowed_access_count()));
        report.push_str(&format!("  Total Denied: {}\n", self.denied_access_count()));
        report.push_str(&format!("  Success Rate: {:.2}%\n", self.access_success_rate() * 100.0));

        report
    }

    /// Get access success rate
    pub fn access_success_rate(&self) -> f64 {
        if self.access_log.is_empty() {
            return 1.0;
        }

        let allowed = self.allowed_access_count() as f64;
        let total = self.access_log.len() as f64;
        allowed / total
    }
}

/// Permission audit
pub struct PermissionAudit {
    findings: Vec<String>,
    recommendations: Vec<String>,
}

impl PermissionAudit {
    /// Create new audit
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Audit access control system
    pub fn audit(&mut self) {
        self.findings.push("Enforce principle of least privilege".to_string());
        self.findings.push("Implement permission inheritance".to_string());
        self.findings.push("Audit admin role assignment".to_string());

        self.recommendations.push("Use role-based access control (RBAC)".to_string());
        self.recommendations.push("Implement attribute-based access control (ABAC) in future".to_string());
        self.recommendations.push("Log all permission changes".to_string());
        self.recommendations.push("Regular permission audits".to_string());
    }

    /// Get findings
    pub fn findings(&self) -> &[String] {
        &self.findings
    }

    /// Get recommendations
    pub fn recommendations(&self) -> &[String] {
        &self.recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions_creation() {
        let role_perms = RolePermissions::new(Role::Admin);
        assert_eq!(role_perms.role, Role::Admin);
        assert!(role_perms.permission_count() > 0);
    }

    #[test]
    fn test_role_permissions_check() {
        let role_perms = RolePermissions::new(Role::Admin);
        assert!(role_perms.has_permission(Permission::ManageUsers));
    }

    #[test]
    fn test_role_permissions_guest() {
        let role_perms = RolePermissions::new(Role::Guest);
        assert!(!role_perms.has_permission(Permission::ManageUsers));
        assert!(role_perms.has_permission(Permission::ReadOnly));
    }

    #[test]
    fn test_user_creation() {
        let user = User::new("user1".to_string(), Role::User);
        assert_eq!(user.id, "user1");
        assert_eq!(user.role, Role::User);
        assert!(user.is_active);
    }

    #[test]
    fn test_user_deactivate() {
        let mut user = User::new("user1".to_string(), Role::User);
        user.deactivate();
        assert!(!user.is_active);

        user.activate();
        assert!(user.is_active);
    }

    #[test]
    fn test_acm_add_user() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        assert!(acm.add_user(user).is_ok());
        assert!(acm.get_user("user1").is_some());
    }

    #[test]
    fn test_acm_duplicate_user() {
        let mut acm = AccessControlManager::new();
        let user1 = User::new("user1".to_string(), Role::User);
        let user2 = User::new("user1".to_string(), Role::Admin);

        acm.add_user(user1).ok();
        assert!(acm.add_user(user2).is_err());
    }

    #[test]
    fn test_acm_remove_user() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        acm.add_user(user).ok();
        assert!(acm.remove_user("user1").is_ok());
        assert!(acm.get_user("user1").is_none());
    }

    #[test]
    fn test_acm_check_permission() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        acm.add_user(user).ok();

        let result = acm.check_permission("user1", Permission::CreateTransaction);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_acm_check_permission_denied() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::Guest);

        acm.add_user(user).ok();

        let result = acm.check_permission("user1", Permission::ManageUsers);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_acm_inactive_user() {
        let mut acm = AccessControlManager::new();
        let mut user = User::new("user1".to_string(), Role::User);
        user.deactivate();

        acm.add_user(user).ok();

        let result = acm.check_permission("user1", Permission::CreateTransaction);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_acm_change_role() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        acm.add_user(user).ok();
        assert!(acm.change_user_role("user1", Role::Admin).is_ok());

        let updated = acm.get_user("user1").unwrap();
        assert_eq!(updated.role, Role::Admin);
    }

    #[test]
    fn test_acm_grant_permission() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::Guest);

        acm.add_user(user).ok();
        acm.grant_permission(Role::Guest, Permission::CreateTransaction);

        let result = acm.check_permission("user1", Permission::CreateTransaction);
        assert!(result.unwrap());
    }

    #[test]
    fn test_acm_revoke_permission() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        acm.add_user(user).ok();
        acm.revoke_permission(Role::User, Permission::CreateTransaction);

        let result = acm.check_permission("user1", Permission::CreateTransaction);
        assert!(!result.unwrap());
    }

    #[test]
    fn test_acm_user_counts() {
        let mut acm = AccessControlManager::new();

        acm.add_user(User::new("user1".to_string(), Role::User)).ok();
        acm.add_user(User::new("user2".to_string(), Role::Admin)).ok();
        acm.add_user(User::new("user3".to_string(), Role::User)).ok();

        assert_eq!(acm.total_users(), 3);
        assert_eq!(acm.users_by_role(Role::User), 2);
        assert_eq!(acm.users_by_role(Role::Admin), 1);
    }

    #[test]
    fn test_acm_access_log() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        acm.add_user(user).ok();
        acm.check_permission("user1", Permission::CreateTransaction).ok();

        assert_eq!(acm.access_log().len(), 1);
        assert_eq!(acm.allowed_access_count(), 1);
    }

    #[test]
    fn test_acm_access_success_rate() {
        let mut acm = AccessControlManager::new();
        let user = User::new("user1".to_string(), Role::User);

        acm.add_user(user).ok();
        acm.check_permission("user1", Permission::CreateTransaction).ok();
        acm.check_permission("user1", Permission::ManageUsers).ok();

        assert_eq!(acm.access_success_rate(), 0.5);
    }

    #[test]
    fn test_acm_generate_report() {
        let mut acm = AccessControlManager::new();
        acm.add_user(User::new("user1".to_string(), Role::Admin)).ok();

        let report = acm.generate_report();
        assert!(report.contains("ACCESS CONTROL REPORT"));
        assert!(report.contains("Admin"));
    }

    #[test]
    fn test_permission_audit() {
        let mut audit = PermissionAudit::new();
        audit.audit();

        assert!(audit.findings().len() > 0);
        assert!(audit.recommendations().len() > 0);
    }

    #[test]
    fn test_role_permissions_add_remove() {
        let mut role_perms = RolePermissions::new(Role::Guest);
        let initial_count = role_perms.permission_count();

        role_perms.add_permission(Permission::CreateTransaction);
        assert!(role_perms.has_permission(Permission::CreateTransaction));
        assert!(role_perms.permission_count() > initial_count);

        role_perms.remove_permission(Permission::CreateTransaction);
        assert!(!role_perms.has_permission(Permission::CreateTransaction));
    }

    #[test]
    fn test_user_login_update() {
        let mut user = User::new("user1".to_string(), Role::User);
        assert!(user.last_login.is_none());

        user.update_login(1000);
        assert_eq!(user.last_login, Some(1000));
    }

    #[test]
    fn test_acm_active_users() {
        let mut acm = AccessControlManager::new();
        let mut user1 = User::new("user1".to_string(), Role::User);
        let user2 = User::new("user2".to_string(), Role::User);

        user1.deactivate();
        acm.add_user(user1).ok();
        acm.add_user(user2).ok();

        assert_eq!(acm.active_users(), 1);
    }
}
