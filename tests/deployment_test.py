"""Exercise explicit deployment configuration without contacting a provider."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).parents[1]


class Deployment(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.base = self.root / "template.toml"
        self.base.write_text('name="example"\nmain="worker.js"\nworkers_dev=false\npreview_urls=false\n[assets]\ndirectory="assets"\nbinding="ASSETS"\n[vars]\nSENTRY_ENVIRONMENT="production"\n')
        self.settings = {
            "name": "example-private-maps", "account_id": "0" * 32,
            "routes": [{"pattern": "maps.example.test", "custom_domain": True}],
            "vars": {"PRIVATE_HOST": "maps.example.test", "ACCESS_TEAM_DOMAIN": "https://example.cloudflareaccess.com", "ACCESS_AUD": "example-audience"},
        }

    def run_config(self, settings, output="deploy.json", private=True):
        path = self.root / "settings.json"
        path.write_text(json.dumps(settings))
        return subprocess.run([
            "python3", str(ROOT / "tools/deployment.py"), "--base", str(self.base),
            "--settings", str(path), "--out", str(self.root / output),
            *(["--private"] if private else []),
        ], capture_output=True)

    def test_explicit_settings_preserve_build_paths_and_protect_output(self):
        result = self.run_config(self.settings)
        self.assertEqual(result.returncode, 0, result.stderr)
        path = self.root / "deploy.json"
        config = json.loads(path.read_bytes())
        self.assertEqual(config["main"], "worker.js")
        self.assertEqual(config["assets"]["directory"], "assets")
        self.assertEqual(config["routes"], self.settings["routes"])
        self.assertEqual(config["vars"], {"SENTRY_ENVIRONMENT": "production", **self.settings["vars"]})
        self.assertFalse(config["workers_dev"])
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        before = path.read_bytes()
        self.assertNotEqual(self.run_config(self.settings).returncode, 0)
        self.assertEqual(path.read_bytes(), before)

    def test_public_config_needs_no_access_settings(self):
        settings = {**self.settings, "vars": {}}
        self.assertEqual(self.run_config(settings, private=False).returncode, 0)

    def test_missing_env_never_falls_back_to_a_bundled_owner(self):
        result = subprocess.run([
            "python3", str(ROOT / "tools/deployment.py"), "--base", str(self.base),
            "--settings-env", "TOPO_MISSING_FIXTURE_SETTINGS", "--out", str(self.root / "deploy.json"),
        ], env={key: value for key, value in os.environ.items() if key != "TOPO_MISSING_FIXTURE_SETTINGS"}, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "deploy.json").exists())

    def test_private_host_issuer_audience_and_unexpected_fields_refuse_without_values(self):
        for field, value in [("PRIVATE_HOST", "other.example.test"), ("ACCESS_TEAM_DOMAIN", "http://example.cloudflareaccess.com"), ("ACCESS_AUD", "")]:
            settings = copy.deepcopy(self.settings)
            settings["vars"][field] = value
            result = self.run_config(settings)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn(b"example.test", result.stderr)
            self.assertFalse((self.root / "deploy.json").exists())
        self.assertNotEqual(self.run_config({**self.settings, "build": {"command": "unexpected"}}).returncode, 0)

    def test_output_cannot_change_the_template_relative_path_base(self):
        (self.root / "different").mkdir()
        self.assertNotEqual(self.run_config(self.settings, "different/deploy.json").returncode, 0)
        self.assertFalse((self.root / "different/deploy.json").exists())
