#!/usr/bin/env python3
"""Regenerate bat/version.json from the Foojay Disco API."""

from __future__ import annotations

import json
import time
import http.client
import urllib.error
import urllib.request
from collections import defaultdict
from datetime import date
from pathlib import Path

API_BASE = "https://api.foojay.io/disco/v3.0"
OUTPUT = Path(__file__).with_name("version.json")
MIN_VERSION = 6
MAX_VERSION = 27
ARCHIVE_PRIORITY = ("zip", "msi", "exe")
MAX_RETRIES = 5


HEADERS = {
    "accept": "application/json",
    "user-agent": "Jir/1.0 (version.json updater; +https://github.com/rururunu/Jir)",
}


def fetch_json(url: str) -> dict:
    last_error: Exception | None = None
    for attempt in range(1, MAX_RETRIES + 1):
        try:
            req = urllib.request.Request(url, headers=HEADERS)
            with urllib.request.urlopen(req, timeout=120) as resp:
                return json.load(resp)
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, http.client.IncompleteRead) as exc:
            last_error = exc
            if attempt == MAX_RETRIES:
                break
            time.sleep(attempt * 2)
    raise RuntimeError(f"failed to fetch {url}: {last_error}") from last_error


def resolve_redirect(url: str) -> str:
    last_error: Exception | None = None
    for attempt in range(1, MAX_RETRIES + 1):
        try:
            return _resolve_redirect_once(url)
        except (urllib.error.URLError, TimeoutError, RuntimeError) as exc:
            last_error = exc
            if attempt == MAX_RETRIES:
                break
            time.sleep(attempt * 2)
    raise RuntimeError(f"failed to resolve redirect {url}: {last_error}") from last_error


def _resolve_redirect_once(url: str) -> str:
    req = urllib.request.Request(
        url,
        method="HEAD",
        headers={
            **HEADERS,
            "accept": "*/*",
        },
    )
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, req, fp, code, msg, headers, newurl):
            return None

    opener = urllib.request.build_opener(NoRedirect)
    try:
        opener.open(req, timeout=60)
    except urllib.error.HTTPError as exc:
        location = exc.headers.get("Location")
        if exc.code in (301, 302, 303, 307, 308) and location:
            return location
        raise
    raise RuntimeError(f"redirect did not return a Location header: {url}")


def pick_package(packages: list[dict]) -> dict | None:
    if not packages:
        return None

    for archive_type in ARCHIVE_PRIORITY:
        candidates = [p for p in packages if p.get("archive_type") == archive_type]
        if not candidates:
            continue

        def sort_key(pkg: dict) -> tuple:
            javafx = 1 if pkg.get("javafx_bundled") else 0
            latest = 0 if pkg.get("latest_build_available") else 1
            return (latest, javafx, pkg.get("java_version", ""))

        return sorted(candidates, key=sort_key)[0]

    return None


def load_firm_names() -> dict[str, str]:
    data = fetch_json(f"{API_BASE}/distributions")
    return {
        item["api_parameter"]: item["name"]
        for item in data.get("result", [])
        if item.get("api_parameter")
    }


def fetch_packages_for_version(feature_version: int, version_query: str) -> list[dict]:
    url = (
        f"{API_BASE}/packages?"
        f"package_type=jdk&operating_system=windows&architecture=x64"
        f"&directly_downloadable=true&version={version_query}"
    )
    data = fetch_json(url)
    grouped: dict[str, list[dict]] = defaultdict(list)
    for pkg in data.get("result", []):
        if pkg.get("major_version") != feature_version:
            continue
        grouped[pkg["distribution"]].append(pkg)

    selected: list[dict] = []
    for distro in sorted(grouped):
        picked = pick_package(grouped[distro])
        if picked is not None:
            selected.append(picked)
    return selected


def fetch_version_packages(feature_version: int) -> list[dict]:
    upper = feature_version + 1
    selected = fetch_packages_for_version(feature_version, f"{feature_version}..%3C{upper}")
    if not selected:
        print("  no GA builds, trying early-access...", flush=True)
        selected = fetch_packages_for_version(feature_version, f"{feature_version}-ea")
    return selected


def to_entry(pkg: dict, firm_names: dict[str, str]) -> dict:
    distro = pkg["distribution"]
    redirect = pkg["links"]["pkg_download_redirect"]
    try:
        url = resolve_redirect(redirect)
    except urllib.error.HTTPError as exc:
        raise RuntimeError(f"redirect failed for {distro} {pkg.get('filename')}: {exc}") from exc

    entry = {
        "version": pkg["major_version"],
        "java_version": pkg["java_version"],
        "firm": firm_names.get(distro, distro.replace("_", " ").title()),
        "distro": distro,
        "filename": pkg["filename"],
        "archive_type": pkg["archive_type"],
        "url": url,
    }
    if pkg.get("release_status") == "ea":
        entry["release_status"] = "ea"
    return entry


def main() -> None:
    firm_names = load_firm_names()
    packages: list[dict] = []

    for feature_version in range(MIN_VERSION, MAX_VERSION + 1):
        print(f"Fetching Java {feature_version}...", flush=True)
        version_packages = fetch_version_packages(feature_version)
        if not version_packages:
            continue
        for pkg in version_packages:
            packages.append(to_entry(pkg, firm_names))
            print(
                f"  {pkg['distribution']} -> {pkg['filename']}",
                flush=True,
            )

    packages.sort(key=lambda item: (item["version"], item["distro"]))
    output = {
        "generated_at": date.today().isoformat(),
        "source": "Foojay Disco API",
        "platform": "windows",
        "architecture": "x64",
        "package_type": "jdk",
        "selection": (
            "one latest directly downloadable package per Java feature version "
            "and distro; zip preferred; falls back to early-access when GA is unavailable"
        ),
        "packages": packages,
    }
    OUTPUT.write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Wrote {len(packages)} packages to {OUTPUT}", flush=True)


if __name__ == "__main__":
    main()
