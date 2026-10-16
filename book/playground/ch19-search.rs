fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if current.len() >= 2 {
            tokens.push(std::mem::take(&mut current));
        } else {
            current.clear();
        }
    }
    if current.len() >= 2 {
        tokens.push(current);
    }
    tokens
}

fn main() {
    let add = tokenize("add two numbers");
    let weather = tokenize("forecast the sky");
    let query = tokenize("add numbers");
    let add_hits = query.iter().filter(|token| add.contains(token)).count();
    let weather_hits = query.iter().filter(|token| weather.contains(token)).count();
    let hit = if add_hits >= weather_hits { "add" } else { "weather" };
    println!("hits: {hit}");
    let mut calls = 0;
    let mut answer = 0;
    for value in [2, 3] {
        if calls >= 2 {
            break;
        }
        calls += 1;
        answer += value;
    }
    println!("calls: {calls}");
    println!("answer: {answer}");
}
