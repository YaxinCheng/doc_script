use doc_script::compile;

fn main() {
    let args = std::env::args();

    compile(args.skip(1).collect());
}
