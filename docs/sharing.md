# Share a map

Give a friend or coworker an offline map they can open without installing Topo,
joining your network or obtaining your credentials.

## Make a recipient-specific run

Collect only the repositories intended for that person:

```sh
topo run --repo shared-app="$HOME/work/shared-app" \
  --out "$HOME/topo-maps/shared-app-for-review"
```

Review the map, `graph.json` and collection metadata before sharing. Exports can
reveal names, paths, packages and relationships even though source bodies are
not included. [Privacy and export contents](/docs/privacy/) owns the full policy.

**Selecting a module or focusing a node does not redact the bundle.** Do not
send an organization/personal portfolio and assume the selected view restricts
what the recipient can read.

## Send the complete folder

Use an approved private channel. You can zip the folder, but keep these files
together when the recipient extracts it:

- `index.html`
- `scope-*.html`
- `graph.json`
- `manifest.json`

Open `index.html` in a desktop browser. Scope navigation uses sibling files, so
sending only one HTML attachment loses the complete navigation path. Some mail
and cloud-drive previews do not run interactive HTML; download and extract it
before opening.

## Help the recipient orient themselves

Tell them which run/date to open and which question to investigate. Link them to
[map controls](/docs/using-the-map/) and [evidence](/docs/evidence/) rather than
asking them to infer health or authority from colored wires.

A recipient can explore the bundle offline. If they also install Topo, they can
[query its graph](/docs/agents/) or render a fresh set of views without your source
checkout. Rendering does not refresh the evidence or change its revision.

<details>
<summary>Serve a bundle on a website</summary>

A static host can serve the whole folder. Use an approved host and independent
access controls suitable for that data. Topo's scope menu, graph checksum,
filters and focus URL are not authorization or redaction.

The public Topo marketing site is not a customer-map upload or hosting service.
Do not publish private output there. Ordinary static-host links and offline
sibling-file links are supported, but authentication is the host's responsibility.

</details>

<details>
<summary>Withdraw access or send an update</summary>

A downloaded offline copy cannot be remotely revoked by Topo. Decide whether
that sharing contract is appropriate before sending it. A new collection is a
new folder/run; tell recipients which one supersedes the earlier evidence.

</details>
