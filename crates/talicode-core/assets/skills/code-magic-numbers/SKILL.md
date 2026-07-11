---
name: code-magic-numbers
description: no unexplained numeric literals
---
No unexplained numeric literals. A raw number that carries meaning (86400, 0.05, a threshold, a limit) should be a named constant. Do NOT flag identity/degenerate values (0, 1, -1), math that defines its own context (n % 2, division by 2), or standard loop scaffolding (i = 0; i < len).
