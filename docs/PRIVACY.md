# PROVE — Privacy

This document describes exactly what PROVE does with the text you give it, what
leaves your computer, what stays on it, and how to delete everything. It
describes the shipped behaviour, not an aspiration; every claim here maps to
code in this repository.

---

## 1. What PROVE processes

Only two pieces of text, both supplied by you:

- the **AI answer** you paste, drop, or pull in from the clipboard, and
- the **question** you optionally paste alongside it.

There is no account, no sign-up, no device identifier, and no profile. PROVE
never reads your clipboard on its own — a clipboard read happens only when you
press the global hotkey or click **Paste from clipboard**.

---

## 2. What leaves your computer

This depends entirely on how you configure the app. There are exactly three
outbound destinations the app can ever reach; they are enforced by the
Content-Security-Policy in `src-tauri/tauri.conf.json`, which permits only
`api.anthropic.com`, `api.search.brave.com`, and `api.github.com`.

| Configuration                    | What is sent                                                                                                                               | Where                                                                                                                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Local CLI provider** (default) | The question and answer are written to the standard input of the command you configured, e.g. `claude -p`.                                 | Nowhere, by PROVE's own doing. The text stays on your machine. Where that tool then sends it is governed by that tool's own configuration, which PROVE cannot see or control. |
| **Anthropic provider**           | The full question and answer text, plus the analysis prompt.                                                                               | `api.anthropic.com`, over HTTPS, authenticated with your own API key.                                                                                                         |
| **Brave Search key stored**      | Each factual claim, individually, as a plain search query. Then a plain HTTPS GET of each result URL so the article body can be extracted. | `api.search.brave.com`, then the third-party sites the search returned. Those fetches carry the user agent `prove/<version>` and no other identifying header.                 |
| **No Brave Search key**          | Nothing.                                                                                                                                   | Nowhere. Factual claims are classified but never web-verified.                                                                                                                |

How many claims are sent as search queries is your setting — **Settings → Web
verification**, default 8 per analysis, selectable up to "All".

Source fetching is deliberately restricted: only `http(s)` URLs are followed, at
most 10 redirects, and every hop is re-checked against a block list of loopback,
private, link-local, carrier-grade-NAT, and unique-local addresses. A page
cannot redirect PROVE into your local network.

### You are asked before any of this happens

By default, PROVE shows a confirmation before every analysis listing exactly
which provider will receive the text, whether web verification will run and for
how many claims, and how many characters are involved. You can dismiss it for
one analysis, or turn it off permanently in **Settings → Privacy**.

---

## 3. The update check

Off by default. When you switch on **Settings → Updates → Check for updates on
launch**, PROVE makes exactly one unauthenticated `GET` to
`api.github.com/repos/lukoplt/ai-prove/releases/latest` at startup and compares
the version string locally. No data about you, your machine, or your analyses is
included in that request, and nothing is downloaded or installed automatically —
the banner only offers to open the release page in your browser.

---

## 4. What is stored locally, and where

Two files, both inside the standard per-app data directory:

- **macOS:** `~/Library/Application Support/app.prove.desktop/`
- **Windows:** `%APPDATA%\app.prove.desktop\`

| File            | Contents                                                                                                                                                                                                                    |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cache.db`      | SQLite. Table `analysis_history` holds every completed analysis (the input text and the full claim/verification result). Table `verification_cache` holds per-claim verification results keyed by a hash of the claim text. |
| `settings.json` | Non-secret preferences: language, theme, hotkey, provider choice, CLI command, verification limit, retention window, and the boolean toggles. **No API keys.**                                                              |

The database is not encrypted at rest. It relies on your operating system's
protection of your home directory. If you need encryption at rest today, use
full-disk encryption (FileVault / BitLocker).

---

## 5. API keys

Your Anthropic and Brave Search keys are stored in the **OS keychain** — macOS
Keychain or Windows Credential Manager — through the `keyring` crate. They are
never written to `settings.json`, never logged, and never rendered back into the
UI: the settings screen only tells you whether a key is present.

---

## 6. Retention and deletion

- **History retention.** _Settings → Privacy → How long to keep history_.
  Default 90 days; also 7, 30, 365, or "Forever". Anything older than the window
  is deleted when the app starts.
- **Delete one analysis.** _History → Delete_ on any row. Immediate and
  permanent.
- **Delete all history.** _History → Delete all history_. Immediate and
  permanent.
- **Verification cache.** Expires on its own after _Settings → Cache TTL_ days,
  default 7.
- **Delete everything.** Quit PROVE and delete the app data directory listed in
  section 4. To also remove the stored API keys, delete the `prove` entries from
  Keychain Access (macOS) or Credential Manager (Windows).

---

## 7. Telemetry

There is none. PROVE contains no analytics SDK, no crash reporter, no
fingerprinting, no usage counters, and no phone-home of any kind. The three
hosts in section 2 are the complete list of network destinations the application
can reach, and each one is reached only because you configured it.

---

## 8. Verifying any of this

Everything above is in this repository:

- Outbound host allow-list: `src-tauri/tauri.conf.json` (`app.security.csp`)
- What the confirmation dialog claims: `src/lib/sendSummary.ts` (and its tests)
- Key storage: `src-tauri/src/storage/keychain.rs`
- History storage, retention, deletion: `src-tauri/src/storage/history.rs`
- Update check: `src-tauri/src/commands/updates.rs`
- SSRF protections on source fetching: `src-tauri/src/search/extract.rs`
