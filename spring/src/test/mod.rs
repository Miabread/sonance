use crate::{interpret::Value, run};

#[test]
fn match_macro() {
    let output = run(r#"
        func main() {
            output!(match(1) {
                1 -> "foo",
                2 -> "bar",
                _ -> "baz",
            });
        }
    "#);

    assert_eq!(output, Ok(vec![Value::String("foo")]));
}

#[test]
fn postfix_match_macro() {
    let output = run(r#"
        func main() {
            2
            .match {
                1 -> "foo",
                2 -> "bar",
                _ -> "baz",
            }
            .output!;
        }
    "#);

    assert_eq!(output, Ok(vec![Value::String("bar")]));
}

#[test]
fn two_functions() {
    let output = run(r#"
        func main() { output!(1) }
        func foobar() { output!(2) }
    "#);

    assert_eq!(output, Ok(vec![Value::Int(1)]));
}
