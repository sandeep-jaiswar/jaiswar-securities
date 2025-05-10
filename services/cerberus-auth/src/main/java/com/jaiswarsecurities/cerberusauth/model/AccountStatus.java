package com.jaiswarsecurities.cerberusauth.model;

public enum AccountStatus {
    ACTIVE,
    INACTIVE,               // Was DEACTIVATED, aligns with DB INACTIVE
    SUSPENDED,              // Matches DB
    PENDING_VERIFICATION,   // Was PENDING_APPROVAL, aligns with DB PENDING_VERIFICATION
    LOCKED                  // Was LOCKED_BAD_CREDENTIALS, aligns with DB LOCKED
}
