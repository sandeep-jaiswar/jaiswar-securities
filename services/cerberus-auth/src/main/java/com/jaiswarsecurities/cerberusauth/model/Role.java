package com.jaiswarsecurities.cerberusauth.model;

import jakarta.persistence.*;
import java.util.Objects;

@Entity
@Table(name = "roles")
public class Role {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(length = 100, unique = true, nullable = false) // Increased length
    private String name; // e.g., ROLE_TRADER_EQUITIES, ROLE_COMPLIANCE_OFFICER

    // Constructors
    public Role() {
    }

    public Role(String name) {
        this.name = name;
    }

    // Getters and Setters
    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Role role = (Role) o;
        return Objects.equals(name, role.name); // Business equality on name
    }

    @Override
    public int hashCode() {
        return Objects.hash(name);
    }

    @Override
    public String toString() {
        return "Role{" +
               "id=" + id +
               ", name='" + name + "'" + // Corrected this line
               '}';
    }
}
