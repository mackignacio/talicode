---
name: code-no-keys
description: >-
  Never hardcode secrets in source. Flag committed API keys, access / refresh /
  bearer tokens, OAuth client secrets, private keys (RSA / EC / SSH / PEM
  blocks), passwords embedded in connection strings or URLs, cloud provider
  access keys (AWS AKIA…, GCP / Azure keys), webhook signing secrets, and
  high-entropy strings that read like live credentials. Secrets belong in
  environment variables or a dedicated secret manager (Vault, AWS / GCP Secrets
  Manager) — never as committed literals, comments, or shipping test fixtures.
  A leaked secret persists in git history after deletion and must be rotated.
  Ignore obvious placeholders. Use when you see a hardcoded secret, api key,
  access token, bearer token, client secret, private key, PEM block, connection
  string password, aws access key, webhook secret, or leaked token.
---

# code-no-keys — no hardcoded secrets in source

Credentials in source are a breach waiting to happen. Anyone with read access to
the repo — contractors, CI logs, a future acquirer, an attacker who cloned it —
holds the key. This lens keeps keys, tokens, and secrets out of committed code and
pushes them into environment variables or a secret manager, where they can be
scoped, rotated, and audited. Its sibling **code-no-credentials** covers account
logins and plaintext passwords; this lens owns API keys, tokens, and machine
secrets.

## The check

1. Identify string literals that carry a credential: API keys, access / refresh /
   bearer / session tokens, OAuth client secrets, signing or webhook secrets, and
   the password segment of a connection string or URL (`scheme://user:pass@host`).
2. Recognize private-key material: PEM / OpenSSH blocks (`BEGIN … PRIVATE KEY`,
   `BEGIN RSA/EC PRIVATE KEY`), and standalone key bytes.
3. Recognize provider-specific shapes: AWS access key ids (`AKIA…`) and secret
   keys, GCP / Azure keys, Stripe / Slack / GitHub token prefixes, JWTs.
4. Assess entropy and intent: a long, random-looking, non-dictionary string
   assigned to a credential-named symbol (`api_key`, `secret`, `token`, `password`)
   is almost certainly real.
5. Inspect comments, docstrings, and any test fixture or seed data that SHIPS — a
   secret is no safer for being commented out or labelled "example".
6. Confirm the source of truth: is the value read from `env` / a secret manager,
   or is it a literal baked into the file? Only the literal is a finding.

## Hard rules

- A hardcoded secret is severity **error**, id `hardcoded-secret`: "A hardcoded
  secret (API key, token, password, connection string, or private key)."
- Secrets are loaded at runtime from environment variables or a dedicated secret
  manager — never committed as literals, defaults, or fallbacks.
- No private-key material of any kind belongs in the repository.
- No secret in a comment, disabled code, or a fixture that ships with the product.
- A leaked secret survives in git history even after the line is deleted or the
  file is removed. Deletion is not remediation — the credential must be **rotated**.
- Report once per file+line; findings are de-duplicated at that granularity.

## What to flag

- A real-looking API key, access / refresh / bearer token, or client secret set
  as a literal, default argument, or config constant.
- Private keys — PEM / OpenSSH / RSA / EC blocks — anywhere in tree.
- A connection string or URL with an embedded password (`db://svc:S3cr3t@…`).
- Cloud provider keys (`AKIA…` plus a 40-char secret), webhook signing secrets.
- High-entropy strings assigned to credential-named identifiers, in prod code,
  comments, or fixtures that ship.

## What NOT to flag

- Obvious placeholders and templates: `YOUR_API_KEY_HERE`, `xxxx`, `<token>`,
  `changeme`, all-zeros, or clearly-fake sample values.
- Values genuinely sourced from `env` / a secret manager at runtime — the point of
  this lens is to encourage exactly that.
- Non-secret config: public keys, public client ids, hostnames, ports, feature
  flags, and other non-sensitive settings.
- Low-entropy dictionary strings that a credential scanner would never trip on.
  When unsure whether a value is live, prefer the real-looking judgment over noise —
  but do not manufacture findings from plausible-but-fake examples.
