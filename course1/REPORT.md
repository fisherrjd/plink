# course1 — Claude's rounds (2026-07-12)

Played autonomously via the Caddy bridge (`PLINK_CADDY=1` + `tools/caddy.py`),
one genuinely aimed stroke per command. Screenshots in this directory are
auto-captured at the start of each hole (current set is from round 2, showing
the speed pads); `scorecard.png` is the in-game final scorecard.

## Scores

| Round | Rules | Total | vs par 62 |
|------:|-------|------:|----------:|
| 1 | v1 physics (no chips/pads, power 5) | 45 | **-17** |
| 2 | power 7 + chip shots + speed pads | 37 | **-25** * |
| 3 | same, hole-17 pad fixed (headless validation) | 38 | **-24** |

\* round 2's hole 17 benefited from a placement bug: the speed pad overlapped
the ball spawn and fired it into the sand for free. Fixed before round 3.

## Round 3 (current rules, honest) hole-by-hole

| Hole | Concept | Par | Strokes | Note |
|-----:|---------|----:|--------:|------|
|  1 | straight            | 2 | 1 | ace — new power reaches in one |
|  2 | L-dogleg            | 3 | 2 | |
|  3 | center block        | 3 | 2 | |
|  4 | funnel + post       | 4 | 3 | |
|  5 | bank gap            | 2 | 1 | **chip-in ace over the wall** |
|  6 | narrow gate         | 3 | 1 | **chip-in ace over the gate** |
|  7 | S-chicane           | 4 | 3 | |
|  8 | pinball posts       | 3 | 2 | |
|  9 | long corridor       | 3 | 1 | **speed-pad ace** |
| 10 | U-turn              | 4 | 3 | pad ride on the return arm |
| 11 | diamond bounce      | 3 | 2 | |
| 12 | Z double-dogleg     | 4 | 3 | one diagonal threads both gaps |
| 13 | sand trap           | 3 | 2 | full-power second stroke banks in |
| 14 | funnel gauntlet     | 4 | 2 | **chip-in eagle over the post** |
| 15 | ring                | 5 | 2 | albatross via the bottom gap |
| 16 | shielded pocket     | 3 | 2 | |
| 17 | sand + posts        | 4 | 3 | pad ride into sand, blast out, tap |
| 18 | finale              | 5 | 3 | pad boost carries through the sand |

Rounds 2 and 3 replayed identically shot-for-shot (physics is deterministic)
except the fixed hole 17.

## Mechanics evaluation

**Swing velocity (power 5 → 7).** Max roll went from ~727px to ~1018px.
Hole 1 became aceable, long lanes stopped being automatic two-putts, and
full-power shots now carry real overshoot risk (bank-backs off the far wall).
Feels much livelier; recommended keeping.

**Chip shot (right-click drag, orange aim line).** The ball goes airborne for
0.22–0.6s scaled by power (swells visually, casts a shadow), ignoring walls,
sand, cup, and pads until it lands, then rolls. Great risk/reward: flight
length is fixed by power, so the rollout is hard to judge — but flying a wall
turns par holes into ace chances (holes 5/6). Balance note: cups placed near
the back wall are generous to chips because the bank-back funnels returns;
future holes could place cups mid-field to punish overshoot.

**Speed pads (teal chevrons).** Fire the ball +700 px/s along the pad's
facing on entry. Best used as a commitment device: you putt gently onto the
pad and surrender control. The hole-9 pad ace and the hole-18
pad-through-sand carry both feel great. Lesson learned: keep pads well clear
of the ball spawn (hole 17 originally auto-launched the ball at hole load).

**Sand (Rough).** With power 7, a 250px sand field still eats ~2 strokes
unless you chip over or pad through — good hazard economy.

## Known rough edges

- Chips can land inside a wall if flight time expires over one; physics
  shoves the ball out, which looks glitchy but plays on. Could clamp flight
  to end only over open ground later.
- Par values are calibrated for machine-precision aim; humans should treat
  par as a good score, not a baseline.
