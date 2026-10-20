"""Open or update the one CI failure issue using the current job's token."""
import json
import os
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def notify(api, repository, token, run_url, sha):
    def request(method, path, data=None):
        body = json.dumps(data).encode() if data is not None else None
        req = Request(f"{api}/repos/{repository}/{path}", body, method=method,
                      headers={"Authorization": f"Bearer {token}",
                               "Accept": "application/vnd.github+json",
                               "X-GitHub-Api-Version": "2022-11-28",
                               "Content-Type": "application/json"})
        with urlopen(req, timeout=30) as response:
            return json.load(response)

    try:
        request("GET", "labels/ci-failure")
    except HTTPError as error:
        if error.code != 404:
            raise
        error.close()
        try:
            request("POST", "labels", {"name": "ci-failure", "color": "b60205",
                                      "description": "Full source verification failed"})
        except HTTPError as conflict:
            if conflict.code != 422:
                raise
            conflict.close()
            request("GET", "labels/ci-failure")

    issues = request("GET", "issues?state=open&labels=ci-failure&sort=created&direction=asc&per_page=100")
    existing = next((issue for issue in issues if "pull_request" not in issue), None)
    content = {"title": "Full source verification failed",
               "body": f"The full source suite failed. Fix forward and verify the exact source.\n\nRun: {run_url}\nSource SHA: `{sha}`\n",
               "labels": ["ci-failure"]}
    if existing:
        # Keep any operator labels when updating the latest failure.
        content.pop("labels")
        issue = request("PATCH", f"issues/{existing['number']}", content)
    else:
        issue = request("POST", "issues", content)
    return issue["number"]


if __name__ == "__main__":
    repository = os.environ["GITHUB_REPOSITORY"]
    run_url = f"{os.environ['GITHUB_SERVER_URL']}/{repository}/actions/runs/{os.environ['GITHUB_RUN_ID']}"
    number = notify(os.environ["GITHUB_API_URL"], repository, os.environ["GH_TOKEN"],
                    run_url, os.environ["GITHUB_SHA"])
    print(f"Updated CI failure issue #{number}")
