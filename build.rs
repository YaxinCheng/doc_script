use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use walkdir::WalkDir;

const LR1_OUTPUT: &str = "grammar.lr1";
const NODE_KIND_FILE_NAME: &str = "node_kind.rs";
const NODE_KIND_HEADER: &[u8] =
    b"#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum NodeKind {\n";

const RULE_FILE_NAME: &str = "rules.rs";
const RULE_FILE_HEADER: &[u8] = b"pub const RULES: [Production; N] = [\n";

const SYMBOL_MAP_FILE_NAME: &str = "symbols.rs";
const SYMBOL_MAP_FILE_HEADER: &[u8] = b"pub fn symbol_to_ord(symbol: &Symbol) -> usize {
match symbol {
";

const ACTION_TABLE_FILE_NAME: &str = "action_table.rs";

const CONVERTER_PATH: &str = "jlalr/Jlalr1.java";
const LR1_CONVERTER: &str = "jlalr.Jlr1";
const PROJECT_PATH: &str = env!("CARGO_MANIFEST_DIR");
const GRAMMAR_RULE_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/grammar.cfg");

const STDLIB_FILE_NAME: &str = "stdlib.rs";
const STD_CONTENT_HEADER: &[u8] = b"pub(crate) const CONTENT: [&str; N] = [\n";
const STD_PATH_HEADER: &[u8] = b"pub(crate) const PATHS: [&str; N] = [\n";

fn main() -> Result<()> {
    let output_dir: PathBuf = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .expect("OUT_DIR not set");
    compile_converter()?;
    generate_lr1_table(&output_dir)?;
    process_lr1_grammar(&output_dir)?;
    generate_stdlib_bridge(&output_dir)?;
    println!("cargo:rerun-if-changed={GRAMMAR_RULE_FILE}");
    println!(concat!(
        "cargo:rerun-if-changed=",
        env!("CARGO_MANIFEST_DIR"),
        "/std"
    ));
    Ok(())
}

fn compile_converter() -> Result<()> {
    Command::new("javac")
        .current_dir(PROJECT_PATH)
        .arg(CONVERTER_PATH)
        .spawn()?
        .wait()
        .map(|_| ())
}

fn generate_lr1_table(output_dir: &Path) -> Result<()> {
    let output_file_path = output_dir.join(LR1_OUTPUT);
    let mut child = Command::new("java")
        .current_dir(PROJECT_PATH)
        .arg(LR1_CONVERTER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    write_grammar_files(child.stdin.take().expect("Failed to capture stdin"))?;
    let output = child.wait_with_output()?.stdout;
    let mut writer = File::create(output_file_path)?;
    writer.write_all(&output)
}

fn write_grammar_files(target: impl Write) -> Result<()> {
    fn write<I: IntoIterator<Item = String>>(
        output: &mut BufWriter<impl Write>,
        line_count: Option<usize>,
        content: I,
    ) -> Result<()> {
        if let Some(line_count) = line_count {
            writeln!(output, "{line_count}")?;
        }
        content
            .into_iter()
            .try_for_each(|line| writeln!(output, "{}", line.trim()))
    }

    let mut writer = BufWriter::new(target);
    let GrammarFile {
        root,
        terminals,
        non_terminals,
        rules,
    } = parse_grammar_file()?;
    write(&mut writer, Some(terminals.len()), terminals)?;
    write(&mut writer, Some(non_terminals.len()), non_terminals)?;
    write(&mut writer, None, std::iter::once(root))?;
    write(&mut writer, Some(rules.len()), rules)
}

struct GrammarFile {
    root: String,
    terminals: Vec<String>,
    non_terminals: HashSet<String>,
    rules: Vec<String>,
}

fn parse_grammar_file() -> Result<GrammarFile> {
    let reader = BufReader::new(File::open(GRAMMAR_RULE_FILE)?);
    let mut non_terminals = HashSet::new();
    let mut terminals = HashSet::new();
    let mut root: Option<String> = None;
    let mut rules = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        for (index, token) in line.trim().split(' ').enumerate() {
            if index == 0 && !non_terminals.contains(token) {
                non_terminals.insert(String::from(token));
                if root.is_none() {
                    root.replace(String::from(token));
                }
            } else if index > 0 && !non_terminals.contains(token) {
                terminals.insert(String::from(token));
            }
        }
        rules.push(line);
    }
    let terminals = terminals
        .into_iter()
        .filter(|token| !non_terminals.contains(token))
        .collect();
    Ok(GrammarFile {
        root: root.expect("Root not found"),
        terminals,
        non_terminals,
        rules,
    })
}

macro_rules! unexpected_eof {
    ($message: expr) => {
        || Error::new(ErrorKind::UnexpectedEof, $message)
    };
}

struct LineReader<T: Iterator> {
    lines: T,
    current_line: usize,
}

impl<T: Iterator<Item = Result<String>>> LineReader<T> {
    pub fn new(lines: T) -> Self {
        LineReader {
            lines,
            current_line: 1,
        }
    }

    /// # WARNING:
    /// Always consume whatever you take. Or the current_line counting will be messed up
    pub fn take(&mut self, n: usize) -> impl Iterator<Item = Result<String>> + '_ {
        let taken = self.lines.by_ref().take(n);
        self.current_line += n;
        taken
    }

    pub fn next<V: FromStr>(&mut self) -> Result<V> {
        let parsed = self
            .lines
            .next()
            .ok_or_else(unexpected_eof!(format!("Line {}", self.current_line)))??
            .parse::<V>()
            .map_err(|_| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Line {}", self.current_line),
                )
            })?;
        self.current_line += 1;
        Ok(parsed)
    }
}

fn process_lr1_grammar(output_dir: &Path) -> Result<()> {
    let grammar_file = BufReader::new(File::open(output_dir.join(LR1_OUTPUT))?);
    let mut lines = LineReader::new(grammar_file.lines());
    let num_of_terminals = lines.next()?;
    let terminals = lines.take(num_of_terminals).collect::<Result<Vec<_>>>()?;
    let num_of_non_terminals = lines.next()?;
    let non_terminals = lines
        .take(num_of_non_terminals)
        .collect::<Result<Vec<_>>>()?;
    generate_node_kind(non_terminals.iter(), output_dir)?;
    let _: String = lines.next()?; // start symbol
    let num_of_rules = lines.next()?;
    generate_symbol_map(&terminals, &non_terminals, output_dir)?;
    let symbol_map = terminals
        .into_iter()
        .chain(non_terminals.into_iter())
        .enumerate()
        .map(|(index, symbol)| (symbol, index))
        .collect::<HashMap<_, _>>();
    generate_rules(lines.take(num_of_rules), num_of_rules, output_dir)?;
    let num_of_states = lines.next()?;
    let num_of_actions = lines.next()?;
    generate_actions(
        lines.take(num_of_actions),
        symbol_map,
        num_of_states,
        output_dir,
    )
}

fn generate_node_kind<'a>(
    mut non_terminals: impl Iterator<Item = &'a String>,
    output_dir: &Path,
) -> Result<()> {
    let mut writer = BufWriter::new(File::create(output_dir.join(NODE_KIND_FILE_NAME))?);
    writer.write_all(NODE_KIND_HEADER)?;
    non_terminals.try_for_each(|line| writeln!(writer, "{},", line))?;
    writeln!(writer, "}}")
}

fn generate_rules(
    mut rules: impl Iterator<Item = Result<String>>,
    size: usize,
    output_dir: &Path,
) -> Result<()> {
    let mut writer = BufWriter::new(File::create(output_dir.join(RULE_FILE_NAME))?);
    writeln!(writer, "const N: usize = {size};")?;
    writer.write_all(RULE_FILE_HEADER)?;
    rules.try_for_each(|line| -> Result<()> {
        let line = line?;
        let mut elements = line.split(' ');
        let lhs = elements
            .next()
            .ok_or_else(unexpected_eof!("No left hand side"))?;
        write_production_rules(&mut writer, lhs, elements)
    })?;
    writeln!(writer, "];")
}

fn write_production_rules<'a>(
    writer: &mut impl Write,
    lhs: &str,
    mut rhs: impl Iterator<Item = &'a str>,
) -> Result<()> {
    write!(writer, "Production {{ lhs: NodeKind::{lhs}, rhs: &[")?;

    rhs.try_for_each(|symbol| {
        if let Some(token_kind) = terminals_to_token_kind(symbol) {
            write!(
                writer,
                r#"Symbol::Terminal(Token {{ kind: {token_kind}, lexeme: "{symbol}" }}), "#,
            )
        } else {
            write!(writer, "Symbol::NonTerminal(NodeKind::{symbol}), ")
        }
    })?;
    writeln!(writer, "] }},")
}

fn terminals_to_token_kind(s: &str) -> Option<&str> {
    let kind = match s {
        "Identifier" | "ParsingStart" | "ParsingEnd" | "NewLine" => s,
        "IntegerLiteral" => "Literal(LiteralKind::Integer)",
        "FloatingLiteral" => "Literal(LiteralKind::Floating)",
        "BooleanLiteral" => "Literal(LiteralKind::Boolean)",
        "StringLiteral" => "Literal(LiteralKind::String)",
        "BinaryLiteral" => "Literal(LiteralKind::Binary)",
        "HexLiteral" => "Literal(LiteralKind::Hex)",
        "break" | "default" | "const" | "continue" | "else" | "for" | "fn" | "if" | "impl"
        | "return" | "super" | "struct" | "self" | "use" | "trait" => "Keyword",
        "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | ":" => "Separator",
        "=" | "==" | ">" | ">=" | ">>" | ">>=" | "<" | "<=" | "<<" | "<<=" | "!" | "!=" | "~"
        | "+" | "+=" | "-" | "-=" | "*" | "*=" | "/" | "/=" | "&" | "&&" | "&=" | "|" | "||"
        | "|=" | "^" | "^=" | "%" | "%=" | "->" | "=>" => "Operator",
        _ => None?,
    };
    Some(kind)
}

fn generate_symbol_map(
    terminals: &[String],
    non_terminals: &[String],
    output_dir: &Path,
) -> Result<()> {
    let mut writer = BufWriter::new(File::create(output_dir.join(SYMBOL_MAP_FILE_NAME))?);
    writer.write_all(SYMBOL_MAP_FILE_HEADER)?;
    for (index, symbol) in terminals.iter().chain(non_terminals.iter()).enumerate() {
        if let Some(token_kind) = terminals_to_token_kind(symbol) {
            if should_ignore_lexeme(symbol) {
                writeln!(
                    writer,
                    "Symbol::Terminal(Token {{ kind: TokenKind::{token_kind}, lexeme: _ }}) => {index},",
                )?
            } else {
                writeln!(
                    writer,
                    r#"Symbol::Terminal(Token {{ kind: TokenKind::{token_kind}, lexeme: "{symbol}" }}) => {index},"#,
                )?
            }
        } else {
            writeln!(
                writer,
                "Symbol::NonTerminal(NodeKind::{symbol}) => {index},",
            )?
        }
    }
    writeln!(
        writer,
        r#"sym => panic!("Unexpected symbol: {{sym:?}}")
}}}}"#
    )
}

enum ActionKind {
    Reduce,
    Transition,
}
impl From<&str> for ActionKind {
    fn from(kind: &str) -> Self {
        match kind {
            "reduce" => ActionKind::Reduce,
            "shift" => ActionKind::Transition,
            unknown => panic!("Invalid action kind: {unknown}"),
        }
    }
}

struct Action<'a> {
    pub state: usize,
    pub symbol: &'a str,
    pub kind: ActionKind,
    pub target: usize,
}

impl<'a> Action<'a> {
    fn from_str(line: &'a str) -> Result<Self> {
        let mut elements = line.split(' ');
        let parse_error = |e: std::num::ParseIntError| Error::new(ErrorKind::InvalidData, e);
        let state = elements
            .next()
            .ok_or_else(unexpected_eof!("No state"))?
            .parse::<usize>()
            .map_err(parse_error)?;
        let symbol = elements
            .next()
            .ok_or_else(unexpected_eof!("No rhs symbol"))?;
        let kind = elements
            .next()
            .ok_or_else(unexpected_eof!("No action type"))?
            .into();
        let target = elements
            .next()
            .ok_or_else(unexpected_eof!("No target state"))?
            .parse::<usize>()
            .map_err(parse_error)?;
        Ok(Action {
            state,
            symbol,
            kind,
            target,
        })
    }
}

fn generate_actions(
    mut lines: impl Iterator<Item = Result<String>>,
    symbol_map: HashMap<String, usize>,
    num_of_states: usize,
    output_dir: &Path,
) -> Result<()> {
    let mut action_writer = BufWriter::new(File::create(output_dir.join(ACTION_TABLE_FILE_NAME))?);
    // reduce_actions and transition_actions are both vectors of min-heaps
    let mut reduce_actions = vec![BinaryHeap::<Reverse<(usize, usize)>>::new(); num_of_states];
    let mut transition_actions = vec![BinaryHeap::<Reverse<(usize, usize)>>::new(); num_of_states];
    lines.try_for_each(|line| -> Result<()> {
        let line = line?;
        let action = Action::from_str(&line)?;
        let symbol_index = *symbol_map
            .get(action.symbol)
            .expect("Invalid symbol in action");

        match action.kind {
            ActionKind::Reduce => &mut reduce_actions[action.state],
            ActionKind::Transition => &mut transition_actions[action.state],
        }
        .push(Reverse((symbol_index, action.target)));
        Ok(())
    })?;
    write_lookup_table(&mut action_writer, reduce_actions, "REDUCTIONS")?;
    write_lookup_table(&mut action_writer, transition_actions, "TRANSITIONS")
}

fn write_lookup_table(
    writer: &mut impl Write,
    lookup_table: Vec<BinaryHeap<Reverse<(usize, usize)>>>,
    name: &str,
) -> Result<()> {
    writeln!(
        writer,
        "const {}: [&[(usize, usize)]; {}] = [",
        name,
        lookup_table.len()
    )?;
    for mut entry in lookup_table {
        write!(writer, "&[")?;
        while let Some(Reverse((symbol, target))) = entry.pop() {
            write!(writer, "({}, {}), ", symbol, target)?;
        }
        writeln!(writer, "],")?;
    }
    writeln!(writer, "];")
}

fn should_ignore_lexeme(symbol: &str) -> bool {
    matches!(
        symbol,
        "Identifier"
            | "IntegerLiteral"
            | "FloatingLiteral"
            | "BooleanLiteral"
            | "StringLiteral"
            | "BinaryLiteral"
            | "HexLiteral"
            | "NewLine"
            | "ParsingStart"
            | "ParsingEnd"
    )
}

fn generate_stdlib_bridge(output_dir: &Path) -> Result<()> {
    let entries = WalkDir::new("std")
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| matches!(entry.path().extension().and_then(OsStr::to_str), Some("ds")))
        .collect::<Vec<_>>();
    let mut paths = entries.iter().map(|entry| entry.path().display());

    let mut std_file_writer = BufWriter::new(File::create(output_dir.join(STDLIB_FILE_NAME))?);
    writeln!(std_file_writer, "const N: usize = {};", entries.len())?;

    std_file_writer.write_all(STD_CONTENT_HEADER)?;
    paths.clone().try_for_each(|path| {
        writeln!(std_file_writer, r#"include_str!("{PROJECT_PATH}/{path}"),"#)
    })?;
    std_file_writer.write_all(b"];\n")?;

    std_file_writer.write_all(STD_PATH_HEADER)?;
    paths.try_for_each(|path| writeln!(std_file_writer, r#""{path}","#))?;
    std_file_writer.write_all(b"];\n")
}
