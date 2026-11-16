use std::fs;
use std::path::Path;

#[test]
fn root_readme_has_valid_site_and_github_markers() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root_readme = book_dir.parent().unwrap().join("README.md");
    let content = fs::read_to_string(&root_readme).expect("missing root README.md");

    let mut mode = "";
    let mut releases_markers = 0;

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        let line_no = idx + 1;
        match line {
            "<!-- site-only" => {
                assert!(
                    mode.is_empty(),
                    "README.md:{line_no}: nested site-only marker"
                );
                mode = "site";
            }
            "-->" => {
                assert_eq!(mode, "site", "README.md:{line_no}: unmatched --> marker");
                mode = "";
            }
            "<!-- github-only -->" => {
                assert!(
                    mode.is_empty(),
                    "README.md:{line_no}: nested github-only marker"
                );
                mode = "github";
            }
            "<!-- end github-only -->" => {
                assert_eq!(
                    mode, "github",
                    "README.md:{line_no}: unmatched end github-only marker"
                );
                mode = "";
            }
            "<!-- wireup:releases -->" => {
                assert!(
                    mode.is_empty(),
                    "README.md:{line_no}: wireup:releases marker must be outside conditional blocks"
                );
                releases_markers += 1;
            }
            _ => {}
        }
    }

    assert!(mode.is_empty(), "README.md has an unclosed '{mode}' block");
    assert_eq!(
        releases_markers, 1,
        "README.md must contain exactly one <!-- wireup:releases --> marker"
    );
}
