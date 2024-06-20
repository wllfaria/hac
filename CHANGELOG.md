# Changelog

## [0.2.0] - 2024-06-20

### Bug Fixes

- Fixing resizing crashing when drawing hint
- Using chars().count() instead of len()
- Fixing broken tests

### Features

- Rendering request headers onscreen
- Selecting and moving around headers
- Deleting headers and help overlays
- Header forms editing and creation
- Sending headers through request
- Redesign of create request form
- Editing headers on the sidebar
- Creating directories on the sidebar
- Initial implementation of parent selector
- Deleting requests and headers
- Removing parent from a request
- Editing directories on the sidebar

### Refactor

- Separating body editor to request editor to implement headers
- Hints are now controlled by its own component

### Wip

- Headers pane rendering
- Header edit form without key handling

## [0.1.1] - 2024-06-03

### Bug Fixes

- Re-enabling tab switching on editor

### Miscellaneous Tasks

- Packaging hac as nix flake
- Adding how to try with nix to readme
- Release hac v0.1.1

### Refactor

- Moving response viewer logic to its module
- Creating a store to hold collection state
- Centralizing state management

### Testing

- Testing collection store

### Wip

- Fixing bugs and migrating logic

## [0.1.0] - 2024-05-30

### Bug Fixes

- Fixing broken tests for tree traversal
- Fixing screen manager tests failing
- Only showing cursor when editing the uri

### Features

- Synchronization problems are now gone
- Empty state for responses
- Enabling dry run and better readme

### Miscellaneous Tasks

- Renaming project to hac
- Setting up publishing to crates.io
- Setting packages metadata
- Renaming tui to client for publishing
- Release hac v0.1.0


