# P0 determinism report

- Scenario: 12x12 room, 6 objects, 3 bodies, random policy, 20000 ticks (1 tick = 1 game second)
- Reference hash (seed 42): `8ce6fe43538a7317`
- Physical contacts in reference run: 977 (227 with FIRE_A)
- Wall time per run: 71.8 ms

| Check | Result |
|---|---|
| 100 runs, same seed: identical final + 20 checkpoint hashes | 100/100 |
| 10 other seeds all differ from reference | true |
| Snapshot at tick 7000, resume, equals uninterrupted run | true |
| 8 parallel threads equal sequential | true |

**P0: PASS**
