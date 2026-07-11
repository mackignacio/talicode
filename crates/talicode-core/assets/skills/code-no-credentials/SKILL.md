---
name: code-no-credentials
description: >-
  Never embed a login or account credential in code, and never store, log,
  transmit, or compare a user password in the clear. Flag hardcoded
  username+password pairs, account credentials, and basic-auth logins baked
  into source or config; passwords persisted unhashed; passwords leaked into
  logs, error messages, or exceptions; passwords compared with plain equality
  instead of a constant-time verify against a salted one-way hash; credentials
  embedded in connection strings or Authorization headers. Correct handling is
  salted password hashing (bcrypt/scrypt/argon2), constant-time comparison, and
  credentials sourced from a secret manager or environment. Use when reviewing
  authentication, login, sign-up, password reset, credential storage, or
  connection setup — triggers: credentials, plaintext password, hardcoded
  login, unhashed password, password in logs, basic auth, connection string.
---

# code-no-credentials

A human or account credential — a username paired with a password, a login for
a database or service, a basic-auth pair — is a long-lived secret that
identifies a person or account. It must never appear as a literal in source or
config, and a password must never exist in a readable, reversible, or
comparable-in-the-clear form. Passwords are verified, not retrieved: store only
a salted one-way hash, compare in constant time, and pull real credentials from
a secret store at runtime. This lens is the sibling of code-no-keys; that lens
owns API keys, tokens, and service secrets, this one owns human/account
credentials and password handling.

## The check

1. Scan every added or changed line for a credential literal: a username beside
   a password, an account login, or a basic-auth pair written directly in
   source, config, tests, fixtures, or a checked-in default.
2. Follow each password value along its whole path — where it is stored, logged,
   serialized, returned, and compared — and confirm it is never persisted or
   emitted in the clear.
3. At storage, confirm the password becomes a salted one-way hash from an
   adaptive algorithm (bcrypt, scrypt, argon2, PBKDF2), not plaintext, not a
   bare fast digest (MD5/SHA-1/SHA-256 with no salt or work factor).
4. At verification, confirm the comparison is a constant-time check of the
   candidate against the stored hash — never `==`, `!=`, string equality, or an
   early-returning byte-by-byte compare on the raw password.
5. Inspect connection strings, URLs, and `Authorization` headers for an embedded
   `user:password`, and confirm the value is composed from an environment
   variable or secret manager, not a literal.
6. Check logging, error messages, exceptions, and telemetry near an auth path
   for a password, credential object, or request body that carries one.

## Hard rules

- No credential literal — no hardcoded username+password, account login, or
  basic-auth pair — in source, config, or tests. Source it from the environment
  or a secret manager.
- Never store a password in the clear or under a reversible/unsalted transform.
  Persist only a salted, adaptive one-way hash.
- Never compare a password with plain equality. Verify the candidate against the
  stored hash with a constant-time function.
- Never write a password or credential into a log line, error message,
  exception, stack trace, or telemetry payload.
- Never embed `user:password` in a connection string, URL, or `Authorization`
  header baked into code.
- A placeholder is not an exemption: an obvious real-looking default credential
  shipped as a fallback is still a finding.

## What to flag / What NOT to flag

Flag: a username and password written as literals; a service or database login
in a connection string; a password saved to a column, file, or cache without
hashing; a password hashed with an unsalted fast digest; `if password ==
stored_password`; a logger, print, or exception that includes a password or a
full credential-bearing request; a basic-auth header assembled from string
literals.

Do NOT flag: passwords already handled by a salted adaptive hash and a
constant-time verify; credentials read from environment variables, a secret
manager, or a config source resolved at runtime; hashed or masked values in
logs; opaque session tokens or API keys — those belong to code-no-keys, not
here. Do not double-report a single credential that another lens already flags
at the same file and line.

Concrete rule — id `embedded-credential`, severity error: an embedded
login/account credential or a password handled in plaintext.
