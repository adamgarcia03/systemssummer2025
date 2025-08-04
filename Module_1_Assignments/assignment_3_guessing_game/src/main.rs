fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {
    let secret = 7;
    let test_guesses = [3, 6, 8, 7];
    let mut attempts = 0;

    for guess in test_guesses {
        attempts += 1;
        let result = check_guess(guess, secret);

        if result == 0 {
            println!("Guess {guess} is correct!");
            break;
        } else if result == 1 {
            println!("Guess {guess} is too high.");
        } else {
            println!("Guess {guess} is too low.");
        }
    }

    println!("Number of attempts: {attempts}");
}
