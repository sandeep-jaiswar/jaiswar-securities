package com.jaiswarsecurities.cerberusauth.repository;

import com.jaiswarsecurities.cerberusauth.model.User;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.EntityGraph;
import org.springframework.stereotype.Repository;

import java.util.Optional;

@Repository
public interface UserRepository extends JpaRepository<User, Long> {

    // Find by username, also fetch roles eagerly to avoid N+1 issues in UserDetailsService
    @EntityGraph(attributePaths = "roles")
    Optional<User> findByUsername(String username);

    Optional<User> findByEmail(String email);

    Optional<User> findByEmployeeId(String employeeId);

    Boolean existsByUsername(String username);

    Boolean existsByEmail(String email);

    Boolean existsByEmployeeId(String employeeId);
}
