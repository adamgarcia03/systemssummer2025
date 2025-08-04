fn is_even(n: i32) -> bool {
    n % 2 == 0
}

fn main() {
    let numbers = [5, 3, 15, 22, 10, 9, 8, 30, 7, 13];

    for &n in &numbers {
        if n % 3 == 0 && n % 5 == 0 {
            println!("{n} -> FizzBuzz");
        } else if n % 3 == 0 {
            println!("{n} -> Fizz");
        } else if n % 5 == 0 {
            println!("{n} -> Buzz");
        } else {
            if is_even(n) {
                println!("{n} -> Even");
            } else {
                println!("{n} -> Odd");
            }
        }
    }

    let mut sum = 0;
    let mut i = 0;
    while i < numbers.len() {
        sum += numbers[i];
        i += 1;
    }
    println!("Sum of all numbers: {sum}");

    let mut max = numbers[0];
    for &n in &numbers {
        if n > max {
            max = n;
        }
    }
    println!("Largest number: {max}");
}
