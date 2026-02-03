# Implementation Plan: CI/CD Testing Enhancement

## Overview

This implementation plan transforms the current sequential CI/CD pipeline into a modular, concurrent system using GitHub Actions reusable workflows. The approach follows a bottom-up strategy: first implementing component workflows (reusable building blocks), then entry-point workflows (orchestrators). Workflow correctness is validated through actionlint and real-world usage.

## Tasks

- [x] 1. Create component workflows for individual test categories
  - [x] 1.1 Implement pre-commit validation component workflow
    - Create `.github/workflows/pre-commit.yml` with workflow_call trigger
    - Accept `ref` input parameter
    - Set up Python and pre-commit environment
    - Implement cache for pre-commit environments
    - Run pre-commit hooks on all files
    - _Requirements: 1.3, 1.6, 4.1, 4.2, 4.3, 4.4, 4.5, 12.1, 12.2_
  
  - [x] 1.2 Implement code formatting component workflow
    - Create `.github/workflows/format.yml` with workflow_call trigger
    - Accept `ref` input parameter
    - Install Rust toolchain with rustfmt component
    - Implement cargo cache strategy
    - Run `cargo fmt --all -- --check`
    - _Requirements: 1.3, 1.7, 5.1, 5.2, 5.3, 5.4, 12.1, 12.2_
  
  - [x] 1.3 Implement security audit component workflow
    - Create `.github/workflows/security-audit.yml` with workflow_call trigger
    - Accept `ref` input parameter
    - Install cargo-audit tool
    - Implement cargo cache strategy
    - Run `cargo audit` against RustSec Advisory Database
    - _Requirements: 1.3, 1.7, 6.1, 6.2, 6.3, 6.4, 6.5, 12.1, 12.2_
  
  - [x] 1.4 Implement linting component workflow
    - Create `.github/workflows/lint.yml` with workflow_call trigger
    - Accept `ref` input parameter
    - Install Rust toolchain with clippy component
    - Implement cargo cache strategy
    - Run `cargo clippy --all-targets --all-features -- -D warnings`
    - _Requirements: 1.3, 1.7, 11.1, 11.2, 11.3, 11.4, 11.5, 12.1, 12.2_
  
  - [x] 1.5 Implement build component workflow with platform matrix
    - Create `.github/workflows/build.yml` with workflow_call trigger
    - Accept `platform`, `ref`, and `release-mode` input parameters
    - Install Rust toolchain
    - Implement cargo cache strategy
    - Build in debug mode when release-mode is false
    - Build in release mode when release-mode is true
    - Upload build artifacts with platform-specific naming
    - _Requirements: 1.3, 1.7, 1.9, 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 12.1, 12.2, 12.4_

- [x] 2. Create component workflows for test execution
  - [x] 2.1 Implement unit tests component workflow
    - Create `.github/workflows/unit-tests.yml` with workflow_call trigger
    - Accept `ref` input parameter
    - Install Rust toolchain
    - Implement cargo cache strategy
    - Run `cargo test --lib --bins --verbose`
    - Output test count and status
    - _Requirements: 1.3, 1.7, 8.1, 8.2, 8.3, 8.4, 8.5, 12.1, 12.2_
  
  - [x] 2.2 Implement property-based tests component workflow
    - Create `.github/workflows/property-tests.yml` with workflow_call trigger
    - Accept `ref` input parameter
    - Install Rust toolchain
    - Implement cargo cache strategy
    - Run `cargo test --test '*' --verbose` with PROPTEST_CASES=100
    - Output test count and status
    - _Requirements: 1.3, 1.7, 7.1, 7.2, 7.3, 7.4, 7.5, 12.1, 12.2_
  
  - [x] 2.3 Implement coverage measurement component workflow
    - Create `.github/workflows/coverage.yml` with workflow_call trigger
    - Accept `ref`, `min-line-coverage`, `min-branch-coverage`, `min-function-coverage` inputs
    - Install Rust toolchain with llvm-tools-preview component
    - Install cargo-llvm-cov
    - Implement cargo cache strategy
    - Generate coverage with `cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info`
    - Generate JSON coverage report
    - Parse line, branch, and function coverage percentages
    - Validate coverage against threshold inputs
    - Fail if any threshold is not met
    - Upload coverage artifacts
    - _Requirements: 1.3, 1.7, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 12.1, 12.2_

- [x] 3. Checkpoint - Verify component workflows
  - Ensure all component workflow files are created with correct structure
  - Verify workflow_call triggers are properly configured
  - Ensure all tests pass, ask the user if questions arise

- [x] 4. Create release management component workflows
  - [x] 4.1 Implement release tagging component workflow
    - Create `.github/workflows/release-tag.yml` with workflow_call trigger
    - Accept `ref`, `branch`, and `tag-type` input parameters
    - Checkout with full history (fetch-depth: 0)
    - Extract major.minor version from release branch name
    - Find latest vX.Y.N tag (non-RC) for the release branch
    - Calculate next patch version by incrementing N
    - Find latest vX.Y.(N+1)-rcM tag
    - Calculate next RC number by incrementing M
    - Create and push new RC tag vX.Y.(N+1)-rc(M+1)
    - Output the created tag name
    - _Requirements: 14.2, 14.4, 14.5, 14.6, 14.7, 14.8, 14.9, 14.15, 14.16_
  
  - [x] 4.2 Implement changelog generation component workflow
    - Create `.github/workflows/changelog.yml` with workflow_call trigger
    - Accept `ref` and `tag` input parameters
    - Checkout with full history (fetch-depth: 0)
    - Find previous release tag (excluding RCs)
    - Generate changelog using git log between previous and current tag
    - Format changelog in markdown
    - Upload changelog as artifact
    - _Requirements: 14.13_
  
  - [x] 4.3 Implement version alias tag component workflow
    - Create `.github/workflows/version-alias-tags.yml` with workflow_call trigger
    - Accept `tag` (vX.Y.Z) and `ref` input parameters
    - Checkout with full history (fetch-depth: 0)
    - Extract major.minor from tag (vX.Y.Z → X.Y)
    - Find latest vX.Y.N tag (non-RC) for major.minor version
    - If input tag is latest, create or update vX.Y alias tag pointing to it
    - Find latest vX.Y.Z tag (non-RC) for major version
    - If input tag is latest, create or update vX alias tag pointing to it
    - Use force-push to update existing alias tags
    - Output the updated alias tag names
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.7_

- [x] 5. Create entry-point workflow for pull requests
  - [x] 5.1 Implement pull-request-default entry-point workflow
    - Create `.github/workflows/pull-request-default.yml`
    - Configure pull_request trigger for default branch
    - Configure workflow_dispatch trigger with `ref` and `branch` inputs
    - Call pre-commit component workflow
    - Call format component workflow
    - Call security-audit component workflow
    - Call lint component workflow
    - Call build component workflow with matrix for [ubuntu-latest, macos-latest, windows-latest]
    - Call unit-tests component workflow with needs: build dependency
    - Call property-tests component workflow with needs: build dependency
    - Call coverage component workflow with needs: [unit-tests, property-tests] and 90% thresholds
    - Pass ref input to all component workflows using `${{ inputs.ref || github.ref }}`
    - _Requirements: 1.1, 1.2, 1.4, 1.5, 1.8, 1.9, 1.10, 1.11, 1.12, 2.1, 2.2, 2.3, 3.1, 3.3, 3.4, 13.1, 13.2, 13.3, 13.4, 13.5_

- [x] 6. Create entry-point workflows for release management
  - [x] 6.1 Implement release-candidate entry-point workflow
    - Create `.github/workflows/release-candidate.yml`
    - Configure push trigger for branches matching `release/v*`
    - Configure workflow_dispatch trigger with `ref` and `branch` inputs
    - Implement validate-issue-reference job that runs first
    - In validation job, fetch all history and get PR commits between base and head
    - Search commit messages for issue references using pattern: (close[sd]?|fix(e[sd])?|resolve[sd]?|related-to)\s+#([0-9]+)
    - Extract issue numbers from matched references
    - Use gh CLI to verify at least one referenced issue exists and is in OPEN state
    - Fail validation if no valid open issue reference is found
    - Call pull-request-default workflow with needs: validate-issue-reference dependency
    - Call release-tag component workflow with needs: ci-checks dependency
    - Pass tag-type: release-candidate to release-tag workflow
    - _Requirements: 1.1, 1.2, 1.4, 1.10, 1.11, 1.12, 14.1, 14.3, 14.4, 14.5, 14.6, 14.7, 14.13_
  
  - [x] 6.2 Implement release-publish entry-point workflow
    - Create `.github/workflows/release-publish.yml`
    - Configure push trigger for tags matching `v[0-9]+.[0-9]+.[0-9]+` (exact semver only, excludes RCs and aliases)
    - Configure workflow_dispatch trigger with `tag` input
    - Implement verify-tag job that runs first
    - In verify-tag job, fetch all history and extract version components from tag
    - Verify tagged commit exists within corresponding release/vX.Y branch
    - Check that tagged commit is reachable from release branch head using git merge-base --is-ancestor
    - For tags vX.Y.Z where Z > 0, verify previous tag vX.Y.(Z-1) exists
    - For tags vX.Y.Z where Z > 0, verify previous tag is direct ancestor of current tag
    - Skip ancestry check for vX.Y.0 tags (initial release in series)
    - Fail workflow if any verification check fails with clear error message
    - Call pull-request-default workflow with needs: verify-tag dependency
    - Call build component workflow with matrix for all platforms and release-mode: true
    - Call changelog component workflow
    - Implement publish-release job with needs: [build-artifacts, generate-changelog]
    - Download all build artifacts
    - Create GitHub release using gh CLI with changelog and artifacts
    - Call version-alias-tags component workflow with needs: publish-release
    - Pass tag and ref inputs to version-alias-tags workflow
    - _Requirements: 1.1, 1.2, 1.4, 1.10, 1.11, 1.12, 14.1, 14.14, 14.15, 14.16, 14.17, 14.18, 14.19, 14.20, 14.21, 14.22, 14.23, 14.26, 15.1, 15.2, 15.5, 15.6_

- [x] 7. Checkpoint - Verify entry-point workflows
  - Ensure all entry-point workflows are created with correct structure
  - Verify triggers are properly configured
  - Verify component workflow calls use correct inputs
  - Ensure all tests pass, ask the user if questions arise

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- The implementation follows a bottom-up approach: component workflows first, then entry-points
- All workflows use aggressive caching to improve CI performance
- Matrix strategies enable concurrent execution across platforms and test categories
- Workflow correctness is validated through actionlint and real-world usage
