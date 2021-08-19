use doc_script::tokenize;

fn main() {
    for token in tokenize("test true 0.3 3 0xAB 0b10") {
        println!("{:?}", token);
    }
}
