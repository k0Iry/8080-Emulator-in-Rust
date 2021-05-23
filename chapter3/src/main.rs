use std::io;

fn main() {
    convert_temperatures();
    fibonacci(99);
}

fn convert_temperatures() {
    loop {
        println!("Please input degree in Fahrenheit");
        let mut temperature = String::new();
        io::stdin()
            .read_line(&mut temperature)
            .expect("Failed to read line");

        let temperature: f32 = match temperature.trim().parse() {
            Ok(temp) => {
                if temp < -459.67 {
                    println!("Cannot be lower that absolute zero -459.67°F");
                    continue;
                }
                temp
            },
            Err(_) => {
                println!("Invalid input!!! Please choose another one that can be turned into a number.");
                continue
            }
        };

        println!("Got temp {}°F in Fahrenheit", temperature);
        let temperature = (temperature - 32.0) / 1.8;
        println!("Converted to Celsius {}°C", temperature);
        break;
    }
}

fn fibonacci(n: usize) {
    let mut num1: u128 = 0;
    let mut num2: u128 = 1;
    for _ in 0..n {
        let temp = num1;
        num1 = num2;
        num2 += temp;
    }

    println!("{}th Fibonacci Number is {}", n, num1);
}