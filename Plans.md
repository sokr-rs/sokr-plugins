# Plans.md — sokr-plugins Phase 1: sokr-cpu Implementation

## Active Sprint — sokr-cpu Phase 1 Complete ✓

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 1.1 | Implement `dispatch_fn` + `completion_fn` + `destroy_fn` (consolidated) | (a) dispatch_fn generates unique tokens, stores state (b) completion_fn polls token status correctly (c) destroy_fn clears slots (d) No panics in FFI (e) cargo build --release passes | — | cc:done [8b8150f] |
| 1.2 | Verify dispatch/completion round-trip with tests | test_1_2_dispatch_completion_roundtrip, test_1_2_multiple_concurrent_dispatches (5 dispatches) — all dispatch→complete workflows validated | 1.1 | cc:done [HEAD] |
| 1.3 | Verify destroy_fn invalidates plugin state with tests | test_1_3_destroy_invalidates_tokens, test_1_3_destroy_clears_all_slots (10 dispatches) — post-destroy queries return Failed | 1.1 | cc:done [HEAD] |
| 1.4 | Full integration test with sokr core | test_1_4_full_lifecycle (capability→dispatch→completion→destroy), test_1_4_multiple_computations (20 computations, interleaved polling) — roundtrip end-to-end validated | 1.1, 1.2, 1.3 | cc:done [HEAD] |

## Backlog

See TODO.md for Phase 2+ (GPU, CUDA, Metal, etc.).

## Archive

(Completed tasks moved here as sprints finish.)

---

**Harness Integration**: Use `/harness-work all` to execute all tasks in order (4 tasks → Breezing mode auto-selected).
