# Mako

Before building or changing UI, read DESIGN.md.

To drive and verify the app, use the `verify-mako` skill in `.claude/skills/verify-mako/`.

## CI

- **CI runs every check on every push. Never skip a job because of which paths changed.**
  Why: a path filter risks missing something that changed, and CI doesn't need optimizing yet.
  <!-- 2026-11-15: "afraid it misses something changed. dont need to optimize this yet." -->
