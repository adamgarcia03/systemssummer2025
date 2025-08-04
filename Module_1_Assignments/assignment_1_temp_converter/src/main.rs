const FREEZING_POINT_F: f64 = 32.0;

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    (c * 9.0 / 5.0) + 32.0
}

fn main() {
    let mut fahrenheit = FREEZING_POINT_F;
    let celsius = fahrenheit_to_celsius(fahrenheit);
    println!("{fahrenheit}°F is {celsius:.2}°C");

    for _ in 1..=5 {
        fahrenheit += 1.0;
        let celsius = fahrenheit_to_celsius(fahrenheit);
        println!("{fahrenheit}°F is {celsius:.2}°C");
    }

    let c = 0.0;
    let f = celsius_to_fahrenheit(c);
    println!("{c}°C is {f:.2}°F");
}
