# ice — Claude's round (2026-07-16)

Played autonomously via the Caddy bridge (`PLINK_CADDY=1 PLINK_COURSE=1` +
`tools/caddy.py`), one genuinely aimed stroke per command. Screenshots in this
directory are auto-captured at the start of each hole; `scorecard.png` is the
in-game final scorecard.

## Score

| Round | Rules | Total | vs par 62 |
|------:|-------|------:|----------:|
| 1 | current physics (chips, pads, power 7) | 32 | **-30** |

## Hole-by-hole

| Hole | Concept | Par | Strokes | Note |
|-----:|---------|----:|--------:|------|
|  1 | straight, ice patch      | 2 | 2 | calibration hole |
|  2 | long ice sheet           | 3 | 1 | ace — ice damp measured at ~0.3/px |
|  3 | diagonal + snow strip    | 3 | 1 | ace through the snow brake |
|  4 | walled ice corridor      | 2 | 1 | ace |
|  5 | iceberg rink             | 3 | 1 | ace — straight line threads all four bergs |
|  6 | over-the-top ice band    | 4 | 3 | snow patch guards the cup |
|  7 | water pool center        | 3 | 2 | banked over the top |
|  8 | S-chicane                | 4 | 2 | **eagle — top-gap bank thread dropped in** |
|  9 | speed pad + snow guard   | 3 | 1 | **pad ace: 600 boost dies to 245 px/s at the cup** |
| 10 | wall bar, ice top route  | 4 | 2 | eagle |
| 11 | ice bridge over water    | 3 | 1 | ace |
| 12 | angled banks rink        | 4 | 3 | tip-clearance ricochet cost a stroke |
| 13 | cup on open ice          | 2 | 1 | ace — dying arrival on ice |
| 14 | ice staircase            | 4 | 3 | direct diagonal threads all hazards |
| 15 | double-gap + water pool  | 5 | 3 | **eagle — three-lane lay-up route** |
| 16 | funnel + snow plug       | 3 | 1 | ace straight through the plug |
| 17 | glacier block            | 4 | 2 | eagle via top bank |
| 18 | launcher pad finale      | 5 | 2 | **eagle — tuned pad entry vector** |

## Mechanics evaluation

**Ice (damp ~0.05 vs felt 1.1).** Measured effective loss ≈ 0.3 px/s per px
travelled (vs 1.1 on felt), so shots glide ~3.5× further. Distance control is
*easier* than on felt for machine aim — the loss model is linear and
predictable — which is why this round went so low. For humans the opposite
will hold (small power errors translate to huge distance errors). The
sheen-streak visual reads clearly.

**Snow (Rough damp 3.0, white).** Excellent as a brake: strips and plugs
(holes 3, 6, 9, 16) let a hot ice shot die near the cup, which actively
*helps* aggressive play. As a wall-off hazard (hole 10) it forces real
routing. Good dual use of one mechanic.

**Icebergs / glaciers (walls).** Skattered small walls on ice (hole 5)
read well. The hole-12 angled banks are brutal on ice because incoming speed
is preserved — my tip-clearance shot ricocheted the full field back. Fun, but
tip clearances behave like knife edges at glide speeds.

**Pads on ice.** Hole 9's pad ace (600 boost + snow brake + 260 capture
speed) is beautifully tuned — the ball arrives at the cup just under capture.
Hole 18's angled launcher rewards entry-vector planning since entry velocity
adds to the boost vector.

**Cup capture at 260 px/s** is generous and the right call on ice — without
it nothing would ever drop.

## Known rough edges

- Par feels loose: 10 of 18 holes fell to a single computed stroke. If the
  course should resist machine-precision play, the big open sheets (2, 4, 5,
  11, 13) could use offset cups or berg guards near the hole.
- Wall banks on ice lose only ~5% — intended? It makes double-bank plans
  viable, which is fun but further lowers effective difficulty.
