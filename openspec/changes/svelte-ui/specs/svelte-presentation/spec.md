# Svelte Presentation

## Purpose

Define the shared Svelte rendering stack and preserve the operator experience across the desktop and frozen report.

## ADDED Requirements

### Requirement: Shared Svelte Presentation

The desktop and frozen report SHALL use Svelte 5 and share their document and plot components. The frontend presentation and its build SHALL have one component framework.

#### Scenario: Build Both Surfaces

- **WHEN** the desktop frontend and standalone report build from the workspace
- **THEN** both compile their shared Svelte components and the workspace graph contains no React presentation dependency

### Requirement: Desktop Workflow Parity

The desktop SHALL preserve its routes, shell and host-driven workflows. Interactive controls SHALL remain operable by keyboard and pointer. Tables SHALL preserve existing column sorting and omit generic search and pagination controls.

#### Scenario: Open and Restore a Desktop View

- **WHEN** an operator follows or directly opens a desktop route with route parameters and search state, then navigates back or forward
- **THEN** the desktop restores the corresponding view, selection and applicable search state

#### Scenario: Work With a Connected Host

- **WHEN** an operator plans and runs a cycle, answers a prompt, browses its evidence, or changes settings
- **THEN** the desktop displays the host's current state and issues the same actions through the existing client contracts

#### Scenario: Use a Menu and Sort a Table

- **WHEN** an operator uses keyboard navigation to activate a menu item and toggles an existing sortable column
- **THEN** the corresponding action runs and the rows display in the selected order
- **AND** the table does not present generic search or pagination controls

### Requirement: Visual and Plot Parity

The shared presentation SHALL preserve layout, label and title capitalization, units in square brackets, number formatting, theme tokens and plot interactions.

#### Scenario: Compare Both Themes

- **WHEN** the same desktop or report view is rendered for the same fixture in Light and Dark
- **THEN** each view preserves the corresponding layout, theme, labels and formatted values

#### Scenario: Explore Plot Evidence

- **WHEN** an operator inspects a plot, changes its window, zooms or uses playback
- **THEN** the plot uses the existing calculations and displays the corresponding frame, axes, markers, legend and summary

### Requirement: Offline Frozen Report

The report SHALL remain one self-contained HTML file using the current embedded capture and native decoder contract.

#### Scenario: Open a Saved Report Offline

- **WHEN** an operator opens a saved report in an isolated browser with network requests disabled
- **THEN** the embedded capture renders the recorded cycle, document and plots with its saved appearance
- **AND** the report uses no external assets or live host calls
