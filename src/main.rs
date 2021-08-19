use doc_script::compile;

fn main() {
    let text = "const i = 3\n";
    let tokens = compile(text);
}
