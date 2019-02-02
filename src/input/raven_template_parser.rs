use combine::*;
use combine::char::*;
use std::collections::HashMap;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateBuilder {
    tokens: Vec<Token>
}

impl TemplateBuilder {

    pub fn new(target_string: &str) -> TemplateBuilder {
        let parsed_tokens = parse_to_token(target_string)
            .expect(&["failed to parse template string:", target_string].join(" "));

        TemplateBuilder { tokens : parsed_tokens }
    }

    /// embded value to string.
    /// 
    /// ```
    /// use std::collections::HashMap;
    /// use raven::input::raven_template_parser::*;
    /// 
    /// let mut key_val_map: HashMap<String, String> = HashMap::new();
    /// key_val_map.insert("id".to_owned(), "tsuboshy".to_owned());
    /// key_val_map.insert("number".to_owned(), "1234567".to_owned());
    /// 
    /// let builder = TemplateBuilder::new("http://localhost/{{id}}/{{number}}");
    /// 
    /// let result1 = builder.build_string(&key_val_map);
    /// assert_eq!(result1, Ok("http://localhost/tsuboshy/1234567".to_owned()));
    /// 
    /// let empty: HashMap<String, String> = HashMap::new();
    /// let result2 = builder.build_string(&empty);
    /// assert_eq!(result2, Err("could not find value: id".to_owned()));
    /// ```
    pub fn build_string(&self, key_map: &HashMap<String, String>) -> Result<String, String> {
        let mut built = String::new();
        for token in &self.tokens {
            match token {
                Token::PlainText(token) => built.push_str(token),
                Token::Key(key) => match key_map.get(key) {
                    Some(value) => built.push_str(value),
                    None => return Err(["could not find value:".to_owned(), key.to_owned()].join(" "))
                }
            }
        }

        Ok(built)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    PlainText(String),
    Key(String)
}

/// parse string to vec of token.
fn parse_to_token(target_string: &str) -> Result<Vec<Token>, combine::error::StringStreamError> {
    let single_left_brace = char('{')
        .skip(not_followed_by(char('{')))
        .map(|c: char| c.to_string());
    let single_right_brace = char('}')
        .skip(not_followed_by(char('}')))
        .map(|c: char| c.to_string());

    let not_left_brace = many1(none_of("{".chars()));
    let not_right_brace = many1(none_of("}".chars()));
    
    let key_name_parser = many1(
        choice((
            attempt(not_right_brace),
            attempt(single_right_brace)
        ))
    ).map(|key_name: Vec<String>| key_name.concat());
    
    let double_left_braces = string("{{");
    let double_right_braces = string("}}");
    let key_parser = between(double_left_braces, double_right_braces, key_name_parser)
        .map(|key| Token::Key(key));
    
    let plain_left_brace = double_left_braces.map(|_| Token::PlainText("{{".to_owned()));

    let plan_text_parser = many1(
        choice((
            attempt(not_left_brace),
            attempt(single_left_brace)
        ))
    ).map(|plan_text: Vec<String>| Token::PlainText(plan_text.concat())) ;
    
    let mut main_parser = many1::<Vec<Token>, _>(
        choice((
            attempt(key_parser).or(attempt(plain_left_brace)),
            attempt(plan_text_parser),
        ))
    ).skip(eof());

    Ok(main_parser.parse(target_string)?.0)
}

#[test]
fn name() {
    let plain = parse_to_token("https://raven/");
    assert_eq!(plain, Ok(vec![Token::PlainText("https://raven/".to_owned())]));

    let contain_keys = parse_to_token("https://raven/{{number}}");
    assert_eq!(contain_keys, Ok(vec![Token::PlainText("https://raven/".to_owned()), Token::Key("number".to_owned())]));

    let chaus = parse_to_token("https://raven//{{numer{}}}/{index}}/{{{number}}}/{{item");
    let expected = vec![
        Token::PlainText("https://raven//".to_owned()),
        Token::Key("numer{".to_owned()),
        Token::PlainText("}/{index}}/".to_owned()),
        Token::Key("{number".to_owned()),
        Token::PlainText("}/".to_owned()),
        Token::PlainText("{{".to_owned()),
        Token::PlainText("item".to_owned()),
    ];
    assert_eq!(chaus, Ok(expected));

}

// pub fn new(template: &str) -> Emdeber {

// }