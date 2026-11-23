use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::sorted_files;

struct FencedBlock {
    line_no: usize,
    info: String,
    lines: Vec<String>,
}

struct ParsedFence {
    block: FencedBlock,
    next_idx: usize,
}

#[test]
fn chapters_have_valid_directives_and_no_raw_rust_programs() {
    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let chapters_dir = book_dir.join("chapters");
    if !chapters_dir.exists() {
        return;
    }

    sorted_files(&chapters_dir, "md")
        .into_iter()
        .filter(|path| path.file_name().and_then(|s| s.to_str()) != Some("README.md"))
        .for_each(|md_file| verify_chapter_file(book_dir, &md_file));
}

fn verify_chapter_file(book_dir: &Path, md_file: &Path) {
    let file_name = md_file.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let content = fs::read_to_string(md_file).expect("failed to read chapter file");

    extract_fenced_blocks(&content)
        .iter()
        .for_each(|block| validate_fenced_block(book_dir, file_name, block));
}

fn extract_fenced_blocks(content: &str) -> Vec<FencedBlock> {
    let raw_lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    let mut idx = 0;

    while idx < raw_lines.len() {
        if let Some(info) = raw_lines[idx].trim_end().strip_prefix("```") {
            let parsed = collect_single_block(&raw_lines, idx, info.trim());
            blocks.push(parsed.block);
            idx = parsed.next_idx;
        } else {
            idx += 1;
        }
    }
    blocks
}

fn collect_single_block(raw_lines: &[&str], open_idx: usize, info: &str) -> ParsedFence {
    let body: Vec<String> = raw_lines[open_idx + 1..]
        .iter()
        .take_while(|line| !line.starts_with("```"))
        .map(|s| (*s).to_string())
        .collect();
    let next_idx = (open_idx + 1 + body.len() + 1).min(raw_lines.len());
    let block = FencedBlock {
        line_no: open_idx + 1,
        info: info.to_string(),
        lines: body,
    };
    ParsedFence { block, next_idx }
}

fn validate_fenced_block(book_dir: &Path, file_name: &str, block: &FencedBlock) {
    assert!(
        !(block.info == "rust" && block.lines.len() > 2),
        "{file_name}:{}: raw ```rust blocks are forbidden in chapters; use a directive",
        block.line_no
    );
    if !matches!(block.info.as_str(), "example" | "output" | "compile-error") {
        return;
    }

    let kv = parse_directive_kv(file_name, block.line_no, &block.lines);
    let name = kv.get("name").unwrap_or_else(|| {
        panic!(
            "{file_name}:{}: {} directive missing 'name'",
            block.line_no, block.info
        )
    });
    verify_directive_target(book_dir, file_name, block, name, kv.get("region"));
}

fn verify_directive_target(
    book_dir: &Path,
    file_name: &str,
    block: &FencedBlock,
    name: &str,
    region: Option<&String>,
) {
    let line_no = block.line_no;
    match block.info.as_str() {
        "example" => verify_example_directive(book_dir, file_name, line_no, name, region),
        "output" => assert!(
            book_dir
                .join("expected")
                .join(format!("{name}.txt"))
                .exists(),
            "{file_name}:{line_no}: missing book/expected/{name}.txt"
        ),
        "compile-error" => assert!(
            book_dir
                .join("compile-errors")
                .join(format!("{name}.rs"))
                .exists()
                && book_dir
                    .join("compile-errors")
                    .join(format!("{name}.txt"))
                    .exists(),
            "{file_name}:{line_no}: missing compile-error fixture '{name}'"
        ),
        _ => {}
    }
}

fn verify_example_directive(
    book_dir: &Path,
    file_name: &str,
    line_no: usize,
    name: &str,
    region: Option<&String>,
) {
    let rs_path = book_dir.join("examples").join(format!("{name}.rs"));
    assert!(
        rs_path.exists(),
        "{file_name}:{line_no}: missing example book/examples/{name}.rs"
    );
    if let Some(region_name) = region {
        assert!(
            has_valid_anchor(&rs_path, region_name),
            "{file_name}:{line_no}: missing or empty ANCHOR '{region_name}' in {name}.rs"
        );
    }
}

fn has_valid_anchor(rs_path: &Path, region: &str) -> bool {
    let source = fs::read_to_string(rs_path).expect("failed to read example source");
    let open_marker = format!("ANCHOR: {region}");
    let close_marker = format!("ANCHOR_END: {region}");
    let lines: Vec<&str> = source.lines().collect();
    let Some(start) = lines.iter().position(|l| l.trim().ends_with(&open_marker)) else {
        return false;
    };
    lines[start + 1..]
        .iter()
        .position(|l| l.trim().ends_with(&close_marker))
        .is_some_and(|end| end > 0)
}

fn parse_directive_kv(
    file_name: &str,
    line_no: usize,
    lines: &[String],
) -> HashMap<String, String> {
    lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_key_value_line(file_name, line_no, l))
        .collect()
}

fn parse_key_value_line(file_name: &str, line_no: usize, line: &str) -> (String, String) {
    let Some((k, v)) = line.split_once(':') else {
        panic!("{file_name}:{line_no}: invalid directive line '{line}'");
    };
    (k.trim().to_string(), v.trim().to_string())
}
