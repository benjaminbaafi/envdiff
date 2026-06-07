# envdiff

A fast, local `.env` auditor. Diff, lint, validate, and guard your environment files — in milliseconds, entirely on your machine.

```
$ envdiff check
→ Found 3 env file(s):
  .env
  .env.local
  .env.production

Key coverage:
  ✓ DATABASE_URL
  ✓ PORT
  ✗ STRIPE_KEY — missing in: .env.production
  ✗ SENTRY_DSN — missing in: .env, .env.local

✗ 2 missing key(s) detected.
```

---

## Why

`.env` files are where bugs hide. A key present in `.env.local` but missing in production causes a silent failure at 2am. A `sk_live_` key accidentally committed to git ends your weekend. A hyphenated variable name crashes your Kubernetes deployment.

`envdiff` catches all of it — before it ships.

---

## Install

**From crates.io:**
```sh
cargo install envdiff
```

**From source:**
```sh
git clone https://github.com/yourname/envdiff
cd envdiff
cargo install --path .
```

---

## Commands

### `check` — scan a directory for key mismatches

Finds all `.env*` files in a directory and reports which keys are missing across environments.

```sh
envdiff check           # scans current directory
envdiff check ./myapp   # scans a specific directory
```

---

### `diff` — colored diff between two files

Shows keys only in one file, keys only in the other, and keys with different values. Secret values are automatically masked.

```sh
envdiff diff .env .env.production
```

```
--- .env
+++ .env.production

− DEBUG (only in .env)
+ SENTRY_DSN (only in .env.production)
~ DATABASE_URL value differs:
    − post••••••••
    + post••••••••
```

---

### `lint` — catch formatting issues

Checks for problems that break tools silently: hyphenated keys, keys starting with digits, duplicate keys, BOM characters, mismatched quotes, trailing whitespace.

```sh
envdiff lint            # lints .env
envdiff lint .env.local
```

```
4 lint issue(s) in .env:

  [error] line 1 [bad-key]: Key contains a hyphen — invalid in POSIX, crashes systemd/k8s
  [error] line 2 [1TOKEN]: Key starts with a digit — invalid in most environments
  [warn ] line 5 [API_KEY]: Duplicate key — also defined on line 3
  [error] line 8 [CERT]: Mismatched quotes in value
```

---

### `validate` — POSIX/systemd/Kubernetes compliance

Strict check that all key names are valid: no hyphens, no leading digits, no special characters.

```sh
envdiff validate
envdiff validate .env.production
```

---

### `missing` — cross-reference against `.env.example`

Reports keys required by `.env.example` that are absent from `.env`, and keys in `.env` not yet documented in `.env.example`.

```sh
envdiff missing
envdiff missing .env .env.example
```

---

### `sync` — keep `.env.example` up to date

Copies key names from `.env` into `.env.example` without ever copying values. Use `--dry-run` to preview changes.

```sh
envdiff sync --dry-run   # preview what would change
envdiff sync             # write the changes
```

---

### `audit` — detect exposed secrets

Scans values against a pattern library covering AWS keys, GitHub tokens, Stripe keys, Slack tokens, JWTs, PEM certificates, Google API keys, NPM tokens, SendGrid, and Twilio. Exits non-zero on critical findings.

```sh
envdiff audit
envdiff audit .env.production
```

```
✗ 1 secret(s) detected in .env:

  [critical] STRIPE_KEY — Stripe Live Secret Key

Run `envdiff mask` to redact values for safe sharing.
```

---

### `mask` — redact values for safe sharing

Replaces all values with `****`. Outputs to stdout by default so you can pipe it safely — use `--in-place` to overwrite the file.

```sh
envdiff mask                  # print redacted version to stdout
envdiff mask --in-place       # overwrite file with redacted values
envdiff mask .env.production  # mask a specific file
```

---

### `guard` — block secret leaks at commit time

Installs a git pre-commit hook that runs `audit` on any staged `.env` files and blocks the commit if secrets are detected. Safely appends to an existing hook if one already exists.

```sh
envdiff guard       # installs hook in current repo
envdiff guard ./app # installs hook in a specific repo
```

Once installed, every `git commit` that includes a `.env` file is automatically scanned.

---

## Parser support

`envdiff` handles real-world `.env` syntax correctly:

| Syntax | Supported |
|---|---|
| `KEY=value` | ✓ |
| `export KEY=value` | ✓ |
| `KEY="quoted value"` | ✓ |
| `KEY='single quoted'` | ✓ |
| `KEY=value # inline comment` | ✓ |
| Multi-line double-quoted values | ✓ |
| `# full-line comments` | ✓ |
| Blank lines | ✓ |

---

## Use in CI

Add a step to your GitHub Actions workflow to block merges with secret leaks or missing keys:

```yaml
- name: Audit .env files
  run: |
    cargo install envdiff
    envdiff audit .env.example
    envdiff validate .env.example
```

---

## License

MIT
