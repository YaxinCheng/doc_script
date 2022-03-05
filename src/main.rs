use doc_script::compile;

fn main() {
    let args = std::env::args();
    let arguments = args.skip(1).collect();
    compile(arguments);
}
