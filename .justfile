# List just recipes.
_help:
    just -l

# Run all tests using nextest.
@test:
    cargo nextest run

# Format with rust nightly
@fmt:
    cargo +nightly fmt

# Run the same checks we run in CI. Requires nightly.
@ci: test
    cargo clippy --all-targets -- -D warnings
    cargo +nightly fmt --check

# Clean everything up
@lint:
    cargo clippy --fix --allow-dirty
    cargo +nightly fmt

# Install required tools
@setup:
    brew tap ceejbot/tap
    brew install fzf cargo-nextest tomato semver-bump
    rustup install nightly

# Tag a new version for release.
version BUMP:
    #!/usr/bin/env bash
    set -e
    current=$(tomato get package.version Cargo.toml)
    version=$(semver-bump {{ BUMP }} "$current")
    tomato set package.version "$version" Cargo.toml &> /dev/null
    cargo generate-lockfile
    git commit Cargo.toml Cargo.lock -m "v${version}"
    git tag "v${version}"
    echo "Release tagged for version v${version}"
