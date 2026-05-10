-- The previous reveal counter only incremented when a hit's guess phase
-- finished (state -> Confirming) and never when a hit was skipped, which
-- produced impossible rows like reveals = 0 with skips > 0. Going forward
-- reveals are recorded the moment a hit becomes the front of the stack
-- (start, confirm, skip), so each old skip event corresponds to exactly
-- one missing reveal. Backfill by adding skips into reveals.
UPDATE hit_stats SET reveals = reveals + skips;
