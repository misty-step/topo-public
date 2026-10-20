"""Consequential observation-age contract; generation is not observation."""

import datetime
import json
import runpy
import unittest
from pathlib import Path
from unittest.mock import patch

helpers = runpy.run_path(str(Path(__file__).parents[1] / "tools/infrastructure.py"))
freshness = helpers["freshness"]
deployed_bindings = helpers["deployed_bindings"]
binding_edges = helpers["binding_edges"]


class Freshness(unittest.TestCase):
    def test_old_unknown_and_future_observations_are_not_refreshed_by_generation(self):
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        maps = [
            {
                "id": "old",
                "label": "Old",
                "observations": [
                    {"observed_at": "2000-01-01T00:00:00Z"},
                    {"observed_at": now},
                    {"observed_at": None},
                ],
                "snapshot": {"gaps": []},
            },
            {
                "id": "unknown",
                "label": "Unknown",
                "observations": [{"observed_at": now}, {"observed_at": None}],
                "snapshot": {"gaps": []},
            },
            {
                "id": "future",
                "label": "Future",
                "observations": [{"observed_at": "2999-01-01T00:00:00Z"}],
                "snapshot": {"gaps": []},
            },
        ]
        result = freshness({"maps": maps})["maps"]
        self.assertEqual([m["status"] for m in result], ["Stale", "Unknown", "Unknown"])
        self.assertEqual(result[0]["oldest_observed_at"], "2000-01-01T00:00:00+00:00")
        self.assertIsNone(result[2]["oldest_observed_at"])


class Bindings(unittest.TestCase):
    def test_serving_split_versions_exact_ids_and_private_value_projection(self):
        account = "0" * 32
        prefix = "cf-live:" + account + ":"
        private = "SYNTHETIC_PRIVATE_VALUE_MUST_NOT_ESCAPE"
        responses = {
            "workers/scripts/caller/deployments": {
                "deployments": [
                    {
                        "id": "deployed",
                        "created_on": "2026-01-01T00:00:00Z",
                        "versions": [
                            {"version_id": "a", "percentage": 25},
                            {"version_id": "b", "percentage": 75},
                        ],
                    }
                ]
            },
            "workers/scripts/caller/versions/a": {
                "id": "a",
                "resources": {
                    "script_runtime": {"unsafe": private},
                    "bindings": [
                        {"name": "DB", "type": "d1", "id": "db-id", "value": private},
                        {"name": "TEXT", "type": "plain_text", "text": private},
                        {"name": "JSON", "type": "json", "json": {"nested": private}},
                        {"name": "KEY", "type": "secret_text", "text": private},
                    ],
                },
            },
            "workers/scripts/caller/versions/b": {
                "id": "b",
                "resources": {
                    "bindings": [
                        {
                            "name": "API",
                            "type": "service",
                            "service": "callee",
                            "environment": "production",
                            "entrypoint": "Api",
                            "value": private,
                        },
                        {
                            "name": "WRONG_ENV",
                            "type": "service",
                            "service": "callee",
                            "environment": "staging",
                        },
                        {"name": "UNKNOWN_ENV", "type": "service", "service": "callee"},
                        {"name": "NAME_ONLY", "type": "d1", "id": "database-name"},
                        {
                            "name": "CACHE",
                            "type": "kv_namespace",
                            "namespace_id": "uninspected",
                            "value": private,
                        },
                    ]
                },
            },
        }
        with patch.dict(
            deployed_bindings.__globals__,
            {"cloudflare": lambda account, path: (responses[path], {})},
        ):
            observations = deployed_bindings(account, "caller")
        self.assertEqual(
            [(v["version_id"], v["percentage"]) for v in observations],
            [("a", 25), ("b", 75)],
        )
        snapshot = {
            "nodes": [
                {"id": prefix + "workers:caller", "sources": []},
                {"id": prefix + "workers:callee", "sources": []},
                {
                    "id": prefix + "databases:db-id",
                    "label": "database-name",
                    "sources": [],
                },
            ],
            "sources": [],
            "edges": [],
        }
        binding_edges(snapshot, account, "caller", observations)
        known = [e for e in snapshot["edges"] if e["to"].startswith(prefix)]
        self.assertEqual(
            {e["to"] for e in known},
            {prefix + "databases:db-id", prefix + "workers:callee"},
        )
        self.assertEqual(
            {e["from"] for e in snapshot["edges"]}, {prefix + "workers:caller"}
        )
        references = [
            n for n in snapshot["nodes"] if n.get("kind") == "resource-reference"
        ]
        self.assertEqual(
            {json.loads(n["details"][0])["name"] for n in references},
            {"WRONG_ENV", "UNKNOWN_ENV", "NAME_ONLY", "CACHE"},
        )
        self.assertTrue(all(n["status"] == "unknown" for n in references))
        self.assertNotIn(
            private, json.dumps({"observations": observations, "snapshot": snapshot})
        )
        self.assertEqual(len(snapshot["sources"]), 2)

    def test_unavailable_reference_cannot_erase_prior_positive_binding(self):
        account = "0" * 32
        prefix = "cf-live:" + account + ":"
        observation = {
            "deployment_id": "d",
            "deployed_at": "2026-01-01T00:00:00Z",
            "version_id": "v",
            "percentage": 100,
            "observed_at": "2026-01-02T00:00:00Z",
            "bindings": [{"name": "DB", "type": "d1"}],
        }
        source = "cf-binding:" + account + ":caller:d:v"
        prior = {
            "id": source + ":DB",
            "from": prefix + "workers:caller",
            "to": prefix + "databases:db-id",
            "sources": [source],
        }
        snapshot = {
            "nodes": [
                {"id": prefix + "workers:caller", "sources": [source]},
                {"id": prefix + "databases:db-id", "sources": [source]},
            ],
            "sources": [{"id": source, "date": "2026-01-01T00:00:00Z"}],
            "edges": [prior],
        }
        with self.assertRaises(ValueError):
            binding_edges(snapshot, account, "caller", [observation])
        observation["bindings"] = []
        with self.assertRaises(ValueError):
            binding_edges(snapshot, account, "caller", [observation])
        self.assertEqual(snapshot["edges"], [prior])
        self.assertEqual(snapshot["sources"][0]["date"], "2026-01-01T00:00:00Z")

class ScopeInputs(unittest.TestCase):
    def test_refresh_cli_uses_only_explicit_fictional_scopes(self):
        import copy
        import subprocess
        import tempfile

        script = Path(__file__).parents[1] / "tools/infrastructure.py"
        snapshot = {
            "schema": "topo.infrastructure-map.v1", "nodes": [], "edges": [], "sources": [], "gaps": [],
            "scenarios": [{"id": "current", "node_ids": [], "edge_ids": []}],
        }
        maps = [{"id": scope, "label": scope, "snapshot": copy.deepcopy(snapshot),
                 "cloudflare_accounts": [], "dns_domains": [], "observations": []}
                for scope in ["example-one", "example-two", "example-three"]]
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            source, output = root / "input.json", root / "output.json"
            source.write_text(json.dumps({"schema": "topo.infrastructure.v1", "maps": maps}))
            result = subprocess.run(["python3", str(script), "--input", str(source), "--out", str(output)], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            document = json.loads(output.read_bytes())
            self.assertEqual([m["id"] for m in document["freshness"]["maps"]], [m["id"] for m in maps])
            self.assertEqual([m["status"] for m in document["freshness"]["maps"]], ["Unknown"] * 3)
            self.assertEqual([m["snapshot"] for m in document["maps"]], [snapshot] * 3)
            for bad in [[], [maps[0], maps[0]], [{**maps[0], "id": "../outside"}]]:
                source.write_text(json.dumps({"schema": "topo.infrastructure.v1", "maps": bad}))
                refused = root / "refused.json"
                result = subprocess.run(["python3", str(script), "--input", str(source), "--out", str(refused)], capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(refused.exists())


if __name__ == "__main__":
    unittest.main()
