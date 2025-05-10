package com.jaiswarsecurities.cerberusauth.model;

import jakarta.persistence.*;
import java.time.LocalDateTime;
import java.util.HashSet;
import java.util.Set;
import java.util.Objects;
import java.util.UUID;

@Entity
@Table(name = "users", indexes = {
    @Index(name = "idx_user_username", columnList = "username", unique = true),
    @Index(name = "idx_user_email", columnList = "email", unique = true),
    @Index(name = "idx_user_employee_id", columnList = "employee_id", unique = true) // Corrected columnList
})
public class User {

    @Id
    @GeneratedValue(strategy = GenerationType.AUTO) // For UUIDs
    @Column(name = "user_id")
    private UUID id;

    @Column(name = "username", length = 100, unique = true, nullable = false)
    private String username;

    @Column(name = "hashed_password", length = 255, nullable = false)
    private String hashedPassword;

    @Column(name = "email", length = 255, unique = true, nullable = false)
    private String email;

    @Column(name = "employee_id", length = 100, unique = true, nullable = true)
    private String employeeId;

    @Column(name = "first_name", length = 100, nullable = true)
    private String firstName;

    @Column(name = "last_name", length = 100, nullable = true)
    private String lastName;

    // department, jobTitle, managerEmployeeId removed as they are not in the central schema

    @Enumerated(EnumType.STRING)
    @Column(name = "status", length = 50, nullable = false) // Length should match DB enum definition, e.g., user_status_enum
    private AccountStatus accountStatus = AccountStatus.PENDING_VERIFICATION; // Default value aligned with DB

    // 'enabled' field removed; its logic is now derived from 'accountStatus' via isEnabled() for Spring Security.

    @Column(name = "last_login_at")
    private LocalDateTime lastLoginAt;

    // passwordLastChanged removed

    @Column(name = "failed_login_attempts", columnDefinition = "INT DEFAULT 0")
    private int failedLoginAttempts = 0;

    // twoFactorEnabled removed

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "user_roles", // Ensure this table name is correct as per db-manager schema (017_create_user_roles_table.yaml)
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "role_id")
    )
    private Set<Role> roles = new HashSet<>();

    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @Column(name = "updated_at", nullable = false)
    private LocalDateTime updatedAt;

    // created_by and updated_by fields from schema are omitted for now for simplicity

    @PrePersist
    protected void onCreate() {
        LocalDateTime now = LocalDateTime.now();
        if (this.createdAt == null) { // Allow manual setting for seeding etc.
            this.createdAt = now;
        }
        this.updatedAt = now;
        // Default accountStatus is PENDING_VERIFICATION set at field declaration.
    }

    @PreUpdate
    protected void onUpdate() {
        this.updatedAt = LocalDateTime.now();
    }

    // Constructors
    public User() {
    }

    // Getters
    public UUID getId() { return id; }
    public String getUsername() { return username; }
    public String getHashedPassword() { return hashedPassword; }
    public String getEmail() { return email; }
    public String getEmployeeId() { return employeeId; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
    public AccountStatus getAccountStatus() { return accountStatus; }
    public LocalDateTime getLastLoginAt() { return lastLoginAt; }
    public int getFailedLoginAttempts() { return failedLoginAttempts; }
    public Set<Role> getRoles() { return roles; }
    public LocalDateTime getCreatedAt() { return createdAt; }
    public LocalDateTime getUpdatedAt() { return updatedAt; }

    // Spring Security's UserDetails interface typically requires isEnabled(), etc.
    // We derive these from accountStatus.
    public boolean isEnabled() {
        // Define which statuses mean the account is effectively enabled for login
        return this.accountStatus == AccountStatus.ACTIVE;
    }

    public boolean isAccountNonExpired() {
        return true; // Or add logic if your statuses include an "expired" state
    }

    public boolean isAccountNonLocked() {
        // Example: if LOCKED or SUSPENDED means locked
        return this.accountStatus != AccountStatus.LOCKED && 
               this.accountStatus != AccountStatus.SUSPENDED;
    }

    public boolean isCredentialsNonExpired() {
        return true; // Or add logic for password expiry if that feature is reintroduced
    }

    // Setters
    public void setId(UUID id) { this.id = id; }
    public void setUsername(String username) { this.username = username; }
    public void setHashedPassword(String hashedPassword) { this.hashedPassword = hashedPassword; }
    public void setEmail(String email) { this.email = email; }
    public void setEmployeeId(String employeeId) { this.employeeId = employeeId; }
    public void setFirstName(String firstName) { this.firstName = firstName; }
    public void setLastName(String lastName) { this.lastName = lastName; }
    public void setAccountStatus(AccountStatus accountStatus) { 
        this.accountStatus = accountStatus;
    }
    public void setLastLoginAt(LocalDateTime lastLoginAt) { this.lastLoginAt = lastLoginAt; }
    public void setFailedLoginAttempts(int failedLoginAttempts) { this.failedLoginAttempts = failedLoginAttempts; }
    public void setRoles(Set<Role> roles) { this.roles = roles; }
    // createdAt and updatedAt typically managed by @PrePersist/@PreUpdate or DB
    public void setCreatedAt(LocalDateTime createdAt) { this.createdAt = createdAt; }
    public void setUpdatedAt(LocalDateTime updatedAt) { this.updatedAt = updatedAt; }

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
        // Prefer ID for hashCode if available, otherwise business key components.
        if (id != null) return Objects.hash(id);
        return Objects.hash(username, employeeId);
    }

    @Override
    public String toString() {
        return "User{" +
               "id=" + id +
               ", username='" + username + "'" +
               ", employeeId='" + employeeId + "'" +
               ", accountStatus=" + accountStatus +
               "}";
    }
}
