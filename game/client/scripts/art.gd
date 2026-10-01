extends RefCounted
## Shared procedural art helpers: colours derived from genetics and ids, so the
## world sprites and the inspector portrait agree. Use via preload, no globals.

const CLOTH := [
	Color("#b5733f"),  # ochre hide
	Color("#8c5a3c"),  # brown leather
	Color("#7d8a4a"),  # moss
	Color("#a8503f"),  # rust
	Color("#c9a66b"),  # straw
	Color("#5f6f7d"),  # slate wool
	Color("#9b7a55"),  # tan
	Color("#6e4f6b"),  # berry dye
]


static func hash01(id: int, k: int) -> float:
	var h := (id * 2654435761 + k * 40503 + 12345) & 0xffffffff
	h = ((h ^ (h >> 15)) * 739982445) & 0xffffffff
	h = h ^ (h >> 13)
	return float(h & 0xffff) / 65535.0


static func skin_color(t: float) -> Color:
	return Color("#f3cfac").lerp(Color("#8d5a3b"), clampf(t * 1.4, 0.0, 1.0)).lerp(Color("#4a2e1f"), clampf(t * 1.4 - 0.4, 0.0, 1.0) * 0.8)


static func hair_color(t: float, age: float) -> Color:
	var c := Color("#e2c27a").lerp(Color("#7a4a2a"), clampf(t * 1.6, 0.0, 1.0)).lerp(Color("#1c1410"), clampf(t * 1.6 - 0.6, 0.0, 1.0))
	if age > 45.0:
		c = c.lerp(Color("#d6d6d2"), clampf((age - 45.0) / 20.0, 0.0, 0.92))
	return c


static func cloth_color(id: int) -> Color:
	var c: Color = CLOTH[int(hash01(id, 1) * CLOTH.size()) % CLOTH.size()]
	return c.lightened((hash01(id, 2) - 0.5) * 0.2)


## 0 short, 1 long, 2 bun / topknot.
static func hair_style(id: int, female: bool, age: float) -> int:
	var r := hash01(id, 3)
	if age < 6.0:
		return 0
	if female:
		return 1 if r < 0.6 else (2 if r < 0.85 else 0)
	return 0 if r < 0.7 else (1 if r < 0.9 else 2)


static func has_beard(id: int, female: bool, age: float) -> bool:
	return not female and age >= 18.0 and hash01(id, 4) < 0.45
