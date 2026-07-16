# desert — Claude's round (2026-07-16)

Played autonomously via the Caddy bridge (`PLINK_CADDY=1 PLINK_COURSE=2` +
`tools/caddy.py`), one genuinely aimed stroke per command. Screenshots in this
directory are auto-captured at the start of each hole; `scorecard.png` is the
in-game final scorecard.

## Score

| Round | Rules | Total | vs par 63 |
|------:|-------|------:|----------:|
| 1 | current physics (chips, pads, power 7) | 47 | **-16** |

## Hole-by-hole

| Hole | Concept | Par | Strokes | Note |
|-----:|---------|----:|--------:|------|
|  1 | quicksand intro          | 2 | 2 | straight lane between hazards |
|  2 | central quicksand        | 3 | 2 | banked over the top |
|  3 | pad into quicksand       | 3 | 2 | declined the pad — the math says swallow |
|  4 | pools + windmill + dune  | 4 | 7 | **swallowed once, spinner-batted into the dune** |
|  5 | center windmill          | 3 | 1 | ace — banked over the swept disc |
|  6 | four-slab sand slalom    | 4 | 3 | bank into the one clean lane |
|  7 | oasis + cacti            | 3 | 2 | banked under the water |
|  8 | wall + guarded low route | 4 | 1 | **chip-in ace over the wall** |
|  9 | twin spinners            | 3 | 1 | ace — both sweeps clear y=360 by ~7px |
| 10 | twin pools channel       | 4 | 1 | **ace down the 48px channel** |
| 11 | sand horseshoe           | 2 | 1 | ace into the mouth |
| 12 | pad relay through dune   | 4 | 2 | eagle — double boost carries the sand |
| 13 | big pool, top lane       | 3 | 3 | 58px lane forces three positional putts |
| 14 | S-chicane + cacti        | 4 | 4 | every lane cactus-pinched |
| 15 | walled courtyard         | 5 | 5 | swallowed once probing a 2px corridor |
| 16 | chevron wall             | 3 | 1 | ace via top bank |
| 17 | four pools + pin spinner | 4 | 4 | forced three-leg detour below the windmill |
| 18 | finale gauntlet          | 5 | 5 | decoy pad; threaded the 23px pool-spinner gap |

## Mechanics evaluation

**Quicksand (drag 2.2 + suction 650, sink at 150 px/s).** The star hazard and
much scarier than plain sand: a rolling entry under ~700 px/s for a 200px pool
dies inside and eats a penalty + replay. Both of my swallows came from
misjudging the *contact zone* — the rect inflated by ball radius (with rounded
corners) — not the pool proper. The penalty-and-replay loop is harsh but
readable; hole 4 stacks it with a windmill and turned into the hardest hole
across all three courses (nearly every line is pixel-tight).

**Windmills (Spinner).** Swept disc = bar/2 + ball radius; at speed 2.0-2.3
rad/s the bar covers ~70° while a mid-speed ball crosses, so "time it" is
really "gamble" — I routed around every disc after hole 4's bat sent the ball
into the dune. As pin guards (17, 18) they force genuinely creative routes.
Consider exposing the phase in some visual rhythm cue if timing is meant to
be part of the skill.

**Dunes (Rough damp 4.5, vs 3.0 snow).** Effectively walls for ground balls
(a 200px dune eats ~1100 px/s) — hole 18's pad-into-dune is a designed decoy.
Only hole 12's double-pad relay legitimately powers through, which makes that
eagle feel earned.

**Chips in the desert.** This is the chip course: hole 8's chip-in ace over
the divider and the hole-15 recovery over quicksand were round-savers. Flight
= v·(0.22+0.38·power) with no in-flight decay, so landings are hot — planning
the rollout (including wall banks) matters more than the carry.

**Cacti (26px walls).** Lovely micro-hazards: they never block a route
outright but pinch every comfortable line by a few pixels (14, 17, 18).

## Known rough edges

- Hole 4's inter-pool gate is sub-pixel for ground routes given the two pool
  contact zones; every legitimate line either grazes quicksand or enters the
  windmill disc. A ~15px widening of the gate (or slower windmill) would take
  it from luck to skill.
- Spinner bats send the ball anywhere, including into hazards the victim had
  no agency over (my hole-4 dune burial). A capped deflection speed would
  keep the chaos without the full-field punts.
- Quicksand contact edges are invisible: the visual pool reads ~11px smaller
  than the effective hazard. A faint darkened ring at the true contact
  boundary would prevent "I wasn't even touching it" moments.
