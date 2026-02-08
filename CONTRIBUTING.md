# Contributing to ParkHub

Thank you for considering a contribution to ParkHub! This guide will help you get started.

## How to Contribute

1. **Fork** the repository
2. **Create a branch** from `main`: `git checkout -b feat/my-feature`
3. **Make your changes** with clear, focused commits
4. **Test** your changes: `cargo test` and `cd parkhub-web && npm test`
5. **Push** your branch and open a **Pull Request**

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add waitlist notification emails
fix: correct slot availability calculation
docs: update API reference for bookings
chore: upgrade dependencies
refactor: simplify auth middleware
```

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. By participating, you agree to:

- Be respectful and constructive in all interactions
- Welcome newcomers and help them contribute
- Focus on what is best for the community
- Show empathy towards other community members

Unacceptable behavior includes harassment, trolling, personal attacks, and publishing private information without consent.

## Pull Request Process

1. Ensure your PR targets the `main` branch
2. Include a clear description of what changed and why
3. Link any related issues (`Fixes #123`)
4. Ensure CI passes (lint, test, build)
5. Request a review from a maintainer
6. Squash commits before merge if requested

## Reporting Bugs

Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md) and include:

- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, browser, ParkHub version)
- Screenshots if applicable

## Requesting Features

Use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md) and describe:

- The problem you're trying to solve
- Your proposed solution
- Alternatives you've considered

## Development Setup

See the [Development Guide](docs/DEVELOPMENT.md) for setting up your local environment.

---

Thank you for helping make ParkHub better!
