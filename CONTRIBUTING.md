# Contributing to Kayfabe

Thank you for your interest in contributing to Kayfabe! This document provides guidelines and instructions for contributing.

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/sarangat/kayfabe.git
   cd kayfabe
   ```

2. **Install Rust** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

## Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Follow Rust 2021 edition idioms
- Write clear, self-documenting code with comments where needed

## Testing

- Add tests for new functionality
- Ensure all existing tests pass
- Test on both macOS and Linux if possible

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in present tense (e.g., "Add", "Fix", "Update")
- Reference issues when applicable

## Reporting Issues

- Use the GitHub issue tracker
- Provide clear reproduction steps
- Include system information (OS, Rust version)
- Attach relevant logs or error messages

## Feature Requests

- Open an issue with the "enhancement" label
- Describe the use case and expected behavior
- Be open to discussion and feedback

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Help maintain a positive community

## Questions?

Feel free to open an issue for questions or join our discussions!
