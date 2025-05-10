package com.jaiswarsecurities.cerberusauth.model;

import jakarta.persistence.*;
import java.time.LocalDateTime;
import java.util.HashSet;
import java.util.Set;
import java.util.Objects;

@Entity
@Table(name = "users", indexes = {
    @Index(name = "idx_user_username", columnList = "username", unique = true),
    @Index(name = "idx_user_email", columnList = "email", unique = true),
    @Index(name = "idx_user_employee_id", columnList = "employeeId", unique = true)
})
public class User {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(length = 50, unique = true, nullable = false)
    private String username;

    @Column(length = 255, nullable = false)
    private String password;

    @Column(length = 100, unique = true, nullable = false)
    private String email;

    @Column(length = 50, unique = true, nullable = true)
    private String employeeId;

    @Column(length = 100, nullable = true)
    private String firstName;

    @Column(length = 100, nullable = true)
    private String lastName;

    @Column(length = 100)
    private String department;

    @Column(length = 100)
    private String jobTitle;

    @Column(length = 50)
    private String managerEmployeeId;

    @Enumerated(EnumType.STRING)
    @Column(length = 30, nullable = false)
    private AccountStatus accountStatus = AccountStatus.PENDING_APPROVAL;

    @Column(nullable = false)
    private boolean enabled = false; // Initial state, can be changed by admin or accountStatus logic

    private LocalDateTime lastLoginDate;

    private LocalDateTime passwordLastChanged;

    @Column(nullable = false, columnDefinition = "INT DEFAULT 0")
    private int failedLoginAttempts = 0;

    @Column(nullable = false)
    private boolean twoFactorEnabled = false;

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "user_roles",
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "role_id")
    )
    private Set<Role> roles = new HashSet<>();

    @Column(nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @Column(nullable = false)
    private LocalDateTime updatedAt;

    @PrePersist
    protected void onCreate() {
        LocalDateTime now = LocalDateTime.now();
        this.createdAt = now;
        this.updatedAt = now;
        if (this.passwordLastChanged == null) {
            this.passwordLastChanged = now;
        }
        // Initial alignment of enabled based on accountStatus
        synchronizeEnabledWithAccountStatus();
    }

    @PreUpdate
    protected void onUpdate() {
        this.updatedAt = LocalDateTime.now();
        synchronizeEnabledWithAccountStatus();
    }

    // Call this method to ensure 'enabled' reflects 'accountStatus' unless overridden.
    private void synchronizeEnabledWithAccountStatus() {
        if (this.accountStatus == AccountStatus.ACTIVE) {
            // If an admin hasn't explicitly disabled it, and status is ACTIVE, it should be enabled.
            // For now, we assume setting to ACTIVE implies it should be enabled.
            this.enabled = true; 
        } else if (this.accountStatus == AccountStatus.DEACTIVATED || 
                   this.accountStatus == AccountStatus.SUSPENDED) {
            this.enabled = false;
        }
        // For PENDING_APPROVAL or LOCKED_BAD_CREDENTIALS, 'enabled' might remain as is,
        // or you could enforce 'enabled = false'. Let's make them explicitly disable for now.
        else if (this.accountStatus == AccountStatus.PENDING_APPROVAL || 
                 this.accountStatus == AccountStatus.LOCKED_BAD_CREDENTIALS) {
            this.enabled = false;
        }
    }

    // Constructors
    public User() {
    }

    // Getters
    public Long getId() { return id; }
    public String getUsername() { return username; }
    public String getPassword() { return password; }
    public String getEmail() { return email; }
    public String getEmployeeId() { return employeeId; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
    public String getDepartment() { return department; }
    public String getJobTitle() { return jobTitle; }
    public String getManagerEmployeeId() { return managerEmployeeId; }
    public AccountStatus getAccountStatus() { return accountStatus; }
    public boolean isEnabled() { return enabled; } // Spring Security uses this
    public LocalDateTime getLastLoginDate() { return lastLoginDate; }
    public LocalDateTime getPasswordLastChanged() { return passwordLastChanged; }
    public int getFailedLoginAttempts() { return failedLoginAttempts; }
    public boolean isTwoFactorEnabled() { return twoFactorEnabled; }
    public Set<Role> getRoles() { return roles; }
    public LocalDateTime getCreatedAt() { return createdAt; }
    public LocalDateTime getUpdatedAt() { return updatedAt; }

    // Setters
    public void setId(Long id) { this.id = id; }
    public void setUsername(String username) { this.username = username; }
    public void setPassword(String password) { this.password = password; }
    public void setEmail(String email) { this.email = email; }
    public void setEmployeeId(String employeeId) { this.employeeId = employeeId; }
    public void setFirstName(String firstName) { this.firstName = firstName; }
    public void setLastName(String lastName) { this.lastName = lastName; }
    public void setDepartment(String department) { this.department = department; }
    public void setJobTitle(String jobTitle) { this.jobTitle = jobTitle; }
    public void setManagerEmployeeId(String managerEmployeeId) { this.managerEmployeeId = managerEmployeeId; }
    public void setAccountStatus(AccountStatus accountStatus) { 
        this.accountStatus = accountStatus;
        synchronizeEnabledWithAccountStatus(); // Keep 'enabled' in sync
    }
    // Allows explicit administrative override of 'enabled' status
    public void setEnabled(boolean enabled) { 
        this.enabled = enabled;
        // If an admin explicitly enables, should we change accountStatus from e.g. SUSPENDED?
        // For now, setting 'enabled' does not automatically change 'accountStatus' from a non-active state.
        // If 'enabled' is set to true, but accountStatus is SUSPENDED, login would still fail in UserDetailsService.
    }
    public void setLastLoginDate(LocalDateTime lastLoginDate) { this.lastLoginDate = lastLoginDate; }
    public void setPasswordLastChanged(LocalDateTime passwordLastChanged) { this.passwordLastChanged = passwordLastChanged; }
    public void setFailedLoginAttempts(int failedLoginAttempts) { this.failedLoginAttempts = failedLoginAttempts; }
    public void setTwoFactorEnabled(boolean twoFactorEnabled) { this.twoFactorEnabled = twoFactorEnabled; }
    public void setRoles(Set<Role> roles) { this.roles = roles; }
    public void setCreatedAt(LocalDateTime createdAt) { this.createdAt = createdAt; } // Usually not set manually
    public void setUpdatedAt(LocalDateTime updatedAt) { this.updatedAt = updatedAt; } // Usually not set manually

    public void addRole(Role role) { this.roles.add(role); }
    public void removeRole(Role role) { this.roles.remove(role); }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        User user = (User) o;
        return Objects.equals(id, user.id) || 
               (username != null && Objects.equals(username, user.username)) || 
               (employeeId != null && Objects.equals(employeeId, user.employeeId));
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, username, employeeId);
    }

    @Override
    public String toString() {
        return "User{" +
               "id=" + id +
               ", username='" + username + ''' +
               ", employeeId='" + employeeId + ''' +
               ", accountStatus=" + accountStatus +
               ", enabled=" + enabled +
               '}';
    }
}
