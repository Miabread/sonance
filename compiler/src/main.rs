use compiler::tokenize;

fn main() {
    let input = include_str!("../../scratch/scratch.son");
    println!("{:?}", tokenize(input));
}
