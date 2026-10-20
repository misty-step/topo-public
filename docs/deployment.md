# Self-host deployment

`wrangler.toml` and `wrangler.maps.toml` are build templates. They contain no
operator account, custom domain, Access issuer, audience, or private hostname.
Local collection needs none of these settings. Hosting and recovery belong to
your deployment system. The source CI builds and tests without deploying.

## Supply explicit settings

Keep a JSON settings file outside the checkout. Public hosting requires exactly
`name`, `account_id`, `routes`, and `vars`. Private hosting also requires
`PRIVATE_HOST`, `ACCESS_TEAM_DOMAIN`, and `ACCESS_AUD` in `vars`.
This fictional private example is not a working deployment:

```json
{
	"name": "example-private-maps",
	"account_id": "00000000000000000000000000000000",
	"routes": [{"pattern": "maps.example.test", "custom_domain": true}],
	"vars": {
		"PRIVATE_HOST": "maps.example.test",
		"ACCESS_TEAM_DOMAIN": "https://example.cloudflareaccess.com",
		"ACCESS_AUD": "example-audience"
	}
}
```

Generate an untracked configuration beside its template so relative asset and
build paths keep their meaning:

```sh
python3 tools/deployment.py --base wrangler.maps.toml \
	--settings /path/to/private/settings.json --private \
	--out .topo-private-deploy.json
```

For public hosting, select `wrangler.toml`, omit `--private`, and choose a fresh
output filename. Public settings need no Access variables.

`--settings-env VARIABLE_NAME` explicitly selects JSON from an environment
variable instead of a file. There is no default settings search or bundled
owner state. Invalid or missing settings fail before any hosting operation.
Existing generated files are not overwritten. Generated files have mode `0600`.
Keep credentials in your deployment system's encrypted injection, outside this JSON.
Access audience identifiers alone are configuration.

## Protect private assets

Configure your own Access application and restrict the private route to
`PRIVATE_HOST`. The Worker rejects another hostname, missing host settings,
and failed Access verification before serving assets. Keep scans, inventories,
source selections, settings, and receipts outside published source and artifacts.

Build the private bundle from explicit inputs with `topo-regenerate`; it does
not publish it. Select the generated configuration explicitly in your hosting
system. After a write, verify the served manifest and its source identity through
your authenticated route. Preserve the last working artifact and provider version.
After uncertain readback, reconcile those identities before retry or recovery.
Code recovery does not restore private datasets. Maintain their backups separately.
