# Privacy and sharing

## Local by default

The CLI reads repositories you explicitly select and writes files to a directory
you choose. It does not upload source, inventory, usage events or model prompts.
It does not use provider/model credentials, execute your project, install its
dependencies or modify infrastructure.

The public Cloudflare site serves the fictional demo, docs and downloads. It is
not a map-upload service. Normal website/download requests reach Cloudflare;
there is no added application analytics script or account system.

## What an export contains

A map can disclose repository names, relative paths, requested packages,
relationships, source hashes, Git context, dates and collection errors. Imported
inventories can additionally contain private resource names, recorded ownership
and historical cost information. Source bodies and target scripts are not
exported, but an export is still sensitive.

Ignored/generated and credential-file paths are excluded; leaf/ancestor symlinks
are not followed. This is not a universal secret-redaction service. Static import
literals and already-redacted imported metadata must be appropriate for the
intended recipients. Review what you share.

## Share a deliberately selected run

Run Topo on only the repositories intended for the recipient, review the bundle,
then share the complete output folder through an approved private channel.
Recipients can open `index.html` without installing Topo. Keep sibling files
for scope navigation. No tailnet invitation is required for an offline bundle.

**A focused view is not a privacy boundary.** The folder's `graph.json` contains
the entire run. Hiding nodes, choosing a module or sharing a focus URL does not
redact other scopes. Offline exports do not provide restricted subsets or
per-recipient authorization.

Use a fresh output directory for each run. Topo refuses to overwrite earlier
maps. Deleting or rolling back the executable does not delete your maps.

## Opt-in private hosting

The separate private-map Worker serves deliberately selected static outputs at
an explicitly configured private hostname. Cloudflare Access restricts the whole
hostname; the Worker additionally verifies the issuer, audience, validity, and
RS256 signature before any asset lookup. A missing host or Access configuration
fails closed. Workers.dev and previews are disabled. Responses are private/no-store.
Every admitted reader can read the entire selected graph and project dossier;
there is no repository-level permission boundary.

`topo-home` composes existing exports. Collection and runtime observation dates
are retained, including stale imported evidence and partial coverage. Publication
does not prove fresh host or provider observations.

Keep source selections, inventories, operator settings, and credentials outside
Git. Use your deployment system's encrypted credential injection. Access audience
identifiers alone are configuration. See [deployment configuration](/docs/deployment/).

## Restore and authority

Keep a protected copy of the complete bundle and the matching executable if
long-term reproducibility matters. `topo render` can recreate view files from a
valid graph without recollecting. The revision hash is integrity, not access
control, complete recovery proof or retirement authority.

[Collection limits](/docs/limitations/)
