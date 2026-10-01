# Project Genesis

A god game in which every human is driven by **Artificial Mind V0** (see
`ARTIFICIAL_MIND_V0.md`). Nothing about civilization is scripted: there are no
houses, farms, religions, money or tech tree. What people do comes from what their
own minds have learned about the world they live in.

- **Simulation:** Rust, deterministic (`game/sim`, crate `genesis-sim`)
- **Bridge:** GDExtension (`game/gdext`, crate `genesis-gdext`, godot-rust 0.5, API 4.4)
- **Client:** Godot 4.4+ (`game/client`), GL Compatibility renderer

## Build and run

```sh
game/client/build.sh                 # cargo build --release -p genesis-gdext, copy the library into game/client/bin
godot --path game/client             # play (Godot 4.4 or newer)
godot --path game/client -- --seed=42   # another world
```

Headless simulation (no graphics), useful for balance checks:

```sh
cargo run --release -p genesis-sim --bin headless -- <hours> <speed> <seed>
GENESIS_DEBUG_DEATHS=1 ...           # print every death and its cause to stderr
```

Automated screenshots (needs `xvfb-run`): `game/client/shot.sh <name> <frames> [--select] [--tab=N] [--zoom=Z] [--speed=I] [--warp-days=D] [--god=power,power] [--click=x,y]`.

## Controls

| Input | Action |
|---|---|
| WASD / arrows / screen edge / right- or middle-drag | Pan |
| Mouse wheel / pinch | Zoom toward cursor |
| Left click | Inspect creature (Select tool) or use the chosen god power |
| Space | Pause / resume |
| 1–5 | 1×, 5×, 25×, 100×, 1000× |
| F | Follow the selected creature |
| H | Show or hide history |
| Esc | Back to Select, then close the inspector |

## Time

1 tick = 1 game second. A human life is compressed: **1 year = 2 game days**.
1× = 1 game minute per real second. Thinking costs about 90 µs per human per
second, so at high speeds the game degrades **cognition**, never time, and says so
in the time bar:

| Speed | Humans think every | Animals act every |
|---|---|---|
| 1×, 5× | 1 s | 1 s |
| 25× | 2 s | 2 s |
| 100× | 6 s | 4 s |
| 1000× | 20 s | 10 s |

The selected human always thinks every second. If the machine cannot keep up, the
bar shows the speed actually achieved ("running 85× (asked 1000×)").

## What is in the world

- **Terrain:** procedural (value-noise fbm): deep water, lakes, rivers, sand,
  grass, forest, hills, mountains, snow. Seasons, day/night, temperature by season,
  hour, altitude and weather (clear, cloudy, rain, storm with lightning).
- **Plants:** berry bushes (fruit seasonally, spread by seed, look green when
  stripped), trees, grass that grazers eat and that regrows.
- **Materials:** stone, flint (knapping it on stone makes a sharp flake), fire
  (lightning, spreads, burns, cooks meat, gives warmth), carcasses, remains.
- **Humans:** body (Artificial Mind V0's biology: energy, water, temperature,
  skin and nociceptors, wounds, sleep pressure), needs, life stages, genetics
  (diploid genome giving appearance, physiology and temperament), relationships
  (familiarity, affection, trust, attraction, fear, resentment), pair bonds,
  pregnancy, birth, death (starvation, thirst, cold, wounds, old age, wolves),
  grief, skills, and the full mind: perception, concepts, memories, learned
  associations, beliefs (own, observed, told), emotion, three decision systems.
- **Grazer** (herbivore) and **wolf** (predator), with simple innate behaviours.

## God powers

Create human, grazer, wolf or plants; add food (berry bushes) or water (pond);
heal, bless, curse or smite a creature; lightning, fire, rain; make the world
warmer or colder.

## Inspector

Click any creature. The tabs are Overview (what they are doing and why, in plain
words), Body, Brain (the active pathway graph sense → learned cue → expectation /
memory → feeling → goal → action, plus the full decision trace), Memories,
Knowledge (learned links with evidence counts), Beliefs (with source: own
experience, watched someone, told by someone), Relations, Family (tree), Skills
and Genetics.

The inspector is an **observer** view. The mind receives only salted, meaningless
tokens ("thing#51d1"). The inspector replaces those tokens with what is really there
("berry bush") so the player can read the trace. The mind never sees these names.

## Gameplay milestone (status)

| # | Step | Status |
|---|---|---|
| 1 | Generate a world | Done |
| 2 | Spawn humans and animals | Done (two bands of 5, six grazer herds, two wolf packs) |
| 3 | Press play | Done |
| 4 | Watch humans explore | Done |
| 5 | Watch hunger, thirst, tiredness | Done (inspector meters, sleep, drinking, eating) |
| 6 | Watch discoveries | Done (berries edible, meat, flint flake, fire; "firsts" in history) |
| 7 | Watch learning from experience | Done (Knowledge tab; burns, bites, food) |
| 8 | Click a human and see why | Done (Overview "why" and Brain tab) |
| 9 | Relationships and memories | Done |
| 10 | Time passing | Done (clock, seasons, ageing) |
| 11 | God powers | Done (all listed powers) |
| 12 | Fast-forward | Done (honest cognition LOD) |
| 13 | Births | Done (couples, pregnancy, birth, family tree) |
| 14 | Deaths | Done (causes recorded, remains, grief) |
| 15 | Knowledge passed on | Done ("X told Y: 'hue:red' means food") |
| 16 | History | Done (history panel, toasts, "look here") |

## Balance (world physics, not behaviour)

Measured with the headless runner, 30 game days (= 16 years) at 1000× on seeds
7, 11 and 23 after the changes below.

| Seed | Humans alive | Births | Deaths | Grazers | Wolves |
|---|---|---|---|---|---|
| 7 | 5 | 1 | 6 | 92 | 6 |
| 11 | 4 | 2 | 8 | 82 | 2 |
| 23 | 13 | 5 | 2 | 72 | 11 |

Before these changes every seed collapsed (all humans dead within 6–16 game days).
These changes were made, all in the world, none in the mind:

- **Berry yield:** 60 kcal per portion and 20 portions per bush. Bushes fruit
  every 25–40 minutes (every 2 h in winter) and spread by seed. A body burns
  about 1300 kcal a day, and the old bushes (30 kcal, one portion every 6 h)
  could not feed a band.
- **Stripped bushes look stripped:** a stripped bush now looks green. Before,
  it kept the red berry look after it was empty, so humans kept tasting empty
  bushes.
- **Felt temperature:** includes metabolic heat (+6 °C awake, +3 °C asleep),
  forest canopy (+3 °C) and huddling (+1.5 °C per adjacent human, up to 3).
  Staying close is the mind's choice; the warmth is physics.
- **Wolves:**
  - They gorge on a kill until full, so one kill feeds them for a day or two.
    Before, they ate one portion and hunted again, wiping out the grazers.
  - They drink before hunting.
  - They bite a human and then back off for 30 minutes.
  - They attack humans only when starving and only a human with fewer than
    two others nearby.
- **Grazers:** spawn in herds; a mate within 20 tiles; breed a little faster.
- **Animal navigation:** animals reach water using a breadth-first
  distance-to-water field. Walking straight toward water left them stuck
  behind mountains.
- **Innate attachment prior:** undirected wandering (the mind's `Wander`
  command) drifts back toward a partner more than 5 tiles away, or toward a
  young child's mother. This is a species-level prior, like the grazers' innate
  fear of wolves. Targeted actions stay entirely the mind's. Without it, couples
  drifted 100+ tiles apart and conception almost never happened.

## Fixed in the second pass

- **Social credit:** comfort from being touched is credited to the toucher, not to whatever you were doing.
- **People vs objects:** innate face and motion senses; people form their own category.
- **Readable labels:** "Touching C5 (person) → comfort", "Putting C1 (water) in the mouth → drink".
- **Navigation:** humans walk around lakes and ridges.
- **Learning by watching:** children learn to eat and drink by watching adults.
- **Nursing:** toddlers are carried until 3 and nursed until 5.
- **Population:** 30 game days at 1000×, 7 seeds starting from 10 people: 16, 15, 4, 11, 10, 17 and 19 alive. Seed 42 did not finish. Most deaths were from old age.
- **Visuals:** painted terrain at 16 px per tile, water shimmer, new sprites.
- **Save/load:** Save and Load buttons (F5 / F9). A save is the world's seed plus a journal of outside actions. Loading replays it and is verified exactly.

## Known issues (still open)

1. **P1 criterion C5** (experienced children faster than naive ones to first swallow) is still FAIL: 6 s vs 6 s. The P1 report predates the last round of mind changes.
2. **Overgeneralised beliefs:** people sometimes tell each other vague beliefs ("visible things mean drink").
3. **Tasting other people** happens about 1–3% of the time in adults (down from 1.5–2.9%).
4. **Tracing affects behaviour:** turning on decision traces (selecting a human) changes that human's think rate. A watched run therefore diverges from an unwatched one, although each is deterministic and save/load records the selection.
5. **Loading time:** loading replays the whole history, so it takes longer the older the world is.
6. Wolves and grazers use simple innate rules, not minds.
