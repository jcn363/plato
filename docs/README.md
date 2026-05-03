# Plato Documentation

This directory contains planning, architecture, and design documentation for the Plato project.

## User Documentation

For user-facing documentation, see the [doc/](../doc/) directory:

- [Installation Guide](../doc/GUIDE.md)
- [User Manual](../doc/MANUAL.md)
- [Build Instructions](../doc/BUILD.md)
- [Library Management](../doc/LIBRARY.md)
- [Hooks Configuration](../doc/HOOKS.md)
- [Article Fetcher](../doc/ARTICLE_FETCHER.md)
- [OCR and TTS](../doc/OCR_TTS.md)
- [Navigation Guide](../doc/NAVIGATION.md)
- [Not Implemented Features](../doc/NOT_IMPLEMENTED.md)
- [PDF Features](../doc/PDF_FEATURES.md)
- [OPDS Catalog](../doc/OPDS_CATALOG.md)
- [Theme System](../doc/THEME_AWARE.md)

## Project Documentation

Root level documentation:

- [README.md](../README.md) - Project overview and quick start
- [CHANGES.md](../CHANGES.md) - Recent changes and implementation progress
- [DEVELOPMENT_SETUP.md](../DEVELOPMENT_SETUP.md) - Development environment setup
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contributing guidelines
- [AGENTS.md](../AGENTS.md) - AI coding agent guidance
- [API_OVERVIEW.md](API_OVERVIEW.md) - Developer reference (traits, errors, helpers)
- [TESTING.md](TESTING.md) - Testing guide (running tests, mocks, linting)

## Crate‑level READMEs

Each top‑level crate now has its own `README.md` describing purpose, public API surface and optional features:

- [core/README.md](../crates/core/README.md) – `plato‑core` library
- [ai/README.md](../crates/ai/README.md) – `plato‑ai` embeddings & LLM providers
- [thumbnail/README.md](../crates/thumbnail/README.md) – `plato‑thumbnail` background thumbnail generation
- [plato‑android/README.md](../crates/plato-android/README.md) – Android‑specific glue
- [plato‑view/README.md](../crates/plato-view/README.md) – UI view‑tree infrastructure

## Planning Documents

- [ROADMAP.md](ROADMAP.md) - Consolidated roadmap of active and planned items

### Architecture & Design

- [PLAN.md](PLAN.md) - Overall project plan
- [PLAN2.md](PLAN2.md) - Updated project plan

### Active Plans

- [APPLE-PLAN.md](APPLE-PLAN.md) - iPhone and iPad support plan

### Architecture

- [architecture/](architecture/) - Architecture documentation

## Feature‑specific documentation

- [AI Integration](AI_INTEGRATION.md) – provider abstraction, embeddings, cache.
- [Thumbnail System](THUMBNAIL_SYSTEM.md) – worker pool, LRU cache, sizing.
- [Text‑to‑Speech](TTS.md) – desktop & Android implementations, limitations.

## Documentation Standards

When creating or updating documentation:

- Use clear, concise language
- Include examples where appropriate
- Keep documentation up-to-date with code changes
- Use Markdown formatting consistently
- Add diagrams for complex concepts when helpful
- Cross-reference related documents
