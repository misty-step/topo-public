# Topo specification

The collection, query, privacy, export, design and installation contracts are
[US-001 through US-012](USER_STORIES.md). They remain the program's behavioral
specification; this index does not duplicate them.

## US-013 Receive verified public code continuously

As a user, I can get working public documentation and native downloads without
waiting for an operator to publish a private map.

- A verified current main revision promotes its tested public Worker/assets.
  Failed, skipped, missing or superseded verification cannot promote code.
- Every advertised native target is tested after relocation with only Git/date
  available. Older versioned downloads keep their original bytes and checksums.
- The public manifest distinguishes serving code version/source from the last
  published native version/source. Development builds say development; they do
  not claim to be a tagged release.
- Installation and local scans need no account or upload. Public publication
  contains only docs, fictional demo, dependency notices and released binaries.

## US-014 Read the notes included in the serving build

As a user, I can follow Changelog from the public navigation to the canonical
notes included in this build, on desktop and phone, without a GitHub credential.

- Conventional changes determine semantic versions. Generated version/notes
  changes go through a release PR with real CI on its exact current head.
- A merged release PR publishes an immutable tag and GitHub Release, attaching
  the native executables tested from that same source in the publishing run.
  Missing or mismatched tag source never attaches that run's binaries. Artifact
  source binding compares the build with the tag, not a moving branch head.
- A no-release change still deploys verified code and does not invent a release.
- Private inventories and private operational notes never enter the public notes.
- Public notes preserve canonical note text but render private source-repository
  references as text, not inaccessible links. Useful public links remain usable;
  the source changelog and repository visibility are unchanged.

## US-015 Recover code without restoring data

As an operator, I can restore an identified successful public Worker version
and verify its exact manifest and the docs/demo/install journey.

- Recovery refuses a version not bound to the requested source, a failed source
  run, or an artifact manifest from another source.
- Code promotion and recovery serialize. A disposable candidate/previous-code/
  candidate rehearsal leaves the tested candidate bytes unchanged.
- Private maps retain their existing separate serialization, Access boundary,
  source/artifact/manifest reconciliation guard and dated data. Code recovery
  does not restore private datasets, local scans or credentials.

## US-016 Identify public failures without uploading private content

As an operator, I can identify the public site's serving source and environment
and receive its unexpected failures through the existing Sentry triage intake.

- Public GET/HEAD `/health` checks the serving public manifest and returns a
  non-cacheable healthy identity or an unavailable response. This checks public
  publication only, never private maps, local scans or overall estate health.
- Error events identify the environment, source and provider version, using fixed
  failure categories. Requests, headers, bodies, arbitrary exception text, private
  inventories and user identities never enter telemetry. Ordinary 404/405 responses
  are not errors. There is no public error-trigger or mutation endpoint.
- Error delivery runs outside the response path, with bounded volume/time and no
  retries. An unavailable or misconfigured telemetry sink cannot stall public reads.
- Native external uptime coverage requires an authorized monitor allocation;
  exposing health or capturing errors alone does not claim uptime/alert recovery.
