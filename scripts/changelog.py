"""
Summarizes PRs.

Requires the `gh` CLI.
"""
from __future__ import annotations

import argparse
import multiprocessing
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass
from typing import Any
import json

from git import Repo  # pip install GitPython
from tqdm import tqdm

OWNER = "abey79"
REPO = "vsvg"
OFFICIAL_DEVS = [
    "abey79",
]


def eprint(*args, **kwargs) -> None:  # type: ignore
    print(*args, file=sys.stderr, **kwargs)  # type: ignore


@dataclass
class PrInfo:
    gh_user_name: str
    pr_title: str
    labels: list[str]


@dataclass
class CommitInfo:
    hexsha: str
    title: str
    pr_number: int | None


# Slow
def fetch_pr_info_from_commit_info(commit_info: CommitInfo) -> PrInfo | None:
    if commit_info.pr_number is None:
        return None
    else:
        return fetch_pr_info(commit_info.pr_number)


def run_command(command: list[str]) -> str:
    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    output, _ = process.communicate()
    return_code = process.returncode

    if return_code != 0:
        raise Exception(
            f"Command `{' '.join(command)}` failed with return code {return_code}"
        )

    return output


# Slow
def fetch_pr_info(pr_number: int) -> PrInfo | None:
    output = run_command(
        [
            "gh",
            "pr",
            "view",
            "--repo",
            f"{OWNER}/{REPO}",
            f"{pr_number}",
            "--json",
            "title,author,labels",
        ]
    )

    data = json.loads(output)

    return PrInfo(
        gh_user_name=data["author"]["login"],
        pr_title=data["title"],
        labels=[label["name"] for label in data["labels"]],
    )


def get_commit_info(commit: Any) -> CommitInfo:
    match = re.match(r"(.*) \(#(\d+)\)", commit.summary)
    if match:
        return CommitInfo(
            hexsha=commit.hexsha,
            title=str(match.group(1)),
            pr_number=int(match.group(2)),
        )
    else:
        return CommitInfo(hexsha=commit.hexsha, title=commit.summary, pr_number=None)


def print_section(title: str, items: list[str]) -> None:
    if len(items) > 0:
        print(f"## {title}")
        print()
        for line in items:
            print(f"- {line}")
        print()


def change_in_changlog(commit_info: CommitInfo, previous_changelog: str) -> bool:
    hexsha = commit_info.hexsha
    pr_number = commit_info.pr_number

    if pr_number is None:
        return f"[{hexsha}]" in previous_changelog
    else:
        return f"[#{pr_number}]" in previous_changelog


def format_change(commit_info: CommitInfo, pr_info: PrInfo | None) -> str:
    hexsha = commit_info.hexsha
    title = commit_info.title
    pr_number = commit_info.pr_number

    if pr_number is None:
        summary = (
            f"{title} [{hexsha}](https://github.com/{OWNER}/{REPO}/commit/{hexsha})"
        )
    else:
        if pr_info is None:
            eprint(f"Warning: PR #{pr_number} not found")

        title = pr_info.pr_title if pr_info else title
        title = title.rstrip(".").strip()  # Some PR end with an unnecessary period

        summary = f"{title} [#{pr_number}](https://github.com/{OWNER}/{REPO}/pull/{pr_number})"

        if pr_info is not None:
            gh_user_name = pr_info.gh_user_name
            if gh_user_name not in OFFICIAL_DEVS:
                summary += (
                    f" (thanks [@{gh_user_name}](https://github.com/{gh_user_name})!)"
                )

    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate a changelog.")
    parser.add_argument("--commit-range", help="e.g. 0.11.0..HEAD")
    args = parser.parse_args()

    # Because how we branch, we sometimes get duplicate commits in the changelog unless we check for it
    previous_changelog = pathlib.Path("CHANGELOG.md").read_text()

    repo = Repo(".")
    commits = list(repo.iter_commits(args.commit_range))
    commits.reverse()  # Most recent last
    commit_infos = list(map(get_commit_info, commits))

    with multiprocessing.Pool() as pool:
        pr_infos = list(
            tqdm(
                pool.imap(fetch_pr_info_from_commit_info, commit_infos),
                total=len(commit_infos),
                desc="Fetch PR info commits",
            )
        )

    categories = {
        "whiskers": "`whiskers`",
        "msvg": "`msvg`",
        "vsvg-cli": "`vsvg`",
        "vsvg": "vsvg",
        "vsvg-viewer": "vsvg",
        "common": "Common",
        "web-demo": "Web Demos",
        "release": "Release",
    }
    by_category = {}

    for commit_info, pr_info in zip(commit_infos, pr_infos):
        if pr_info is not None:
            if "exclude-from-changelog" in pr_info.labels:
                continue

            if "release" in pr_info.labels:
                continue

        if change_in_changlog(commit_info, previous_changelog):
            eprint(f"Ignoring dup: {commit_info}")
            continue

        change = format_change(commit_info, pr_info)
        labels = pr_info.labels if pr_info else []

        has_category = False
        for category in categories.keys():
            if category in labels:
                by_category.setdefault(category, []).append(change)
                has_category = True
                break

        if not has_category:
            by_category.setdefault("unsorted", []).append(change)

    for category, title in categories.items():
        print_section(title, by_category.get(category, []))

    print_section("!!! UNSORTED !!!", by_category.get("unsorted", []))

    print(
        f"**Full Changelog**: https://github.com/{OWNER}/{REPO}/compare/{'...'.join(args.commit_range.split('..'))}"
    )


if __name__ == "__main__":
    main()
