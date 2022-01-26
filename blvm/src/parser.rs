use middle::*;

pub fn tokenize(src: &str) -> Vec<Instr> {
    let mut result = Vec::new();

    for line in src.lines() {
        // <label>: <instr> <arg>+
        let mut parts = line.split_whitespace();

        let label = parts.next();
        let instr = parts.next();
        let args = parts.collect::<Vec<_>>();
        println!("{:?} {:?} {:?}", label, instr, args);
    }

    result
}