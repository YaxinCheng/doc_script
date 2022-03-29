# DocScript

A domain specific language (DSL) to generate your document

This project is in progress and missing many features. You can see the rough roadmap at the end of this README file. Also, if you are interested, you can check on the issues to keep track of the progress.

## Purpose

The purpose of this DSL is to enable quick documentation in the style of programming. 

This language will provide multiple predefined layout elements that can be used like classes. Using these elements, documentation layouts can be easily created, modified, and removed.

This language will also provide straightforward abstractions for content data, such as text, images, and fonts. With that, users can produce a documentations with multimedia with no extra efforts. 

## Language Specification

As a DSL focusing on generating documents, the language is designed to be simple but powerful.

DocScript supports a folder based module system and highly abstract-able trait system. Its type system is simple but strict to guarantee the correctness of content. Users can define constant, struct, and trait based on their needs, and they will all be compiled statically.

A full language specification documentation will be provided ...

## Getting Started

This section talks about how to compile and execute the compiler `doc` .

### Prerequisites

* Rust 1.58 or above
  * Can be installed from [rust-lang.org](https://www.rust-lang.org)
* Cargo
* Git

### Compile

```bash
cd SOME_DIR
git clone https://github.com/YaxinCheng/doc_script.git
cd doc_script
cargo build --release
```

Once compiled, you can execute it from

```bash
target/release/doc
```

## Roadmap

- [ ] std library
- [ ] arithmetic operations
- [ ] intermedia representation
- [ ] interpretation to HTML
- [ ] interpretation to PDF

## License

This project is licensed under the GPLv3 Licence - see the [LICENSE.md](https://github.com/YaxinCheng/doc_script/blob/master/LICENSE)