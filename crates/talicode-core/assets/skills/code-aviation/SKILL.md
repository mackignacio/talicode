---
name: code-aviation
description: DO-178C safety-critical profile (opt-in, strict)
---
The DO-178C safety-critical standard for airborne software (opt-in, strict): absolute predictability over convenience. Enforce no dynamic allocation after init, no recursion at all, compile-time-constant loop bounds, a single clear exit, strong typing with no implicit conversions, no dead/unreachable code, and locked priority-based scheduling. Enable deliberately for safety-critical code; it is stricter than the default lenses.
