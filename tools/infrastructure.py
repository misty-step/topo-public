"""Refresh approved Cloudflare metadata; retain explicitly supplied private snapshots."""

import argparse
import datetime
import json
import os
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

SCOPE_ID = re.compile(r"[a-z0-9][a-z0-9_-]{0,63}")
ENDPOINTS = {
    "workers": ("workers/scripts", "id", "service", "compute"),
    "domains": ("workers/domains", "id", "domain", "domain"),
    "databases": ("d1/database", "uuid", "database", "data"),
}
BINDING_FIELDS = {
    "d1": ("id",),
    "service": ("service", "environment", "entrypoint"),
    "kv_namespace": ("namespace_id",),
    "r2_bucket": ("bucket_name",),
    "durable_object_namespace": ("namespace_id", "class_name", "script_name"),
    "hyperdrive": ("id",),
    "queue": ("queue_name",),
    "vectorize": ("index_name",),
}


def validate(data):
    if data["schema"] != "topo.infrastructure.v1":
        raise ValueError("Expected reviewed infrastructure payload")
    maps = data["maps"]
    if not maps or any(not isinstance(m["id"], str) or not SCOPE_ID.fullmatch(m["id"]) for m in maps) or len({m["id"] for m in maps}) != len(maps):
        raise ValueError("Expected nonempty distinct infrastructure scopes with safe IDs")
    accounts = set()
    for m in maps:
        s = m["snapshot"]
        if s["schema"] != "topo.infrastructure-map.v1":
            raise ValueError("Unsupported native snapshot")
        for rows in (s["nodes"], s["edges"], s["sources"]):
            if len({r["id"] for r in rows}) != len(rows):
                raise ValueError("Duplicate inventory identity")
        nodes = {n["id"] for n in s["nodes"]}
        sources = {v["id"] for v in s["sources"]}
        for row in s["nodes"] + s["edges"]:
            if not set(row["sources"]) <= sources:
                raise ValueError("Missing evidence")
        for e in s["edges"]:
            if e["from"] not in nodes or e["to"] not in nodes:
                raise ValueError("Dangling inventory relationship")
        current = s["scenarios"]
        if len(current) != 1 or current[0]["id"] != "current":
            raise ValueError("Only observed baseline is admitted")
        if set(current[0]["node_ids"]) != nodes or set(current[0]["edge_ids"]) != {
            e["id"] for e in s["edges"]
        }:
            raise ValueError("Baseline must cover every supplied row")
        for a in m["cloudflare_accounts"]:
            if not re.fullmatch(r"[0-9a-f]{32}", a["id"]) or a["id"] in accounts:
                raise ValueError("Invalid or cross-scope account assignment")
            accounts.add(a["id"])
        for domain in m["dns_domains"]:
            if (
                not re.fullmatch(r"[a-z0-9][a-z0-9.-]{0,252}", domain["name"])
                or domain["node"] not in nodes
            ):
                raise ValueError("Invalid approved domain assignment")


def cloudflare(account, endpoint):
    """Internal GET only; raw responses never reach output or error messages."""

    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args):
            return None

    opener = urllib.request.build_opener(NoRedirect)
    request = urllib.request.Request(
        f"https://api.cloudflare.com/client/v4/accounts/{account}/{endpoint}",
        headers={"Authorization": "Bearer " + os.environ["CLOUDFLARE_API_TOKEN"]},
    )
    with opener.open(request, timeout=20) as response:
        result = json.load(response)
    if result.get("success") is not True:
        raise ValueError("Provider metadata unavailable; do not publish")
    return result["result"], result.get("result_info") or {}


def listing(account, endpoint):
    rows = []
    page = 1
    while True:
        query = f"?page={page}&per_page=100" if endpoint == "d1/database" else ""
        batch, info = cloudflare(account, endpoint + query)
        if not isinstance(batch, list):
            raise TypeError("Provider listing malformed")
        rows.extend(batch)
        total = info.get("total_count")
        more = info.get("total_pages", page) > page or (
            total is not None and len(rows) < total
        )
        if not more:
            if endpoint == "d1/database" and total is None and len(batch) == 100:
                more = True
            else:
                return rows
        if endpoint != "d1/database" or not batch or page >= 100:
            raise ValueError("Provider listing incomplete; do not publish")
        page += 1


def deployed_bindings(account, worker):
    """Project deployed versions to safe references before anything is retained."""
    path = "workers/scripts/" + urllib.parse.quote(worker, safe="")
    result, _ = cloudflare(account, path + "/deployments")
    deployment_observed_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
    deployments = result["deployments"]
    if not deployments:
        raise ValueError("Deployed-version metadata unknown; do not publish")
    # The first deployment is serving, not the newest uploaded version.
    deployment = deployments[0]
    observations = []
    for selected in deployment["versions"]:
        if selected["percentage"] <= 0:
            continue
        version_id = selected["version_id"]
        version, _ = cloudflare(
            account, path + "/versions/" + urllib.parse.quote(version_id, safe="")
        )
        bindings = version.get("resources", {}).get("bindings")
        if version["id"] != version_id or not isinstance(bindings, list):
            raise ValueError("Deployed binding metadata unknown; do not publish")
        safe = []
        for binding in bindings:
            item = {"name": binding["name"], "type": binding["type"]}
            if not all(isinstance(value, str) for value in item.values()):
                raise ValueError("Binding name/type metadata malformed")
            for field in BINDING_FIELDS.get(item["type"], ()):
                if isinstance(binding.get(field), str):
                    item[field] = binding[field]
            safe.append(item)
        observations.append(
            {
                "deployment_id": deployment["id"],
                "deployed_at": deployment["created_on"],
                "deployment_observed_at": deployment_observed_at,
                "deployed_versions": [
                    {"version_id": v["version_id"], "percentage": v["percentage"]}
                    for v in deployment["versions"]
                ],
                "version_id": version_id,
                "percentage": selected["percentage"],
                "observed_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "bindings": safe,
            }
        )
    if not observations:
        raise ValueError("No serving version observed; do not publish")
    return observations


def binding_edges(snapshot, account, worker, observations):
    """Only exact same-account IDs join; uninspected resource references stay unknown."""
    prefix = "cf-live:" + account + ":"
    worker_id = prefix + "workers:" + worker
    nodes = {n["id"]: n for n in snapshot["nodes"]}
    for observation in observations:
        source_id = (
            "cf-binding:"
            + account
            + ":"
            + worker
            + ":"
            + observation["deployment_id"]
            + ":"
            + observation["version_id"]
        )
        prior_edges = {e["id"]: e for e in snapshot["edges"]}
        observed_resources = {
            b["name"] for b in observation["bindings"] if b["type"] in ("d1", "service")
        }
        if any(
            e["id"].startswith(source_id + ":")
            and e["to"].startswith(prefix)
            and e["id"][len(source_id) + 1 :] not in observed_resources
            for e in prior_edges.values()
        ):
            raise ValueError(
                "Prior binding absent from available metadata; do not publish"
            )
        additions = []
        for binding in observation["bindings"]:
            kind = binding["type"]
            if kind in (
                "plain_text",
                "json",
                "secret_text",
                "secret_key",
                "version_metadata",
            ):
                continue  # Names/types are evidence, never values or resource dependencies.
            target = None
            if kind == "d1" and "id" in binding:
                target = prefix + "databases:" + binding["id"]
            elif (
                kind == "service"
                and binding.get("environment") == "production"
                and "service" in binding
            ):
                target = prefix + "workers:" + binding["service"]
            if target not in nodes:
                target = None
            edge_id = source_id + ":" + binding["name"]
            reference_id = edge_id + ":reference"
            if target is None:
                if edge_id in prior_edges and prior_edges[edge_id]["to"].startswith(
                    prefix
                ):
                    raise ValueError(
                        "Prior known binding cannot be replaced by unavailable metadata"
                    )
                target = reference_id
                snapshot["nodes"] = [
                    n for n in snapshot["nodes"] if n["id"] != reference_id
                ]
                snapshot["nodes"].append(
                    {
                        "id": reference_id,
                        "label": kind + " · " + binding["name"],
                        "kind": "resource-reference",
                        "category": "data",
                        "status": "unknown",
                        "summary": "Typed deployed binding reference; resource identity/presence uninspected, not health.",
                        "details": [json.dumps(binding)],
                        "sources": [source_id],
                    }
                )
            else:
                snapshot["nodes"] = [
                    n for n in snapshot["nodes"] if n["id"] != reference_id
                ]
            additions.append(
                {
                    "id": edge_id,
                    "from": worker_id,
                    "to": target,
                    "relation": "uses_binding",
                    "type": "data" if kind == "d1" else "binding",
                    "evidence": "read",
                    "sources": [source_id],
                    "label": binding["name"]
                    + " · "
                    + kind
                    + " · version "
                    + observation["version_id"]
                    + " · "
                    + str(observation["percentage"])
                    + "%; configured, not health",
                }
            )
        snapshot["edges"] = [
            e for e in snapshot["edges"] if e["id"] not in {a["id"] for a in additions}
        ] + additions
        snapshot["sources"] = [s for s in snapshot["sources"] if s["id"] != source_id]
        snapshot["sources"].append(
            {
                "id": source_id,
                "label": "Cloudflare deployed resource bindings · " + worker,
                "date": observation["observed_at"],
                "level": "api",
                "result": "Selected by current deployment; safe names/types/resource references only. Not runtime invocation or health. "
                + json.dumps(observation),
            }
        )
    # Prior version observations remain dated evidence, not silently refreshed or erased.
    nodes[worker_id]["sources"].extend(
        s["id"]
        for s in snapshot["sources"]
        if s["id"].startswith("cf-binding:" + account + ":" + worker + ":")
        and s["id"] not in nodes[worker_id]["sources"]
    )


def refresh(data):
    for m in data["maps"]:
        s = m["snapshot"]
        for domain in m["dns_domains"]:
            source_id = "dns-current:" + domain["name"]
            answers = []
            for record_type in ("NS", "MX"):
                query = urllib.parse.urlencode(
                    {"name": domain["name"], "type": record_type}
                )
                with urllib.request.urlopen(
                    "https://dns.google/resolve?" + query, timeout=20
                ) as response:
                    result = json.load(response)
                if result["Status"] != 0:
                    raise ValueError("DNS observation failed; do not publish")
                answers.append(
                    {"type": record_type, "answers": result.get("Answer", [])}
                )
            observed = datetime.datetime.now(datetime.timezone.utc).isoformat()
            s["sources"] = [r for r in s["sources"] if r["id"] != source_id]
            s["sources"].append(
                {
                    "id": source_id,
                    "label": "Public DNS NS/MX · " + domain["name"],
                    "date": observed,
                    "level": "dns",
                    "result": "Resolver metadata, not account custody, mail delivery or service health. "
                    + json.dumps(answers),
                }
            )
            node = next(n for n in s["nodes"] if n["id"] == domain["node"])
            if source_id not in node["sources"]:
                node["sources"].append(source_id)
            m["observations"] = [r for r in m["observations"] if r["id"] != source_id]
            m["observations"].append(
                {
                    "id": source_id,
                    "observed_at": observed,
                    "status": "observed-dns",
                    "coverage": "Public NS/MX only",
                }
            )
        for a in m["cloudflare_accounts"]:
            prefix = "cf-live:" + a["id"] + ":"
            for field in ("nodes", "edges", "sources"):
                s[field] = [r for r in s[field] if not r["id"].startswith(prefix)]
            m["observations"] = [
                r for r in m["observations"] if not r["id"].startswith(prefix)
            ]
            account_id = prefix + "account"
            worker_ids = set()
            for kind, (endpoint, key, entity_kind, category) in ENDPOINTS.items():
                rows = listing(a["id"], endpoint)
                observed = datetime.datetime.now(datetime.timezone.utc).isoformat()
                source_id = prefix + kind
                s["sources"].append(
                    {
                        "id": source_id,
                        "label": "Cloudflare GET " + endpoint,
                        "date": observed,
                        "level": "api",
                        "result": f"Observed metadata: {len(rows)} credential-visible records in approved account. Not health, traffic, legal ownership or complete estate discovery.",
                    }
                )
                m["observations"].append(
                    {
                        "id": source_id,
                        "observed_at": observed,
                        "status": "observed-metadata",
                        "records": len(rows),
                        "coverage": endpoint,
                    }
                )
                if kind == "workers":
                    s["nodes"].append(
                        {
                            "id": account_id,
                            "label": a["label"],
                            "kind": "account",
                            "category": "business",
                            "status": "configured",
                            "summary": "Approved source assignment; API-visible Cloudflare account, not legal billing custody.",
                            "details": [],
                            "sources": [source_id],
                        }
                    )
                for row in rows:
                    identity = row[key]
                    node_id = prefix + kind + ":" + identity
                    label = (
                        row.get("hostname")
                        if kind == "domains"
                        else row.get("name", identity)
                    )
                    s["nodes"].append(
                        {
                            "id": node_id,
                            "label": label,
                            "kind": entity_kind,
                            "category": category,
                            "status": "configured",
                            "summary": "Current provider metadata; runtime health and product/source binding unknown.",
                            "sources": [source_id],
                            "details": ["Provider observation: " + observed],
                            "last_verified": observed,
                        }
                    )
                    s["edges"].append(
                        {
                            "id": node_id + ":account",
                            "from": account_id,
                            "to": node_id,
                            "type": "account",
                            "relation": "contains",
                            "evidence": "read",
                            "label": "Listed in approved account",
                            "sources": [source_id],
                        }
                    )
                    if kind == "workers":
                        worker_ids.add(identity)
                    elif kind == "domains":
                        service = row["service"]
                        if service not in worker_ids:
                            raise ValueError(
                                "Domain service missing from observed Worker listing"
                            )
                        s["edges"].append(
                            {
                                "id": node_id + ":routes",
                                "from": node_id,
                                "to": prefix + "workers:" + service,
                                "type": "routing",
                                "evidence": "read",
                                "label": "Provider-declared service; environment="
                                + row.get("environment", "unknown")
                                + "; enabled="
                                + str(row.get("enabled", "unknown")),
                                "sources": [source_id],
                            }
                        )
            for worker in sorted(worker_ids):
                observations = deployed_bindings(a["id"], worker)
                binding_edges(s, a["id"], worker, observations)
                for observation in observations:
                    source_id = (
                        "cf-binding:"
                        + a["id"]
                        + ":"
                        + worker
                        + ":"
                        + observation["deployment_id"]
                        + ":"
                        + observation["version_id"]
                    )
                    m["observations"] = [
                        o for o in m["observations"] if o["id"] != source_id
                    ]
                    m["observations"].append(
                        {
                            "id": source_id,
                            "observed_at": observation["observed_at"],
                            "status": "observed-deployed-bindings",
                            "coverage": "Safe explicit deployed resource references; not health",
                        }
                    )
            s["scenarios"] = [
                {
                    "id": "current",
                    "node_ids": [n["id"] for n in s["nodes"]],
                    "edge_ids": [e["id"] for e in s["edges"]],
                }
            ]
    validate(data)
    return data


def freshness(data):
    now = datetime.datetime.now(datetime.timezone.utc)
    maps = []
    for m in data["maps"]:
        observations = m["observations"]
        dates = [
            datetime.datetime.fromisoformat(r["observed_at"].replace("Z", "+00:00"))
            for r in observations
            if r["observed_at"] is not None
        ]
        known = [d for d in dates if d.tzinfo is not None and d <= now]
        status = (
            "Stale"
            if known and (now - min(known)).total_seconds() >= 48 * 3600
            else "Unknown"
            if len(known) != len(observations) or not known
            else "Fresh"
        )
        maps.append(
            {
                "id": m["id"],
                "label": m["label"],
                "scope": "snapshot:" + m["id"],
                "status": status,
                "partial": True,
                "oldest_observed_at": min(known).isoformat() if known else None,
                "sources": observations,
                "gaps": m["snapshot"]["gaps"],
            }
        )
    return {
        "schema": "topo.infrastructure-freshness.v1",
        "generated_at": now.isoformat(),
        "stale_after_hours": 48,
        "maps": maps,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    if args.out.exists():
        raise ValueError("Output exists; use a fresh private path")
    data = json.loads(args.input.read_bytes())
    validate(data)
    refresh(data)
    data["freshness"] = freshness(data)
    with args.out.open("x") as f:
        json.dump(data, f)
    print(
        f"{len(data['maps'])} private infrastructure scopes refreshed; bounded provider metadata only."
    )


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError):
        # Do not echo provider errors, private paths, account IDs or credentials.
        print(
            "Infrastructure preparation failed; previous publication unchanged.",
            file=sys.stderr,
        )
        sys.exit(1)
