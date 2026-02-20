# Automove Pro Strategy Interview Notes

Captured: 2026-02-20
Purpose: Preserve pro-player heuristics as a reference input for future automove improvements.

## Strategy Q&A

### 1. What are your top 3 win conditions, in priority order, when a turn starts?
always attack opponent drainer, get supermana if there's a safe way, get opponent's mana if there's a safe way

### 2. What are the first 5 board cues you use to judge winning vs losing quickly?
if one player scored or secured and going to score supermana, the other should have a chance to score one of opponents mana to have a chance to win. holding potion creates strong threats and gives lots of opportunities. your mana pieces total distance to be moved to reach 5 points score target.

### 3. What board cues make you switch from "score race" to "tactical kill" plan?
always attack drainer, always attack spirit if there's a quick way and it creates risks, attacking other mons depends on context, might be needed to protect your mons

### 4. Which white openings are strongest in practice, and against which opponent setups?
get potion soon, steal supermana soon or get ready for stealing it soon

### 5. What exact signals tell you to abandon an opening line early?
my drainer will be attacked if i leave it there before i'm able to score any mana with it

### 6. Is same-turn opponent drainer attack always correct when available? What are the exceptions?
the only exception is scoring supermana or opponents mana on that turn or winning the game on that turn

### 7. How much tempo/material are you willing to spend to kill opponent drainer?
max

### 8. Which drainer-attack routes (mystic, demon, bomb) are most reliable by position type?
whoever can get it, doesn't matter

### 9. What are common fake-good drainer attacks that actually lose on reply?
not sure

### 10. When your own drainer is exposed, what is your defense priority order?
walk away or protect with angel depending on a context, or attack the potential attacker before it creates threat

### 11. In which situations is it correct to leave your drainer vulnerable anyway?
winning same turn or scoring supermana or opponent's mana same turn

### 12. When should spirit be moved off base immediately, even if no immediate gain?
always

### 13. When is it actually correct to keep spirit on base?
never

### 14. What are the top 5 highest-value spirit actions and setups?
steadily moving your own mana closer to your pools, helping drainer or your other mons move, stealing supermane closer to your drainer, pulling opponent's mana closer to your side, scoring opponents mana

### 15. How do you choose between spirit deployment and direct carrier progress on the same turn?
go spirit, you'll be able to move further the next turn with it

### 16. Which mana moves are "opponent help" and should almost never be played?
towards their side of the board

### 17. When is mana handoff acceptable because compensation is strong enough?
scoring supermana or opponents mana or winning same turn

### 18. Supermana vs regular mana: what concrete thresholds decide priority?
both needed, securing supermana significantly increases winning chances

### 19. What must be checked every turn before allowing any quiet move?
attack opponent drainer

### 20. What bomb patterns win most often, and what bomb mistakes lose games?
use bomb to attack opponents drainer

### 21. What potion timing patterns are strongest (immediate use vs hold)?
hold and create threat for scoring opponents mana with 2 spirit actions in a row

### 22. What are your anti-roundtrip rules to avoid wasting tempo?
going anywhere always choose the shortest path

### 23. In near-target-score endgames, what is your exact move-selection checklist?
same as always, there's no much difference between start and end, gotta do the right thing at any given position

### 24. What bot mistakes do strong players punish most consistently?
leaving their drainer vurnarable and not going for supermana or caring about it when i go for it

### 25. Can you give labeled positions where best move is non-obvious, plus why alternatives fail?
preparing a decisive supermana or opponent mana capture for the next turn

## Intended Usage

- Treat these as strong priors for candidate design and tactical guardrails.
- Validate all rule-like assumptions with the existing promotion experiments before runtime promotion.
