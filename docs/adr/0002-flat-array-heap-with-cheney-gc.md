# Heap: flat-array cells with a Cheney two-space copying GC

## Context

The ION-style VM reduces the program on a heap of cells that reference each other, updated in place to realise sharing. Lazy K programs process streams indefinitely, so unreferenced cells must be reclaimed, and lazy fixpoints can create cyclic structure. Performance is a goal.

## Decision

Represent the heap as a flat array of fixed-size cells addressed by index, and reclaim with a **Cheney two-space copying collector**. Roots are the spine stack plus the input/output cursors. Allocation is a bump pointer into to-space; collection copies live cells to the other semi-space and compacts.

## Considered options

- **Reference counting** — deterministic and Rust-friendly, but lazy fixpoints can build cycles that leak. Rejected: we can't guarantee acyclicity for arbitrary programs.
- **Mark-sweep + free list** — no 2× space, simpler roots, but no compaction, so locality degrades over long runs.

## Consequences

- Costs 2× address space and requires enumerating roots from the spine stack and I/O cursors.
- Compaction restores cache locality; the collector can shortcut indirection chains while copying.
- Cell layout must be uniform and support in-place update plus an indirection tag.
