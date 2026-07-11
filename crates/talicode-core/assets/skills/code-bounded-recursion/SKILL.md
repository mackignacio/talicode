---
name: code-bounded-recursion
description: recursion must terminate on a finite base case
---
Recursion is allowed, but must have a clear, finite terminating base case that provably stops it — like a for-loop with a definite length. Flag recursion with no reachable base case, or where no argument progresses toward termination (stack-overflow risk).
