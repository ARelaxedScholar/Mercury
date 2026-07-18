# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-07-18

### Added
- Repeatable contributor verification gate covering formatting, default and all-feature targets, doctests, strict Clippy, and crate packaging.
- Feature-gated semantic LLM example that compile-checks the README's primary semantic workflow.
- State-machine validity specification defining structural invariants, optional policies, diagnostics, and static/runtime conformance requirements.
- Typed execution-semantics specification defining phase commits, failure ownership, mutation visibility, interruption, and retry boundaries.
- Implementation-level graph-definition and validation core covering state categories, route coverage, graph policies, and stable diagnostics.
- Graph-definition and sync/async execution-adapter architecture decisions for the future compiler-verified API.
- Experimental procedural macro with full structural validation, generated typestate execution skeletons, and compile-pass/fail coverage for graph and phase diagnostics.
- Typed lifecycle example covering retry cycles, terminal completion, absorbing cancellation, and complete runtime route registration.
- Additive recovery-capable typed execution methods that preserve source-phase state across node, transition, branch-dispatch, and branch-configuration failures.
- Experimental macro execution slice with fallible direct effects, exhaustive typed route dispatch, destination-typed outcomes, and recoverable mutation-before-error behavior.
- `experimental-graph` root-crate feature re-exporting the compiler-verified macro through lockstep-versioned implementation crates.
- Typed workflow branch builder API with framework-owned route-to-transition execution via `Branch::branch::<O>()`, `.on(...)`, `.on_finish(...)`, and `.finish()`.
- Explicit typed branch errors with `BranchBuildError<R>` and `BranchExecuteError<R, E>`.
- Typed prelude/module exports for branch builder types (`BranchBuilder`, `ConfiguredBranchBuilder`) and branch error types.

### Changed
- Migrated the typed workflow example and typed integration tests to the framework-owned branch builder path instead of ad hoc `resolve(...)` closures.
- Clarified typed workflow docs so compile-time guarantees cover phase legality for nodes, transitions, and branch handlers, while route coverage and finish coverage remain runtime-validated.
- Removed the inherited empty binary target and orphan root module from the published package.

### Fixed
- Synchronized `Cargo.nix` with the current lockfile and package version.
- Removed the obsolete Arrow compatibility pin from Chrono after eliminating unused Arrow dependencies.
- Corrected the semantic quick start and the exported `signature!` macro path for external callers.
- Removed unused `json`, Arrow, and Parquet dependencies, eliminating stale dependency advisories and reducing the release surface.
- Replaced inherited development-stage panic messages and comments with release-appropriate diagnostics and documentation.

### Notes
- `Branch::resolve(...)` remains available as an advanced escape hatch; the builder path is now the canonical typed branching API.
- The current branch builder intentionally uses boxed erased executors and linear runtime matching to keep heterogeneous branch arms simple in v1.

## [0.4.1] - 2026-04-21

### Fixed
- Restored default-feature builds and crate packaging by aligning feature gating with the code that depends on it.
- Resolved release-blocking formatting, clippy, rustdoc, and Nix toolchain issues so the release gate passes reproducibly in `nix develop`.
- Updated the transitive `lz4_flex` dependency to `0.11.6` and regenerated `Cargo.nix` to remove the yanked lockfile entry.

## [0.4.0] - 2026-02-10

### Added
- **Semantic Layer**: Introduced `Signature`, `Field`, and `Sealable` traits for defining structural contracts.
- **Sealed Nodes**: Added `SealedNode` for validated, globally identifiable task instances.
- **Telemetry**: Implementation of `Telemetry` trait with `MemoryTelemetry` for recording and inspecting execution traces.
- **Validation**: Added `ValidationIssue` and `ValidationResult` for flow-level contract verification.
- Builder pattern for LLM completion methods (`deepseek_complete`, `gemini_complete`, `ollama_complete`)
- Multi-turn fluent message support (`.system()`, `.user()`, `.assistant()`) in builders
- Implicit model validation with thread-safe caching
- Convenience default methods for client configuration (`with_deepseek`, `with_gemini`, `with_ollama`)
- Standardized model selection across all providers with best-in-class defaults (e.g., `gemini-1.5-flash`, `deepseek-chat`, `phi4`)
- Model discovery API (`list_models`) for each provider

### Changed
- Refactored LLM client configuration to use simpler default methods
- Moved custom URL configuration to `with_*_at` methods
- Standardized builder-based completions across all providers

### Fixed
- Improved API discoverability and reduced boilerplate in common use cases

## [0.3.0] - 2025-12-26

### Added
- AsyncFlow: full asynchronous flow implementation for async node orchestration
- AsyncNode: asynchronous node logic with async trait support
- AsyncBatchNode: batch processing for async nodes
- AsyncParallelBatchNode: parallel batch processing for async nodes
- Flake.nix and cargo2nix support for Nix users
- More professional README with comprehensive examples
- Convenience function to get successors from any node
- Edit function to Ollama LLM client (feature-gated)

### Changed
- Node now expects `Executable` instead of `Node` as next step, enabling mixed sync/async workflows
- Flow is now aware of `Executable` types (though synchronous flow still only handles sync nodes)
- Improved internal architecture with better separation of sync and async implementations

### Fixed
- Fixed lifetime errors in AsyncFlow implementation
- Fixed logic for sequential async batch processing
- Fixed trait bound on wrong struct that broke client-side functionality
- Cleaned up code with clippy fixes

### Notes
- This release introduces a complete async counterpart to the existing synchronous API.
- The async API is feature-complete with parallel batch processing capabilities.
- The crate now supports Nix-based development environments via flake.nix.
