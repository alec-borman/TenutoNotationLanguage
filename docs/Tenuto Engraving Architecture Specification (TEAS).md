# Tenuto Engraving Architecture Specification (TEAS)

**Target:** `tenuto-engrave` (Rust Native Render Engine)  
**Status:** Normative Draft / Phase VII Architecture  
**License:** MIT  

---

## 1. Abstract & Scope

The `tenuto-engrave` crate acts as the final typographical backend for the Tenuto Compiler (`tenutoc`). It receives a mathematically resolved `VisualScore` (containing absolute ticks, spelled pitches, and measure grids) and translates it into deterministic, production-grade Scalable Vector Graphics (SVG).

### 1.1 The Legacy Paradigm Failure
Traditional notation engines (LilyPond, MuseScore) rely on Object-Oriented Programming (OOP). In OOP, a `Measure` object owns a `Chord` object, which owns a `Note` object, which owns a `Stem` object. This creates fatal flaws:
*   **Cyclic Dependencies:** If a cross-staff beam connects a note in the Treble staff to a note in the Bass staff, traversing the OOP tree requires massive pointer indirection and violates Rust’s strict aliasing rules.
*   **Cache Misses:** Trees allocate memory randomly on the heap, destroying CPU cache locality.
*   **Monolithic Re-renders:** Changing a single note forces a complete recalculation of the entire page layout.

### 1.2 The Tenuto Engraving Paradigm
To achieve sub-50ms rendering for 100-page symphonies, `tenuto-engrave` implements four cutting-edge computer science paradigms:
1.  **Data-Oriented Design (ECS/Arenas):** Flat memory arrays for zero-cost graph traversals.
2.  **Incremental Computation (`salsa`):** Memoized query graphs for real-time, partial re-rendering.
3.  **Linear Constraint Solving:** Utilizing the Cassowary algorithm for optimal horizontal spring-mass justification.
4.  **SIMD-Accelerated Skylines:** Quantized, vectorized 1D arrays for instantaneous vertical collision detection.

---

## 2. The Incremental Memory Model

To satisfy Rust’s borrow checker and achieve real-time responsiveness (crucial for the future `tenutod` live-coding daemon), the engine strictly forbids hierarchical memory ownership.

### 2.1 Generational Arenas (`slotmap`)
Every musical element (Notehead, Stem, Beam, Slur, Lyric) is instantiated as a flat `Entity` inside a tightly packed array called an Arena.

*   **Identifiers:** Entities reference each other exclusively via `EntityId` (a 64-bit integer combining an index and a generation counter). 
*   **Safety:** If a note is deleted, its generation counter increments. If a beam attempts to query the deleted note using the old ID, it safely returns `None` instead of triggering a segfault or dangling pointer.
*   **Cache Locality:** Because all `Notehead` data lives in one contiguous array, the CPU can calculate the bounding boxes for an entire measure in a single cache line fetch.

### 2.2 The Incremental Query Graph (`salsa`)
The engraving pipeline is constructed as a Directed Acyclic Graph (DAG) of pure functions (Queries) managed by the `salsa` crate.

**The Introspection Loop:**
Layout requires cyclic logic (e.g., *a beam's slope depends on the stems, but the stems' lengths depend on the beam's slope*). `salsa` tracks dependencies automatically.

1.  **Input:** The user alters `measure 5`.
2.  **Invalidation:** `salsa` invalidates the `BoundingBox` query for `measure 5`.
3.  **Cascade:** It checks if the new bounding box exceeds the `LineBreak` threshold. If it does not, the `LineBreak` query remains cached.
4.  **Result:** The engine instantly outputs the new SVG by recalculating *only* the pixels inside `measure 5`, skipping the other 99 measures entirely.

---

## 3. The Entity Taxonomy (The Relational Graph)

Entities in the `tenuto-engrave` ECS are categorized into distinct structural tiers. They do not hold data directly; instead, they hold IDs pointing to components.

### 3.1 Structural Entities
Entities that define the invisible layout grid.
*   **`SystemId`:** A single horizontal line of music across the page.
*   **`StaffId`:** A single 5-line string within a System.
*   **`TimeColumnId`:** An absolute horizontal X-coordinate shared by simultaneous events across all staves in a System (the foundation of the Spring-Rod model).

### 3.2 Graphical Entities (Glyphs & Paths)
Entities that result in physical ink on the page.
*   **`GlyphId`:** A static SMuFL character (e.g., Notehead, Clef, Rest). Contains a `smufl_code` and an `(x, y)` relative offset.
*   **`SpannerId`:** A continuous graphical element connecting two `TimeColumnId`s (e.g., Beams, Slurs, Hairpins, Octave Lines). Contains `kurbo::BezPath` data.

### 3.3 The Component Tables
Instead of a `Note` struct holding a `Stem`, the ECS maintains parallel arrays mapping IDs to physical properties:

```rust
pub struct EcsWorld {
    // Relationships
    pub parent_column: SecondaryMap<GlyphId, TimeColumnId>,
    pub staff_membership: SecondaryMap<TimeColumnId, StaffId>,
    
    // Geometry
    pub absolute_x: SecondaryMap<TimeColumnId, f64>,
    pub absolute_y: SecondaryMap<StaffId, f64>,
    
    // SMuFL Data
    pub smufl_metadata: SecondaryMap<GlyphId, SmuflGlyph>,
}
```

By decoupling the data, a "System" can iterate exclusively over `absolute_x` to solve horizontal spacing without ever loading font metadata into the CPU cache, maximizing throughput.



## 4. The SMuFL Subsystem (Standard Music Font Layout)

To achieve deterministic, pixel-perfect engraving, the engine must never "guess" where to place a symbol. It must rely on a rigorous mapping of musical semantics to physical vectors. `tenuto-engrave` natively implements the W3C **SMuFL 1.4 Specification**, utilizing the reference font *Bravura*.

### 4.1 Coordinate Space & Units

All internal calculations in `tenuto-engrave` operate in a universal, resolution-independent unit called the **Staff Space (`ss`)**.
*   **Definition:** `1.0 ss` is exactly the vertical distance between two adjacent lines on a standard 5-line musical staff.
*   **Origin:** The local origin `(0.0, 0.0)` for any glyph is standardly placed at the horizontal center and vertical baseline (usually resting on the bottom staff line).
*   **Y-Axis Inversion:** In SVG, Y increases downwards. In musical coordinates (and SMuFL), Y increases *upwards*. The final SVG renderer applies an affine transform matrix `[1 0 0 -1 0 0]` to flip the Y-axis at export time, allowing the core engine to perform math where "higher pitch = higher Y value."

### 4.2 Ingesting `metadata.json`

At initialization, the engine parses the font's SMuFL `metadata.json` into an $O(1)$ lookup HashMap, structurally mapped to the `SmuflGlyph` data type.

#### 4.2.1 The Axis-Aligned Bounding Box (AABB)
The fundamental unit of collision detection. The metadata defines the exact extents of the ink.
```rust
pub struct BoundingBox {
    pub sw: (f64, f64), // South-West corner (e.g.,[-0.58, -0.16])
    pub ne: (f64, f64), // North-East corner (e.g., [ 0.58,  0.16])
}
```

#### 4.2.2 Attachment Anchors
SMuFL provides strict floating-point coordinates for attaching disparate graphical elements. The engine queries these anchors to build composite objects without mathematical guesswork.
*   **Stem Connections:** `stemUpSE` and `stemDownNW` define exactly where a vertical stem attaches to a specific notehead design (e.g., placing the stem flush against the right edge of a quarter note, but slightly inset for a whole note).
*   **Optical Centers:** `opticalCenter` defines the visual weight of irregular glyphs (like flat signs), used for horizontally centering elements over a `TimeColumnId`.

### 4.3 Optical Kerning and Cut-Outs

Standard AABBs are sufficient for gross collision avoidance but fail at professional typography. For instance, the bounding box of a Flat sign (`b`) contains massive negative space in its upper-right quadrant. If a whole notehead uses AABB collision against a Flat sign, they will be forced unnaturally far apart.

To solve this, `tenuto-engrave` implements **SMuFL Cut-outs**.

*   **Definition:** A cut-out is a predefined polygonal sub-region that trims away the negative space from an AABB.
*   **The Intersection Algorithm:** When evaluating the horizontal `Rod` length (minimum distance) between a Flat sign and a Notehead, the engine intersects the Notehead's left bounding edge against the Flat sign's cut-out polygon.
*   **Result:** The rounded left edge of the notehead visually "tucks" underneath the overhanging top hook of the flat sign, achieving true optical kerning and saving critical horizontal page space.

---

## 5. The Spring-Mass Model (Horizontal Spacing)

Music is not monospaced. The spacing between a whole note and a quarter note is not a 4:1 linear ratio; it is logarithmic. `tenuto-engrave` translates time into space using the **Spring-Rod Physical Model** (based on Gourlay's algorithm).

### 5.1 The Elements of Tension

Within a `SystemId`, every unique onset tick across all staves forms a vertical `TimeColumnId`. The layout engine models the horizontal distance between these columns mechanically.

1.  **Rods (Rigid Constraints):** A Rod defines the absolute minimum physical distance between two columns required to prevent glyph collisions. 
    *   *Calculation:* `RodLength = BBox(Column_A_Right) + BBox(Column_B_Left) + OpticalPadding`.
    *   *Rule:* The physical distance can **never** be less than the Rod length.
2.  **Springs (Flexible Constraints):** A Spring defines the ideal, un-stretched optical space between two columns based on their musical duration.
    *   *Calculation:* `SpringLength = BaseSpace * (Duration / BaseDuration)^K`.
    *   *(K is the optical scaling factor, typically `0.5` to `0.6` logarithmically).*

### 5.2 The Cassowary Constraint Solver

Instead of writing a custom, iterative physics loop to calculate how the springs compress against the rods, `tenuto-engrave` treats the entire system as a matrix of linear inequalities, solvable by the **Cassowary Algorithm** (the same algorithm powering Apple's iOS Auto Layout).

The ECS feeds the following rules into the Cassowary solver:
*   **Constraint 1 (Rods):** `Column[i+1].x - Column[i].x >= RodLength`
*   **Constraint 2 (Springs):** `Column[i+1].x - Column[i].x == IdealSpringLength * SystemStretchFactor`
*   **Constraint 3 (Justification):** `Column[last].x == PageRightMargin`

The Cassowary engine instantly optimizes the $X$ coordinates of every `TimeColumnId` to minimize the stretch penalty while strictly obeying the rigid rod boundaries. If a measure is overly crowded, the rods engage, and the springs of adjacent, less-crowded columns absorb the stretch factor.



## 6. Global Line Breaking (Gourlay / Knuth-Plass)

Once the Cassowary solver has determined the absolute minimum rod constraints and ideal spring spaces for every measure, the engine must distribute the continuous timeline across finite physical systems (lines) and pages. 

Naive engines use "First-Fit" greedy algorithms, stuffing measures onto a line until it overflows, resulting in right-margins that are jarringly crammed or artificially bloated. `tenuto-engrave` adapts the **Knuth-Plass Dynamic Programming Algorithm** (originally designed for TeX paragraphs) into two-dimensional musical space, following the architectural research of John S. Gourlay.

### 6.1 The Directed Acyclic Graph (DAG)
The layout engine constructs a massive network graph representing every possible combination of line breaks for the entire musical movement.
*   **Nodes:** Every valid breakpoint in the IR. Typically, these are barlines, but in unmetered cadenzas, they can be calculated sub-measure beat boundaries.
*   **Edges:** A proposed, unbroken line of music spanning from Node $i$ to Node $j$.

### 6.2 The Demerit Cost Function (Badness)
For every edge in the graph, the engine calculates a quantifiable "Badness" score ($B$). This score evaluates how severely the line had to be deformed by the Cassowary solver to fit the physical page width $W_{target}$.

Let $W_{natural}$ be the sum of all Rods and ideal Springs in the proposed line. The stretch/shrink ratio $R$ is defined as:
$$R = \frac{W_{target} - \sum \text{Rods}}{\sum S_{ideal}}$$

The Demerit score penalizes deformation exponentially:
*   If $R = 1.0$ (perfect fit), $B = 0$.
*   If the line is compressed ($R < 1.0$) or stretched ($R > 1.0$), $B = |1.0 - R|^3$.
*   *Additional Demerits:* Penalties are added for breaking between tightly phrased motifs or stranding a single measure on a new page (widows/orphans).

### 6.3 The Viterbi Optimization
With the DAG fully weighted, the engine utilizes dynamic programming (specifically the Viterbi algorithm) to find the "Shortest Path" from the first measure to the final measure. This guarantees a mathematically optimal global layout where no single system looks disproportionately cramped compared to its neighbors.

---

## 7. Vertical Collision Avoidance (The Skyline Algorithm)

While the Spring-Mass system resolves horizontal time, the vertical axis dictates pitch and articulation. Because music notation is intensely dense, standard Axis-Aligned Bounding Boxes (AABBs) waste massive amounts of vertical space—treating a delicate, arching slur like a giant, solid rectangular brick. 

`tenuto-engrave` implements a highly optimized, modern evolution of LilyPond’s **Skyline Algorithm**.

### 7.1 The Quantized 1D Skyline Array
A Skyline is a topological map tracing the extreme outer contour of a musical staff. Every staff maintains two skylines: `TopSkyline` and `BottomSkyline`.

Instead of using slow, continuous polynomial math to calculate curves during intersections, `tenuto-engrave` aggressively quantizes the skyline into a dense, flat 1-Dimensional array of `f32` height values.
*   **Resolution:** 1 array index = `0.1` Staff Spaces (ss).
*   **Structure:** `Vec<f32>` representing the maximum Y-extent at any given X-coordinate slice.

When a new glyph (e.g., a Staccato dot) is placed, its local SMuFL geometry is sampled and "dropped" onto the array. The staff's master skyline updates by taking the maximum height at every index:
$$S'_{top}[x] = \max(S_{top}[x], Glyph_{top}[x])$$

### 7.2 SIMD-Accelerated Intersections
Because the skylines are strictly aligned arrays of `f32` primitives, the engine leverages Rust’s `std::simd` (Single Instruction, Multiple Data) intrinsics to perform collision detection at hardware-accelerated speeds.

When the layout engine needs to place a dynamic marking (like `fff`) underneath a complex chord, it does not iterate through individual noteheads.
1.  It loads the `fff` bounding geometry.
2.  It loads 16 indices of the staff's `BottomSkyline` into the CPU's AVX/NEON registers simultaneously.
3.  It executes a single, vectorized `f32x16::max` operation to find the absolute closest the `fff` can be placed without touching the ink of the descending stems.

### 7.3 Staff Interlocking (System Compression)
To assemble individual staves into a full orchestral System without wasting page space, the engine cross-references their skylines.
To find the absolute minimum vertical distance $D$ required between Staff A (top) and Staff B (bottom):
$$D = \max_{x} (S_{bottom, A}[x] - S_{top, B}[x]) + P_{padding}$$
This allows Staff A's descending ledger lines to seamlessly interlock with Staff B's ascending cross-staff beams, nesting perfectly into the negative space.


## 8. Continuous Curves & Routing (The `kurbo` Subsystem)

Music notation requires highly expressive, continuous vector shapes—specifically ties, slurs, hairpins, and glissando lines. These elements cannot be rendered as static glyphs from a SMuFL font; they must be dynamically calculated at render-time. 

Slurs, in particular, present a complex geometric challenge. They must arch gracefully to connect noteheads or stems, but must dynamically deform their trajectory to rigorously avoid clipping through accidentals, articulation dots, and protruding ledger lines. 

`tenuto-engrave` outsources all 2D vector mathematics to **`kurbo`**, the industry-standard Rust crate for high-precision Bezier curve manipulation.

### 8.1 The Cubic Bezier Architecture

A slur is mathematically represented as a cubic Bezier curve $B(t)$, defined by four control points:
*   $P_0$: The start anchor (e.g., the exact SMuFL `stemUpSE` fractional coordinate of the first note).
*   $P_3$: The end anchor.
*   $P_1, P_2$: The internal control points dictating the height and inflection of the arch.

$$B(t) = (1-t)^3 P_0 + 3(1-t)^2 t P_1 + 3(1-t)t^2 P_2 + t^3 P_3 \quad \text{for } 0 \le t \le 1$$

### 8.2 Candidate Generation & Demerit Scoring

Because of the infinite variability of the Skyline it must cross, the routing algorithm cannot calculate a single "perfect" curve analytically. It employs a generate-and-test heuristic.

1.  **Generation:** The engine computes a vast search space of potential Bezier configurations by iteratively shifting $P_1$ and $P_2$ upward (increasing the arch) or outward (flattening the apex).
2.  **Sampling (`kurbo::flatten`):** The candidate curve is flattened into discrete line segments.
3.  **The Demerit Function:** The engine samples the curve at discrete $X$ intervals and evaluates it against the SIMD-accelerated Top/Bottom Skyline array. It computes an aggregate "Badness" score based on rigid penalties:

| Demerit Variable | Algorithmic Penalty Trigger | Mitigation Strategy |
| :--- | :--- | :--- |
| **Intersection** | The curve dips below the skyline: $B_y(x) < S_{top}[x]$. | Triggers a near-infinite penalty, instantly rejecting the curve. |
| **Variance** | The vertical distance between the slur and the enclosed noteheads fluctuates wildly. | Forces the algorithm to select a curve whose trajectory aesthetically mirrors the melodic contour of the notes beneath it. |
| **Asymmetry** | The angle vector between $P_1$ and $P_2$ contradicts the general slope between $P_0$ and $P_3$. | Prevents "lopsided" or visually unbalanced arches. |

The candidate Bezier curve with the absolute lowest aggregate demerit score is selected. If *no* candidate clears the intersection penalty, the engine falls back to shifting the primary anchors ($P_0$, $P_3$) to the opposite vertical side of the noteheads.

### 8.3 Parallel Offset Curves (Tapering)

Slurs are not rendered as lines of uniform thickness. High-end typography requires them to taper to a hairline at the anchors and thicken at the apex. 

To draw this in SVG, the engine must generate a closed shape representing the *outline* of the thick slur. This requires calculating a parallel offset curve. Because the mathematical offset of a cubic Bezier is a degree-10 polynomial (which SVG cannot render), the engine leverages `kurbo`'s `offset_cubic` algorithm to rapidly approximate the offset path using multiple optimized cubic segments.

---

## 9. Cross-Staff Synchronization & Voice Routing

Advanced Tenuto code allows polyphonic voices to physically cross between distinct staves while remaining logically attached to their parent stream (e.g., a pianist playing a continuous arpeggio that spans from the bass clef to the treble clef).

**Syntax Trigger:** `pno_rh: c4.cross(pno_lh)`

### 9.1 Cross-Staff Beaming (The Knee Beam)
When a beam group spans two staves, the layout engine executes a secondary topological analysis.
1.  **Beam Ownership:** The beam `SpannerId` is injected into the ECS referencing the Source Staff, inheriting its color and visibility properties.
2.  **Slope Calculation:** The engine calculates the absolute vertical distance between the two staves within the specific `SystemId`. The beam's mathematical slope ($P_0$ to $P_3$) is calculated to connect the outermost noteheads across this visual void.
3.  **The Knee Beam Protocol:** If the vertical distance between the highest and lowest notehead in the beamed group exceeds a standard threshold (typically $> 1.5$ octaves), the engine automatically triggers a "Knee Beam." The beam physically alters its trajectory mid-stream, breaking the primary slope to keep the stems within readable lengths.

### 9.2 The "Invisible Node" Anchor
To maintain the mathematical integrity of the Spring-Mass system, a note crossing to a secondary staff leaves an "Invisible Proxy Node" in its original `TimeColumnId` on the source staff. This ensures the Cassowary solver still calculates the correct horizontal space for the logical rhythm, even though the physical ink is rendered elsewhere in the vertical hierarchy.


## 10. Deterministic SVG Export

The culmination of the engraving pipeline is the `tenutoc::svg` backend. Unlike HTML or CSS, the exported SVG must not rely on the browser’s internal layout algorithms or text-rendering engines. It must be **100% deterministic**, meaning the file will look exactly the same across every device, PDF converter, or operating system for the next century.

### 10.1 Absolute Paths and Transformations

To achieve absolute determinism, `tenuto-engrave` bypasses the `<text>` SVG tag entirely. 
*   **Vectorization:** Every SMuFL character (clefs, noteheads, flags) and every dynamically calculated shape (slurs, beams, ties) is explicitly rendered as raw `<path>` data using SVG `d="..."` commands (Move, Line, Cubic Bezier).
*   **The Y-Axis Inversion:** Because the internal IR uses a Cartesian coordinate system (where Y increases upwards) and SVG uses a screen coordinate system (where Y increases downwards), the root `<svg>` or `<g>` group applies a universal affine transformation:
    ```xml
    <g transform="matrix(1 0 0 -1 0 page_height)">
    ```

### 10.2 Structural Grouping (The DOM Hierarchy)

While the visual coordinates are flat, the output DOM is hierarchically grouped to enable downstream interactivity (e.g., highlighting a measure during playback in a web app).

```xml
<svg viewBox="0 0 2100 2970" xmlns="http://www.w3.org/2000/svg">
  <!-- System 1 -->
  <g class="system" id="sys-1">
    
    <!-- Measure 1 -->
    <g class="measure" id="m-1">
      
      <!-- Staff Lines -->
      <path class="staff-lines" d="M 0 500 L 1800 500 ... " stroke="black"/>
      
      <!-- Note Entity -->
      <g class="note" data-tick="0" data-pitch="60">
        <!-- Notehead (SMuFL Bravura path) -->
        <path class="glyph" d="M..." fill="black" transform="translate(150, 520)"/>
        <!-- Stem -->
        <line x1="162" y1="520" x2="162" y2="555" stroke-width="1.2"/>
      </g>
      
    </g>
  </g>
</svg>
```
*   **`data-tick` Integration:** By embedding the absolute `Timeline` ticks directly into the SVG output as data attributes, web developers can easily synchronize the playing MIDI audio with a visual cursor moving across the screen.

---

## 11. The `salsa` Implementation Strategy (Real-Time Responsiveness)

To fulfill Phase VI of the roadmap (the `tenutod` live-coding daemon), the engraving engine must be able to re-render a score in milliseconds as the user types. Re-running the Cassowary constraint solver and Viterbi line-breaking algorithms across a 50-page symphony on every keystroke is computationally impossible.

The architecture solves this using **Incremental Computation** powered by the `salsa` crate.

### 11.1 The Query Group Graph

The compilation pipeline is restructured into distinct, memoized `salsa` queries. 

```rust
#[salsa::query_group(EngraverStorage)]
pub trait EngraverDatabase {
    // 1. The Raw Input
    #[salsa::input]
    fn raw_ir(&self) -> Arc<Timeline>;

    // 2. Measure-level caching
    fn measure_bounds(&self, measure_id: MeasureId) -> Arc<BoundingBox>;
    
    // 3. System-level layout
    fn line_breaks(&self) -> Arc<Vec<LineBreak>>;
    
    // 4. Final SVG string generation
    fn render_measure(&self, measure_id: MeasureId) -> Arc<String>;
}
```

### 11.2 The Cascade of Invalidation

When the user alters a single note in Measure 5 via the text editor:
1.  **The Input Changes:** The daemon updates the `raw_ir` in the `salsa` database.
2.  **Granular Invalidation:** `salsa` detects that only the events in `measure_id: 5` have changed. It invalidates the `measure_bounds(5)` cache.
3.  **The Circuit Breaker:** `salsa` re-runs `measure_bounds(5)`. 
    *   *Scenario A (No Ripple):* The new note does not drastically alter the horizontal width of Measure 5. The total width remains within the existing line-break thresholds. `salsa` halts the invalidation cascade. It grabs the pre-rendered SVG strings for all other measures from the cache, renders the new SVG string only for Measure 5, and returns the full document in $< 5\text{ms}$.
    *   *Scenario B (Ripple Effect):* The user pasted a massive 32nd-note run into Measure 5. The new width forces the Cassowary solver to push Measure 6 onto the next page. `salsa` invalidates the `line_breaks()` query, recalculates the Viterbi DAG, and updates the layout of the subsequent pages.

### 11.3 Memory Optimization

Because `salsa` clones data between query layers to detect changes, the engine must heavily utilize `Arc` (Atomic Reference Counting) to prevent massive memory allocations during the caching process. The flat, `slotmap`-based ECS World defined in Chunk 1 pairs perfectly with this, as `salsa` only needs to compare lightweight `EntityId` arrays to detect state mutations.

---
**End of the Tenuto Engraving Architecture Specification (TEAS).** 
