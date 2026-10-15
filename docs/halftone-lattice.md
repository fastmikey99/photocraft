# Explicit raster-lattice halftone prototype

`image.mode.bitmap` accepts `method:"halftoneLattice"` with integer `latticeX`,
`latticeY`, `originX`, and `originY`. The basis vectors are `(a,b)` and `(-b,a)`;
the determinant `a²+b²` must be 4–65536. Origins refer to integer pixel indices,
not pixel centres. All processing remains in the normal engine command path.

This opt-in method is a clean-room prototype based on observed periodic bitmap
behaviour. It does **not** map Photoshop's nominal frequency/angle settings to
its raster lattice. The existing `halftone` method is unchanged.

The screen ranks the distinct lattice residues with a circular spot that becomes
complementary circular holes above half coverage. For signed cell coordinates
`u,v` in `[-0.5,0.5)`, the increasing area score is:

- `2*(u*u+v*v)` when `abs(u)+abs(v)<0.5`;
- `1-2*((0.5-abs(u))²+(0.5-abs(v))²)` otherwise.

The implementation calculates equivalent integer scores, sorts tied scores by
descending Cartesian Y then X, and uses threshold `(rank+0.5)/site_count`.
A pixel becomes black when `1-gray > threshold`. Pure black and pure white are
preserved. The calculation is depth independent; tests exercise 8/16/32-bit
command execution, invalid parameters, undo, lattice translations, coverage,
and endpoints. Setup work is bounded independently of image size.

The ordering of equal scores and threshold quantization are prototype conventions,
not a claim of exact Photoshop parity. A local reference comparison achieved
99.9314% binary-alpha agreement after reconstructing the source sampling geometry;
all residual disagreements were in partial-alpha pixels. No reference pixels or
measured per-phase threshold table are included in the algorithm or tests.
