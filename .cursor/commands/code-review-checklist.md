# Code Review Checklist

## Overview

Comprehensive checklist for conducting thorough code reviews to ensure quality, security, and maintainability. After going through the checklist, perform code changes for the issues found. Do this iteration 3 times. Perform for both the backend and frontend codebase and other folders present.

## Review Categories

### Functionality

- [ ] Code does what it's supposed to do
- [ ] Edge cases are handled
- [ ] Error handling is appropriate
- [ ] No obvious bugs or logic errors

### Code Quality

- [ ] Code is readable and well-structured
- [ ] Functions are small and focused
- [ ] Variable names are descriptive
- [ ] No code duplication
- [ ] Follows project conventions
- [ ] Cyclomatic complexity is very very low especially in Rust
- [ ] No dangling TODO comments
- [ ] No unused code

### Security

- [ ] No obvious security vulnerabilities
- [ ] Input validation is present
- [ ] Sensitive data is handled properly
- [ ] No hardcoded secrets
