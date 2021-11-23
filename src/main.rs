use doc_script::compile;

fn main() {
    let args = std::env::args();
    let compiled = compile(args.skip(1).collect());
    println!("{}", compiled)
}
