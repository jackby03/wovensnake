#!/usr/bin/env python3
"""Fetch metadata for python-build-standalone releases and emit a JSON catalog."""
import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from typing import Optional

GITHUB_API = "https://api.github.com/repos/astral-sh/python-build-standalone/releases"
USER_AGENT = "woven-metadata-fetch/1.0"  # mimic a harmless client


def fetch_releases(page: int, token: Optional[str]):
    headers = {"User-Agent": USER_AGENT}
    if token:
        headers["Authorization"] = f"token {token}"

    url = f"{GITHUB_API}?page={page}&per_page=50"
    request = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(request) as response:
        if response.status != 200:
            raise RuntimeError(f"GitHub API returned {response.status} for page {page}")
        return json.load(response)


def parse_asset(asset: dict):
    name = asset.get("name", "")
    if not name.lower().startswith("cpython-"):
        return None

    parts = name.split("-")
    if len(parts) < 3:
        return None

    version_part = parts[1].split("+")[0]
    flavor_part = parts[-1].split(".")[0]
    platform = "-".join(parts[2:-1])
    flavor = flavor_part
    shared = "shared" in name

    return {
        "name": parts[0],
        "version": version_part,
        "platform": platform,
        "flavor": flavor,
        "shared": shared,
        "url": asset.get("browser_download_url"),
    }


def main():
    parser = argparse.ArgumentParser(description="Generate python asset metadata catalog from astral releases.")
    parser.add_argument("--output", default="metadata/python_downloads.json", help="Path to write the JSON catalog")
    parser.add_argument("--pages", type=int, default=3, help="Number of release pages to scan")
    parser.add_argument("--token", help="GitHub token to raise rate limits")
    args = parser.parse_args()

    catalog = []
    seen = set()

    for page in range(1, args.pages + 1):
        try:
            releases = fetch_releases(page, args.token)
        except urllib.error.HTTPError as exc:
            print(f"Failed to fetch page {page}: {exc}", file=sys.stderr)
            break
        except Exception as exc:
            print(f"Unexpected error: {exc}", file=sys.stderr)
            break

        if not releases:
            break

        for release in releases:
            for asset in release.get("assets", []):
                metadata = parse_asset(asset)
                if not metadata or not metadata.get("url"):
                    continue
                key = (metadata["version"], metadata["platform"], metadata.get("flavor"))
                if key in seen:
                    continue
                seen.add(key)
                catalog.append(metadata)

    catalog.sort(key=lambda entry: (entry["version"], entry["platform"]))
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as fd:
        json.dump(catalog, fd, indent=2)
    print(f"Wrote {len(catalog)} entries to {args.output}")


if __name__ == "__main__":
    main()
