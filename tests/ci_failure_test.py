import importlib.util
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import unittest

spec = importlib.util.spec_from_file_location("notify", Path(__file__).parents[1] / "tools/notify_ci_failure.py")
notification = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notification)


class NotificationTest(unittest.TestCase):
    def test_fresh_repository_and_repeated_failure_use_one_issue(self):
        state = {"label": False, "issues": [], "writes": []}
        token = "synthetic-job-token"
        case = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def handle_request(self):
                case.assertEqual(self.headers["Authorization"], f"Bearer {token}")
                data = json.loads(self.rfile.read(int(self.headers["Content-Length"]))) if self.headers.get("Content-Length") else None
                path = self.path.removeprefix("/repos/example/source/")
                status = 200
                if path == "labels/ci-failure":
                    status, result = (200, {"name": "ci-failure"}) if state["label"] else (404, {})
                elif path == "labels":
                    case.assertEqual(data["name"], "ci-failure")
                    state["label"] = True
                    status, result = 201, data
                elif path.startswith("issues?"):
                    # PRs can carry the same label but cannot receive this issue update.
                    result = [{"number": 1, "pull_request": {}}] + state["issues"]
                elif path == "issues":
                    case.assertEqual(len(state["issues"]), 0)
                    result = {"number": 2, **data}
                    state["issues"].append(result)
                    state["writes"].append(data)
                    status = 201
                elif path == "issues/2":
                    case.assertNotIn("labels", data)
                    state["issues"][0].update(data)
                    state["writes"].append(data)
                    result = state["issues"][0]
                else:
                    raise AssertionError(path)
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(result).encode())

            do_GET = do_POST = do_PATCH = handle_request

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            for index, sha in enumerate(["a" * 40, "b" * 40], 1):
                run_url = f"https://github.com/example/source/actions/runs/{index}"
                self.assertEqual(notification.notify(f"http://127.0.0.1:{server.server_port}",
                                                     "example/source", token, run_url, sha), 2)
                self.assertIn(run_url, state["writes"][-1]["body"])
                self.assertIn(sha, state["writes"][-1]["body"])
            self.assertEqual(len(state["issues"]), 1)
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
