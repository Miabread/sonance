# Overview

```sonance
import {
    std { compare.Ordering, io.stdin },
    random { Random, thread_rng },
};

func read_number(buffer: $String) -> Result<U32, ParseError> {
    buffer.clear;
    stdin().read_line($buffer).expect;
    buffer.trim.parse
}

func main() {
    let mut buffer = String.new;

    print("Please enter a number: ");

    let number = while_failing {
        read_number($buffer)
    } {
        print("Input must be a number.");
    };

    let correct = 0.to(number).random(thread_rng());
    print("Guess a number 0 to \(number): ");

    loop continue!{
        let guess = read_number($buffer).catch {
            print("Input must be a number.");
            continue!();
        };

        correct.compare(&guess).match {
            Ordering.Greater -> print("Higher..."),
            Ordering.Less -> print("Lower..."),
            Ordering.Equal -> {
                print("Correct!");
                return!();
            },
        };
    };
};
```
