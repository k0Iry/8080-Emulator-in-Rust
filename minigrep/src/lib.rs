use std::error::Error;
use std::fs;
use std::env;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.filename)?;
    let query = config.query;

    let results = if config.case_sensitive {
        search(&query, &contents)
    } else {
        search_case_insensitive(&query, &contents)
    };

    for line in results {
        println!("{}", line)
    }

    Ok(())
}

pub struct Config {
    query: String,
    filename: String,
    case_sensitive: bool,
}

impl Config {
    pub fn new(args: &mut env::Args) -> Result<Config, &str> {
        args.next();
        let query = match args.next() { // move the ownership out to query
            Some(query) => query,
            None => return Err("Didn't get a query string")
        };
        let filename = match args.next() {
            Some(filename) => filename,
            None => return Err("Didn't get a file name")
        };
        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();

        println!("CASE_INSENSITIVE = {:?}", case_sensitive);

        Ok(Config { query, filename, case_sensitive })
    }
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query)).collect()
}

fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query.to_lowercase().as_str())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}