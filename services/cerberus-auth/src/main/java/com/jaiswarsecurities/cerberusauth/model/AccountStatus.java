package com.jaiswarsecurities.cerberusauth.model;

public enum AccountStatus {
    ACTIVE,
    SUSPENDED, // Manually suspended by an admin
    LOCKED_BAD_CREDENTIALS, // Locked due to too many failed attempts
    PENDING_APPROVAL, // New account, awaiting activation/approval
    DEACTIVATED // User account is no longer active (e.g., employee left)
}
