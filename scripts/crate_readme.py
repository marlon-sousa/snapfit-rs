"""The crate's README, `snapfit/README.md`, written from the root README as the site sees it.

Its rules must stay in step with site/src/lib/readme-strip.mjs, which the site reads the root README with.
"""

from enum import Enum
from pathlib import Path

FILE = Path("snapfit") / "README.md"

HEADER = (
    "<!-- Written by `cargo xtask crate-readme` from the README at the repository's root."
    " Edit that one, then run it again. -->\n\n"
)

RELEASES = (
    "Each release's edition of the book is on"
    " [the snapfit site](https://marlon-sousa.github.io/snapfit-rs/releases/)."
)

RELEASES_MARKER = "<!-- snapfit:releases -->"


class Block(Enum):
    """A block of the root README that only one of GitHub and the site shows, named by its opening marker."""

    GITHUB_ONLY = "<!-- github-only -->"
    SITE_ONLY = "<!-- site-only"

    @property
    def closer(self) -> str:
        return "<!-- end github-only -->" if self is Block.GITHUB_ONLY else "-->"

    @property
    def kind(self) -> str:
        return "github-only" if self is Block.GITHUB_ONLY else "site-only"


OPENERS = {block.value: block for block in Block}
CLOSERS = {block.closer: block for block in Block}

# What JavaScript's `trim()`, which the site trims lines with, takes as space: ECMAScript's WhiteSpace and LineTerminator.
JS_SPACE = "\t\n\v\f\r \u00a0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u2028\u2029\u202f\u205f\u3000\ufeff"


def from_root(readme: str) -> tuple[str, list[str]]:
    """The crate's README that `readme` gives, and a sentence for each marker out of place.

    Lines are split and trimmed as the site splits and trims them: on `\\n` alone,
    and of what JavaScript takes as space.
    """
    kept: list[str] = []
    problems: list[str] = []
    # The blocks open around the line read, innermost last, with the number of the line that opened each.
    open_blocks: list[tuple[Block, int]] = []
    releases = False
    for number, line in enumerate(readme.split("\n"), start=1):
        marker = line.strip(JS_SPACE)
        inner = open_blocks[-1] if open_blocks else None
        if marker in OPENERS:
            opened = OPENERS[marker]
            if inner:
                problems.append(
                    f"README.md opens a {opened.kind} block on line {number},"
                    f" inside the {inner[0].kind} block opened on line {inner[1]}."
                )
            open_blocks.append((opened, number))
        elif marker in CLOSERS:
            closed = CLOSERS[marker]
            if inner and inner[0] is closed:
                open_blocks.pop()
            else:
                problems.append(f"README.md closes a {closed.kind} block on line {number}, where none is open.")
        elif inner and inner[0] is Block.GITHUB_ONLY:
            continue
        elif marker == RELEASES_MARKER:
            if releases:
                problems.append(f"README.md holds a second releases marker, on line {number}.")
            releases = True
            kept.append(RELEASES)
        else:
            kept.append(line)
    problems += [
        f"README.md never closes the {block.kind} block it opens on line {opened}." for block, opened in open_blocks
    ]
    if not releases:
        problems.append(f"README.md has no releases marker, {RELEASES_MARKER}, which the site needs.")
    return HEADER + "\n".join(kept), problems


def wanted(root: Path) -> tuple[str, list[str]]:
    """What `snapfit/README.md` should hold, and the problems that keep it from it."""
    try:
        with open(root / "README.md", encoding="utf-8", newline="") as readme:
            return from_root(readme.read().replace("\r\n", "\n"))
    except OSError as error:
        return "", [f"README.md could not be read: {error}"]


def check(root: Path) -> bool:
    """Whether `snapfit/README.md` is what the root README gives, saying why not."""
    print("\n>>> Checking the crate's README against the root README...")
    text, problems = wanted(root)
    if not problems:
        try:
            with open(root / FILE, encoding="utf-8", newline="") as written:
                stale = written.read() != text
        except OSError as error:
            problems = [f"{FILE.as_posix()} could not be read: {error}. Run cargo xtask crate-readme."]
        else:
            if stale:
                problems = [f"{FILE.as_posix()} is not what the root README gives; run cargo xtask crate-readme."]
    return reported(problems)


def write(root: Path) -> bool:
    """Writes `snapfit/README.md` from the root README, unless a marker is out of place."""
    print(f"\n>>> Writing {FILE.as_posix()} from the root README...")
    text, problems = wanted(root)
    if not problems:
        try:
            (root / FILE).write_text(text, encoding="utf-8", newline="\n")
        except OSError as error:
            problems = [f"{FILE.as_posix()} could not be written: {error}"]
    return reported(problems)


def reported(problems: list[str]) -> bool:
    """Prints each of `problems`, and gives whether there were none."""
    for problem in problems:
        print(f"❌ {problem}")
    return not problems
