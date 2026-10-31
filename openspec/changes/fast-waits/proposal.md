# Phase A Test Speed-Up

## Why

Integration and acceptance stories spend most of their runtime awaiting real
cadences. Independent native desktop workflows also serialize unnecessarily.

## What Changes

Use existing configuration and plugin settings for short fixture cadences,
replace avoidable polling with available progress signals, and reduce redundant
fixture work while retaining behavior and boundary coverage. Run native desktop
workflows concurrently with isolated roots and ports. Keep product deadlines,
checks, policies and story budgets unchanged. Harmos is outside this change.

## Impact

Dokime runtime, test fixtures, OBC plugin settings, desktop automation and
contributor guidance. Fresh baseline and final Windows traces provide per-story
comparisons and identify remaining upstream waits. Delivery is a local
Conventional Commit only, without pushing.
