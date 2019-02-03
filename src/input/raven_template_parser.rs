use combine::*;
use combine::char::*;
use std::collections::HashMap;
use chrono::prelude::*;


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
fn parse_to_token_test() {
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


/// try to expand numeric list strings.
/// 
/// ```
/// use raven::input::raven_template_parser::try_expand_list;
/// 
/// let numeric_list_pattern = "[1..5]";
/// let result1 = try_expand_list(numeric_list_pattern);
/// assert_eq!(result1, vec!["1","2","3","4","5"]);
///
/// let not_numeric_list_pattern = "a1234";
/// let result2 = try_expand_list(not_numeric_list_pattern);
/// assert_eq!(result2, vec!["a1234".to_owned()]);
///
/// let contain_other_strings = "id-[1..2]-[1..2]";
/// let result3 = try_expand_list(contain_other_strings);
/// assert_eq!(
///     result3, 
///     vec![ "id-1-1".to_owned()
///         , "id-1-2".to_owned()
///         , "id-2-1".to_owned()
///         , "id-2-2".to_owned()
///         ]
/// )
///```
pub fn try_expand_list(target_string: &str) -> Vec<String> {
    let expand_list_parser = ( 
        char('['), 
        many1(digit()),
        many1(char('.')),
        many1(digit()),
        char(']')
    ).map(|t: (_, String, String, String, _) | {
        let start = t.1.parse::<i32>().unwrap();
        let end = t.3.parse::<i32>().unwrap() + 1;
        (start .. end)
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
    });

    let plain_text = many1(none_of("[".chars())).map(|plain_text: String| vec![plain_text]);
    let plain_left_brace = char('[').map(|_| vec!["[".to_owned()]);

    let mut main_parser = many1::<Vec<Vec<String>>, _>(
        choice((
            attempt(expand_list_parser).or(attempt(plain_left_brace)),
            attempt(plain_text)
        ))
    ).skip(eof());

    let parsed = main_parser.parse(target_string);

    match parsed {
        Ok((result, _)) => result.iter()
            .fold(vec!["".to_owned()], |result, vec| {
                product_list(&result, vec)
                    .iter()
                    .map(|string_list| string_list.concat())
                    .collect()
            }),
        Err(_) => vec![target_string.to_owned()]
    }        
}

fn product_list<T: Clone>(vec1: &Vec<T>, vec2: &Vec<T>) -> Vec<Vec<T>> {
    let mut result = Vec::with_capacity(vec1.len() * vec2.len());
    for item_1 in vec1 {
        for item_2 in vec2 {
            result.push(vec![item_1.clone(), item_2.clone()]);
        }
    }
    result
}


#[test]
fn try_expand_list_test() {
    let numeric_list_pattern = "[1..5]";
    let result1 = try_expand_list(numeric_list_pattern);
    assert_eq!(result1, vec!["1","2","3","4","5"]);

    let not_numeric_list_pattern = "a1234";
    let result2 = try_expand_list(not_numeric_list_pattern);
    assert_eq!(result2, vec!["a1234".to_owned()]);

    let contain_other_strings = "id-[1..2]-[1..2]";
    let result3 = try_expand_list(contain_other_strings);
    assert_eq!(
        result3, 
        vec![ "id-1-1".to_owned()
            , "id-1-2".to_owned()
            , "id-2-1".to_owned()
            , "id-2-2".to_owned()
            ]
    )

}


pub fn parse_key_value_map(map: HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    if map.is_empty() {
        return vec![HashMap::new()];
    }

    let now: DateTime<Local> = Local::now();

    let mut single_map_lists: Vec<Vec<HashMap<String, String>>> = vec![];
    for (key, values) in map.into_iter() {
        let single_maps = values.iter()
            .map(|val| now.format(val).to_string())
            .flat_map(|val| try_expand_list(&val))
            .map(|val| { 
                let mut single_map = HashMap::new();
                single_map.insert(key.to_owned(), val);
                single_map
            })
            .collect::<Vec<HashMap<String, String>>>();
        single_map_lists.push(single_maps);
    };

    single_map_lists.into_iter()
        .fold(vec![HashMap::new()], |result_list, list_of_map| {
            let mut new_result_list: Vec<HashMap<String, String>> = Vec::new();
            for result_item in &result_list {
                for map in &list_of_map {
                    let mut result_map: HashMap<String, String> = HashMap::new();
                    result_map.extend(result_item.to_owned());
                    result_map.extend(map.to_owned());
                    new_result_list.push(result_map);
                }
            }
            new_result_list
        })
}


#[test]
fn parse_key_value_map_and_template_parser_testtest() {
    let now_y_m_d = Local::now().format("%Y-%m-%d").to_string();

    let builder = TemplateBuilder::new("https://raven/{{a}}/{{b}}/{{c}}");

    let mut map = HashMap::new();
    map.insert("a".to_owned(), vec!["a1".to_owned(), "a2".to_owned()]);
    map.insert("b".to_owned(), vec!["b[1..2]".to_owned()]);
    map.insert("c".to_owned(), vec!["c1-%Y-%m-%d".to_owned()]);

    let mut expected = vec![
        ["https://raven/a1/b1/c1-", &now_y_m_d].concat(),
        ["https://raven/a1/b2/c1-", &now_y_m_d].concat(),
        ["https://raven/a2/b1/c1-", &now_y_m_d].concat(),
        ["https://raven/a2/b2/c1-", &now_y_m_d].concat()        
    ];

    for parsed in parse_key_value_map(map) {
        assert_eq!(builder.build_string(&parsed), Ok(expected.pop().unwrap()));
    };
}