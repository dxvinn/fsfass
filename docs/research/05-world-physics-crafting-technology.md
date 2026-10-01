# 05 — World, Physics, Crafting, Skills & Technology Emergence

Research track: (19) Weather/Climate, (20) Terrain/Geology/Worldgen, (21) Physics/Object abstraction,
(22) Crafting & Invention, (23) Skills, (24) Technology emergence.

Date of research: 2026-10-01. No game code. All descriptions are in our own words.

## How verification was done (read first)

- GitHub repos were verified by fetching their github.com page (stars/license as shown on the page at research time).
  Star counts are approximate. "Last activity" is often not shown on the rendered page; where it was not, it says so.
- The research sandbox blocked several domains (arxiv.org, pnas.org, redblobgames.com, mewo2.com,
  dwarffortresswiki.org, bookdown.org, gameaipro.com). For those, facts come from search-result snippets,
  from source code fetched via raw.githubusercontent.com, or from well-known published formulas; these are marked
  **(secondary)**. Papers are cited by title/venue so they can be looked up.
- Cataclysm-DDA skill/rust mechanics were read directly from `src/skill.cpp`, `src/character.cpp`
  and `src/character_proficiency.cpp` on the master branch (fetched 2026-10-01). mapgen4's rainfall
  algorithm was read directly from `map.ts`. Crafter's recipe table was read from `crafter/data.yaml`.
- **License flags for a possibly commercial, closed-source game**: GPL/LGPL/AGPL/CC-BY-SA/non-commercial items are
  marked "ALGORITHM ONLY". MIT/Apache/BSD/CC0 code could be reused with attribution, but the recommendation
  throughout is to re-implement algorithms in our own engine (data layout, determinism and LOD needs differ).

Candidate count: ~50 entries (worldgen 16, climate 5, physics/materials 7, crafting/open-endedness 12, skills 3,
technology/cultural evolution 9; the CDDA skills entry points back to Part C).

---

# Part A — Terrain, geology and worldgen (track 20)

### plate-tectonics (platec)
- URL: https://github.com/Mindwerks/plate-tectonics
- License: LGPL-3.0
- Language: C++ with Python bindings (pyplatec)
- Activity/maintenance: ~96 stars, ~745 commits; last commit date not shown; low activity (mature library).
- Architecture: 2D wrapped heightmap; N plates each own a raster "crust" layer (height + age per cell) that is
  translated by a velocity vector every step. Where plates overlap, collision rules decide: continental crust
  folds/aggregates onto the overriding plate (mountain building, "folding ratio"), oceanic crust subducts;
  divergent gaps are filled with new young oceanic crust. A periodic erosion pass smooths. Driven by
  `step()` until plates stop or iteration limit; parameters: seed, size, sea level, erosion period,
  folding ratio, aggregation overlap thresholds, number of plates, cycle count.
- Concept modeled: kinematic plate tectonics on a flat torus, producing realistic mountain chains and continent shapes.
- Computational cost: seconds to tens of seconds for 512x512 on one core; scales with area x steps.
- What we can learn: raster-per-plate representation; crust age as a by-product (useful to place rock types:
  old cratons vs young basalt); collision classification by crust type.
- What we could integrate: ALGORITHM ONLY (LGPL — dynamic linking would technically be allowed, but static
  linking on consoles/closed builds is awkward). Re-implement the idea.
- What we should NOT integrate: the flat torus topology (wrong for a globe; fine for a cylinder/torus world);
  its fixed raster resolution.
- Scalability: offline worldgen only; not a runtime system.

### WorldEngine
- URL: https://github.com/Mindwerks/worldengine
- License: MIT
- Language: Python (uses pyplatec)
- Activity/maintenance: ~1.1k stars; largely dormant.
- Architecture: pipeline = platec plates -> noise detail -> temperature (latitude + elevation) -> precipitation
  (latitude bands + rain shadow) -> erosion and river tracing -> humidity and permeability -> biome by the
  **Holdridge life zones** model (biotemperature x annual precipitation x potential-evapotranspiration ratio,
  hexagonal classification). Outputs are layered rasters (elevation, ocean, precipitation, temperature, biome, rivers).
- Concept modeled: complete "physical world" pipeline in the classic order.
- Computational cost: tens of seconds to minutes for 512x512 in Python.
- What we can learn: ordering of stages and the Holdridge alternative to Whittaker (Holdridge yields ~38 zones
  and naturally incorporates evapotranspiration, which matters for agriculture later).
- What we could integrate: MIT — code reusable, but we'd port the ideas (Python is too slow for our target).
- What we should NOT integrate: its simplistic precipitation (no seasonal cycle).
- Scalability: offline.

### Tectonics.js
- URL: https://github.com/davidson16807/tectonics.js
- License: CC-BY-4.0 (unusual for code; attribution required)
- Language: JavaScript (WebGL)
- Activity/maintenance: ~233 stars, ~1.7k commits; author's later work moved elsewhere; low recent activity.
- Architecture: 3D sphere represented by a fixed vertex grid; plates as rigid bodies rotating around Euler poles;
  crust carried as fields (thickness, density, rock-type masses: sediment, sedimentary, metamorphic, felsic/mafic);
  isostasy computes elevation from crust thickness/density; simple climate (insolation, temperature, precipitation)
  and a rock cycle (weathering -> sediment -> lithification -> metamorphism).
- Concept modeled: long-term geology over hundreds of millions of years with a rock cycle.
- Computational cost: interactive in the browser at ~10k vertices.
- What we can learn: **isostasy-based elevation** (elevation as consequence of crust thickness/density — more robust
  than hand-tuned uplift), and **rock-type mass fields per cell** — exactly what we need to place flint, clay,
  ores and building stone later.
- What we could integrate: algorithm, and code with attribution if desired (CC-BY permits commercial use).
- What we should NOT integrate: its renderer/UI.
- Scalability: good for our coarse planet mesh (10k–100k cells).

### Undiscovered Worlds
- URL: https://github.com/JonathanCRH/Undiscovered_Worlds (classic flat version: https://github.com/JonathanCRH/Undiscovered_Worlds_Classic)
- License: GPL-3.0
- Language: C++ (raylib + ImGui)
- Activity/maintenance: ~106 stars, ~149 commits; new spherical version is experimental (no save/load yet).
- Architecture (secondary, from author's blog/search): global map with "tectonic-like" mountain ranges built from
  ridge templates, monthly temperature/precipitation from latitude, ocean currents and prevailing winds, detailed
  river routing incl. lakes, salt lakes, deltas, and a zoomable **regional map** that procedurally upsamples the
  global map with consistent rivers.
- Concept modeled: Earth-like plausible geography at two scales.
- Computational cost: tens of seconds for a world.
- What we can learn: two-scale approach (global coarse model + deterministic regional upsampling that respects
  global rivers/coasts) — directly matches our LOD needs.
- What we could integrate: ALGORITHM ONLY (GPL-3).
- What we should NOT integrate: any code.
- Scalability: regional on-demand detail is the scalable idea.

### World Orogen (planet_heightmap_generation)
- URL: https://github.com/raguilar011095/planet_heightmap_generation (also mirrored at mattfoxley/...); site orogen.studio
- License: GPL-3.0
- Language: JavaScript (Three.js, Delaunator)
- Activity/maintenance: ~105 stars, ~132 commits, active issues/PRs.
- Architecture: Fibonacci-sphere points -> spherical Delaunay/Voronoi; plates by farthest-point seeding + round-robin
  flood fill with directional bias & compactness; elevation from blended distance fields to boundaries,
  stress-driven uplift, asymmetric mountain profiles, shelf/slope/abyss; erosion: glacial (fjords), hydraulic via
  priority-flood routing, thermal talus, soil creep; atmosphere: seasonal ITCZ that varies with longitude, Gaussian
  pressure bands, Coriolis-deflected geostrophic wind (summer & winter); rule-based ocean gyres with western
  boundary intensification; precipitation blends moisture advection (convective lift at ITCZ, orographic lift,
  lee cyclogenesis) with a zonal heuristic; Köppen-Geiger classification with lapse rate and continentality.
- Concept modeled: the most complete open browser-based "tectonics -> climate -> Köppen" pipeline found.
- Computational cost: seconds in a browser for ~100k cells.
- What we can learn: this is close to the pipeline we want; especially the *blend* of physical advection with a
  zonal heuristic (keeps results Earth-like even when the toy physics misbehaves).
- What we could integrate: ALGORITHM ONLY (GPL-3).
- What we should NOT integrate: code.
- Scalability: fine for 100k–500k cells offline.

### Andy Gainey's planet generator (Experilous)
- URL: https://github.com/againey/planet-generator
- License: CC0-1.0 (public domain)
- Language: JavaScript (three.js)
- Activity/maintenance: ~14 stars, 4 commits, 2014 prototype, unmaintained.
- Architecture: subdivided icosahedron with random edge rotations ("distorted" mesh for organic tiles); plates from
  random seed tiles + flood fill; each plate gets rotation axis + angular rate (+ drift); at boundary edges compute
  relative motion and split into pressure (normal component) and shear (tangential); boundary elevation chosen by
  case (collision, subduction, divergence, shear) and blended inward by distance; air currents for heat/moisture;
  rule-based biomes.
- Concept modeled: cheap, non-iterative "one-shot" tectonics on a sphere.
- Computational cost: about one second for ~10k–40k tiles.
- What we can learn: boundary classification via relative velocity decomposition; one-shot distance blending instead of
  iterative simulation. This is the fastest way to get believable mountains.
- What we could integrate: code or algorithm freely (CC0).
- What we should NOT integrate: its weather (very rough).
- Scalability: excellent.

### Procedural Tectonic Planets (Cortial, Peytavie, Galin, Guérin, Eurographics 2019)
- URL: https://hal.science/hal-02136820v1/file/2019-Procedural-Tectonic-Planets.pdf
- License: paper (no code verified)
- Language: n/a
- Activity/maintenance: published 2019 (Computer Graphics Forum 38(2)).
- Architecture (secondary): plates on a sphere as sets of sample points carrying crust attributes (type, thickness,
  age, orogeny type, local ridge direction); interactive time steps apply subduction uplift (function of distance
  to front, relative speed, and crust), continental collision (terranes merge, orogeny), rifting events, oceanic
  crust generation at ridges with ridge-aligned features; final relief is amplified with procedural/exemplar detail.
- Concept modeled: controllable, event-driven tectonics at interactive rates — ideal for a "god" that triggers rifts.
- Computational cost: interactive (sub-second steps) on ~500k samples per their claims.
- What we can learn: **god-tool design**: "split this continent", "push these plates together" as discrete events
  that re-run local uplift; storing orogeny type for later mineral placement.
- What we could integrate: algorithm.
- What we should NOT integrate: exemplar-based amplification (needs DEM datasets & licensing).
- Scalability: good for our global mesh.

### Azgaar's Fantasy Map Generator
- URL: https://github.com/Azgaar/Fantasy-Map-Generator
- License: MIT (verified from LICENSE file; note: earlier assumptions that it is GPL are wrong for the current repo)
- Language: JavaScript, transitioning to TypeScript
- Activity/maintenance: ~6k stars; actively maintained.
- Architecture: jittered points -> Voronoi cells (packed graph); heightmap from templates (hill, pit, range, trough,
  strait, mask) composed by a small DSL; temperature from latitude and altitude; precipitation by winds per latitude
  band with moisture depletion over land and mountains; rivers by downhill flux with depression handling; biomes from
  a temperature x moisture matrix; then cultures, states, burgs, religions, routes, names; layered architecture
  (state / generation / editing / rendering).
- Concept modeled: fantasy atlas with political layers; physically plausible "enough".
- Computational cost: ~1–3 s for 10k–100k cells in browser.
- What we can learn: the heightmap template DSL (great for god-mode "brushes"); editor-friendly separation of data
  vs generation vs rendering; cultural expansion as cost-weighted flood fill (relevant to other tracks).
- What we could integrate: MIT code allowed with notice; we'd still port to our engine.
- What we should NOT integrate: its political generation (our civs must emerge, not be stamped).
- Scalability: good up to ~100k cells; not a time-stepped sim.

### mapgen4 (Red Blob Games, Amit Patel)
- URL: https://github.com/redblobgames/mapgen4
- License: Apache-2.0
- Language: TypeScript (WebGL)
- Activity/maintenance: ~900 stars; maintained occasionally.
- Architecture (read from `map.ts`): compact **dual mesh** (Delaunay triangles + Voronoi regions, half-edge arrays).
  Rainfall: regions are sorted once by their projection on the wind vector; then a single sweep in wind order
  sets each region's humidity to the average of upwind neighbors' humidity; ocean regions gain humidity by
  evaporation; when humidity exceeds a cap that shrinks with elevation (roughly 1 − elevation), the excess falls as
  orographic rain scaled by a rain-shadow parameter; rainfall also has a base term proportional to humidity.
  Rivers: triangles are visited in a priority-flood order from the coast; each triangle's flow = flow constant x
  moisture², accumulated downstream along "downslope sides"; river width from flow.
- Concept modeled: rain shadows and river networks in one O(n log n) pass (sort) + O(n) sweep.
- Computational cost: real-time regeneration at ~25k–1M cells.
- What we can learn: the **wind-ordered single sweep** is the key trick for our seasonal climate: no iteration needed,
  cost O(n) per season once sorted per wind direction.
- What we could integrate: Apache-2.0 — code reuse allowed with NOTICE; algorithm is what matters.
- What we should NOT integrate: single global wind direction (we need latitude-dependent winds — sort per
  wind band or use iterative relaxation instead).
- Scalability: excellent.

### Martin O'Leary — terrain (Uncharted Atlas)
- URL: https://github.com/mewo2/terrain (notes: mewo2.com/notes/terrain — secondary, domain blocked)
- License: MIT
- Language: JavaScript (d3)
- Activity/maintenance: ~3k stars; author declares it finished, not maintained.
- Architecture (secondary, well known): Lloyd-relaxed points -> Voronoi mesh; heightmap from primitives (slope, cone,
  blobs, noise); **Planchon–Darboux** depression filling so every cell drains; flux = accumulated upstream area;
  erosion rate proportional to sqrt(flux) x slope (a stream-power-law form); clean coastlines; city placement by a
  score combining flux (rivers), coast and distance to other cities; Markov-ish naming.
- Concept modeled: minimal believable fluvial terrain.
- Computational cost: seconds for ~16k cells.
- What we can learn: "erosion ∝ sqrt(flux)·slope" is the cheapest credible erosion; settlement scoring as an emergent
  attractor (water access, flatness) — agents in our game should *compute* the same signals.
- What we could integrate: MIT; algorithm.
- What we should NOT integrate: naming system (other track).
- Scalability: good.

### SimpleHydrology / SoilMachine (Nick McDonald, "weigert")
- URL: https://github.com/weigert/SimpleHydrology
- License: MIT
- Language: C++ (TinyEngine)
- Activity/maintenance: ~750 stars; last significant change Jan 2023.
- Architecture: particle ("droplet") erosion on a heightmap with **persistent discharge and momentum maps**: each
  particle descends, picks up sediment up to a capacity that grows with speed & discharge, deposits when slowing;
  the discharge map is an exponential moving average of particle passage (so rivers persist), and momentum maps
  feed back into particle motion, creating meanders; vegetation layer stabilizes soil.
- Concept modeled: long-lived river channels and meanders at local scale.
- Computational cost: ~hundreds of thousands of particles for a 256²–512² map; seconds on CPU.
- What we can learn: **persistent discharge field** is the right "memory" for rivers in a time-stepped world — when the
  god changes rainfall, rivers can migrate gradually instead of being regenerated.
- What we could integrate: MIT; algorithm (we'd want it on GPU or chunk-local).
- What we should NOT integrate: as a global process (too slow for a planet).
- Scalability: chunk-local detail only (zoomed-in regions).

### Hydraulic-Erosion (Sebastian Lague)
- URL: https://github.com/SebLague/Hydraulic-Erosion
- License: MIT
- Language: C# (Unity, compute shader variant)
- Activity/maintenance: ~1k stars; finished project.
- Architecture: droplet erosion (based on Hans Theobald Beyer's thesis): bilinear gradient, inertia blending of
  direction, sediment capacity = max(min slope, −Δh) x speed x water x capacity factor, erode with a radius brush,
  deposit when over capacity, evaporate water each step; ~70k droplets for a demo map.
- Concept modeled: visual erosion detailing.
- Computational cost: fast on GPU; 1–3 s CPU for 256².
- What we can learn: parameterization set (inertia, capacity, erode/deposit speed, evaporation, radius) for chunk detail.
- What we could integrate: MIT; algorithm.
- What we should NOT integrate: as gameplay hydrology (no persistent rivers).
- Scalability: local only.

### terrain-erosion-3-ways (Daniel Andrino)
- URL: https://github.com/dandrino/terrain-erosion-3-ways
- License: MIT
- Language: Python
- Activity/maintenance: ~950 stars; finished.
- Architecture: three methods compared: (1) grid hydraulic simulation (water, sediment, gradients); (2) GAN trained on
  USGS DEMs; (3) **river-network-first**: Poisson-disc points + Delaunay, grow a river tree from the coast, then set
  terrain height as a function of distance/upstream position along the tree.
- Concept modeled: trade-offs between simulation, learning, and constructive generation.
- Computational cost: simulation O(N³)-ish in grid side; river-network O(N² log N); GAN days of training.
- What we can learn: river-network-first generation is cheapest for guaranteed drainage; use when a god "draws" a river.
- What we could integrate: MIT; algorithm.
- What we should NOT integrate: the GAN (data/licensing, opacity).
- Scalability: river-first method scales well.

### RichDEM (Priority-Flood, Barnes et al.)
- URL: https://github.com/r-barnes/richdem
- License: GPL-3.0
- Language: C++ (header-only) + Python
- Activity/maintenance: ~330 stars, ~2.5k commits.
- Architecture: Priority-Flood (seed priority queue with boundary/ocean cells, pop lowest, raise/flag neighbors) for
  depression filling or breaching in O(n log n) (O(n) variants for integer DEMs); D8 / D-infinity flow
  directions; flow accumulation; flat resolution; watershed labeling.
- Concept modeled: hydrologically correct drainage on DEMs.
- Computational cost: millions of cells per second.
- What we can learn: **Priority-Flood** is the standard; also yields lake extents (filled volume) and spill points.
- What we could integrate: ALGORITHM ONLY (GPL-3). The algorithm is published (Barnes, Lehman, Mulla 2014) and is
  simple to reimplement.
- What we should NOT integrate: code.
- Scalability: excellent.

### Fastscape (stream power law landscape evolution)
- URL: https://github.com/fastscape-lem/fastscape (core: fastscapelib)
- License: BSD-3-Clause
- Language: Python wrapper over Fortran/C++ core
- Activity/maintenance: ~76 stars; GFZ Potsdam group; maintained.
- Architecture: landscape evolution model: dh/dt = U − K·A^m·S^n + D·∇²h (uplift, stream-power fluvial incision with
  drainage area A and slope S, hillslope diffusion). Braun & Willett (2013) implicit O(n) solver: order nodes from
  base level upstream (stack), then solve each node implicitly given its receiver — unconditionally stable,
  big time steps.
- Concept modeled: geologic-time landscape shaping consistent with tectonic uplift.
- Computational cost: O(n) per step; 1M nodes ~ sub-second per step in native code.
- What we can learn: this is the right *large-scale* erosion for worldgen (and for god "fast-forward 1 Myr" actions).
- What we could integrate: BSD — code reuse allowed; we'd port the Braun–Willett solver to our mesh.
- What we should NOT integrate: xarray-simlab framework.
- Scalability: excellent; usable on our global mesh.

### Dwarf Fortress world generation (reference)
- URL: https://www.gamedeveloper.com/design/interview-the-making-of-dwarf-fortress ; GameAIPro 2 ch. 41 "Simulation Principles from Dwarf Fortress" (Tarn Adams) — secondary (PDF domain blocked)
- License: proprietary game; reference only
- Language: C++
- Activity/maintenance: commercial, active.
- Architecture (secondary): separate scalar fields seeded on a coarse grid and fractally refined (midpoint displacement):
  elevation, rainfall, temperature, drainage, volcanism, savagery/wildness; temperature biased by latitude and
  elevation; rainfall biased by orographic effects/rain shadows; "temporary rivers" erode mountains, then permanent
  rivers carve to the sea; biome = function of these fields; then mineral layers, caves, then history simulation.
  Material properties defined in raw text files (see DF materials entry below).
- Concept modeled: biome from independent interacting fields; world history after physical generation.
- Computational cost: minutes for large worlds including history.
- What we can learn: **keep fields separate** and let biome be a derived classification — exactly what lets a god edit
  rainfall without breaking things. Also: "drainage" as its own field (soil permeability) matters for swamps vs steppes.
- What we could integrate: ideas only.
- What we should NOT integrate: anything copied.
- Scalability: n/a.

---

# Part B — Weather and climate (track 19)

### climlab
- URL: https://github.com/climlab/climlab
- License: MIT
- Language: Python (+ Fortran radiation codes)
- Activity/maintenance: ~250 stars; v0.9.2 March 2025.
- Architecture: process-oriented: models are trees of coupled processes (insolation, radiation, diffusion, convection)
  stepping shared state. Includes: 1D **diffusive energy balance models** (Budyko–Sellers / North style) with
  seasonal insolation for arbitrary orbital parameters; moist EBM; radiative-convective columns (RRTMG, CAM3, grey);
  1D advection-diffusion solvers.
- Concept modeled: textbook climate physics.
- Computational cost: 1D EBM with 90 latitudes: microseconds per step; years in milliseconds.
- What we can learn: the EBM equation: heat capacity · dT/dt = absorbed solar Q(lat, day)·(1 − albedo(T)) − (A + B·T)
  + divergence of (D·grad T). With ice–albedo feedback this gives snowball/hothouse bifurcations — a fun god lever
  ("dim the sun 3%").
- What we could integrate: MIT — reference implementation to validate ours.
- What we should NOT integrate: RRTMG etc. (overkill).
- Scalability: EBM generalizes to a 2D mesh at negligible cost.

### ExoPlaSim
- URL: https://github.com/alphaparrot/ExoPlaSim
- License: GPL-2.0
- Language: Fortran + Python API
- Activity/maintenance: ~125 stars; 3.x final maintenance release.
- Architecture: PlaSim spectral GCM extended for exoplanets (rotation, pressure, tidal locking, ice, etc.); T21 (32x64
  grid) default with 10 vertical layers.
- Concept modeled: real 3D general circulation.
- Computational cost (secondary, from paper Paradise et al. 2022 MNRAS): ~30–60 s walltime per simulated year at T21
  with 45-min timestep on a parallel node; a few minutes/year with 4 MPI threads. Needs decades to equilibrate →
  tens of minutes to hours per world.
- What we can learn: an offline "ground truth" to tune our toy climate (e.g., check that our rain shadows and monsoons
  land in the right places for test continents).
- What we could integrate: ALGORITHM ONLY / offline tool (GPL-2; Fortran).
- What we should NOT integrate: as a runtime component (far too expensive, and GPL).
- Scalability: none for gameplay.

### Köppen / Whittaker / Holdridge classification (concept + small repos)
- URL: e.g. https://github.com/PeteMichaud/koppen (generates Köppen maps from temperature + rainfall maps) — verified listed in search; not inspected
- License: unverified for the small repos
- Language: various
- Activity/maintenance: unverified
- Architecture: Köppen needs 12 monthly mean temperatures + 12 monthly precipitation totals: group E (warmest month
  < 10 °C), B (dry: annual P below a threshold ≈ 20·T_mean + offset depending on whether rain falls in summer or
  winter), A (coldest month ≥ 18 °C), C/D split by coldest month (0 °C or −3 °C), then subletters by seasonality
  of rain and summer heat. Whittaker: 2D lookup of annual mean T and annual P. Holdridge: log-scale biotemperature,
  precipitation, PET ratio.
- Concept modeled: climate -> biome labels.
- Computational cost: trivial per tile.
- What we can learn: **we need at least 4 (better 12) seasonal samples per tile** to classify correctly; Köppen is a
  good debug overlay; Whittaker is a good driver for vegetation fields.
- What we could integrate: thresholds are public scientific knowledge; implement ourselves.
- What we should NOT integrate: discrete biome labels as the *simulation* state (vegetation must be continuous fields).
- Scalability: trivial.

### Stochastic weather generators (Richardson 1981 "WGEN" class) — concept
- URL: unverified (concept from hydrology literature; "Richardson, Stochastic simulation of daily precipitation, temperature and solar radiation", Water Resources Research 1981)
- License: n/a
- Language: n/a
- Activity/maintenance: classic, widely re-implemented.
- Architecture: precipitation occurrence as a 2-state Markov chain per day (P(wet|dry), P(wet|wet)); amount from a gamma
  (or exponential) distribution; temperature and radiation as AR(1) residuals around seasonal means, conditioned on
  wet/dry state.
- Concept modeled: daily weather statistics consistent with a climatology.
- Computational cost: a few random draws per region per day.
- What we can learn: separates *climate* (slow, simulated) from *weather* (fast, sampled) — the key LOD idea for us.
- What we could integrate: the method (public).
- What we should NOT integrate: nothing problematic.
- Scalability: excellent; spatial correlation by sampling on a coarse grid and interpolating.

### Wind/moisture sweep + seasonal ITCZ (mapgen4 + World Orogen pattern)
- URL: see mapgen4 and World Orogen entries above
- License: Apache-2.0 / GPL-3.0 respectively (algorithm-only for the latter)
- Language: TypeScript / JavaScript
- Activity/maintenance: see above
- Architecture: combine (a) zonal pressure belts whose latitudes shift with the season-following thermal equator, (b)
  Coriolis-deflected winds from pressure gradient, (c) monsoon perturbation from land–sea temperature contrast,
  (d) moisture carried downwind with evaporation over warm water and rain-out on uplift.
- Concept modeled: plausible seasonal precipitation patterns including monsoons and rain shadows.
- Computational cost: O(n log n) sort per wind configuration, O(n) per season.
- What we can learn: see "Recommended design".
- What we could integrate: algorithm.
- What we should NOT integrate: n/a.
- Scalability: excellent.

---

# Part C — Physics, materials, objects and affordances (track 21)

### Dwarf Fortress material definition tokens (reference)
- URL: https://dwarffortresswiki.org/index.php/DF2014:Material_science (secondary; domain blocked from sandbox)
- License: proprietary game; documentation on community wiki (GFDL/CC-BY-SA style — do not copy text)
- Language: raw text data
- Activity/maintenance: active game.
- Architecture (secondary): each material has thermal tokens (SPEC_HEAT, IGNITE_POINT, MELTING_POINT, BOILING_POINT,
  HEATDAM_POINT, COLDDAM_POINT, MAT_FIXED_TEMP), densities (SOLID_DENSITY, LIQUID_DENSITY), and six mechanical
  modes each with YIELD / FRACTURE / STRAIN_AT_YIELD (IMPACT, COMPRESSIVE, TENSILE, TORSION, SHEAR, BENDING), plus
  MAX_EDGE (sharpness ceiling; e.g. obsidian far above generic stone) and usage flags (ITEMS_WEAPON, IS_STONE …).
  Combat: edged attacks are resolved against SHEAR_* values and the weapon's edge; blunt attacks against IMPACT_*.
  Material templates (STONE_TEMPLATE, METAL_TEMPLATE) give inherited defaults.
- Concept modeled: per-material physics for weapons/armor/temperature, with state changes by temperature.
- Computational cost: cheap lookups.
- What we can learn: **mechanical modes x (yield, fracture) is a compact, general schema**; MAX_EDGE as a material ceiling
  vs. the object's current edge; template inheritance.
- What we could integrate: the *idea* of the schema (not data).
- What we should NOT integrate: DF's numbers (copyright of data tables) or text.
- Scalability: data-driven lookups scale fine.

### Cataclysm: Dark Days Ahead — materials, items, skills (data + code)
- URL: https://github.com/CleverRaven/Cataclysm-DDA (files: data/json/materials.json, src/skill.cpp, src/character.cpp, src/character_proficiency.cpp)
- License: CC-BY-SA 3.0 (some files Apache-2.0 / OFL) — **share-alike: ALGORITHM ONLY for a closed game**
- Language: C++ + JSON data
- Activity/maintenance: ~13k stars, ~130k commits, very active.
- Architecture: materials in JSON with density, specific heat (solid/liquid), latent heat, freezing point, chip_resist,
  soft, conductive, breathability, wind_resist, resist {bash, cut, stab, bullet, acid, heat}, burn_data
  (fuel, smoke, burn rate per intensity), fuel_data, rotting, repaired_with, salvaged_into. Items reference
  materials (with portions); recipes are authored JSON with skill/proficiency/tool-quality requirements
  (tool *qualities* like CUT 1, HAMMER 2 are an affordance abstraction).
  Skills (read directly from source): two tracks per skill — **practical exercise** and **theoretical knowledge**.
  XP to next level = 10,000 x (level+1)². Practice converts an "amount" (capped, scaled by focus) into practical XP;
  when knowledge > practice, practice is multiplied by a "catch-up" factor (≈ 1 + (2·INT + PER)/24, ~2 for average)
  times the level gap — relearning is faster. Rust: no rust within 24 h of last practice; afterwards once per day
  practical XP drops by an amount starting at ~4% of a level and shrinking with sqrt of accumulated rust (rust
  slows as it accumulates, and stops when accumulated rust exceeds 3 levels' worth); a rust_resist trait scales it;
  **knowledge never rusts**, so rust only opens a gap that catch-up closes quickly. Teaching/book learning gives
  knowledge XP scaled by 2/(gap+1) where gap = teacher knowledge − learner knowledge (best teacher is only somewhat
  better). Focus: practice drains a focus pool (drain roughly quadratic in amount), and XP is scaled by focus;
  proficiencies (e.g. knapping, pottery) are time-practiced binary competencies that also use focus.
- Concept modeled: detailed survival crafting with skill rust and relearning.
- Computational cost: per-character, per-action trivial.
- What we can learn: the **knowledge/practice split** is the best design found for rust + relearning; tool *qualities*
  as derived affordances (we'd compute them from material+shape rather than author them).
- What we could integrate: formulas as inspiration (re-derived, not copied).
- What we should NOT integrate: JSON data or code (CC-BY-SA); authored recipe lists (contradicts our goal).
- Scalability: per-agent cost fine for 10k agents; aggregate for 1M.

### The Powder Toy
- URL: https://github.com/The-Powder-Toy/The-Powder-Toy
- License: GPL-3.0
- Language: C++ (SDL), Lua API
- Activity/maintenance: ~5.3k stars; active (2026).
- Architecture: one particle per grid cell; per-element property table (state, weight, heat conductivity, flammability,
  explosiveness, melting/boiling transitions with target elements); coarser grids for air pressure and velocity
  (simple fluid solver) and ambient heat; per-element update functions for special reactions.
- Concept modeled: emergent physics/chemistry sandbox.
- Computational cost: ~600x400 cells at 60 fps on one core.
- What we can learn: **phase transitions as property thresholds with target materials**; coarse air/heat grid coupled to a
  fine material grid (multi-resolution).
- What we could integrate: ALGORITHM ONLY (GPL-3).
- What we should NOT integrate: per-cell particle physics at world scale.
- Scalability: local only (fire/flood in a settlement when the camera is there).

### Noita (GDC 2019, "Exploring the Tech and Design of Noita", Petri Purho)
- URL: https://www.gdcvault.com/play/1025695/Exploring-the-Tech-and-Design ; video https://www.youtube.com/watch?v=prXuyMCgbTc
- License: proprietary game; talk is reference
- Language: C++
- Activity/maintenance: shipped 2020.
- Architecture: falling-sand per-pixel simulation; world split into 64x64 chunks, each with a dirty rectangle so only
  active regions update; multithreading via a 4-pass checkerboard of chunks where each thread may write within its chunk
  plus a 32-pixel margin; destructible rigid bodies via marching squares + Box2D.
- Concept modeled: large continuous falling-sand worlds.
- Computational cost: real-time for a screen-sized active region.
- What we can learn: **dirty-rect/sleeping chunks** — the same idea applies to our thermal/fire CA and to any per-tile
  physics: only simulate "awake" regions.
- What we could integrate: ideas.
- What we should NOT integrate: pixel-level material sim as the core world representation (contrast case: too
  expensive for a planet with 1M organisms).
- Scalability: local.

### Sandspiel
- URL: https://github.com/MaxBittker/sandspiel
- License: MIT
- Language: Rust -> WebAssembly, WebGL
- Activity/maintenance: ~3.2k stars; ~346 commits.
- Architecture: cellular automaton grid with per-species update rules (sand, water, fire, plant, acid, ice …) plus a
  GPU fluid sim for wind that pushes particles; compact cell struct (species, two small registers, clock).
- Concept modeled: falling sand with simple material interactions.
- Computational cost: real-time in browser.
- What we can learn: tiny per-cell state (a few bytes) suffices for rich interactions; wind field coupling.
- What we could integrate: MIT; ideas.
- What we should NOT integrate: species-specific hand rules as our interaction system.
- Scalability: local.

### AI2-THOR (object state changes)
- URL: https://github.com/allenai/ai2thor
- License: Apache-2.0
- Language: Python + Unity/C#
- Activity/maintenance: ~1.8k stars; active.
- Architecture: objects carry boolean affordance/state properties (pickupable, sliceable→sliced, cookable→cooked,
  breakable→broken, fillable→filled with liquid, toggleable, dirtyable, temperature hot/cold/room via heat sources),
  and actions are valid only if the property holds.
- Concept modeled: household affordances for embodied agents.
- Computational cost: Unity physics; not relevant.
- What we can learn: a minimal **state-change vocabulary** — useful as the *effect tokens* agents perceive.
- What we could integrate: the vocabulary idea.
- What we should NOT integrate: boolean authored affordances (we derive them from continuous properties).
- Scalability: n/a.

### Affordance theory and computational affordance models (Gibson; Şahin et al.; Zech et al.; Jamone et al.)
- URL: Zech et al. 2017 "Computational models of affordance in robotics: a taxonomy and systematic classification", Adaptive Behavior https://journals.sagepub.com/doi/10.1177/1059712317726357 ; Jamone et al. 2016 survey (IEEE TCDS) https://iris.unife.it/retrieve/e309ade4-f113-3969-e053-3a05fe0a2c94/jamone16affordancesSurvey.pdf
- License: papers
- Language: n/a
- Activity/maintenance: active field.
- Architecture: Gibson: affordances are action possibilities relative to an agent, not object labels. Şahin et al. (2007)
  formalism: an affordance is a learned relation (effect, (entity, behavior)) — "applying behavior b to an entity with
  features f generates effect e". Robots learn it by exploration: act, measure feature change, cluster effects into
  categories, train predictors from object features (+ action) to effect class; then plan by chaining predicted effects.
  Tool affordances add a tool entity: (effect, (tool features, target features, behavior)).
- Concept modeled: learning what objects are good for from interaction.
- Computational cost: tiny if features are symbolic/low-dimensional (our case).
- What we can learn: this is the **core representation for our invention system**: agents learn
  (behavior, feature-predicate-of-tool, feature-predicate-of-target) -> effect; generalization happens over
  *properties*, so "this flint flake cuts" generalizes to "anything with edge > 0.5 cuts soft things".
- What we could integrate: the formalism.
- What we should NOT integrate: vision-based deep affordance models.
- Scalability: belief tables per agent are small; shared per culture at LOD.

---

# Part D — Crafting, invention, open-ended discovery (track 22)

### Crafter (Hafner 2021)
- URL: https://github.com/danijar/crafter
- License: MIT
- Language: Python
- Activity/maintenance: ~600 stars; stable benchmark.
- Architecture: 64x64 procedurally generated 2D world; 22 achievements; recipes in `data.yaml` as three tables:
  `collect` (e.g. stone requires wood_pickaxe; iron requires stone_pickaxe; diamond requires iron_pickaxe; sapling from
  grass with 10% probability), `place` (table costs 2 wood; furnace costs stone and must be on grass/sand/path; plant
  requires sapling), `make` (wood pickaxe = 1 wood near a table; stone pickaxe = wood + stone near table; iron
  pickaxe = wood + coal + iron near table and furnace). Score = geometric-mean-like aggregate over achievement rates;
  humans ~50%.
- Concept modeled: a tiny, authored tech tree with "nearby station" requirements.
- Computational cost: ~thousands of steps/s per env.
- What we can learn: the **"nearby" requirement** (stations/context) is a useful pattern — our analogue is "near heat
  source ≥ 600 °C". It also shows the failure mode we must avoid: fixed recipes → closed-ended.
- What we could integrate: as a test harness for agent learning only.
- What we should NOT integrate: authored recipes.
- Scalability: n/a.

### Craftax (Matthews et al. 2024)
- URL: https://github.com/MichaelTMatthews/Craftax
- License: MIT
- Language: Python/JAX
- Activity/maintenance: ~450 stars.
- Architecture: Crafter + NetHack-like mechanics (floors, magic, farming, archery) entirely in JAX, vectorized: whole
  environment state is arrays, step is a pure function → millions of steps/s on GPU.
- Concept modeled: open-endedish RL benchmark at scale.
- Computational cost: very high throughput on GPU.
- What we can learn: **state-as-arrays, pure step function** is how we'd batch-simulate thousands of low-LOD agents.
- What we could integrate: MIT; design pattern.
- What we should NOT integrate: content.
- Scalability: excellent (design lesson).

### Voyager (Wang et al. 2023) — skill library
- URL: https://github.com/MineDojo/Voyager
- License: MIT
- Language: Python + JS (Mineflayer)
- Activity/maintenance: ~7k stars; research code, inactive.
- Architecture: LLM agent in Minecraft with (1) automatic curriculum proposing next tasks by exploration progress,
  (2) **skill library**: verified programs stored with an embedding of their description, retrieved by similarity and
  composed, (3) iterative prompting with environment feedback, execution errors, and a self-verification critic.
- Concept modeled: lifelong accumulation of reusable, composable skills.
- Computational cost: many GPT-4 calls per task — expensive.
- What we can learn: techniques should be **stored as composable, parameterized programs** (sequences of verbs with
  role predicates), verified by execution; retrieval by similarity to the current goal. Our "culture" is a
  population-level skill library with imperfect copying.
- What we could integrate: the library structure (no LLM required).
- What we should NOT integrate: LLM-in-the-loop for every agent (cost) — at most for rare "notable" individuals/narration.
- Scalability: LLM version does not scale; symbolic version does.

### OMNI (Zhang, Lehman, Stanley, Clune; ICLR 2024)
- URL: https://github.com/jennyzzt/omni
- License: unverified
- Language: Python (unverified)
- Activity/maintenance: research code (unverified).
- Architecture: learning-progress-based task sampling + an LLM "model of interestingness" that filters tasks that are
  learnable *and* interesting (avoids trivially similar variants), demonstrated on Crafter tasks.
- Concept modeled: curiosity that avoids boring near-duplicates.
- Computational cost: LLM queries per task-set update (cheap relative to RL).
- What we can learn: our agents' curiosity must weight **novelty of effect** not novelty of item ID; otherwise they
  will "discover" 500 variants of "stick+stone".
- What we could integrate: idea.
- What we should NOT integrate: n/a.
- Scalability: idea scales.

### OMNI-EPIC (Faldor, Zhang, Cully, Clune; ICLR 2025)
- URL: https://github.com/maxencefaldor/omni-epic
- License: Apache-2.0
- Language: Python (+ JS frontend)
- Activity/maintenance: ~80 stars; research code.
- Architecture: LLM writes new environments+reward code; archive of tasks; interestingness model compares candidate to
  archive; success detector; DreamerV3 trains agents; iterate.
- Concept modeled: open-ended generation of tasks as code.
- Computational cost: high (LLM + RL).
- What we can learn: archive + novelty filter + success check loop — mirrors how our tech registry should accept a new
  technique only if its effect differs meaningfully from known ones.
- What we could integrate: idea.
- What we should NOT integrate: code-generating LLM inside the sim.
- Scalability: low.

### POET (Wang, Lehman, Clune, Stanley 2019)
- URL: https://github.com/uber-research/poet
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~270 stars; archived-style research code.
- Architecture: population of (environment, agent) pairs; environments mutate; a **minimal criterion** keeps only
  environments neither too easy nor too hard; periodic transfer of agents between environments.
- Concept modeled: co-evolution of challenges and solutions.
- Computational cost: large compute.
- What we can learn: **minimal criterion** = "zone of proximal development" for both skill learning and invention
  (agents attempt experiments slightly beyond current capability); **transfer** = diffusion of techniques between groups
  facing different environments (a technique invented in a dry region later solves a problem elsewhere).
- What we could integrate: ideas.
- What we should NOT integrate: code.
- Scalability: concept only.

### OpenAI hide-and-seek (multi-agent emergence environments)
- URL: https://github.com/openai/multi-agent-emergence-environments
- License: MIT
- Language: Python (MuJoCo worldgen)
- Activity/maintenance: ~1.8k stars; archived (read-only), no updates.
- Architecture: team-competitive RL with movable boxes/ramps; agents discovered 6 strategy phases including ramp use,
  box surfing (physics exploit) — emergent tool use from autocurricula.
- Concept modeled: tool use emerging from pressure + physics affordances.
- Computational cost: billions of env steps.
- What we can learn: emergent tool use needs (a) physics with exploitable affordances, (b) pressure, (c) enormous
  sampling — we must **shortcut sampling** with structured curiosity and property-level generalization; and expect
  exploits (agents will find physics bugs: design invariants carefully).
- What we could integrate: nothing directly.
- What we should NOT integrate: deep RL for tool discovery.
- Scalability: not feasible at our scale.

### pyribs (quality-diversity)
- URL: https://github.com/icaros-usc/pyribs
- License: MIT
- Language: Python
- Activity/maintenance: ~260 stars; active.
- Architecture: Archive (grid/CVT over "measure" descriptors, best solution per cell), Emitters (propose solutions:
  Gaussian, CMA-ME/MAE, etc.), Scheduler (ask/tell loop).
- Concept modeled: find diverse high-quality solutions.
- Computational cost: depends on evaluation; library overhead tiny.
- What we can learn: model a culture's **technique repertoire as a QD archive** over descriptors (function, material
  class, effort) — keeps diversity (several axe variants per niche) instead of collapsing to one.
- What we could integrate: MIT; but the archive is ~100 lines to write ourselves.
- What we should NOT integrate: Python dependency.
- Scalability: fine.

### QDax
- URL: https://github.com/adaptive-intelligent-robotics/QDax
- License: MIT
- Language: Python/JAX
- Activity/maintenance: ~360 stars; active.
- Architecture: JAX-accelerated MAP-Elites and variants (CMA-ME, PGA-ME, AURORA with learned descriptors).
- Concept modeled: massively parallel QD.
- Computational cost: GPU-parallel.
- What we can learn: AURORA's learned descriptors could auto-discover "kinds" of artifacts; offline tooling for balancing.
- What we could integrate: offline design tool only.
- What we should NOT integrate: runtime dependency.
- Scalability: offline.

### Word2World (Nasir, James, Togelius 2024)
- URL: https://github.com/umair-nasir14/Word2World (paper arXiv:2405.06686)
- License: Apache-2.0
- Language: Python
- Activity/maintenance: ~70 stars; research code.
- Architecture: LLM writes a story, extracts characters/tiles/goals, places tiles on a 2D character grid in two passes,
  iterates with feedback for coherence and playability.
- Concept modeled: LLM-based PCG of worlds.
- Computational cost: several LLM calls per world.
- What we can learn: LLMs are good at *naming and narrating* content but need hard validators.
- What we could integrate: idea — use LLMs to *name/describe* discovered techniques, never to decide physics outcomes.
- What we should NOT integrate: LLM-determined world rules.
- Scalability: offline/occasional.

### Infinite Craft (Neal Agarwal, 2024) — LLM-generated recipes
- URL: https://neal.fun/infinite-craft/ (write-up: https://debarghyadas.com/writes/infinite-craft/ ; agent: https://github.com/shitwolfymakes/NoFunNeal)
- License: proprietary web game
- Language: n/a
- Activity/maintenance: live.
- Architecture (secondary): pair of element names -> lookup in a global DB; if unseen, prompt an LLM (Llama 2) for the
  result, then cache forever so results are deterministic and shared; "First Discovery" flag.
- Concept modeled: unbounded combinatorial crafting via semantic association.
- Computational cost: one LLM call per novel pair, then cached.
- What we can learn: **memoize discoveries globally and deterministically** (we will do the same with our physics
  oracle). Also shows the danger: semantic/pun logic ("Fire + Book = Knowledge") is not physical; it breaks a
  simulation's causal consistency.
- What we could integrate: caching pattern.
- What we should NOT integrate: LLM-as-physics.
- Scalability: caching makes it scale.

### Little Alchemy 2 as an exploration testbed (Brändle, Stocks, Tenenbaum, Gershman, Schulz 2023; LLM follow-ups)
- URL: "Empowerment contributes to exploration behaviour in a creative video game" (Nature Human Behaviour 2023) — secondary; LLM follow-up "Large Language Models Think Too Fast To Explore Effectively" https://arxiv.org/html/2501.18009v1
- License: papers
- Language: n/a
- Activity/maintenance: 2023–2025.
- Architecture: ~720 elements, fixed combination table; human exploration is better explained by **empowerment**
  (choosing elements that lead to many further combinations) plus uncertainty, than by uncertainty alone; most LLMs
  explore with uncertainty-driven strategies and underperform humans.
- Concept modeled: how intelligent explorers choose which combinations to try.
- Computational cost: n/a.
- What we can learn: agent curiosity should include an **empowerment heuristic**: prefer experimenting with items that
  have many untested affordances / were involved in recent discoveries (Kauffman-style adjacent possible as a felt
  attraction).
- What we could integrate: the decision heuristic.
- What we should NOT integrate: fixed combination tables.
- Scalability: cheap heuristic.

---

# Part E — Skills (track 23)

### Power law vs exponential law of practice (Newell & Rosenbloom 1981; Heathcote, Brown & Mewhort 2000)
- URL: https://en.wikipedia.org/wiki/Power_law_of_practice ; Heathcote et al., "The power law repealed", Psychonomic Bulletin & Review 7(2):185–207
- License: papers
- Language: n/a
- Activity/maintenance: classic + ongoing (piecewise power laws, S-shaped curves).
- Architecture: performance time T(N) = A + B·N^(−β) (power) vs T(N) = A + B·e^(−αN) (exponential). Heathcote et al.:
  across 7,910 individual learning series, exponentials fit individuals better; averaging many exponentials with
  different rates produces an apparent power law.
- Concept modeled: speed/accuracy improvement with practice.
- Computational cost: trivial.
- What we can learn: **simulate individuals with exponential approach to an asymptote; aggregates will look power-law
  automatically** — this makes LOD aggregation self-consistent.
- What we could integrate: formulas.
- What we should NOT integrate: n/a.
- Scalability: trivial.

### ACT-R declarative memory decay (pyactr)
- URL: https://github.com/jakdot/pyactr
- License: GPL-3.0 (ALGORITHM ONLY; the equations are published science)
- Language: Python
- Activity/maintenance: ~185 stars; ~112 commits.
- Architecture: ACT-R base-level activation B = ln(Σ_j t_j^(−d)) with t_j time since each past use and d ≈ 0.5; retrieval
  probability is logistic in activation; spacing effects emerge naturally.
- Concept modeled: forgetting with spacing and frequency effects.
- Computational cost: O(#uses) per skill unless approximated (Petrov's hybrid approximation keeps only last k uses + a
  closed form for the rest).
- What we can learn: **rust as decaying activation of procedural knowledge**, with spaced practice more durable than
  massed — a nice emergent apprenticeship dynamic.
- What we could integrate: equations (public).
- What we should NOT integrate: pyactr code.
- Scalability: with the approximation, O(1) per skill per agent.

### Cataclysm-DDA skills, focus & rust
- See the Cataclysm-DDA entry in Part C (verified source: `src/skill.cpp` `SkillLevel::train`, `SkillLevel::rust`,
  `SkillLevel::knowledge_train`; `src/character.cpp` `practice` with catch-up and focus drain;
  `src/character_proficiency.cpp` `practice_proficiency`).
- License: CC-BY-SA 3.0 — ALGORITHM ONLY.
- Key lessons summarized: quadratic XP per level; dual track knowledge/practice; rust grace period; rust rate decays
  with accumulated rust; catch-up multiplier when practice < knowledge; teacher effectiveness 2/(gap+1).

---

# Part F — Technology emergence & cumulative cultural evolution (track 24)

### Henrich 2004 — Demography and cultural evolution (Tasmanian case)
- URL: https://henrich.fas.harvard.edu/publications/demography-and-cultural-evolution-how-adaptive-cultural-processes-can-produce ; tutorial: Mesoudi "Model 9" (bookdown, secondary)
- License: paper
- Language: n/a
- Activity/maintenance: classic (American Antiquity 69(2), 2004), debated (e.g., Read 2006 reply).
- Architecture: N learners each copy the most skilled individual; each learner's resulting skill = model skill −
  α + Gumbel noise with dispersion β (most copies are worse; a few are better by luck). Expected change in mean skill
  per generation ≈ −α + β(γ + ln N) with γ ≈ 0.577; cumulative improvement needs N > N* = exp(α/β − γ). Harder skills
  (larger α/β) need exponentially larger populations; shrinking N below N* causes loss.
- Concept modeled: population size–dependent maintenance vs loss of complex skills.
- Computational cost: O(N) per generation; analytic shortcut O(1).
- What we can learn: our aggregate tech model can use the analytic recurrence per technique per community.
- What we could integrate: the equation.
- What we should NOT integrate: the assumption that everyone copies the single best model (use prestige/success-biased
  sampling among *reachable* practitioners).
- Scalability: O(1) per technique per community per generation.

### Mesoudi — Simulation models of cultural evolution in R
- URL: https://github.com/amesoudi/cultural_evolution_ABM_tutorial
- License: GPL-3.0 (ALGORITHM ONLY; models are textbook)
- Language: R (RMarkdown)
- Activity/maintenance: ~110 stars; v1.3.0 May 2025.
- Architecture: 19 small ABMs: unbiased/biased transmission, conformity, vertical/horizontal transmission, migration,
  blending, **demography and gain/loss (Henrich)**, polarization, group selection, social networks, Bayesian iterated
  learning, reinforcement learning, evolution of social learning strategies.
- Concept modeled: canonical cultural evolution dynamics.
- Computational cost: tiny.
- What we can learn: ready-made test cases to validate our transmission system (we should reproduce Henrich's N*
  threshold and conformity results in unit simulations).
- What we could integrate: models as test specifications.
- What we should NOT integrate: R code.
- Scalability: n/a.

### Kolodny, Creanza & Feldman 2015 — Evolution in leaps
- URL: https://cehg.stanford.edu/publications/evolution-leaps-punctuated-accumulation-and-loss-cultural-innovations (PNAS 112(49), 2015)
- License: paper (open access)
- Language: n/a
- Activity/maintenance: 2015.
- Architecture (secondary): cultural repertoire grows via several innovation types — independent invention, combination
  of existing traits, and modification/refinement of existing traits; traits are organized so some are
  **foundational** and facilitate whole clusters of related traits; innovation probabilities rise with repertoire
  size and population size; traits can be lost. Result: long stasis punctuated by bursts (when a foundational trait
  appears, its cluster fills quickly) and correlated losses (losing a foundational trait cascades).
- Concept modeled: punctuated tech accumulation and cascading loss.
- Computational cost: small stochastic simulation.
- What we can learn: **facilitation graph** — techniques make related techniques more discoverable; losing "fire
  control" should cascade (pottery, cooking, resin glue). Matches what our property-affordance system produces
  naturally (fire unlocks a whole region of property space).
- What we could integrate: model structure for aggregate LOD.
- What we should NOT integrate: n/a.
- Scalability: excellent.

### Lewis & Laland 2012 — Transmission fidelity is the key to cumulative culture
- URL: Phil. Trans. R. Soc. B 367:2171–2180 (2012) — https://www.researchgate.net/publication/228067282_Transmission_fidelity_is_the_key_to_the_build-up_of_cumulative_culture
- License: paper
- Language: n/a
- Activity/maintenance: 2012; widely cited.
- Architecture: population with traits produced by novel invention, modification and combination; each trait copied
  with fidelity f; small increases in f produce large, threshold-like increases in the number and longevity of
  traits and in cumulative (combination) depth.
- Concept modeled: fidelity threshold for cumulative culture.
- Computational cost: small.
- What we can learn: **teaching and language should be "fidelity technologies"** — discovering teaching or language
  increases f and should visibly change a civilization's trajectory. A god action ("bless with speech") has a big effect.
- What we could integrate: model.
- Scalability: excellent.

### Derex et al. 2013 & Muthukrishna et al. 2014 — lab evidence for group size / sociality
- URL: Derex, Beugin, Godelle, Raymond, Nature 503:389 (2013) (with critique/reply https://www.nature.com/articles/nature13411 , https://www.nature.com/articles/nature13412); Muthukrishna et al., Proc R Soc B 281:20132511 (2014) https://pmc.ncbi.nlm.nih.gov/articles/PMC3843838/ ; Derex & Boyd PNAS 2016 "Partial connectivity increases cultural accumulation within groups"
- License: papers
- Language: n/a
- Activity/maintenance: 2013–2016, debated.
- Architecture: lab transmission chains/groups: more models (and more sociable access) → complex skills maintained and
  improved; small/isolated chains lose them. Derex & Boyd 2016: **partially connected** groups out-innovate fully
  connected ones (diversity preserved, then recombined).
- Concept modeled: network structure effects on innovation.
- Computational cost: n/a.
- What we can learn: N_eff should be **reachable skilled models**, and moderate fragmentation (several bands with
  occasional contact) can beat one big band — emergent from our settlement/migration topology.
- Scalability: n/a.

### Weitzman 1998 — Recombinant growth
- URL: https://scholar.harvard.edu/weitzman/publications/recombinant-growth (QJE 113(2):331–360); PDF https://mattsclancy.com/wp-content/uploads/2023/01/Recombinant-Growth.pdf
- License: paper
- Language: n/a
- Activity/maintenance: classic.
- Architecture: new ideas are produced by pairing existing ideas; number of possible pairs grows ~ C(n,2), but each
  hybrid requires R&D effort and succeeds with a probability depending on resources; long run growth is limited by
  the capacity to process combinations, not by the supply of combinations.
- Concept modeled: combinatorial explosion of possibilities vs limited exploration effort.
- Computational cost: n/a.
- What we can learn: **exploration effort (time, curiosity, surplus) is the bottleneck**, not the number of possible
  combos — tie invention rate to leisure/surplus time and population.
- Scalability: n/a.

### TAP equation — Theory of the Adjacent Possible (Kauffman; Cortês, Kauffman, Liddle, Smolin)
- URL: "The TAP equation: evaluating combinatorial innovation in biocosmology" arXiv:2204.14115 (secondary); Phys Rev Research 7, 023127 (2025)
- License: paper (no reference implementation found — search came up empty)
- Language: n/a
- Activity/maintenance: active 2022–2025.
- Architecture: M_{t+1} = M_t(1 − μ) + Σ_{i=1..M_t} α_i·C(M_t, i), with μ an extinction rate and α_i decreasing (often
  α^i) for combinations of i items. Generic solution: long plateau then sudden super-exponential blow-up ("hockey
  stick"); variants with finite resources avoid blow-up.
- Concept modeled: combinatorial technology growth with loss.
- Computational cost: trivial.
- What we can learn: unchecked combinatorics blows up — our system needs **costs and limits** (effort, material
  availability, fidelity loss) so tech growth is a plateau-then-takeoff curve, not infinite.
- What we could integrate: as a sanity-check macro curve for balancing.
- Scalability: trivial.

### Tria, Loreto, Servedio & Strogatz 2014 — Urn model with triggering
- URL: https://www.nature.com/articles/srep05890 (Sci Rep 4:5890)
- License: paper (open access)
- Language: n/a
- Activity/maintenance: follow-ups through 2023 (ABM version: PLOS ONE 2023 "Simulating emergence of novelties using agent-based models").
- Architecture: Pólya urn; drawing a ball reinforces it (ρ copies added); drawing a never-seen ball also adds ν+1 brand
  new balls (expanding the adjacent possible); semantic variant groups related novelties. Yields Heaps' law for
  rate of novelty (D(t) ~ t^(ν/ρ) when ν<ρ) and Zipf's law for usage.
- Concept modeled: "one novelty opens the door to others".
- Computational cost: trivial.
- What we can learn: a cheap stochastic model of discovery for **far-LOD regions**: each discovered technique "adds
  balls" (opens candidate affordance experiments) to the regional urn; usage reinforcement = practice frequency.
- Scalability: excellent.

### Morgan & Feldman 2024 — Human culture is uniquely open-ended
- URL: https://www.researchgate.net/publication/385644332_Human_culture_is_uniquely_open-ended_rather_than_uniquely_cumulative (Nature Human Behaviour, Nov 2024)
- License: paper
- Language: n/a
- Activity/maintenance: 2024.
- Architecture (secondary): argues many animals have cumulative culture, but human culture is distinctively
  **open-ended** — able to keep generating new trait types without a ceiling, enabled by generative,
  compositional representations.
- Concept modeled: the difference between refinement within a fixed space and open-ended expansion.
- What we can learn: our system needs *compositionality* (objects composed of parts; techniques composed of steps;
  products used as inputs/tools) so the space keeps growing; property-based interactions + composition give this.
- Scalability: n/a.

---

# Recommended design for our game

Guiding principles (distilled from the above):

1. **Fields, not labels.** World state is continuous fields (elevation, rock, soil, temperature, moisture, vegetation);
   biomes/Köppen are derived views (DF, Holdridge, World Orogen).
2. **Climate is simulated slowly; weather is sampled fast** (EBM + moisture sweep; WGEN-style stochastic weather).
3. **Materials have physics properties; objects have shape + state; interactions are formulas over properties**
   (DF materials, CDDA, Powder Toy phase rules) — no authored recipes.
4. **Agents learn affordances** as (behavior, tool-predicate, target-predicate) → effect (Şahin; Gibson) and store
   techniques as composable programs (Voyager library), transmitted with fidelity (Lewis & Laland; Henrich).
5. **Everything has an aggregate form** with matching statistics for fast-forward (exponential individuals → power-law
   aggregates; Henrich recurrence; urn-with-triggering for regional discovery; memoized physics oracle).

## 1. Worldgen pipeline (ordered)

All steps are deterministic from (seed, parameters); god edits re-run only downstream steps locally.

| # | Step | Algorithm | Output fields | Cost (100k–250k cells) |
|---|------|-----------|---------------|------------------------|
| 0 | Planet params | radius, rotation period, axial tilt, year length, solar constant, ocean fraction | globals | – |
| 1 | Mesh | Fibonacci sphere + spherical Delaunay/Voronoi dual mesh (mapgen4 half-edge layout; World Orogen) ; for a flat-map variant, Poisson-disc + Lloyd | cells, neighbors | <1 s |
| 2 | Plates | farthest-point seeds, weighted round-robin flood fill (Orogen); each plate: Euler pole + angular speed, crust type, density | plate id per cell | <0.2 s |
| 3 | Boundary stress | per boundary edge: relative velocity split into normal (convergent/divergent) and tangential (transform) (Gainey) | boundary type, stress | <0.1 s |
| 4 | Crust & base elevation | continental vs oceanic crust thickness; isostatic elevation (Tectonics.js idea); distance-field blending from boundaries with asymmetric profiles: trench+arc for ocean–continent, orogen plateau for continent–continent, ridge for divergent, island arcs ocean–ocean (Gainey/Orogen/Cortial) | elevation, crust thickness, orogeny type, crust age | <0.5 s |
| 5 | Lithology | per cell a 3–5 layer strata stack chosen from tectonic context: basalt (oceanic/rift/hotspot), granite (cratons), metamorphic (orogens), andesite/obsidian (arcs/volcanic), limestone/chalk (shallow seas, older), sandstone/shale (basins) | strata stack | <0.1 s |
| 6 | Detail noise | ridged/fBm noise amplitude scaled by stress & orogeny type | elevation | <0.1 s |
| 7 | Drainage pre-pass | Priority-Flood (Barnes) → fill or breach depressions; record lakes (spill elevation, volume) | flow receivers, lakes | <0.2 s |
| 8 | Large-scale erosion | Braun–Willett implicit stream-power + hillslope diffusion (Fastscape), 50–200 steps with uplift from step 4; thermal (talus) relaxation; record eroded volume and deposit sediment in basins/floodplains/deltas | elevation, sediment thickness | 1–5 s |
| 9 | Climate spin-up | run seasonal climate (section 2) for ~10 model years to equilibrium | T, P per season | <1 s |
| 10 | Rivers & lakes | discharge = accumulated (P − ET) along receivers (mapgen4/O'Leary); river where discharge > threshold; Strahler order; floodplain width ∝ discharge^0.5 | discharge, river graph | <0.2 s |
| 11 | Soils | CLORPT (Jenny): parent rock + climate + relief + time + organisms → depth, texture (sand/silt/clay %), fertility, drainage class, organic content; **clay beds** in floodplains, lake beds, weathered shale; peat in cold wet flats | soil fields | <0.1 s |
| 12 | Deposits / resources | rule table by context: **flint/chert** nodules in chalk/limestone; **obsidian** at young volcanic arcs; **quartzite** in metamorphic belts; **copper** porphyry at subduction arcs; **tin** in granites; **bog iron** in wetlands; **placer gold/tin** downstream of orogens in river sediments; **salt** in evaporite basins/arid lakes; fibers & resin come from vegetation | deposit lists per cell (type, grade, depth, quantity) | <0.1 s |
| 13 | Vegetation & biome | continuous fields (tree cover, grass, shrub, biomass) from Whittaker/Holdridge-like response curves of T, P, soil; Köppen computed for display | veg fields, biome label | <0.1 s |
| 14 | Local detail (on demand) | when the camera/agents are in a region: upsample to a fine raster consistent with coarse rivers (Undiscovered Worlds idea), particle erosion with persistent discharge (SimpleHydrology) for 50k–200k droplets, place individual stones/trees by Poisson-disc weighted by fields | chunk heightmaps & objects | 50–300 ms per chunk |

God edits: raising a mountain → re-run 7–13 in a region bounding box plus the downwind climate band; rifting a continent
→ re-run 3–13 globally (seconds; show it as an animated geological epoch).

## 2. Climate and weather per tile per season

State per cell: T_s[season] (surface temp), P_s[season], soil moisture W, snow S, sea-ice/ice fraction, vegetation.
Seasons: use **12 months** for classification (or 4 seasons with interpolation if budget is tight).

Per cell per month update:

1. **Insolation**: declination δ = tilt·sin(2π·(month_phase)); daily-mean insolation from the standard formula using
   latitude φ and hour angle h0 where cos h0 = −tan φ·tan δ (clamped for polar day/night).
2. **Energy balance (2D diffusive EBM)**: C·ΔT = Q·(1 − albedo) − (A + B·T) + D·Laplacian(T), with
   C large over ocean (thermal inertia → maritime vs continental climates), albedo from snow/ice/vegetation/desert,
   A,B ≈ Earth OLR fit (~203 W/m² and ~2.1 W/m²/K), D tuned for meridional transport. Apply lapse rate
   (−6.5 K per km) for surface temperature at elevation. Solve with 2–4 Jacobi iterations per month (cheap; climate is
   quasi-equilibrium).
3. **Pressure & wind**: zonal belts (ITCZ low, subtropical high ~±30°, subpolar low ~±60°, polar high) whose latitude
   shifts toward the summer hemisphere following the thermal equator; add monsoon perturbation ∝ −(T_land −
   T_ocean_nearby); wind = pressure gradient rotated by Coriolis (geostrophic-ish), damped near the equator.
   Result: a wind vector per cell per month.
4. **Moisture**: process cells in upwind-to-downwind order (mapgen4 sweep; since winds vary, use a small number of
   ordered sweeps per wind band, or 3–5 iterations of a semi-Lagrangian upwind advection). Humidity gains evaporation
   over water ∝ q_sat(T)·(1 − RH) where q_sat grows ~7%/K (Clausius–Clapeyron); land adds evapotranspiration from soil
   moisture and vegetation. Precipitation = convective term (strong where ITCZ / warm humid) + orographic term ∝
   max(0, uplift = wind · ∇elevation)·humidity + frontal term (where temperature gradient is strong, mid-latitudes) +
   rain-out when humidity exceeds a cap that falls with elevation and temperature. Leeward cells inherit depleted
   humidity → rain shadows.
5. **Surface water**: bucket model for soil moisture W (capacity from soil texture), runoff to rivers, snow
   accumulation/melt by degree-days; drought index = rolling W anomaly.
6. **Classification**: after a model year, compute Köppen (needs 12 monthly T and P) and Whittaker/Holdridge inputs.

Cost: ~100–300 flops per cell per month → 250k cells × 12 months ≈ 0.1–0.5 GFLOP per simulated year ≈ 10–50 ms on one
core; parallelizes trivially. Recompute climatology only when the god changes something or every N years for slow
drift (vegetation/ice feedback).

**Weather layer** (sampled, not simulated globally):
- Daily state per *weather region* (coarse grid, ~1–4k regions): wet/dry Markov chain whose transition probabilities
  are fitted to the cell's monthly P (number of wet days and mean intensity); amounts from gamma distribution; daily
  temperature = monthly climatology + AR(1) anomaly; spatial coherence by drawing anomalies on a coarse noise field
  advected along the prevailing wind each day.
- **Storm agents**: mid-latitude cyclones spawn where meridional temperature gradient is strong (more in winter), travel
  with the westerlies, bring a frontal rain band and wind; tropical cyclones spawn over water > 26–27 °C in late
  summer, move west then poleward, deliver extreme rain/wind and storm surge to coasts. Typically 0–50 active storm
  objects globally.
- **Low-frequency variability**: a regional "ENSO-like" oscillator (damped noisy oscillator, period 3–7 years) and red-
  noise (AR(1) with yearly ρ≈0.5–0.8) in precipitation produce multi-year droughts/pluvials — important drivers of
  migrations and technology loss.
- **Fire weather**: fire danger index from temperature, humidity, wind, and days since rain.
- Locally (camera region), daily weather drives cloud/rain visuals and agents' perceptions; far regions use monthly
  totals.

God tools: change tilt/solar constant (re-run EBM), seed rain (inject humidity), summon storm (spawn storm agent),
drought curse (bias the anomaly), volcanic winter (albedo/aerosol term for 1–3 years).

## 3. Object / material property schema

### 3.1 Material (shared definition; ~50–300 materials, generated from geology & biology, not hand-authored per recipe)
| Property | Unit/range | Used for |
|---|---|---|
| density ρ | kg/m³ | mass, floating, throwing |
| hardness H | 1–10 (Mohs-like) | scratching, cutting, wear |
| toughness K | J/m² (normalized 0–1) | resists fracture under impact |
| fracture_mode | conchoidal / granular / fibrous / ductile / plastic | what impact produces (sharp flakes vs crumbs vs bending) |
| max_edge E_max | 0–1 | sharpest edge the material can hold (obsidian ≈ 1, flint 0.9, granite 0.3, copper 0.6, bronze 0.75) |
| tensile_strength σ_t | MPa | ropes, lashings, hafts |
| shear_strength τ | MPa | resistance to being cut |
| flexibility | 0–1 | bows, baskets, levers |
| plasticity(wetness) | function 0–1 | shaping by hand (clay high when wet, ~0 when dry) |
| water_content w | 0–1 (state, see 3.2) | drying, firing risk |
| porosity | 0–1 | holds water?, absorbs |
| solubility/slaking | 0–1 | dissolves/softens when wet (unfired clay, salt) |
| ignition_T, heat_of_combustion, burn_rate | °C, MJ/kg, kg/s per m² | fuel and fire |
| melting_T, boiling_T | °C | smelting, glass |
| specific_heat c, conductivity k | J/kg/K, W/m/K | heating/cooling, CA |
| transforms | list of (condition → product material, rate, reversible?) | phase/chemical changes: dry, fire, char, melt, cook, rot, sinter |
| nutrition / toxicity | kcal/kg, 0–1 | food |
| decay_rate | 1/day by T & moisture | spoilage |
| adhesive(T) | 0–1 | resins/pitch become glue when heated then cooled |
| fiber | bool + strand strength | cordage |
| friction | 0–1 | grip, fire-by-friction |
| luster/color | visual + value | aesthetics, prestige |

Material **generation**: rocks from lithology (e.g., flint = microcrystalline silica: H 7, conchoidal, E_max 0.9),
woods from tree species traits (density, flexibility), animal materials from creatures (bone, sinew, hide), plants
(fiber, resin). Values are physically motivated so new materials (god-created "starmetal") just plug in.

### 3.2 Object instance (per item)
- materials: list of (material, mass fraction, part-id)
- parts & joints (composition graph): parts with roles inferred from geometry (head, handle, edge, container wall);
  joints with type (lashed, glued, socketed, fired-bond) and joint strength J
- shape descriptors: mass m, length L, edge_sharpness e (current, ≤ E_max), point p, flatness, cavity volume V,
  graspability g (size vs hand), symmetry/quality q (0–1)
- state: temperature T, water_content w, integrity/durability D (0–1), firing state, wear
- provenance: maker, technique id, date (for archaeology/history views)

Affordances are **derived** on the fly (and cached): e.g. *can_cut* = e > 0.3 ∧ g > 0.5; *can_contain_liquid* =
V > 0 ∧ porosity < 0.3 ∧ solubility < 0.1; *can_lever* = L > 0.5 m ∧ σ_t, flexibility in range; *fuel* = ignition_T
reachable ∧ heat_of_combustion > 5 MJ/kg.

### 3.3 Interaction formulas (physics-lite, deterministic + seeded noise)
- **Strike** (A struck onto B, or B hit by A swung): impact energy E_imp = ½·m_eff·v². Hand-held v ≈ 3–6 m/s;
  swung on a haft v = ω·L_haft (lever amplifies). If E_imp / contact_area > K_B·f(H_A/H_B): B fractures.
  Fragments n ∝ E_imp/K_B. For conchoidal B, each fragment gets edge e ~ E_max_B·Beta(a,b) where the Beta mean
  increases with the striker's *knapping skill* and with H_A ≥ H_B−1 (a hard hammerstone), and decreases with flaws.
  Granular materials produce crumbs (e ≈ 0); ductile materials deform (shape change).
- **Cut** (tool edge drawn/chopped into target): cut_depth per stroke =
  k · e_tool · max(0, H_tool − H_target + 1)/(H_target+1) · F_or_E / (τ_target · contact_length).
  Edge wear per stroke: Δe = −w0 · (H_target / H_tool)² · F; integrity loss if E_imp > K_tool (chipping).
  Example: flint flake (H7, e 0.8) on hide (H≈1.5, low τ) → deep cuts; on oak (H≈3, high τ) → shallow, edge dulls;
  hafted flint axe raises F_or_E ×5–10.
- **Join**: joint strength J = min(binder strength, friction·grip contact, socket fit). Lashing with fiber of strand
  strength s and n wraps: J ≈ μ·n·s; resin glue: J = adhesive(T_cooled)·area. On use, if impulse > J → joint fails
  (head flies off), which is itself an observable effect that teaches.
- **Heat/fire (thermal CA)**: per local cell or per object: dT/dt = Σ conduction k·ΔT/dist + radiation/convection
  term from nearby fires − losses. Combustion when T ≥ ignition_T and oxygen available and fuel mass > 0: heat release =
  burn_rate · heat_of_combustion; spreads to neighbors via the same equation (+ wind bias). Sleeping/dirty-rect
  regions (Noita) so only fires are simulated. Open campfire peaks ≈ 600–800 °C; pit/kiln with insulation and fuel
  load can exceed 900 °C — this emerges from the heat balance with an "enclosure" factor reducing losses.
- **Transforms** (Powder Toy style but with time integration): each transform has a condition (T range, w range,
  oxygen, duration) and accumulates progress while the condition holds: progress += dt/τ_transform. Examples:
  wet clay → dry clay (w decreases by evaporation ∝ (1 − RH)·temperature); clay at T ≥ ~600 °C for ≥ ~30 min →
  **earthenware** (irreversible: H 3→5, solubility 1→0, porosity → 0.15–0.25); wood in low oxygen at 300–500 °C →
  charcoal; meat at 60–100 °C → cooked (digestibility up, pathogens down); resin + heat → liquid adhesive → cools to
  glue; copper ore + charcoal at ≥ 1085 °C in reducing atmosphere → copper metal.
- **Thermal shock**: when a porous object with water content w is heated with rate dT/dt, crack probability
  p = σ(a·w·dT/dt − b·temper_fraction − c·skill). Temper (sand/grit/shell) is a *material mixture* that lowers p —
  discoverable.
- **Containment**: liquid volume held V_hold = V_cavity, leak rate ∝ porosity·permeability; fired pot leaks slowly
  (porous earthenware) until sealed (resin, burnished, glazed).

Granularity: objects exist as individuals only near the focus (L0/L1); elsewhere stocks are counts per (material, form,
quality bucket) per household.

## 4. Invention without authored recipes

### 4.1 Representation
- **Effect vocabulary** (perceptible deltas, ~30 tokens, like AI2-THOR states): fractured, sharp_fragment, cut,
  pierced, shaped, dried, hardened, softened, burned, charred, cooked, melted, joined, joint_failed, holds_water,
  leaks, floats, hot, light_emitted, food_obtained_faster, felled, etc. Each effect also carries magnitudes
  (e.g., cut depth, time saved).
- **Affordance belief** per agent (Şahin formalism): key = (behavior, tool_predicate, target_predicate,
  context_predicate) where predicates are *bins over properties* (e.g. "edge ≥ 0.6", "H ≥ 6 & conchoidal", "wet
  plastic", "near heat ≥ 500 °C"); value = (expected effect distribution, count, confidence, utility estimate).
  Predicates are discovered by **generalization**: after k ≥ 2 successes with different objects, the agent intersects
  their property bins (version-space style) to form a broader predicate; failures specialize it.
- **Technique** = a parameterized program (Voyager-style) of steps; each step = (behavior, role predicates,
  parameters such as angle/force/duration), plus preconditions and expected outcome; techniques can call techniques
  (composition: "make_flake" is a sub-step of "make_axe").
- **Global discovery registry** (UI/history only, not a recipe list): first time any agent anywhere produces a
  new (effect-class, product-signature) we create a registry entry with name (optionally LLM-generated description),
  inventor, place, date. Names are cosmetic.
- **Physics oracle cache** (Infinite Craft caching, but physical): (behavior, binned tool props, binned target props,
  binned context) → effect distribution, filled by actually running the interaction formulas; used by far-LOD
  regions to evaluate experiments without object instances. Deterministic per world seed.

### 4.2 Experiment selection (curiosity + need)
Each idle/low-pressure tick an agent may experiment with probability ∝ surplus_time × curiosity trait × safety.
Candidate experiments = (behavior × held/nearby objects) pairs. Score:
score = w_need · predicted_goal_relevance + w_nov · novelty(effect-uncertainty of this predicate-bin) +
w_emp · empowerment(objects involved in many untried or recently fruitful affordances) + w_soc · observed_others_doing −
risk. Novelty is computed over **property bins and effects**, not item IDs (OMNI lesson), so trivial variants are
uninteresting. Difficulty is kept in the zone of proximal development (POET minimal criterion): prefer experiments
whose predicted success is 0.2–0.8.

### 4.3 Worked example A — sharp rock → hafted axe (exact state transitions)

Initial: band at a river with flint nodules eroded from chalk (deposit from worldgen step 12). Agent A: knapping 0,
hunger high; a carcass nearby; no stone tools known.

| t | Action / event | Objects (before → after) | Physics | Perceived effect | Belief / technique update |
|---|---|---|---|---|---|
| 1 | A throws/drops a flint cobble onto a riverbed boulder while playing or cracking nuts | flint_cobble{m 0.8 kg, e 0, D 1} → 3 fragments {m 0.4, 0.25, 0.15; e = 0.72, 0.1, 0.35} | E_imp = ½·0.8·5² = 10 J > threshold for flint (conchoidal) ; e sampled from Beta scaled by E_max 0.9 | fractured, sharp_fragment | belief(strike, H≥6&conchoidal, hard_surface) → fractured (n=1) |
| 2 | A picks up fragment #1; grips it badly | A.hand integrity −0.05 | contact with e 0.72 | cut (self!) | salient: "sharp fragment cuts soft things" belief(press, e≥0.6, soft) → cut |
| 3 | A presses fragment on carcass hide | hide integrity 1 → 0.7 ; meat accessible | cut_depth high: H 7 vs 1.5, τ low | cut, food_obtained_faster (×3 vs hands/teeth) | technique T1 "cut with sharp stone" (utility ↑ from food rate) |
| 4 | Next day A wants another sharp stone; repeats strike deliberately with different stones (granite, flint, sandstone) | granite → crumbs (e≈0); sandstone → crumbs; flint → flakes | fracture_mode differs | only flint gives sharp_fragment | generalization: predicate narrows to "conchoidal & glassy" → technique T2 "make_flake" = strike(conchoidal_core, hard_hammer) |
| 5 | Repeated practice | knapping skill 0 → 0.15 | flake edge Beta mean rises with skill; waste ratio falls | — | T2 parameters tuned (angle, force) |
| 6 | Observers B, C watch A (see §6) | — | — | — | B,C acquire T1/T2 with fidelity f_obs |
| 7 | A tries chopping a sapling with hand-held flake | sapling integrity −0.02/stroke; flake e 0.75 → 0.6 | low energy: hand v ≈ 4 m/s, m 0.15 kg → 1.2 J | slow cut, dulled | utility of "chop" low; need persists (firewood/spear shafts) |
| 8 | A already has belief "swing stick → hits harder/further" from knocking fruit (lever) | — | — | — | empowerment: stick & flake both "fruitful" → experiment "join(stick, flake)" scores high |
| 9 | A presses flake into a split stick end (no binder) | composite{head flake, handle stick, J = friction ≈ 15 N} | swing v = ω·L ≈ 12 m/s at head → E ≈ 11 J; impulse > J | joint_failed (head flies off) | belief(join, no binder) → joint_failed; need for binder |
| 10 | A (or someone with cordage technique — fiber from bark, invented separately) lashes it: n=6 wraps of bark fiber s 40 N | composite J ≈ μ·n·s ≈ 0.5·6·40 = 120 N | impact impulse under J for light blows | felled sapling in 12 strokes vs 80 | technique T3 "hafted_chopper" = T2 + split_stick + lash(fiber) + seat; product signature {edge head + lever handle + lashed joint} → **registry: "hafted axe" (first discovery)** |
| 11 | Later refinement: heated pine resin as glue + lashing | J ↑ to ~300 N | adhesive(T) transform (resin heated then cooled) | fewer joint failures | T3' variant (QD archive keeps both) |

Note what was never authored: "axe", "requires: stick + flint". Only properties (fracture_mode, E_max, H, τ, σ_t of
fiber, lever mechanics) and the joint formula existed.

### 4.4 Worked example B — clay → pottery

Initial: floodplain soil with clay fraction 0.45 (worldgen step 11), band has controlled fire (hearth technique F).

| t | Action / event | Object state | Physics | Perceived effect | Learned |
|---|---|---|---|---|---|
| 1 | Child/agent plays with wet riverbank mud | lump{clay, w 0.30, plasticity 0.85} → shape: ball/figure | plasticity(w) high → shape persists | shaped | belief(knead, wet plastic) → shaped |
| 2 | Lump left in sun 2 days | w 0.30 → 0.08 ("leather-hard" at ~0.15, dry < 0.1) | evaporation ∝ (1−RH)·T | hardened (dry) | belief(leave_in_sun, shaped wet clay) → hardened |
| 3 | Rain falls on dried figure | w ↑, solubility 1 → slumps | slaking | softened / ruined | belief: dry clay + water → softened (negative) |
| 4 | Clay used to line the hearth pit or a figurine sits next to the fire for an evening | object T 20 → 650 °C for 3 h (pit with embers; enclosure factor) | transform progress = Σ dt/τ(T); passes 1.0 → **earthenware** (H 3→5, solubility 0, porosity 0.2). If w was 0.15 and heating fast: crack prob p = σ(a·0.15·rate − …) ≈ 0.4 → some crack | hardened_irreversibly, (sometimes) fractured | belief(heat ≥ 600 °C, dry shaped clay) → permanent hardness & water-proof. **Key observation: after rain, the fired lining does not soften** (contrast with step 3 → high surprise → high learning rate) |
| 5 | Agent with containment need (carry water/ cook) and empowerment heuristic combines beliefs: "shaped" + "fire-hardened" + existing belief "hollow things hold water" (gourds, skulls) | pinch-pot shape (cavity V 1.5 L), dried 3 days, fired | same transform | holds_water (leaks slowly) | technique P1 "make_pot" = dig_clay → knead → shape(cavity) → dry(≥2 days) → fire(≥600 °C, ≥30 min) |
| 6 | Pot placed on fire with water + meat | water 100 °C; meat cooked; pot T rises | heat conduction through wall | cooked (boiling), food yield ↑ (fat/marrow extraction, softer plants) | new affordance "boil" opens cooking techniques (Kolodny facilitation cluster) |
| 7 | Many cracks in firing → variance in outcomes; an agent mixes sand/crushed shell into clay (experiment "mix A into B" scored by novelty) | temper_fraction 0.2 | crack p falls (b·temper term) | fewer fractured | P1' with tempering (registry: "tempered pottery") |
| 8 | Later: enclosed pit / updraft kiln (enclosure factor ↑ → 900 °C) | stoneware-like hardness, lower porosity | transform branch at higher T | better containment | ... and kilns later enable smelting (copper needs ≥1085 °C + reducing atmosphere + charcoal) — adjacent possible expands |

Pottery requires fire control (F) + clay deposit + drying climate (or shelter) + containment need: populations in
arid, clay-poor or highly mobile contexts may never invent it — and that is a feature.

## 5. Skill learning and decay formulas

Per agent per *technique family* (skills are attached to technique families — knapping, cordage, pottery, hunting —
not to an authored skill list; a family is created when a technique is learned and similar techniques share it via
predicate similarity).

State: practical p ∈ [0,1), knowledge k ∈ [0,1) (k ≥ p is "knowing how it should go"), activation A for decay, last
practice time t_last, total practice n_total.

- **Practice gain** (individual exponential approach; aggregates look power-law per Heathcote):
  Δp = η · aptitude · focus · zpd(d − p) · (1 − p) · (1 + c·max(0, k − p)/max(k, ε))
  where zpd(x) = exp(−(x − δ)²/(2σ²)) peaks when task difficulty d is slightly above p (δ ≈ 0.1); the last factor is the
  CDDA-style **catch-up** (relearning a rusty skill is faster). Knowledge rises too: Δk = 0.9·Δp_raw (bounded by p + 0.1
  from practice alone; more from teaching).
- **Performance**: success probability P = σ((p_eff − d)/τ); output quality q ~ Normal(μ = p_eff·material_quality, s =
  s0·(1 − p_eff)); time per action T = T_min + (T_0 − T_min)·exp(−r·n_effective).
- **Decay/rust** (ACT-R-inspired, CDDA grace): after a grace period g (e.g., 3 days game time),
  p_eff = p · [f_floor + (1 − f_floor) · (1 + (t − t_last)/τ_r)^(−ψ)], with f_floor = 1 − exp(−n_total/N0)·0.5
  (overlearned skills keep at least ~50–90%), τ_r grows with spacing of past practice (spaced practice is more durable),
  ψ ≈ 0.5. Knowledge k decays far slower (ψ_k ≈ 0.1) — this reproduces CDDA's "rust only lowers practice, knowledge
  stays; catch-up restores it".
- **Focus/fatigue**: focus pool drains with intensive practice (like CDDA, drain ∝ amount² below a cap) and recovers with
  rest; sleep consolidates (small bonus to p after sleep if practiced that day).
- **Observation learning**: watching a practitioner: Δk = η_obs · visibility · attention · (k_model − k_learner)·g(gap)
  where g(gap) = 2/(1 + gap·10) style damping (CDDA's 2/(gap+1): a teacher far above you is less useful).
- **Teaching** (a discoverable technique itself, with its own skill): multiplies η_obs (×2–4), raises fidelity (below),
  and lets k transfer without the model being mid-task.

## 6. Cultural transmission with fidelity and population-size-dependent loss

### 6.1 Individual level (L0/L1)
When learner L acquires technique T from model M (chosen by success/prestige bias among *reachable* practitioners,
plus conformity weight):
- For each step i of T: copy step correctly with prob f_i = f_mode · (1 − complexity_i·0.3) · (0.5 + 0.5·k_L), where
  f_mode = 0.6 (observation only), 0.8 (repeated observation/apprenticeship), 0.9 (teaching with gestures),
  0.97 (teaching with language). Failure → step dropped (prob 0.5), parameter noise (0.45), or random step mutation
  (0.05; occasionally an improvement — the Henrich "lucky copy").
- Learner's initial skill: p_L = max(0, p_M − α_T + β_T·G) with G ~ Gumbel(0,1), α_T increasing with technique
  complexity (#steps, tolerance tightness), β_T inference noise (smaller with teaching). This reproduces Henrich 2004.
- A technique copy that doesn't reproduce the effect when the learner tries it is pruned/repaired by the learner's own
  experimentation (individual learning fills gaps — Kolodny's "modification").

### 6.2 Community level (L2/L3, fast-forward)
For each community c and technique T: state = (n_practitioners, mean skill z̄, var), plus "knowledge holders" count.
Per generation (or per year with scaled rates):
- N_eff = number of practitioners **reachable** by learners (within band + contact rate × neighbors' practitioners;
  Muthukrishna sociality; partial connectivity bonus from Derex & Boyd via occasional cross-band pooling).
- Expected mean change: Δz̄ = −α_T + β_T·(γ + ln N_eff) (Henrich), plus practice drift from usage frequency (techniques
  not used because the environment no longer requires them rust: z̄ decays as in §5).
- **Loss**: T is lost in c when n_practitioners = 0 or z̄ < z_min(T) (cannot produce the effect). Probability of
  practitioners dying out is handled demographically (binomial deaths). Loss of a **facilitating** technique
  (fire, cordage, containers) lowers the effective utility and practice of dependents → cascading decline
  (Kolodny 2015).
- **Diffusion between communities**: per contact event, T crosses with probability ∝ contact_rate · visibility(T) ·
  utility_in_destination(T) · fidelity; arrival with skill = source z̄ − α_T·(1 + distance penalty).
- **Fidelity technologies**: teaching, language, demonstrations, later writing → raise f_mode and lower α/β, which by
  Lewis & Laland creates threshold-like jumps in repertoire size — a natural "age" transition without a tech tree.

Expected emergent phenomena to test: Tasmania-like loss on islanded small populations; punctuated bursts after a
foundational discovery; regional technological styles (QD archive keeps diverse variants); re-invention of lost
techniques when conditions recur (since physics, not recipes, determines possibility).

## 7. Fast-forward aggregation (LOD)

| Tier | Where | Physics | Agents | Invention | Skills/culture |
|---|---|---|---|---|---|
| L0 | camera/focus region (≤ few hundred agents) | full object instances, thermal CA, interaction formulas | full cognition | real experiments on instances; results written to oracle cache | individual, §5–6.1 |
| L1 | active regions (≤ ~10k agents) | objects as stacks; interactions resolved through the **oracle cache** (no geometry) | simplified decisions | experiments sampled as (behavior, binned props) and resolved by the oracle | individual, coarse time step (hours/day) |
| L2 | inactive settlements (≤ ~100k agents represented) | resource stocks per household | households/bands | hazard model below | community state per technique (§6.2) per season |
| L3 | far regions / deep fast-forward (up to 1M organisms represented) | regional stocks | populations | urn-with-triggering per region | per region per generation |

**Aggregate invention hazard** for each candidate technique j in a community's adjacent possible:
λ_j = N_explorers · surplus_time · curiosity · P_oracle(success_j | locally available materials & contexts) ·
(1 + Σ facilitation from known related techniques) · need_j; draw Poisson events per step. The adjacent possible
candidates are generated from (a) known techniques' one-step variations (substitute a material in a predicate bin,
add a step from another known technique, change a context like temperature), and (b) affordance pairs among
locally present material classes — enumerated lazily and cached; resource constraints (no flint nearby → candidates
using conchoidal stone have near-zero P) give geography-dependent paths. At L3 approximate with Tria's urn:
each discovery adds ν new candidate "balls" restricted to materials present in the region.

**Consistency on zoom-in** (L2 → L0): instantiate individuals by sampling skills from (z̄, var), techniques by
sampling per-individual knowledge consistent with counts, objects from stocks with quality distributions;
deterministic seeds per (community, technique, year) so revisiting is stable. On zoom-out, fold individual stats back
(means/variances, counts).

**Climate at fast-forward**: use the monthly climatology plus sampled annual anomalies (AR(1) + oscillator), not daily
weather; storms reduced to annual damage/rain statistics; droughts/pluvials still sampled because they drive history.

**Geology at fast-forward**: frozen except god interventions; river avulsion/meanders can be sampled rarely (century
scale) for long runs.

Budget sketch at 1M represented organisms: climate 10–50 ms per simulated year; L2/L3 culture updates
O(communities × known techniques) ≈ 10k × 200 = 2M simple updates per season (~10–20 ms); oracle lookups cached;
L0/L1 dominates real-time cost and is bounded by focus size.

## Open questions / risks

1. **Combinatorial blow-up vs stagnation** (TAP/Weitzman): too generous generalization → explosive tech;
   too strict → nothing happens for centuries. Needs automated balancing runs (QD/archive metrics, target curves like
   "fire in first 1–5 sim years at L0 scale; pottery 100–1000 years in favorable regions").
2. **Predicate binning design** determines what is discoverable; badly chosen bins make some inventions impossible or
   trivial. Bins must be data-driven per property and reviewed with designers.
3. **Physics exploits** (hide-and-seek lesson): agents may find degenerate strategies in our simplified formulas
   (e.g., infinite sharpness by repeated knapping). Need invariants: conservation of mass, E_max ceilings, wear.
4. **Legibility**: players must understand *why* a band invented pottery. Need an "invention story" log (event chain from
   §4 tables) and a knowledge-graph viewer generated from the registry, not a tech tree.
5. **Time-scale mismatch**: real prehistory took ~10^5 years for some steps; our pacing needs accelerators (curiosity
   multipliers, god "inspiration" nudges) without breaking causality.
6. **Climate toy-model errors**: an EBM + sweep can misplace deserts/monsoons on unusual continents. Mitigate by
   blending with zonal heuristics (World Orogen approach) and validating offline against ExoPlaSim runs (GPL tool used
   only internally, never shipped).
7. **Determinism under LOD switching**: oracle cache and seeded sampling must make the same world history regardless of
   where the camera was; otherwise players see "observer effects". Possibly accept bounded divergence.
8. **Cultural model validity**: Henrich's population-size effect is debated (Read 2006; Derex reanalysis). Keep the
   parameters tunable and expose them as world settings.
9. **LLM use**: tempting for naming/narration and for "creative leaps", but must never decide physical outcomes
   (Infinite Craft semantic logic breaks causality); cost at 1M agents is prohibitive except for highlighted individuals.
10. **Licensing**: several best references are GPL/LGPL/CC-BY-SA (World Orogen, Undiscovered Worlds, RichDEM, Powder Toy,
    platec, CDDA, pyactr, Mesoudi tutorial, ExoPlaSim). Re-implement from papers; keep a clean-room note per algorithm.
    MIT/Apache/BSD/CC0 sources (Azgaar, mapgen4, Fastscape, SimpleHydrology, Lague, Gainey, crafter, Craftax, pyribs,
    QDax) can be consulted freely with attribution if code is reused.
11. **Unverified items**: OMNI repo license; small Köppen repos; DF material token details and Kolodny model specifics
    (domains blocked; based on search snippets/memory) — re-check before relying on exact numbers.
12. **Scale of object instances**: 1M organisms × inventories is too many instances; stacks with quality buckets
    beyond L0 are mandatory, which means some per-object provenance stories are lost at distance.
