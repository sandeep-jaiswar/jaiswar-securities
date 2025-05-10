package com.jaiswarsecurities.cerberusauth.service;

import com.jaiswarsecurities.cerberusauth.model.AccountStatus; // Import AccountStatus
import com.jaiswarsecurities.cerberusauth.model.User;
import com.jaiswarsecurities.cerberusauth.repository.UserRepository;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.security.core.GrantedAuthority;
import org.springframework.security.core.authority.SimpleGrantedAuthority;
import org.springframework.security.core.userdetails.UserDetails;
import org.springframework.security.core.userdetails.UserDetailsService;
import org.springframework.security.core.userdetails.UsernameNotFoundException;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.Set;
import java.util.stream.Collectors;

@Service
public class DatabaseUserDetailsService implements UserDetailsService {

    private final UserRepository userRepository;

    @Autowired
    public DatabaseUserDetailsService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    @Override
    @Transactional(readOnly = true)
    public UserDetails loadUserByUsername(String username) throws UsernameNotFoundException {
        User user = userRepository.findByUsername(username)
                .orElseThrow(() -> 
                    new UsernameNotFoundException("User not found with username: " + username)
                );

        Set<GrantedAuthority> authorities = user.getRoles().stream()
                .map(role -> new SimpleGrantedAuthority(role.getName()))
                .collect(Collectors.toSet());

        // Refined logic for UserDetails flags based on User entity's state
        boolean isEnabled = user.isEnabled(); // Master switch
        
        // Account is considered "non-locked" if its status isn't one that implies a lock or suspension.
        // The `enabled` flag is the primary gatekeeper.
        // If `enabled` is true, then we check `accountStatus` for non-locking states.
        boolean accountNonLocked = user.getAccountStatus() != AccountStatus.LOCKED_BAD_CREDENTIALS &&
                                   user.getAccountStatus() != AccountStatus.SUSPENDED;
                                   // DEACTIVATED and PENDING_APPROVAL should result in user.isEnabled() == false
                                   // due to logic in User.java

        // Placeholder for password expiry logic (e.g., based on user.getPasswordLastChanged())
        boolean credentialsNonExpired = true; 
        // Placeholder for overall account expiry logic (e.g., contract end date for temp employees)
        boolean accountNonExpired = true; 

        return new org.springframework.security.core.userdetails.User(
                user.getUsername(),
                user.getPassword(),
                isEnabled,           // Directly from our User entity's 'enabled' field
                accountNonExpired,   // For now, true. Implement if needed.
                credentialsNonExpired, // For now, true. Implement if needed.
                accountNonLocked,    // Refined logic based on accountStatus
                authorities
        );
    }
}
