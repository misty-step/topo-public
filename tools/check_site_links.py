"""Check internal hrefs against a generated static site's files and anchors."""
from html.parser import HTMLParser
from pathlib import Path
import sys
from urllib.parse import unquote, urljoin, urlsplit


class Page(HTMLParser):
    def __init__(self, path):
        super().__init__(convert_charrefs=True)
        self.links = []
        self.anchors = set()
        self.feed(path.read_text())

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if attrs.get("id"):
            self.anchors.add(attrs["id"])
        if tag == "a" and attrs.get("name"):
            self.anchors.add(attrs["name"])
        if "href" in attrs and attrs["href"] is not None:
            self.links.append(attrs["href"])


def check(root):
    root = root.resolve()
    pages = {path: Page(path) for path in root.rglob("*.html")}
    errors = []
    for path, page in sorted(pages.items()):
        relative = path.relative_to(root).as_posix()
        base = "/" + relative.removesuffix("index.html")
        for href in page.links:
            url = urlsplit(href)
            if url.scheme or url.netloc:
                continue
            resolved = urlsplit(urljoin(base, href))
            target = (root / unquote(resolved.path).lstrip("/")).resolve()
            if not target.is_relative_to(root):
                errors.append(f"{relative}: href escapes site: {href}")
                continue
            if target.is_dir():
                target /= "index.html"
            if not target.is_file():
                errors.append(f"{relative}: missing file for {href}")
            elif resolved.fragment and target in pages:
                anchor = unquote(resolved.fragment)
                if anchor not in pages[target].anchors:
                    errors.append(f"{relative}: missing anchor for {href}")
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Internal hrefs resolve across {len(pages)} generated HTML files")


if __name__ == "__main__":
    check(Path(sys.argv[1]))
