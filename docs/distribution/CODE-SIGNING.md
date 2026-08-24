# Code signing, notarization, and SmartScreen — handover checklist

**Status: nothing in this document has been executed.** No certificate has been
bought, no key exists in this repository, no CI secret has been created, and no
binary has been signed with anything other than an ad-hoc signature. This is the
list of decisions and commands the repository owner has to run themselves,
because every step needs an identity, a payment method, or a password that only
they hold.

---

## 1. Where PROVE stands today

`src-tauri/tauri.conf.json` sets:

```json
"macOS": {
  "signingIdentity": "-",
  "entitlements": "Entitlements.plist"
}
```

`"-"` is an **ad-hoc** signature. It makes the binary loadable on the machine
that built it and nothing more. In practice:

- **macOS.** A downloaded `.dmg` is quarantined. Gatekeeper shows _"PROVE cannot
  be opened because the developer cannot be verified."_ The only way in is
  right-click → Open, or `xattr -d com.apple.quarantine`. Most users stop there.
- **Windows.** The `.msi` and the NSIS `-setup.exe` are unsigned, so SmartScreen
  shows _"Windows protected your PC"_ with **Run anyway** hidden behind _More
  info_. Unsigned installers never accumulate reputation, so this never
  improves on its own.

`src-tauri/Entitlements.plist` is already minimal and correct for signing: it
opts into outbound network, JIT, and unsigned executable memory (WebKit needs
both), and deliberately does **not** request the app sandbox — the sandbox would
break both the OS keychain access and the CLI-provider subprocess.

---

## 2. What the owner has to obtain

Nothing below can be automated from this repo.

### macOS

- [ ] Apple Developer Program membership — 99 USD/year.
- [ ] A **Developer ID Application** certificate (not "Apple Development", not
      "Mac App Distribution"), created in the Apple Developer portal and
      installed in the login keychain.
- [ ] The **Team ID** (10 characters, visible in the Developer portal under
      Membership).
- [ ] Credentials for `notarytool`, either: - an **App Store Connect API key**: Issuer ID + Key ID + the `.p8` file
      (preferred — no 2FA prompts, revocable per key), or - an **app-specific password** generated at appleid.apple.com for the
      Apple ID that owns the membership.

### Windows

- [ ] One of three options — **this is the decision that is still open**:

  | Option                                    | Cost        | SmartScreen reputation                                      | CI friendliness                                                                                                           |
  | ----------------------------------------- | ----------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
  | **OV certificate** (Sectigo, DigiCert, …) | lowest      | Builds slowly — weeks of downloads before the warning stops | Since June 2023 OV keys must also live on an HSM/token, so unattended CI signing needs the vendor's cloud-signing service |
  | **EV certificate**                        | highest     | Immediate — no warning from the first download              | Hardware token; unattended CI is awkward unless the vendor offers cloud signing                                           |
  | **Azure Trusted Signing**                 | low monthly | Immediate, same as EV                                       | Best — designed for CI, no physical token, signs via an Azure endpoint                                                    |

  For a solo open-source project shipping through GitHub Actions, **Azure
  Trusted Signing** is usually the least painful, but it requires an Azure
  subscription and an identity validation that takes a few business days.
  Eligibility currently requires an organisation with 3+ years of verifiable
  history, or an individual validation path — check current terms before
  committing.

---

## 3. macOS: what to change and what to run

### Config

Replace the ad-hoc identity in `src-tauri/tauri.conf.json`:

```json
"macOS": {
  "signingIdentity": "Developer ID Application: YOUR NAME (TEAMID1234)",
  "entitlements": "Entitlements.plist"
}
```

Leave `Entitlements.plist` as it is — it has already been reviewed for minimum
privilege, and the hardened runtime it assumes is what notarization requires.

### CI secrets

`tauri-action` reads these environment variables. Add them as GitHub Actions
repository secrets and reference them **by name only** in
`.github/workflows/release.yml` — never `echo` them, never put them in a job
summary:

| Secret                       | What it is                                                                        |
| ---------------------------- | --------------------------------------------------------------------------------- |
| `APPLE_CERTIFICATE`          | The Developer ID cert exported as `.p12`, then base64-encoded                     |
| `APPLE_CERTIFICATE_PASSWORD` | The password chosen during the `.p12` export                                      |
| `APPLE_SIGNING_IDENTITY`     | The full identity string, e.g. `Developer ID Application: YOUR NAME (TEAMID1234)` |
| `APPLE_ID`                   | The Apple ID email (only for the app-specific-password path)                      |
| `APPLE_PASSWORD`             | The app-specific password (same path)                                             |
| `APPLE_TEAM_ID`              | The 10-character Team ID                                                          |

With the API-key path instead: `APPLE_API_ISSUER`, `APPLE_API_KEY`, and
`APPLE_API_KEY_PATH` (the `.p8` written to disk by an earlier step).

Export the `.p12` locally like this — run it on the machine holding the
certificate, and put the output into the secret:

```bash
base64 -i DeveloperID.p12 | pbcopy
```

### Verifying a signed build

After a signed local build (`pnpm tauri build`), all three must pass:

```bash
codesign --verify --deep --strict --verbose=2 src-tauri/target/release/bundle/macos/PROVE.app
```

```bash
spctl --assess --type execute --verbose src-tauri/target/release/bundle/macos/PROVE.app
```

```bash
xcrun stapler validate src-tauri/target/release/bundle/dmg/PROVE_0.4.4_universal.dmg
```

`spctl` must report `accepted` and `source=Notarized Developer ID`. If it says
`source=Unnotarized Developer ID`, the signature is fine but the notarization
ticket was never stapled — the build is signed but users will still be warned
when offline.

### Submitting for notarization by hand

`tauri-action` does this automatically when the Apple variables are present. The
manual equivalent, for debugging a rejection:

```bash
xcrun notarytool submit PROVE_0.4.4_universal.dmg --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_PASSWORD" --wait
```

```bash
xcrun notarytool log <submission-id> --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_PASSWORD"
```

```bash
xcrun stapler staple PROVE_0.4.4_universal.dmg
```

The `log` command is the one that matters: notarization rejections are almost
always a missing hardened runtime flag or an unsigned nested binary, and the log
names the exact path.

---

## 4. Windows: what to change and what to run

### Config

Add a `windows` block to `bundle` in `src-tauri/tauri.conf.json`:

```json
"windows": {
  "certificateThumbprint": "THUMBPRINT_WITHOUT_SPACES",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

The thumbprint is the certificate's SHA-1 fingerprint with spaces stripped.
Read it from an installed certificate with PowerShell:

```powershell
Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Format-List Subject, Thumbprint
```

**The timestamp URL is not optional.** Without a countersignature, every
installer stops validating the day the certificate expires. With one, binaries
signed before expiry stay valid indefinitely.

### Verifying a signed build

```powershell
signtool verify /pa /v .\src-tauri\target\release\bundle\nsis\PROVE_0.4.4_x64-setup.exe
```

```powershell
signtool verify /pa /v .\src-tauri\target\release\bundle\msi\PROVE_0.4.4_x64_en-US.msi
```

Both the MSI and the NSIS installer need signing — users can download either
from the GitHub release, so signing only one leaves half the downloads warned.

### If Azure Trusted Signing is chosen

The Tauri config keys above do not apply. Signing happens as a separate CI step
using `azure/trusted-signing-action` against the built artefacts, with
`AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, and `AZURE_CLIENT_SECRET` as repository
secrets. Note that this action is a third party in the release path, so pin it
to a full commit SHA exactly like every other action in
`.github/workflows/release.yml` already is.

---

## 5. What changes in CI

In `.github/workflows/release.yml`, the `tauri-apps/tauri-action` step gains an
`env:` block alongside the existing `GITHUB_TOKEN`. Nothing else in the workflow
changes. Two rules:

1. Reference secrets by name; never interpolate them into a `run:` shell line
   where they could land in a log.
2. Keep every third-party action pinned to a full commit SHA. The repo already
   does this and the signing secrets make the release job a far more valuable
   target than before.

---

## 6. Explicitly not covered here

- Mac App Store or Microsoft Store distribution (different certificates,
  different sandbox rules — the app currently cannot be sandboxed).
- The Tauri auto-updater's own signing keypair and key rotation. That belongs to
  `05-distribution.md` and is a separate secret from the platform certificates.
- Linux packaging and signing.

---

## 7. The one decision still open

**Which Windows path?** OV (cheapest, weeks of SmartScreen warnings), EV
(immediate trust, hardware token, awkward CI), or Azure Trusted Signing
(immediate trust, CI-friendly, needs an Azure subscription and identity
validation). Everything else in this document follows mechanically once that is
answered; macOS has only one real path and it is fully specified above.
