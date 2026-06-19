#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# ///

"""Create or update the managed Shields.io compatibility report issue."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


def run_gh(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["gh", *args],
        text=True,
        capture_output=True,
        check=False,
    )


def warn(message: str) -> None:
    print(f"::warning::{message}", file=sys.stderr)


def ensure_label(repo: str, label: str) -> bool:
    result = run_gh(
        [
            "label",
            "create",
            label,
            "--repo",
            repo,
            "--description",
            "Managed Shields.io compatibility report issue",
            "--color",
            "5319e7",
        ]
    )
    if result.returncode == 0:
        return True

    message = result.stderr.strip() or result.stdout.strip()
    if "already exists" in message.lower():
        return True

    warn(f"could not create compatibility report label: {message}")
    return False


def find_issue(repo: str, label: str) -> int | None:
    result = run_gh(
        [
            "issue",
            "list",
            "--repo",
            repo,
            "--state",
            "open",
            "--author",
            "github-actions[bot]",
            "--label",
            label,
            "--json",
            "number,updatedAt",
        ]
    )
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        warn(f"could not search compatibility report issues: {message}")
        return None

    issues = json.loads(result.stdout)
    if not issues:
        return None

    issues.sort(key=lambda issue: issue["updatedAt"], reverse=True)
    return int(issues[0]["number"])


def issue_url(repo: str, number: int) -> str:
    result = run_gh(
        [
            "issue",
            "view",
            str(number),
            "--repo",
            repo,
            "--json",
            "url",
        ]
    )
    if result.returncode != 0:
        return f"https://github.com/{repo}/issues/{number}"
    return str(json.loads(result.stdout)["url"])


def create_issue(repo: str, label: str, title: str, body_file: Path) -> str | None:
    result = run_gh(
        [
            "issue",
            "create",
            "--repo",
            repo,
            "--title",
            title,
            "--label",
            label,
            "--body-file",
            str(body_file),
        ]
    )
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        warn(f"could not create compatibility report issue: {message}")
        return None
    return result.stdout.strip()


def update_issue(repo: str, number: int, body_file: Path) -> str | None:
    result = run_gh(
        [
            "issue",
            "edit",
            str(number),
            "--repo",
            repo,
            "--body-file",
            str(body_file),
        ]
    )
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        warn(f"could not update compatibility report issue #{number}: {message}")
        return None
    return issue_url(repo, number)


def write_output(name: str, value: str) -> None:
    github_output = os.environ.get("GITHUB_OUTPUT")
    if not github_output:
        return
    with Path(github_output).open("a", encoding="utf-8") as output:
        print(f"{name}={value}", file=output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="GitHub repository, owner/name")
    parser.add_argument("--label", required=True, help="Dedicated tracking label")
    parser.add_argument("--title", required=True, help="Issue title to create")
    parser.add_argument(
        "--body-file",
        required=True,
        type=Path,
        help="Markdown report body",
    )
    args = parser.parse_args()

    if not args.body_file.is_file():
        raise SystemExit(f"report body does not exist: {args.body_file}")

    if not ensure_label(args.repo, args.label):
        return 0

    number = find_issue(args.repo, args.label)
    if number is None:
        url = create_issue(args.repo, args.label, args.title, args.body_file)
    else:
        url = update_issue(args.repo, number, args.body_file)

    if url:
        print(f"compatibility report issue: {url}")
        write_output("issue_url", url)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
