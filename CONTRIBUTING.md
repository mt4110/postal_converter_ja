# Contributing to Postal Converter JA

First off, thanks for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to Postal Converter JA. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Code of Conduct

This project and everyone participating in it is governed by the [Postal Converter JA Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for Postal Converter JA. Following these guidelines helps maintainers and the community understand your report, reproduce the behavior, and find related reports.

- **Use the Bug Report Template**: When you create a new issue, please select the "Bug report" template.
- **Provide Reproducible Steps**: Include clear steps to reproduce the issue.
- **Check for Duplicates**: Before creating a new issue, please search existing issues to see if the problem has already been reported.

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Postal Converter JA, including completely new features and minor improvements to existing functionality.

- **Use the Feature Request Template**: When you create a new issue, please select the "Feature request" template.
- **Be Specific**: Describe the behavior you want and why.

### Pull Requests

1.  **Fork the repo** and create your branch from `develop`.
2.  **Ensure CI passes**: Run tests and linting locally before pushing.
    - Backend: `cargo test`
    - Frontend: `yarn lint`
    - Launcher: `go build ./...`
3.  **Use the PR Template**: Fill out the Pull Request template to provide context for your changes.
4.  **Link Issues**: If your PR fixes an issue, link it in the description (e.g., `Fixes #123`).

## Styleguides

### Git Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line
- Consider using [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat:`, `fix:`, `docs:`, `chore:`)

### Rust Styleguide

- Run `cargo fmt` before committing.
- Ensure `cargo clippy` passes without warnings.

### TypeScript/React Styleguide

- Run `yarn lint` to check for style issues.
- Use functional components and hooks.

Thank you for your contribution!
