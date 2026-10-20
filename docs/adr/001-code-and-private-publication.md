# ADR 001: Public source CI and operator-owned hosting

Accepted: 2026-10-15. Updated: 2026-10-20 for the public source tree.

## Decision

This repository distributes development source. The public site and native
installer remain separate distribution endpoints. Source CI neither publishes
releases nor changes hosted code, Access settings, or private maps.

The source branch and its clean publication candidate have identical trees.
The original private repository keeps its history, configuration, deployment
custody, and backups. Its public-source review draft must never merge into its
private main branch. Publication uses fresh history after a separate privacy,
metadata, access, and retained-history audit.

## Verification and notification

`.github/workflows/pr.yml` runs a small advisory set on pull requests:
TypeScript checking and Python unit tests. Superseded PR runs are cancelled.
It does not run the full native, browser, or Worker suite.

`.github/workflows/full.yml` runs twice daily and on manual dispatch. Its run
concurrency is independent of PR checks and never cancels an earlier full run.
It runs `bun run check`, the reusable native relocation matrix on hosted Linux
and both macOS targets, the built public Worker's error tests, the Chromium demo,
and the built private Worker's authorization tests. Fictional inputs and
in-memory signing keys exercise refusal before asset access. All jobs skip in
private repositories. Actions are pinned to full commit identities.

Workflows have read-only contents permission. The full suite's failure job alone
has issue-write permission through its ephemeral `github.token`. It creates the
`ci-failure` label if absent and opens or updates one open issue with the run URL
and source SHA. Notification updates serialize. A manual `drill` input forces a
failure to prove delivery; scheduled runs ignore it. No external notification
service or long-lived credential is needed.

The initial default branch can contain only a generic README and a dispatch-only
workflow at the same `full.yml` path. The audited source commit descends from
that bootstrap, allowing manual proof of the source branch before merge.
Hosted exact-source verification and the failure drill must be observed before
publication is called proven. A local syntax check does not establish delivery.

## Hosting and recovery

Operators supply explicit external settings as described in
[the deployment contract](../deployment.md). Local collection needs no hosting
account. Public site inputs stay limited to docs, design exports, the fictional
demo, component notices, and verified native binaries. Private data never enters
that build.

Each operator owns deployment, version readback, retention, and recovery in their
hosting system. Preserve the last working artifact and reconcile provider and
manifest identities before retrying uncertain writes. Code recovery does not
restore private datasets, local scans, or credentials.

The distribution and recovery stories US-013, US-014, and US-015 remain product
intent for the separately operated distribution. This source repository does
not claim their automated promotion or rollback is implemented here. US-016's
built public error boundary remains exercised with a synthetic local sink.
No first-party license is introduced. Existing third-party notices and grammar
provenance remain applicable.
