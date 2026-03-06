# Phase 2.95 Execution Tracker

Phase: Performance, Platform Reach & UX Completeness
Status: Ready for Implementation
Started: 2026-03-06

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Install Scripts | WP-5 | ISC-001~007 | pending |
| 2 | Remove All Symbol | WP-2 | RMA-001~004 | pending |
| 3 | Windows Junction Fallback | WP-3 | WJN-001~006 | pending |
| 4 | Performance Part 1: Repo-Level Cache | WP-1 pt1 | PSY-001~003, PSY-006~007 | pending |
| 5 | Performance Part 2: Batch Sync + Migration | WP-1 pt2 | PSY-004~006, PSY-008 | pending |
| 6 | Docker Bind Mount + Agent Auto-Detection | WP-4 | DBM-001~007 | pending |
| 7 | Regression + Closeout | — | TM regression | pending |

## Dependency Constraints

- Batches 1, 2, 3, 4, 6 are independent of each other.
- Batch 5 depends on Batch 4 (repo-level cache infrastructure).
- Batch 7 depends on all preceding batches.

## Completion Records

(Populated by Builder after each batch.)
