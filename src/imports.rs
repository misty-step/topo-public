//! Native, non-executing syntax collection. Literal requests are retained;
//! filesystem/package resolution and evidence belong to the collector.
use crate::model::Result;
use tree_sitter::{Language, Node, Parser};

pub struct Parsed {
    pub path: String,
    pub imports: Vec<Import>,
    pub error: Option<String>,
}
pub struct Import {
    pub specifier: String,
    pub line: usize,
}

fn literal(node: Node<'_>, text: &str, go: bool) -> Option<String> {
    let raw = node.utf8_text(text.as_bytes()).ok()?;
    if go && node.kind() == "raw_string_literal" {
        return Some(raw.strip_prefix('`')?.strip_suffix('`')?.replace('\r', ""));
    }
    if node.kind() == "template_string"
        && node
            .named_children(&mut node.walk())
            .any(|n| n.kind() == "template_substitution")
    {
        return None;
    }
    if !["string", "interpreted_string_literal", "template_string"].contains(&node.kind()) {
        return None;
    }
    let quote = raw.chars().next()?;
    if !['\'', '"', '`'].contains(&quote) {
        return None;
    }
    let inner = raw.strip_prefix(quote)?.strip_suffix(quote)?;
    let mut chars = inner.chars().peekable();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let escape = chars.next()?;
        let decoded = match escape {
            '\n' => continue,
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                continue;
            }
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'v' => '\u{b}',
            '\\' | '\'' | '"' | '`' | '/' => escape,
            'x' | 'u' | 'U' => {
                let length = if escape == 'x' {
                    2
                } else if escape == 'u' {
                    4
                } else {
                    8
                };
                let mut digits = String::new();
                for _ in 0..length {
                    digits.push(chars.next()?);
                }
                let value = u32::from_str_radix(&digits, 16).ok()?;
                if (0xd800..=0xdbff).contains(&value) {
                    if chars.next() != Some('\\') || chars.next() != Some('u') {
                        return None;
                    }
                    let mut low = String::new();
                    for _ in 0..4 {
                        low.push(chars.next()?);
                    }
                    let low = u32::from_str_radix(&low, 16).ok()?;
                    if !(0xdc00..=0xdfff).contains(&low) {
                        return None;
                    }
                    char::from_u32(0x10000 + ((value - 0xd800) << 10) + (low - 0xdc00))?
                } else {
                    char::from_u32(value)?
                }
            }
            '0'..='7' if go => {
                let mut digits = String::from(escape);
                for _ in 0..2 {
                    digits.push(chars.next()?);
                }
                char::from_u32(u32::from_str_radix(&digits, 8).ok()?)?
            }
            '0' => '\0',
            _ if !go => escape,
            _ => return None,
        };
        out.push(decoded);
    }
    Some(out)
}
pub fn parse(path: &str, text: &str) -> Result<Parsed> {
    let go = path.ends_with(".go");
    let language: Language = if go {
        tree_sitter_go::LANGUAGE.into()
    } else if path.ends_with(".tsx") {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    } else if path.ends_with(".ts") {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    } else {
        tree_sitter_javascript::LANGUAGE.into()
    };
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    let tree = parser
        .parse(text, None)
        .ok_or("Syntax parser did not produce a tree")?;
    let mut result = Parsed {
        path: path.into(),
        imports: vec![],
        error: tree
            .root_node()
            .has_error()
            .then(|| "Syntax diagnostics; relationships may be incomplete".into()),
    };
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        let source = match node.kind() {
            "import_spec" if go => node.child_by_field_name("path"),
            "import_statement" | "export_statement" if !go => node.child_by_field_name("source"),
            "call_expression" if !go => {
                let function = node.child_by_field_name("function");
                let is_request = function.is_some_and(|f| {
                    f.kind() == "import"
                        || f.kind() == "identifier"
                            && f.utf8_text(text.as_bytes()).ok() == Some("require")
                });
                if is_request {
                    node.child_by_field_name("arguments")
                        .and_then(|a| a.named_child(0))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(source) = source {
            if let Some(specifier) = literal(source, text, go) {
                result.imports.push(Import {
                    specifier,
                    line: source.start_position().row + 1,
                });
            }
        }
        stack.extend(node.named_children(&mut node.walk()));
    }
    result
        .imports
        .sort_by(|a, b| (a.line, &a.specifier).cmp(&(b.line, &b.specifier)));
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn syntax_not_text_drives_requests() {
        let ts = parse(
            "a.ts",
            r#"// import 'comment';
const text="require('string')";
import type {Thing} from './types.js';
export {x} from './export';
require('node:fs');
import(`./literal`);
import(`./${dynamic}`);
import 'bun:test';
import '\u007aod';
"#,
        )
        .unwrap();
        assert!(ts.error.is_none());
        assert_eq!(
            ts.imports
                .iter()
                .map(|i| i.specifier.as_str())
                .collect::<Vec<_>>(),
            vec![
                "./types.js",
                "./export",
                "node:fs",
                "./literal",
                "bun:test",
                "zod"
            ]
        );
        let go = parse(
            "a.go",
            "package main\nimport (alias \"database/sql\"; _ `encoding/json`)\n",
        )
        .unwrap();
        assert!(go.error.is_none());
        assert_eq!(
            go.imports
                .iter()
                .map(|i| i.specifier.as_str())
                .collect::<Vec<_>>(),
            vec!["database/sql", "encoding/json"]
        );
    }
}
