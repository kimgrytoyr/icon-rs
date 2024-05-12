use std::error::Error;

#[derive(Debug, Clone)]
pub enum Symbol {
    Phrase(String),
    Group(Vec<Symbol>),
    And(Box<Symbol>, Box<Symbol>),
    Or(Box<Symbol>, Box<Symbol>),
    Not(Box<Symbol>),
}

fn parse_expression<'a>(tokens: &'a [&'a str]) -> (Symbol, &'a [&'a str]) {
    let (mut left, mut tokens) = parse_primary(tokens);

    while !tokens.is_empty() {
        match tokens[0] {
            "&" | "|" | " " => {
                let op = tokens[0];
                let (right, rest) = parse_primary(&tokens[1..]);
                left = match op {
                    "&" | " " => Symbol::And(Box::new(left), Box::new(right)),
                    "|" => Symbol::Or(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                };
                tokens = rest;
            }
            _ => break,
        }
    }

    (left, tokens)
}

fn parse_primary<'a>(tokens: &'a [&'a str]) -> (Symbol, &'a [&'a str]) {
    match tokens.get(0) {
        Some(&"(") => {
            let (expr, rest) = parse_expression(&tokens[1..]);
            if rest[0] != ")" {
                panic!("Expected ')'");
            }
            (Symbol::Group(vec![expr]), &rest[1..])
        }
        Some(&"!") => {
            let (expr, rest) = parse_primary(&tokens[1..]);
            (Symbol::Not(Box::new(expr)), rest)
        }
        Some(phrase) => (Symbol::Phrase(phrase.to_string()), &tokens[1..]),
        None => panic!("Unexpected end of input"),
    }
}

fn parse_tokens<'a>(to_parse: &'a str) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let delimiters = ['!', '&', '|', '(', ')', ' '];
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut in_token = false;

    for (i, c) in to_parse.chars().enumerate() {
        if delimiters.contains(&c) {
            if in_token {
                tokens.push(&to_parse[start..i]);
                in_token = false;
            }
            tokens.push(&to_parse[i..i + 1]);
        } else if !c.is_whitespace() {
            if !in_token {
                start = i;
                in_token = true;
            }
            if i == to_parse.len() - 1 {
                tokens.push(&to_parse[start..]);
            }
        } else {
            if in_token {
                tokens.push(&to_parse[start..i]);
                in_token = false;
            }
        }
    }

    Ok(tokens)
}

pub fn match_query(icon_string: String, parsed_query: Symbol) -> Result<bool, Box<dyn Error>> {
    match parsed_query {
        Symbol::Phrase(phrase) => Ok(icon_string.contains(&phrase)),
        Symbol::Group(group) => {
            for symbol in group {
                if !match_query(icon_string.clone(), symbol)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Symbol::And(left, right) => {
            Ok(match_query(icon_string.clone(), *left)?
                && match_query(icon_string.clone(), *right)?)
        }
        Symbol::Or(left, right) => {
            Ok(match_query(icon_string.clone(), *left)?
                || match_query(icon_string.clone(), *right)?)
        }
        Symbol::Not(expr) => Ok(!match_query(icon_string, *expr)?),
    }
}

pub fn parse(to_parse: &str) -> Result<Symbol, Box<dyn Error>> {
    let tokens = parse_tokens(to_parse)?;
    let (parsed, _) = parse_expression(&tokens);

    Ok(parsed)
}
