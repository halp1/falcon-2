# Expansion Floodfill Optimization Plan

High-level optimizations for `expand_floodfill`, ranked by impact:

1. Collapse redundant flood-fills into incremental propagation  
   • Instead of re-running a full floodfill from each new seed, track “frontier” bits and only floodfill from those, OR’ing their reach into the master mask.

2. Precompute and cache floor-drop positions  
   • Build a small lookup table for “drop to floor” per column-height combination so you can replace the trailing_zeros+bitmask operations with a single table lookup.

3. Pre-flatten and inline kick data  
   • Expand `config.kicks.data(...)` into a static table at startup so the inner kick loops become simple array accesses, avoiding iterator overhead.

4. Move floodfill into the main loop (inline)  
   • Inlining or macros eliminate the function call overhead for `floodfill` and let the compiler better optimize its hot inner loops.

5. Bit-parallel shifts across all columns  
   • Instead of shifting one `u64` at a time, leverage vector types (e.g. SIMD) to shift multiple columns in parallel.

6. Reuse a single work-stack and bitset buffers  
   • Allocate `stack`, `explored`, `newly` and floodfill buffers once (e.g. as thread-local) to avoid zeroing large arrays on every call.

7. Use unsafe intrinsics for trailing_zeros and bit-scans  
   • On supported targets, call LLVM intrinsics or CPU instructions directly to minimize the cost of bit operations.

8. Parallelize rotations/kicks with Rayon  
   • Split the 4 rotations (or kick pairs) across threads to leverage multi-core CPUs, if allowed by game logic.

9. Special-case very common pieces/configs  
   • For the I- and Z-pieces, hand-tune a specialized routine that fuses kicks and floodflow in one pass.
