"""Combine a source template with explicitly supplied operator deployment settings."""
import argparse
import json
import os
from pathlib import Path
import re
import tomllib


def configure(base, settings, output, private=False):
    if base.resolve().parent != output.resolve().parent:
        raise ValueError("Keep generated config beside its template for relative asset paths")
    if output.exists():
        raise ValueError("Deployment output already exists")
    if not isinstance(settings, dict) or set(settings) != {"name", "account_id", "routes", "vars"}:
        raise ValueError("Expected explicit name, account_id, routes and vars")
    if not isinstance(settings["name"], str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,62}", settings["name"]):
        raise ValueError("Invalid Worker name")
    if not isinstance(settings["account_id"], str) or not re.fullmatch(r"[0-9a-f]{32}", settings["account_id"]):
        raise ValueError("Invalid account identity")
    routes = settings["routes"]
    if not isinstance(routes, list) or not routes or any(
        not isinstance(route, dict) or set(route) != {"pattern", "custom_domain"}
        or route["custom_domain"] is not True
        or not isinstance(route["pattern"], str)
        or not re.fullmatch(r"[a-z0-9]+(?:[.-][a-z0-9]+)+", route["pattern"])
        for route in routes
    ):
        raise ValueError("Expected explicit custom-domain routes")
    variables = settings["vars"]
    if not isinstance(variables, dict) or any(not isinstance(value, str) for value in variables.values()):
        raise ValueError("Expected string deployment variables")
    if private:
        host = variables.get("PRIVATE_HOST", "")
        issuer = variables.get("ACCESS_TEAM_DOMAIN", "")
        if len(routes) != 1 or routes[0]["pattern"] != host:
            raise ValueError("Private route must match PRIVATE_HOST")
        if not re.fullmatch(r"https://[a-z0-9]+(?:-[a-z0-9]+)*\.cloudflareaccess\.com", issuer):
            raise ValueError("Expected an explicit Access team issuer")
        audience = variables.get("ACCESS_AUD", "")
        if not audience or audience.strip() != audience:
            raise ValueError("Expected an explicit Access audience")
    config = tomllib.loads(base.read_text())
    config.update({key: settings[key] for key in ("name", "account_id", "routes")})
    config["vars"] = {**config.get("vars", {}), **variables}
    with os.fdopen(os.open(output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "w") as destination:
        destination.write(json.dumps(config) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", type=Path, required=True)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--settings", type=Path)
    source.add_argument("--settings-env")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--private", action="store_true")
    args = parser.parse_args()
    try:
        raw = args.settings.read_text() if args.settings else os.environ[args.settings_env]
        configure(args.base, json.loads(raw), args.out, args.private)
    except (KeyError, ValueError, OSError, TypeError):
        raise SystemExit("Deployment settings unavailable or invalid; no configuration generated")
