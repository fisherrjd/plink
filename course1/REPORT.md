# course1 — Claude's round (2026-07-12)

Played autonomously via the Caddy bridge (`PLINK_CADDY=1` + `tools/caddy.py`),
one genuine aimed stroke per command. Screenshots in this directory were
auto-captured at the start of each hole; `scorecard.png` is the in-game final
scorecard.

## Final score: 45 strokes — par 62 — **17 under par**

| Hole | Concept          | Par | Strokes | +/- |
|-----:|------------------|----:|--------:|----:|
|  1 | straight            | 2 | 2 | E  |
|  2 | L-dogleg            | 3 | 2 | -1 |
|  3 | center block        | 3 | 2 | -1 |
|  4 | funnel + post       | 4 | 3 | -1 |
|  5 | bank gap            | 2 | 2 | E  |
|  6 | narrow gate         | 3 | 2 | -1 |
|  7 | S-chicane           | 4 | 3 | -1 |
|  8 | pinball posts       | 3 | 2 | -1 |
|  9 | long corridor       | 3 | 2 | -1 |
| 10 | U-turn              | 4 | 3 | -1 |
| 11 | diamond bounce      | 3 | 2 | -1 |
| 12 | Z double-dogleg     | 4 | 3 | -1 |
| 13 | sand trap           | 3 | 3 | E  |
| 14 | funnel gauntlet     | 4 | 3 | -1 |
| 15 | ring                | 5 | 2 | **-3 (albatross)** |
| 16 | shielded pocket     | 3 | 2 | -1 |
| 17 | sand + posts        | 4 | 3 | -1 |
| 18 | finale              | 5 | 4 | -1 |

Front nine: 20 (par 27, -7) · Back nine: 25 (par 35, -10)

## Round notes

- Physics calibration: a full-power putt rolls ~727px; roll distance is
  ~linear in power. Sand (`Rough`, damp 4.5) costs ~4.1x distance.
- Highlight: hole 15 (ring, par 5) fell in 2 — approach to the mouth of the
  ring, then one thread through the bottom gap to the center cup.
- Hole 12's par-4 Z fell to a single diagonal threading both gaps.
- No hole played over par.

## Design observations (for the next iteration)

- Par 62 is soft: with file-precise aim every hole is birdieable; pars should
  assume 2 positioning strokes rarely miss. Human mouse play will be spraying
  more — pars are probably right for humans, easy for a machine.
- The 18-line scorecard overflows its panel and collides with the
  Back-to-Menu button — needs a layout fix.
- Max roll (~727px) can't cross the 990px playfield; every long hole becomes
  drive + tap. More swing velocity would add risk/reward.
