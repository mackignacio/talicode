---
name: code-magic-strings
description: >-
  Flag every bare string literal that carries MEANING the code branches or
  depends on — a status value, a key or field name, a path or URL fragment, a
  mode/type discriminator, a role, or a message dispatched on — and demand it
  become a named constant or an enum member with a single source of truth. A
  repeated or semantically-loaded literal is a typo bug and a scattered-update
  hazard waiting to happen; one authoritative name fixes both. Overlaps
  code-dry (duplicated knowledge) and is the sibling of code-magic-numbers for
  the numeric case. Stay quiet on ordinary human-facing text, log messages,
  and empty literals. Use when a diff hardcodes a "magic string", a "hardcoded
  string", a "string constant", a status/enum value, a dict/JSON key, or a
  path/route literal that logic keys off.
---

# Magic Strings

A string literal that the program *reasons about* — compares, switches on,
looks up by, or routes with — is not text, it is a protocol value wearing a
string's clothes. The moment two places must spell `"pending"`, `"user_id"`,
or `/v1/orders` the same way for the code to work, the literal has become
load-bearing, and a bare literal gives you no compiler, no autocomplete, and no
single place to change it. One misremembered character fails silently. This
lens hoists those meaningful literals into named constants or enums so the
name documents intent, the definition lives in one place, and a typo becomes a
symbol error instead of a runtime mystery. It judges *meaning*, not appearance:
the goal is a single source of truth for every value logic depends on.

## The check

1. For each string literal in the change, ask the deciding question: does any
   logic *depend on its exact value* — is it compared, switched on, used as a
   lookup key, matched, or dispatched? If yes, it is meaningful; if it is only
   shown to a human, it is display text.
2. If the literal is a status/state (`"active"`, `"failed"`), a mode or type
   discriminator (`"admin"`, `"dark"`, `"csv"`), or a role, and code branches
   on it — FLAG it: it belongs in an enum or a named constant.
3. If the literal is a dict/map/JSON key, a field or column name, a header
   name, or an env-var name that appears in more than one place — FLAG it: name
   it once and reference the name everywhere.
4. If the literal is a path, route, URL, or filename fragment that other code
   must match or join against — FLAG it: centralize it as a constant.
5. If the same meaningful literal appears at two or more sites, FLAG it
   regardless of category — that is the duplicated-knowledge case and the
   highest-value fix.
6. If the literal is shown, logged, or formatted for a human and nothing
   branches on it, leave it alone.

## Hard rules

- A meaningful string literal that should be a named constant is a finding: id
  `magic-string`, severity `info` — "A meaningful string literal (status, key,
  path) that should be a named constant."
- Meaning is decided by *use*, not by looks: `"active"` compared in an `if` is
  a magic string; `"Account is active"` rendered to a user is not.
- A protocol/status value that logic branches on is ALWAYS meaningful even on
  its first and only appearance — a status enum with one call site today still
  needs the name, because the name is the contract.
- Repetition alone is sufficient to flag: the same key or path spelled at two
  sites is a single-source-of-truth violation even if each looks harmless.
- The fix is a named constant or an enum member, not a comment — the value must
  have exactly one definition that every site references.
- Do not double-report with code-dry on the same duplicated literal; this lens
  owns the meaningful-literal finding, de-duplicated by file and line at
  aggregation.

## Do NOT flag

Keep noise low. These are not findings:

- Ordinary human-facing display text, UI copy, and error/exception messages
  shown to a person — nothing branches on their exact characters.
- Log and trace messages, including single-use format/template strings composed
  once for output.
- Empty strings (`""`) and whitespace-only literals used as defaults, joins, or
  separators — unless the empty value itself is a status the code tests for.
- A one-off literal with a single call site that is not a protocol value — a
  local default label, a test fixture's arbitrary name, a throwaway.
- Literals whose value is self-evident in place and that no other code must
  agree with — naming them adds indirection without a single-source-of-truth
  payoff.

When unsure, apply one test: if a second developer had to reproduce this exact
string elsewhere for the code to work, it is meaningful — name it. If it only
has to *read well* to a human, leave it as text.
