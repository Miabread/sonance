use crate::run;

#[test]
fn blank() {
    let input = "
    func main() {}";
    assert_eq!(run(input), 0);
}

#[test]
fn return_code() {
    let input = "func main() -> USize { 123 }";
    assert_eq!(run(input), 123);
}

#[test]
fn spacing() {
    let input = "
    func main() -> USize {
        456
    }";
    assert_eq!(run(input), 456);
}

#[test]
fn multi_number() {
    let input = "
    func main() -> USize {
        123;
        456;
        789
    }";
    assert_eq!(run(input), 789);
}
