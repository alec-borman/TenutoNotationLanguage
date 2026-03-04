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



# TEAS Addendum A: Advanced Mechanics & Pagination

## 12. The Zero-Duration Domain (Prefatory & Grace)

The standard Spring-Mass model assumes that graphical space is proportional to temporal duration. However, music is littered with crucial glyphs that consume *zero* metrical time but require rigid physical space. The engine handles these via **Zero-Duration Columns**.

### 12.1 Prefatory Columns (Clefs, Keys, Meters)
Measure boundaries and mid-measure changes often introduce Clefs, Key Signatures, and Time Signatures. 
*   **Architecture:** These are instantiated as `TimeColumnId` entities with a `metrical_duration = 0`.
*   **Constraint Resolution:** In the Cassowary solver, a Prefatory Column possesses a **Rod** (rigid width based on the SMuFL bounding box) but **no Spring**. 
*   **Result:** A treble clef pushes the subsequent notes to the right strictly by its physical width plus standard padding ($L_{min}$), but it will *never* stretch or compress when justification forces ($F$) are applied to the measure.

### 12.2 Grace Note Formatting
Grace notes present a unique paradox: they are visually beamed and stemmed like standard notes, but do not consume metric time.
*   **ECS Flagging:** Grace notes are grouped into their own `TimeColumnId`s flagged with `is_grace = true`.
*   **Scale Transformation:** The engine applies a universal `scale_factor` (typically `0.6` or $60\%$) to the `GlyphId`s associated with the grace column, querying the SMuFL `graceNote` classes.
*   **Cassowary Handling:** Grace columns are assigned infinitely stiff springs ($c = \infty$). Their physical spacing relies entirely on their scaled Rod lengths, keeping them tightly packed against their primary target note, immune to global measure justification.

---

## 13. Complex Glyph Orchestration & Stacking

While single notes are trivial, the engine must systematically assemble complex composite objects before calculating their global bounding boxes.

### 13.1 Accidental Stacking Algorithms
When a chord contains multiple altered pitches (e.g., a clustered D7♭9 chord), drawing all accidentals in a single vertical column will cause catastrophic visual collisions.
*   **The Sub-Column Array:** Accidentals are assigned to a secondary structural entity called an `AccidentalColumnId`. 
*   **The Packing Algorithm:** The engine sorts the chord's accidentals by vertical Y-coordinate. It attempts to place them in a primary vertical column immediately to the left of the chord. If a collision is detected via AABB overlap, it creates a new `AccidentalColumnId` further to the left, cascading outward in a classic "zig-zag" pattern.
*   **Rod Expansion:** The aggregate width of all nested `AccidentalColumnId`s is added to the parent `TimeColumnId`'s Rod constraint.

### 13.2 Centering Articulations and Ornaments
Fermatas, staccato dots, and marcato accents are placed outside the staff using the Skyline algorithm for vertical clearance, but require precise horizontal alignment.
*   **SMuFL `opticalCenter`:** The engine does not center these glyphs based on the total width of the notehead's bounding box. It queries the SMuFL `opticalCenter` anchor.
*   **Execution:** A staccato dot's X-coordinate is locked via Cassowary constraint to `Notehead.x + Notehead.opticalCenter.x`, ensuring perfect visual weight distribution even on asymmetrical glyphs.

### 13.3 The Generalized Kerning Matrix
The optical cut-out logic defined in the core spec must extend beyond Flat/Notehead pairs. 
*   **The Kerning Database:** During the `smufl` parsing phase, the engine constructs a generalized Interaction Matrix. It maps overlapping cut-out polygons for high-friction pairs:
    *   Sharp signs and natural signs adjacent to bar lines.
    *   Tuplets numbers tucked between stems and slurs.
    *   Accidentals nesting into the negative space of adjacent accidentals in a stacked chord.
*   **Performance:** By pre-calculating the maximum safe overlap distances for these pairs into a cached matrix, the engine avoids expensive polygon-intersection math during the active layout loop.

---

## 14. Advanced Spanners & Continuous Lines

The `kurbo` Bezier subsystem must handle significantly more than simple slurs.

### 14.1 Tuplet Brackets and Numerals
Tuplets require a multi-component `SpannerId`.
*   **Components:** A horizontal/sloped line, two vertical "hooks" (if unconnected to a beam), and a SMuFL numeral (e.g., **3**).
*   **Constraint:** The slope of the tuplet bracket must parallel the prevailing slope of the note group beneath it.
*   **Skyline Interaction:** The tuplet number is injected into the Skyline array *before* slurs are routed, forcing overlapping slurs to arch *over* the tuplet numeral, adhering to standard engraving hierarchy.

### 14.2 Feathered Beams (Accelerando/Ritardando)
Contemporary notation requires beams that fan outward or inward.
*   **Architecture:** Instead of a single thick `kurbo` path, a feathered beam is modeled as a parent `SpannerId` containing multiple child paths.
*   **Y-Offset Scaling:** The primary beam connects $P_{start}$ to $P_{end}$. The secondary and tertiary beams compute their Y-offsets dynamically based on the $X$ progression.
*   **Equation:** For a 3-beam accelerando group, the vertical distance between beams at $X_{start}$ is $1.0\text{ss}$, tapering linearly to $0.25\text{ss}$ at $X_{end}$.


## 15. Macro-Typography & Page Formatting

While Gourlay's algorithm successfully distributes measures into optimal Systems (lines), the engine must perform a secondary dynamic programming pass to distribute those Systems across physical Pages. 

### 15.1 Vertical Spring-Mass & Page Justification
Just as horizontal time is elastic, the vertical space between staves (`staff-staff-spacing`) and systems (`system-system-spacing`) acts as a vertical Spring-Mass system.
*   **The Goal:** A page should appear optically balanced, with systems stretching vertically to fill the page height, creating flush top and bottom margins.
*   **The Constraints:** The vertical "Rods" are determined by the maximum interlocking distance calculated by the Skyline algorithm (Section 7). The vertical "Springs" stretch the systems apart based on a global stretchability constant.

### 15.2 Page Breaking (The 2D Knuth-Plass DAG)
The engine constructs a second Directed Acyclic Graph where the nodes are the previously calculated line breaks, and the edges are full printed pages. 
The demerit function ($B_{page}$) evaluates:
1.  **Vertical Stretch/Shrink Penalty:** How much the vertical springs had to deform to fit the page height.
2.  **Widow/Orphan Penalties:** Massive demerits for stranding a single system on the final page, or leaving a single system of a new movement at the bottom of a previous page.
3.  **Page-Turn Optimizations:** The engine scans the IR for rests. It artificially reduces the penalty for breaking a page during a long multi-measure rest, actively aiding human performers in physical page-turning.

### 15.3 Global Layout Elements
Titles, composers, headers, and page numbers bypass the musical constraint solver. They are injected as absolute-positioned `TextEntity` objects during the final rendering frame, utilizing standard typographical bounding boxes. Their presence permanently alters the Top/Bottom page margins fed into the Cassowary solver.

---

## 16. Ossia Staves & Scaling Transformations

Ossia staves (alternative passages or cues) present a unique layout challenge: they exist parallel to the main timeline but are rendered at a significantly reduced scale (typically $66\%$ or $\frac{2}{3}$ of standard size).

### 16.1 ECS Scaling Components
To handle this elegantly without duplicating layout logic, `tenuto-engrave` utilizes the ECS architecture.
*   **The Component:** The `StaffId` entity is granted a `scale_factor` component. For a standard staff, this is `1.0`. For an Ossia, it is `0.66`.
*   **The Application:** When the engine's Realization System queries the SMuFL `metadata.json` for bounding boxes and anchor coordinates, it immediately multiplies the resulting vectors by the parent staff's `scale_factor`.

### 16.2 Skyline Integration for Scaled Staves
Because the Ossia is mathematically scaled *before* collision detection, it seamlessly drops into the standard Skyline algorithm. 
*   **Interlocking:** The small Ossia staff possesses its own Top and Bottom skylines. The vertical spacing algorithm treats it exactly like a standard staff. Because its bounding boxes are smaller, it naturally tucks much tighter against the main staff, creating the expected dense, nested look of professional cues without any custom "Ossia-specific" collision logic.
*   **Timeline Anchoring:** The Ossia's `TimeColumnId`s remain strictly linked to the main measure grid, ensuring horizontal alignment with the parent staff is perfectly preserved regardless of the visual scaling.

---

## 17. Error Handling & Constraint Fallbacks

A robust layout engine must never panic when presented with an "impossible" score (e.g., a user forces 20 complex measures onto a single system via a manual `break: none` override). When the Cassowary solver encounters unsolvable constraints, it must rely on a strict fallback hierarchy.

### 17.1 Soft vs. Hard Constraints
In the Cassowary algorithm, constraints are assigned weights (`Required`, `Strong`, `Medium`, `Weak`). 
*   Normally, Rods (minimum physical widths) are `Required` ($\infty$ weight). 
*   If the required page margin forces the total width to be less than the sum of the Rods, the Cassowary solver throws an `Overconstrained` error.

### 17.2 The Resolution Hierarchy
When an `Overconstrained` state is detected, the engine executes the following recovery protocol:

1.  **Padding Relaxation:** The engine downgrades the optical padding constants ($P_{padding}$) to `Weak` constraints, allowing elements to sit uncomfortably close, but not touching.
2.  **Spring Compression Limit Violation:** If still unsolvable, the engine allows Springs to compress below their mathematical zero-point. 
3.  **Controlled Overlap (Rod Violation):** As a last resort, the engine downgrades Rods from `Required` to `Strong`. This allows graphical glyphs (like accidentals and noteheads) to physically overlap and crash into each other. *Rationale: It is better to emit a cluttered, overlapping SVG that the user can manually debug than to crash the compiler and output nothing.*

### 17.3 Slur & Tie Resolution Fallbacks
If the `kurbo` Bezier router fails to find *any* candidate curve that clears the Skyline intersection demerits (Section 8.2), it executes a hardcoded fallback sequence:
1.  **Anchor Inversion:** It flips the $P_0$ and $P_3$ anchors to the opposite vertical side of the note group (e.g., drawing the slur under the noteheads instead of over the stems).
2.  **Apex Flattening:** It drastically reduces the Y-coordinates of $P_1$ and $P_2$, creating a nearly flat line that snakes between obstacles.
3.  **Skyline Penetration:** If the chord is so dense that no path exists, the engine temporarily removes ledger lines and stems from the Skyline array and recalculates, allowing the slur to slice through stems (which is typographically acceptable in extreme edge cases) while still avoiding noteheads and accidentals.


## 18. Extensibility & Graphic Notation

Western music notation is not a static monolith; contemporary composers frequently invent new symbols and visual paradigms (e.g., spectral music, indeterminate pitch, or graphic scores). A modern layout engine must be extensible without requiring a hard fork of the compiler codebase.

### 18.1 SMuFL Private Use Area (PUA) & Custom Glyphs
The SMuFL specification officially reserves ranges within the Unicode Private Use Area for font-specific and user-defined glyphs.
*   **The Injection API:** `tenuto-engrave` exposes an API to register a `CustomGlyph`. The user provides a raw SVG `<path>` and a manual `BoundingBox` mapping.
*   **ECS Integration:** This custom object is assigned a standard `GlyphId` and dropped into the ECS. Because the engine's algorithms (Spring-Mass and Skyline) operate strictly on abstract bounding boxes and cut-out polygons, the custom glyph interacts with the layout physics exactly like a native notehead, automatically dodging slurs and displacing neighboring columns.

### 18.2 Arbitrary Spanners (Vector Graphic Injections)
Contemporary scores often require arbitrary lines indicating air-noise, gradual bow pressure, or spatial panning.
*   **Custom `SpannerId`:** The engine allows users to define custom continuous lines anchored to `TimeColumnId`s. 
*   **The `kurbo` Pipeline:** A user defines a generic mathematical curve (e.g., a sine wave indicating vibrato speed). The engine anchors the start of the wave to $X_{start}$ and the end to $X_{end}$. As the Cassowary solver stretches the measure, the $X_{end}$ coordinate updates, and the `kurbo` subsystem dynamically recalculates the Bezier control points to smoothly stretch the wave across the new visual space without distorting its Y-amplitude.

---

## 19. Interactive DOM & Playback Synchronization

Tenuto's ultimate goal is to power the "Living Score"—sheet music that lives natively in the browser, perfectly synchronized with audio playback (Web MIDI or Web Audio API). The SVG backend is explicitly architected to facilitate this.

### 19.1 Embedded Metrical Metadata (`data-*` attributes)
Because the visual layout is derived directly from the absolute `Timeline` IR, the engine possesses the exact mathematical tick for every pixel of ink. The SVG exporter permanently embeds this temporal data into the DOM hierarchy.

```xml
<!-- Measure wrapper knows its exact chronological boundaries -->
<g class="tenuto-measure" data-measure="5" data-start-tick="30720" data-end-tick="38400">
    
    <!-- A specific chord group knows its exact trigger time and duration -->
    <g class="tenuto-chord" data-tick="32640" data-duration="1920">
        <path class="notehead" d="..." />
        <path class="accidental" d="..." />
    </g>

</g>
```

### 19.2 The Playhead & CSS Integration
By strictly avoiding `<canvas>` and utilizing a highly structured SVG DOM, front-end developers can achieve 60 FPS synchronized playback with trivial JavaScript.
*   **The Playhead:** A developer simply queries the current audio tick from their Web Audio context, loops through the `<g class="tenuto-measure">` tags, and calculates an exact X-coordinate interpolation for a moving vertical playhead line.
*   **Visual Feedback:** When the audio tick matches a `data-tick` on a `tenuto-chord`, JavaScript can toggle a CSS class (`.playing { fill: #FF0000; }`), instantly highlighting the notes currently sounding without recalculating any layout logic.

---

## 20. Memory Management at Scale (The 100-Page Symphony)

For massive orchestral scores containing millions of discrete `AtomicEvent`s, keeping the entire `EcsWorld` and SMuFL metadata active in primary memory could exhaust consumer hardware or WebAssembly memory limits.

### 20.1 Spatial Chunking & Paging
`tenuto-engrave` mitigates memory pressure by treating **Pages** as isolated spatial chunks within the ECS.
*   Once the Viterbi line-breaking algorithm determines the system and page boundaries (Section 15), the mathematical layout of "Page 1" has absolutely no topological effect on "Page 2".
*   **Memory Eviction:** The ECS can serialize the solved spatial coordinates for Page 1, write the SVG string to disk (or cache), and immediately drop the heavy `GlyphId` bounding boxes and `Skyline` arrays for those measures from memory, drastically reducing peak RAM utilization.
*   **Parallel Rendering:** Because pages are mathematically isolated after line-breaking, the actual generation of Bezier curves and SVG strings is embarrassingly parallel. The engine utilizes the `rayon` crate to spawn worker threads, drawing Page 1 through Page 100 simultaneously across all available CPU cores.


# TEAS Addendum B: Vocal, Semantic, and System-Level Typographical Paradigms

## 21. The Vocal Typography Subsystem (Lyrics)

Vocal music introduces a secondary layout engine running parallel to the musical notes. Lyrics possess their own horizontal justification rules, hyphenation spacing, and vertical stacking constraints. `tenuto-engrave` treats lyrics as a native subsystem within the ECS, avoiding the collision nightmares of legacy GUI applications.

### 21.1 The ECS Lyric Hierarchy
Lyrics are not stored as mere attributes of a note. They are instantiated as discrete relational entities:
*   **`LyricVerseId`:** Represents a single horizontal line of text (e.g., Verse 1, Verse 2).
*   **`SyllableId`:** A specific chunk of text (or SMuFL elision glyph) anchored to a `TimeColumnId`.

### 21.2 Horizontal Alignment & The "Syllable Spring"
A syllable fundamentally alters the Cassowary constraints of its parent `TimeColumnId`.
*   **Alignment:** By default, the optical center of the syllable’s bounding box is aligned with the `opticalCenter` of the target notehead. However, if a syllable initiates a melisma, it is natively left-aligned to the notehead.
*   **The Text Rod:** The width of the text string is converted into an absolute physical Rod. If a whole note is sung with the word "Hallelujah", the measure cannot horizontally compress smaller than the width of the text, overriding the ideal musical Spring length.

### 21.3 Hyphens, Melismas, and Elisions
These elements are not static text; they are dynamic graphical objects reacting to the layout physics.
*   **Hyphens (`-`):** Injected as specialized `SpannerId` objects spanning between two `SyllableId`s. As the Cassowary solver stretches the measure, the hyphen dynamically centers itself in the resulting whitespace. If the space exceeds a predefined maximum threshold, the engine instantiates *multiple* equidistant hyphens.
*   **Melisma Lines (`_`):** Modeled as `SpannerId` straight lines. They begin at the right bounding edge of the originating syllable and terminate at the `TimeColumnId` of the final note in the melisma, rendering a standard baseline underscore, optionally terminating with a vertical SMuFL hook.
*   **Elisions (`~`):** Requires the SMuFL `lyricElisionNarrow` or `lyricElisionWide` glyphs. The engine calculates the bounding box of both syllables, places them tightly adjacent, and anchors the elision glyph between them, treating the entire composite as a single `SyllableId` constraint.

### 21.4 Vertical Stacking & Verse Skylines
Multiple verses must stack perfectly vertically without colliding with descending ledger lines or low dynamics.
*   **Layered Skylines:** The engine generates a localized `BottomSkyline` specifically for the music staff. `Verse 1` is placed below it, acting as a flat rectangular block (calculated from font ascenders/descenders). 
*   **Push-Down Mechanism:** The bottom of `Verse 1` becomes the new skyline for `Verse 2`. This guarantees perfectly equidistant text lines that gracefully dodge descending musical elements.

---

## 22. Semantic & Harmonic Annotations

Standard musical text (Dynamics, Tempos) and harmonic structures (Chord Symbols, Figured Bass) require unified handling. They represent a fusion of standard font typography and SMuFL musical symbols.

### 22.1 The Unified Text Model
Arbitrary text (e.g., "Allegro ♩ = 120" or rehearsal mark "A") is instantiated as an `AnnotationId`.
*   **Rich Bounding Boxes:** The engine utilizes standard font metric libraries (e.g., `rusttype` or `swash`) to calculate the exact extents of standard text, while querying the SMuFL metadata for the injected musical glyphs. The engine merges these into a single composite AABB.
*   **The Annotation Skyline:** To prevent a tempo marking from overlapping with a crescendo hairpin above the staff, annotations are dropped into the staff's `TopSkyline` or `BottomSkyline` *sequentially*, prioritized by semantic importance.

### 22.2 Chord Symbols
Jazz and pop chord symbols (e.g., `Cmaj7/G`) are highly structured semantic tags.
*   **Alignment:** Treated as zero-duration columns sitting strictly above the `TopSkyline`. Their X-coordinate is horizontally locked to the associated beat.
*   **Vertical Stacking:** If a piece includes original and alternate chords (e.g., Coltrane changes), they are grouped into a `ChordStackId`. The engine applies a vertical spring-rod constraint between them to ensure optical separation.

### 22.3 Figured Bass
Figured bass is highly complex, acting as a vertical micro-layout engine operating beneath the staff.
*   **The Figure Stack:** Modeled as a specialized `AnnotationId` attached to a bass note `TimeColumnId`.
*   **Alignment:** The numerals (e.g., `6`, `4`, `2`) and accidentals (e.g., `♯3`) are vertically aligned by their central axes.
*   **Continuation Lines:** Modeled as `SpannerId` elements. The engine routes a horizontal line from the right bounding edge of a specific digit to the target `TimeColumnId`, dynamically adjusting its length as the Cassowary solver stretches the parent measures.

## 23. System-Level Topology & Prefatory Bounds

An orchestral score is not just a collection of staves; it is a highly structured hierarchy of instrument families grouped by brackets, braces, and systemic barlines. These elements exist outside the standard temporal grid but exert massive physical constraints on the page layout.

### 23.1 Instrument Names and the Left Margin
Instrument names (e.g., "Violin I", "Vln. I") sit to the left of the system. They are not tied to a `TimeColumnId`.
*   **The Margin Rod:** The engine evaluates the bounding boxes of all active instrument names within a system. It calculates the maximum width $W_{names}$.
*   **Cassowary Integration:** A global `SystemMarginId` is instantiated as a structural entity. The engine injects a rigid `Required` constraint into the Cassowary solver: `FirstMeasure.X >= LeftPageMargin + W_{names} + Padding`. This automatically indents the entire musical system to perfectly accommodate the longest instrument name.

### 23.2 Brackets, Braces, and Sub-Brackets
Grouping symbols must dynamically stretch to encompass multiple staves, reacting to the vertical Spring-Mass justification (Section 15.1).
*   **The Grouping Entity:** Modeled as a `SystemBracketId` in the ECS, containing an array of `StaffId`s it encompasses.
*   **Dynamic Vertical Routing:** After the vertical Cassowary solver finalizes the Y-coordinates of all staves, the engine computes the $Y_{top}$ of the highest staff in the group and the $Y_{bottom}$ of the lowest staff.
*   **SMuFL Implementation:** For braces (piano), the engine scales a specific SMuFL brace glyph (`brace`) to fit the $Y_{top} - Y_{bottom}$ delta. For orchestral brackets, it draws a continuous thick vertical vector line with SMuFL top/bottom hooks (`bracketTop`, `bracketBottom`), ensuring perfect resolution regardless of how far the staves are stretched apart.

---

## 24. Macro-Document Structure (Multi-Movement)

A complete Tenuto document may contain multiple movements (e.g., a 4-movement symphony), each with distinct titles, differing instrumentations, and independent measure number sequences, all flowing continuously across physical pages.

### 24.1 The Movement DAG Isolation
Gourlay's dynamic programming line-breaking algorithm (Section 6) becomes exponentially slower ($O(n^2)$ or worse) if asked to calculate the entire path for a 2,000-measure opera at once. 
*   **Structural Partitioning:** The engine treats each Movement as an independent structural `FlowId`. 
*   **Isolated Solvers:** The Viterbi DAG is strictly bounded within a single `FlowId`. The layout of Movement 2 has no mathematical impact on the line-breaking of Movement 1, drastically reducing computational overhead.

### 24.2 Continuous Page Flow and Header State Machines
Despite isolated line-breaking, movements must share global page formatting.
*   **The Flow-Merge System:** Once all movements are line-broken into rigid `SystemId`s, a master Page-Breaking pass evaluates them sequentially. If Movement 1 ends halfway down Page 14, Movement 2 can begin on the same page, separated by a localized `SpacerRod`.
*   **Header/Footer ECS System:** Titles and page numbers are managed by a localized state machine. The engine checks the `PageId`: if it contains the start of a new `FlowId`, it triggers the "First Page Template" (large title, composer name). Otherwise, it triggers the "Subsequent Page Template" (small header, instrument name).

---

## 25. Alternative Staff Notations

The Tenuto syntax abstractly handles tab (`0-6`) and percussion (`k`), but the layout engine must physically mutate its rendering rules to accommodate these specialized formats, as well as historical notations like Gregorian chant.

### 25.1 Percussion Grids (Variable Line Counts)
Percussion is often engraved on 1-line, 2-line, or 5-line staves. 
*   **Staff Override Component:** The `StaffId` is granted a `line_count` property. 
*   **Y-Axis Recalculation:** The standard 5-line staff uses Y-coordinates from $-2.0\text{ss}$ to $+2.0\text{ss}$. If `line_count = 1`, the engine dynamically remaps all `absolute_y` queries to anchor to $0.0\text{ss}$.
*   **Glyph Substitution:** The realization system intercepts standard noteheads and replaces them with SMuFL percussion variants (e.g., `noteheadCross`, `noteheadCircle`) based on the Tenuto `.head("x")` attribute.

### 25.2 Lute & Historical Tablature
While standard guitar tab uses numbers, historical tabs (French/Italian) use letters (a, b, c) positioned *between* staff lines, with rhythmic flags hovering above the staff.
*   **The Tablature Translation System:** The engine flags the `StaffId` as `Style::Tablature(Historical)`. 
*   **Coordinate Re-mapping:** Instead of drawing noteheads on lines, the engine renders SMuFL letter glyphs. The vertical Y-offset is mathematically locked to the center of the spaces between lines.
*   **Detached Rhythmic Flags:** Standard stems are suppressed. Instead, the engine instantiates separate `GlyphId` entities for rhythmic duration (e.g., `luteDurationEighth`) and locks their X-coordinates to the `TimeColumnId`, pushing them vertically above the `TopSkyline`.

### 25.3 Gregorian Chant (Neumatic Notation)
Chant notation breaks almost every rule of modern engraving. It uses a 4-line staff, square neumes, no barlines, and is horizontally spaced almost entirely by the lyrics, not mathematical time.
*   **Disabling the Metric Spring:** For a `StaffId` flagged as `Style::Chant`, the engine bypasses the standard $S_{ideal} = f(\text{duration})$ metric spring calculations (Section 5.1). 
*   **Lyric-Driven Constraints:** The horizontal Cassowary constraints are driven strictly by the width of the `SyllableId` Rods. The neumes (musical symbols) are treated as zero-width followers, visually centering themselves over the text rather than dictating the flow of time.
*   **Custom Neume Composites:** The engine utilizes the SMuFL `medRen` (Medieval/Renaissance) class to construct complex ligatures (stacked squares) using localized vertical bounding box intersections, entirely bypassing standard stem/flag logic.


## 26. Advanced Slur & Tie Interactions

While Section 8 defined the core `kurbo` Bezier routing over a standard skyline, real-world music engraving introduces extreme topological edge cases where continuous lines must break, cross spatial voids, or anchor to empty space.

### 26.1 System Break Fragmentation
When a tied note or a phrasing slur spans across a line break (or page break), the single logical `SpannerId` must be visually bifurcated into two distinct graphical paths.
*   **The Bifurcation System:** During the Line-Breaking phase (Section 6), the engine detects if a `SpannerId`'s start and end `TimeColumnId`s sit on different `SystemId`s. If so, it dynamically spawns two transient `RenderSpanner` entities.
*   **Trailing Edge (System 1):** The first curve anchors at $P_0$ (the start note) and routes to a calculated $P_3$ hovering slightly past the right barline of the system. The curve's trajectory is artificially flattened to indicate continuation.
*   **Leading Edge (System 2):** The second curve anchors at a new $P_0$ positioned immediately after the prefatory glyphs (clef/key) of the new system, routing to $P_3$ (the target note).
*   **SMuFL Integration:** For ties, the engine optionally utilizes the SMuFL `tieEndpoint` and `tieStart` glyphs or constructs equivalent tapered Beziers that mimic the traditional "fade out" ink bleed at the system margins.

### 26.2 Cross-Staff Bezier Routing
When a slur connects a note in the Bass staff to a note in the Treble staff, the Bezier curve must traverse the vertical gap (`staff-staff-spacing`) without colliding with dynamics, lyrics, or tempo markings occupying that space.
*   **Absolute System Coordinates:** The `kurbo` router temporarily lifts the control points ($P_0$ through $P_3$) out of the local `StaffId` coordinate space and maps them to the absolute `SystemId` coordinate space.
*   **S-Curve Inflection:** Because the vertical delta ($\Delta Y$) is massive compared to a standard slur, the engine adjusts the internal control points ($P_1, P_2$). Instead of a simple arch, it generates an S-curve or a steep diagonal trajectory. 
*   **Composite Skyline:** The curve is evaluated against a temporary *Composite Skyline* (the `TopSkyline` of the Bass staff merged with the `BottomSkyline` of the Treble staff) to navigate the negative space safely.

### 26.3 Atypical Anchors (Laissez Vibrer & Rests)
*   **Laissez Vibrer (Let Ring):** Triggered by the `.letring` attribute. The engine generates a tie where $P_0$ anchors to the notehead, but $P_3$ anchors to an empty spatial offset $+1.5\text{ss}$ to the right, floating in the void.
*   **Slurs to Rests:** While rare, phrasing slurs occasionally encompass rests. Because rests lack stems, the engine calculates the bounding box of the rest, finds its optical center top, and anchors the Bezier curve directly to that coordinate.

---

## 27. Accessibility & Alternative Formats

A truly modern compiler must output data that is universally accessible. Because `tenutoc` separates logic from physics, generating alternative formats for visually impaired musicians requires zero reverse-engineering.

### 27.1 Semantic SVG & ARIA Roles
The `tenutoc::svg` backend is designed to be natively readable by screen readers. 
*   **ARIA Injection:** Instead of emitting silent paths, the engine wraps logical musical groups in `<g>` tags enriched with `role="group"` and ARIA labels.
*   **Example Output:** 
    ```xml
    <g role="listitem" aria-label="Measure 5, Beat 1: Quarter note C 4, forte.">
        <path d="..." /> <!-- Notehead -->
        <path d="..." /> <!-- Stem -->
    </g>
    ```
*   This allows visually impaired users to "tab" through an SVG score on the web and hear a perfectly accurate, semantic description of the music.

### 27.2 Braille Music Translation
Braille music notation is a highly complex, linear, non-spatial code. Traditional GUI software struggles to generate it because they rely on visual screen coordinates. 
*   **The Braille Exporter (`tenutoc::braille`):** Because Tenuto's IR is a flattened, absolute-time stream of `AtomicEvent`s, it maps natively to Braille formatting. 
*   **Execution:** A future backend module can traverse the `Timeline` IR, applying standard Braille music syntax rules (e.g., combining pitch and duration into single Braille cells, placing octave markers only when leaps occur) and outputting a standard `.brf` (Braille Ready Format) file directly from the `.ten` source code.

---

## 28. Engine Validation & Testing Architecture

To guarantee the stability of the Cassowary constraint solver and the Skyline algorithms across infinite musical possibilities, the `tenuto-engrave` crate utilizes a highly aggressive Quality Assurance pipeline.

### 28.1 Property-Based Testing (`proptest`)
The layout math must never panic (`unwrap()` failures). We utilize the `proptest` crate to generate thousands of randomized, chaotic measures (e.g., a 128th note stacked against a septuplet, with random accidental clusters). The test suite verifies that the Cassowary solver can *always* find a mathematically valid fallback (even if it involves controlled overlap, as defined in Section 17) without crashing the compiler.

### 28.2 Visual Snapshot Regression (`insta`)
Because a single tweak to the Spring-Mass ratio can subtly alter the layout of a 100-page score, the engine uses the `insta` snapshot testing crate.
*   The CI/CD pipeline compiles a suite of "Golden Master" `.ten` files into SVGs.
*   It cryptographically hashes the SVG strings. If a developer's pull request alters the layout logic and changes the hash of a Golden Master, the PR is flagged for manual visual review, ensuring zero unintended typographical regressions.


# TEAS Addendum D: The Avant-Garde, Historical, & Scholarly Frontier

## 34. Aleatoric, Proportional, & Electroacoustic Notation

Contemporary and electroacoustic scores frequently abandon rigid metrical grids in favor of spatial or graphic representations of time and texture.

### 34.1 Strict Proportional Layout (Space = Time)
When a composer specifies `layout @{ proportional: true }`, the Cassowary solver abandons the logarithmic optical Spring formula (Section 5.1).
*   **The Constraint:** The ideal spring length $S_{ideal}$ becomes strictly linear: $S_{ideal} = k \times \Delta t$, where $\Delta t$ is the exact tick duration.
*   **Barline Suppression:** Barlines are either hidden or converted to dashed tick-marks, allowing notes to float in pure physical relation to their absolute time.

### 34.2 Aleatoric Clusters and Indeterminate Pitch
*   **Cluster Bars:** Triggered by `[c4 c5]:4.cluster`. The engine calculates the $Y_{bottom}$ of C4 and the $Y_{top}$ of C5. Instead of rendering discrete noteheads, it emits a `SpannerId` rendering a thick SVG `<polygon>` or `<rect>` spanning the pitch range and the temporal width of the column.
*   **Indeterminate Pitch Boxes:** Rendered via a `BoxSpannerId`. The ECS anchors the four corners to $(Time_{start}, Pitch_{high})$ and $(Time_{end}, Pitch_{low})$, drawing a bounding box (often with SMuFL wavy lines) that strictly repels the Top/Bottom Skylines.

### 34.3 Audio Waveforms & Embedded Graphics
For live electronics, performers require visual cues of the audio buffer.
*   **The Asset Entity:** `AssetId` entities contain base64-encoded SVG or PNG data. 
*   **Timeline Anchoring:** The asset is anchored to a `TimeColumnId`. The Cassowary solver scales the graphic horizontally so its width exactly matches the temporal duration of the electronic cue, ensuring the waveform physically aligns with the acoustic instruments above it.

---

## 35. Advanced Metric topologies & Extreme Microtonality

### 35.1 Composite and Additive Meters
Time signatures like `3+2+3/8` require complex horizontal assembly.
*   **The Composite Glyph:** The engine instantiates a `TimeSignatureId` that holds an array of numerators. 
*   **SMuFL Assembly:** It queries SMuFL for the digits and the `timeSigPlus` glyph, kerning them horizontally into a single, unified AABB before injecting them into the Prefatory Column Rod constraint.

### 35.2 Metric Modulation
Equations like `[Quarter] =[Dotted Eighth]` must be assembled from text and font characters.
*   **The Tempo Equation Entity:** A specialized `AnnotationId`. The engine fetches the SMuFL `metNoteQuarter`, the text `=`, and `metNoteEighth` + `metAugmentationDot`. These are packed into a single bounding box and dropped into the `TopSkyline` above the measure boundary.

### 35.3 Sagittal and Ratio-Based Microtonality
To support Just Intonation (Ben Johnston) or the Sagittal accidental system, standard `alter` arrays are insufficient.
*   **The Extensible Accidental:** The `SpelledPitch` struct's `display` field is upgraded to accept a generic `SmuflCodepoint`.
*   **Execution:** If a user writes `c4.sagittal("U+E300")`, the engine bypasses the Gould State Machine and forcibly injects the specific Sagittal glyph into the `AccidentalColumnId`, utilizing its unique SMuFL cut-outs for collision detection.

---

## 36. Scholarly, Analytical, and Legal Frameworks

Scores published for academia or copyright require meta-typographical elements that sit entirely outside the musical flow.

### 36.1 Schenkerian Analysis
Schenkerian graphs use notes and slurs to represent harmonic hierarchy, not performance timing.
*   **Stemless & Scaled Notes:** Noteheads are stripped of stems (`.stem(none)`) or scaled down.
*   **Hierarchical Slurs:** Slurs in Schenkerian analysis nest deeply. The `kurbo` router calculates the intersection of *other slurs*, forcing background structural slurs to draw massively flattened, sweeping Bezier arches that encapsulate foreground slurs without touching them.

### 36.2 Critical Commentary and Footnotes
*   **The Footnote Entity:** A text annotation marked with an asterisk `*` in the score. 
*   **Page-Level Routing:** The line-breaking DAG identifies the `FootnoteId`. During the Page-Breaking phase (Section 15.2), the engine subtracts the height of the footnote text block from the physical page's $Y_{max}$, forcing the Cassowary solver to compress the musical systems slightly upward to leave room at the bottom of the page.

### 36.3 Copyright and Legal Metadata
*   **The Metadata Block:** Multi-line text arrays defined in `meta @{ copyright:["© 2026", "ISMN 979-0..."] }`.
*   **Absolute Placement:** Injected into the final SVG generation phase at absolute page coordinates (e.g., bottom center of Page 1), entirely decoupled from the musical skyline.

---

## 37. Orchestral and Historical Edge Cases

### 37.1 Divisi and Orchestral Grouping
When two flutes share a staff, they frequently switch between playing in unison (`a2`) and splitting into chords (`divisi`).
*   **Stateful Annotations:** When the IR detects `v1` and `v2` collapsing into identical unison pitches, it automatically generates a `TextAnnotation` reading "a2" positioned above the `TopSkyline`.
*   **Double Stops vs. Divisi:** The engine analyzes the voice assignments. If two notes belong to `v1`, it stems them together (double stop). If they belong to `v1` and `v2`, it stems them oppositely (up/down) to indicate divided players.

### 37.2 Mensural and Advanced Neumatic Notation
For Medieval and Renaissance music, standard metric ticks do not apply.
*   **Coloration and Prolation:** Parsed via specific `.head()` and `.color()` attributes. 
*   **Ligatures:** Rendered by substituting standard notes with SMuFL `mensural` and `medRen` compound glyphs. The Cassowary solver disables the standard Spring entirely, spacing the ligatures based strictly on optical padding and lyric syllable widths.

***
