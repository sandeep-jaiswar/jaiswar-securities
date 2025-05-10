package com.jaiswarsecurities.cerberusauth.controller;

import org.springframework.http.ResponseEntity;
import org.springframework.security.core.Authentication;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.security.core.userdetails.UserDetails;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api")
public class TestController {

    @GetMapping("/greeting")
    public ResponseEntity<String> getGreeting() {
        Authentication authentication = SecurityContextHolder.getContext().getAuthentication();
        String currentPrincipalName;
        if (authentication != null && authentication.getPrincipal() instanceof UserDetails) {
            UserDetails userDetails = (UserDetails) authentication.getPrincipal();
            currentPrincipalName = userDetails.getUsername();
        } else if (authentication != null) {
            currentPrincipalName = authentication.getName();
        } else {
            currentPrincipalName = "Unknown User"; // Should not happen if endpoint is secured
        }
        return ResponseEntity.ok("Hello, " + currentPrincipalName + "! This is a secured endpoint.");
    }
}
