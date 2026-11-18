"""Tests of crate_readme.py, which the gate's `test` mode runs."""

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

from crate_readme import FILE, HEADER, RELEASES, check, from_root, write
from crate_readme import RELEASES_MARKER as MARKER


class FromRoot(unittest.TestCase):
    def test_the_crate_readme_sees_what_the_site_sees(self) -> None:
        cases = [
            (
                "every marker",
                "# Title\n\n<!-- github-only -->\nFor contributors.\n<!-- end github-only -->\n"
                f"<!-- site-only\nFor users.\n-->\n{MARKER}\nShared.\n",
                f"{HEADER}# Title\n\nFor users.\n{RELEASES}\nShared.\n",
            ),
            ("markers with space around them", f"  <!-- site-only \nShown.\n\t-->\n {MARKER}\n", f"{HEADER}Shown.\n{RELEASES}\n"),
            ("a byte order mark before a marker", f"\ufeff<!-- site-only\nShown.\n-->\n{MARKER}\n", f"{HEADER}Shown.\n{RELEASES}\n"),
            ("a form feed, which the site keeps on its line", f"a\fb\n{MARKER}\n", f"{HEADER}a\fb\n{RELEASES}\n"),
            (
                "a comment inside a site-only block, which the site shows",
                f"<!-- site-only\nSee <!-- a note --> here.\n-->\n{MARKER}\n",
                f"{HEADER}See <!-- a note --> here.\n{RELEASES}\n",
            ),
            (
                "a marker after a space JavaScript does not trim, which the site keeps as text",
                f"{MARKER}\n\x85<!-- github-only -->\n",
                f"{HEADER}{RELEASES}\n\x85<!-- github-only -->\n",
            ),
            ("a line separator, which the site keeps on its line", f"a\u2028b\n{MARKER}", f"{HEADER}a\u2028b\n{RELEASES}"),
            ("the releases marker inside a site-only block", f"<!-- site-only\n{MARKER}\n-->\n", f"{HEADER}{RELEASES}\n"),
        ]
        for case, readme, crate in cases:
            with self.subTest(case):
                self.assertEqual(from_root(readme), (crate, []))

    def test_every_marker_out_of_place_is_named_once(self) -> None:
        cases = [
            ("an empty README", "", ["README.md has no releases marker, <!-- snapfit:releases -->, which the site needs."]),
            (
                "a releases marker the site never sees",
                f"<!-- github-only -->\n{MARKER}\n<!-- end github-only -->\n",
                ["README.md has no releases marker, <!-- snapfit:releases -->, which the site needs."],
            ),
            ("a second releases marker", f"{MARKER}\n{MARKER}\n", ["README.md holds a second releases marker, on line 2."]),
            (
                "closers with no block",
                f"{MARKER}\n<!-- end github-only -->\n-->\n",
                [
                    "README.md closes a github-only block on line 2, where none is open.",
                    "README.md closes a site-only block on line 3, where none is open.",
                ],
            ),
            (
                "a block inside another, closed",
                f"{MARKER}\n<!-- site-only\n<!-- github-only -->\n<!-- end github-only -->\n-->\n",
                ["README.md opens a github-only block on line 3, inside the site-only block opened on line 2."],
            ),
            (
                "blocks never closed",
                f"{MARKER}\n<!-- github-only -->\n<!-- site-only\n",
                [
                    "README.md opens a site-only block on line 3, inside the github-only block opened on line 2.",
                    "README.md never closes the github-only block it opens on line 2.",
                    "README.md never closes the site-only block it opens on line 3.",
                ],
            ),
        ]
        for case, readme, problems in cases:
            with self.subTest(case):
                self.assertEqual(from_root(readme)[1], problems)


class OnDisk(unittest.TestCase):
    """Tests of the files `write` and `check` read and write, with what they print kept out of the log."""

    def setUp(self) -> None:
        self.enterContext(contextlib.redirect_stdout(io.StringIO()))
        self.folder = tempfile.TemporaryDirectory()
        self.root = Path(self.folder.name)
        (self.root / FILE).parent.mkdir()
        (self.root / "README.md").write_bytes(f"Shared.\r\n{MARKER}\r\n".encode())

    def tearDown(self) -> None:
        self.folder.cleanup()

    def test_a_written_crate_readme_passes_the_check(self) -> None:
        self.assertTrue(write(self.root))
        self.assertEqual((self.root / FILE).read_bytes(), f"{HEADER}Shared.\n{RELEASES}\n".encode())
        self.assertTrue(check(self.root))

    def test_a_missing_or_stale_crate_readme_fails_the_check(self) -> None:
        self.assertFalse(check(self.root))
        (self.root / FILE).write_text("old", encoding="utf-8")
        self.assertFalse(check(self.root))

    def test_a_marker_out_of_place_writes_nothing(self) -> None:
        (self.root / "README.md").write_text("Shared.\n", encoding="utf-8")
        self.assertFalse(write(self.root))
        self.assertFalse((self.root / FILE).exists())


if __name__ == "__main__":
    unittest.main()
