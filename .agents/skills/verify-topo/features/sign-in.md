# Sign-in

The owner of a self-hosted private deployment opens its private host. Cloudflare
Access sends them to the configured login method. After they sign in, the
`topo-maps` Worker checks the Access JWT against `ACCESS_TEAM_DOMAIN` and
`ACCESS_AUD`, then serves the private home. Anyone else, and any forged
session, gets no map content.

## Sub-features

- `sign-in-gate`: a signed-out visitor is sent off the private host to sign in and sees no private home content.
- `sign-in-wrong-identity`: a forged `CF_Authorization` cookie and an unknown service token are both refused, with no 200.
- `sign-in-owner`: the owner signs in, and `/cdn-cgi/access/get-identity` returns `TOPO_VERIFY_OWNER_EMAIL`. If `TOPO_VERIFY_OWNER_IDP_TYPE` is set, the IdP type must also match it.
- `sign-in-owner-page`: in that session the owner-only home renders (title `Topo`, link `Projects N`), and `/manifest.json` is `topo.home.v1`.
- `sign-in-config`: the deployed owner configuration matches the real owner and nothing wider:
  - the allow rules admit exactly the `TOPO_VERIFY_ALLOWED_EMAILS` set (default: the owner), with no `require` or `exclude`;
  - non-identity rules are service tokens only, and there is no bypass;
  - exactly one login method is allowed;
  - the app audience equals `vars.ACCESS_AUD`.

## How to get to it (user POV)

- Open `https://<PRIVATE_HOST>/` in a browser and sign in with the owner identity through the Access login method.

## Driving it with verify-topo

Preconditions:

- `TOPO_VERIFY_PRIVATE_SETTINGS` names the deployment's settings JSON, kept outside the checkout. `TOPO_VERIFY_OWNER_EMAIL` names the owner.
- `doctor --live` passes the Access audience and owner-policy checks.
- For the owner step, one of these:
  - an existing owner session in `~/.cache/tmp/verify-topo/owner-profile`;
  - the owner's credential in an environment variable you name;
  - the owner present for a headed browser.

- **Config.** Inject `CLOUDFLARE_API_TOKEN` (read-only Access and log scope) and `TOPO_MAPS_ACCESS_CREDENTIAL` from your secret manager, then run `.agents/skills/verify-topo/verify-topo doctor --live`. Every Access line must be `PASS`. A swapped or extra email, an everyone or domain rule, a `require` or `exclude` condition, or a second login method fails it. The `last owner login` line is informational.
- **Signed out.** Run `.agents/skills/verify-topo/verify-topo sign-in`. The gate drive opens the private host in a fresh browser context. It must leave the private host and show no `Projects` link. Set `TOPO_VERIFY_SIGN_IN_HOST` to also require the exact login host.
- **Wrong identity.** The same drive requests the host anonymously, with a forged `CF_Authorization` cookie, and with an unknown `CF-Access-Client-Id`/`CF-Access-Client-Secret` pair. Each must get 401/403, or a redirect to `<ACCESS_TEAM_DOMAIN>/cdn-cgi/access/login/`. A 200, a 5xx or any other redirect fails.
- **Owner session.** The owner drive opens the host in the persistent owner profile. If it lands on the private host, the existing session is reused. Otherwise it needs one of:
  - `--password-env NAME`: for an email-and-password login page. It fills textbox `Email` with the owner and textbox `Password` from `$NAME`, chooses `Sign in`, then `Continue` if shown;
  - `--headed`: opens a visible browser and waits up to 10 minutes for the owner to finish, for any login method.
  With neither, it stops and reports `UNPROVEN`.
- **Owner-only page.** In the session it fetches `/cdn-cgi/access/get-identity`, which must show the owner. It checks title `Topo` and link `Projects N`, then reads `/manifest.json`, which must be `topo.home.v1`.
- **Proof.** `private-evidence/sign-in/result.json`, `gate-signed-out.png` and `.aria.txt`, and `private-evidence/sign-in/owner/result.json`. These sit outside `evidence/` because they name the operator's host and owner. Never commit them, or `doctor --live` output, to this public repository. That file holds the identity email, IdP type, manifest schema, `generated_at`, source revision and page count. It has no private page content.

## Gotchas

- Without `TOPO_VERIFY_PRIVATE_SETTINGS` and `TOPO_VERIFY_OWNER_EMAIL`, sign-in reports `SKIP` (n/a). It never passes. Only claim owner sign-in when `sign-in-owner` passed.
- If no agent can hold the owner's credential, the owner step stays `UNPROVEN`. Name the missing access, for example "owner password for <identity>" or "owner present for `--headed`". Keep the step in every run. Do not swap in a weaker proof.
- A publishing service token passes Access as `non_identity`. It proves the Worker serves the bundle. It is not an owner session and never satisfies `sign-in-owner`.
- The Access log is a passive signal of real owner logins. It is not agent proof.
- Never drive a password-reset or recovery flow. It changes the owner's credential.
- Screenshots and ARIA snapshots of the signed-in home would capture private maps. The owner drive records only identity and manifest metadata. Keep it that way.
- Access sessions expire after the application's session duration. A stale profile lands on the sign-in page again, which is normal.
