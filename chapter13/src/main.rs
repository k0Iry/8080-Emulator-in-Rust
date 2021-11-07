use std::thread;
use std::time::Duration;
use std::collections::HashMap;

struct Cacher<T, K, V>
where
    T: Fn(&K) -> V,
    K: std::hash::Hash + Eq,
    V: Copy
{
    calculation: T,
    table: HashMap<K, V>,
}

impl<T, K, V> Cacher<T, K, V>
where
    T: Fn(&K) -> V,
    K: std::hash::Hash + Eq,
    V: Copy
{
    fn new(calculation: T) -> Cacher<T, K, V> {
        Cacher {
            calculation,
            table: HashMap::new(),
        }
    }

    fn value(&mut self, arg: K) -> V {
        if let Some(value) = self.table.get(&arg) {
            *value
        } else {
            let result = (self.calculation)(&arg);
            self.table.insert(arg, result);
            result
        }
        // *self.table.entry(arg).or_insert((self.calculation)(arg))
    }
}

fn generate_workout(intensity: u32, random_number: u32) {
    let mut expensive_result = Cacher::new(|num| {
        println!("calculating slowly...");
        thread::sleep(Duration::from_secs(2));
        *num
    });

    if intensity < 25 {
        println!("Today, do {} pushups!", expensive_result.value(intensity));
        println!("Next, do {} situps!", expensive_result.value(intensity));
    } else {
        if random_number == 3 {
            println!("Take a break today! Remember to stay hydrated!");
        } else {
            println!(
                "Today, run for {} minutes!",
                expensive_result.value(intensity)
            );
        }
    }

    // try with string slice
    let mut string_length = Cacher::new(|string: &&str| {
        string.len()
    });

    println!("String slice length: {}", string_length.value("this is a string literal"));
}

fn main() {
    let simulated_user_specified_value = 10;
    let simulated_random_number = 7;

    generate_workout(simulated_user_specified_value, simulated_random_number);
}
