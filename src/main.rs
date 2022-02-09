pub mod parser;

fn main() {
    let src = r#"
    (print "Hello, World!")
    (print '(hello, world!))
    "#;

    let parsed = parser::parse(src);
    for result in parsed {
        println!("{:?}", result);
    }
}
