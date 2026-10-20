# Source review bootstrap

This branch registers the manual workflow entry point only.
Dispatch `full.yml` against `source-review` to verify the reviewed source before merge.
The source branch supplies the complete workflow and development tree.
Dispatching this bootstrap branch deliberately fails: it proves no source behavior
or failure notification. Only the actual source workflow and drill count as proof.
