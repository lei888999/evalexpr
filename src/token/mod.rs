use std::str::FromStr;

use crate::{
    error::{EvalexprError, EvalexprResult},
    value::numeric_types::{
        default_numeric_types::DefaultNumericTypes, EvalexprInt, EvalexprNumericTypes,
    },
};

mod display;

#[derive(Clone, PartialEq, Debug)]
pub enum Token<NumericTypes: EvalexprNumericTypes = DefaultNumericTypes> {
    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Hat,

    // Logic
    Eq,
    Neq,
    Gt,
    Lt,
    Geq,
    Leq,
    And,
    Or,
    Not,

    // Precedence
    LBrace,
    RBrace,

    // Assignment
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    HatAssign,
    AndAssign,
    OrAssign,

    // Special
    Comma,
    Semicolon,

    // Values, Variables and Functions
    Identifier(String),
    Float(NumericTypes::Float),
    Int(NumericTypes::Int),
    Boolean(bool),
    String(String),
}

/// A partial token is an input character whose meaning depends on the characters around it.
#[derive(Clone, Debug, PartialEq)]
pub enum PartialToken<NumericTypes: EvalexprNumericTypes = DefaultNumericTypes> {
    /// A partial token that unambiguously maps to a single token.
    Token(Token<NumericTypes>),
    /// A partial token that is a literal.
    Literal(String),
    /// A plus character '+'.
    Plus,
    /// A minus character '-'.
    Minus,
    /// A star character '*'.
    Star,
    /// A slash character '/'.
    Slash,
    /// A percent character '%'.
    Percent,
    /// A hat character '^'.
    Hat,
    /// A whitespace character, e.g. ' '.
    Whitespace,
    /// An equal-to character '='.
    Eq,
    /// An exclamation mark character '!'.
    ExclamationMark,
    /// A greater-than character '>'.
    Gt,
    /// A lower-than character '<'.
    Lt,
    /// An ampersand character '&'.
    Ampersand,
    /// A vertical bar character '|'.
    VerticalBar,
}

// Make this a const fn as soon as is_whitespace and to_string get stable (issue #57563)
fn char_to_partial_token<NumericTypes: EvalexprNumericTypes>(
    c: char,
) -> PartialToken<NumericTypes> {
    match c {
        '+' => PartialToken::Plus,
        '-' => PartialToken::Minus,
        '*' => PartialToken::Star,
        '/' => PartialToken::Slash,
        '%' => PartialToken::Percent,
        '^' => PartialToken::Hat,

        '(' => PartialToken::Token(Token::LBrace),
        ')' => PartialToken::Token(Token::RBrace),

        ',' => PartialToken::Token(Token::Comma),
        ';' => PartialToken::Token(Token::Semicolon),

        '=' => PartialToken::Eq,
        '!' => PartialToken::ExclamationMark,
        '>' => PartialToken::Gt,
        '<' => PartialToken::Lt,
        '&' => PartialToken::Ampersand,
        '|' => PartialToken::VerticalBar,

        c => {
            if c.is_whitespace() {
                PartialToken::Whitespace
            } else {
                PartialToken::Literal(c.to_string())
            }
        },
    }
}

impl<NumericTypes: EvalexprNumericTypes> Token<NumericTypes> {
    #[cfg(not(tarpaulin_include))]
    pub(crate) const fn is_leftsided_value(&self) -> bool {
        match self {
            Token::Plus => false,
            Token::Minus => false,
            Token::Star => false,
            Token::Slash => false,
            Token::Percent => false,
            Token::Hat => false,

            Token::Eq => false,
            Token::Neq => false,
            Token::Gt => false,
            Token::Lt => false,
            Token::Geq => false,
            Token::Leq => false,
            Token::And => false,
            Token::Or => false,
            Token::Not => false,

            Token::LBrace => true,
            Token::RBrace => false,

            Token::Comma => false,
            Token::Semicolon => false,

            Token::Assign => false,
            Token::PlusAssign => false,
            Token::MinusAssign => false,
            Token::StarAssign => false,
            Token::SlashAssign => false,
            Token::PercentAssign => false,
            Token::HatAssign => false,
            Token::AndAssign => false,
            Token::OrAssign => false,

            Token::Identifier(_) => true,
            Token::Float(_) => true,
            Token::Int(_) => true,
            Token::Boolean(_) => true,
            Token::String(_) => true,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub(crate) const fn is_rightsided_value(&self) -> bool {
        match self {
            Token::Plus => false,
            Token::Minus => false,
            Token::Star => false,
            Token::Slash => false,
            Token::Percent => false,
            Token::Hat => false,

            Token::Eq => false,
            Token::Neq => false,
            Token::Gt => false,
            Token::Lt => false,
            Token::Geq => false,
            Token::Leq => false,
            Token::And => false,
            Token::Or => false,
            Token::Not => false,

            Token::LBrace => false,
            Token::RBrace => true,

            Token::Comma => false,
            Token::Semicolon => false,

            Token::Assign => false,
            Token::PlusAssign => false,
            Token::MinusAssign => false,
            Token::StarAssign => false,
            Token::SlashAssign => false,
            Token::PercentAssign => false,
            Token::HatAssign => false,
            Token::AndAssign => false,
            Token::OrAssign => false,

            Token::Identifier(_) => true,
            Token::Float(_) => true,
            Token::Int(_) => true,
            Token::Boolean(_) => true,
            Token::String(_) => true,
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub(crate) fn is_assignment(&self) -> bool {
        use Token::*;
        matches!(
            self,
            Assign
                | PlusAssign
                | MinusAssign
                | StarAssign
                | SlashAssign
                | PercentAssign
                | HatAssign
                | AndAssign
                | OrAssign
        )
    }
}

/// Parses an escape sequence within a string literal.
fn parse_escape_sequence<Iter: Iterator<Item = char>, NumericTypes: EvalexprNumericTypes>(
    iter: &mut Iter,
) -> EvalexprResult<char, NumericTypes> {
    match iter.next() {
        Some('"') => Ok('"'),
        Some('\\') => Ok('\\'),
        Some(c) => Err(EvalexprError::IllegalEscapeSequence(format!("\\{}", c))),
        None => Err(EvalexprError::IllegalEscapeSequence("\\".to_string())),
    }
}

/// Parses a string value from the given character iterator.
///
/// The first character from the iterator is interpreted as first character of the string.
/// The string is terminated by a double quote `"`.
/// Occurrences of `"` within the string can be escaped with `\`.
/// The backslash needs to be escaped with another backslash `\`.
fn parse_string_literal<Iter: Iterator<Item = char>, NumericTypes: EvalexprNumericTypes>(
    mut iter: &mut Iter,
) -> EvalexprResult<PartialToken<NumericTypes>, NumericTypes> {
    let mut result = String::new();

    while let Some(c) = iter.next() {
        match c {
            '"' => return Ok(PartialToken::Token(Token::String(result))),
            '\\' => result.push(parse_escape_sequence(&mut iter)?),
            c => result.push(c),
        }
    }

    Err(EvalexprError::UnmatchedDoubleQuote)
}

fn try_skip_comment<NumericTypes: EvalexprNumericTypes>(
    iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> EvalexprResult<bool, NumericTypes> {
    let mut matched = false;
    if let Some(lookahead) = iter.peek() {
        if *lookahead == '/' {
            matched = true;
            iter.next();
            // line comment
            for c in iter {
                if c == '\n' {
                    break;
                }
            }
        } else if *lookahead == '*' {
            // inline commment
            iter.next();
            while let Some(c) = iter.next() {
                if let Some(next) = iter.peek() {
                    if c == '*' && *next == '/' {
                        matched = true;
                        iter.next();
                        break;
                    }
                }
            }
            if !matched {
                return Err(EvalexprError::CustomMessage(
                    "unmatched inline comment".into(),
                ));
            }
        }
    }

    Ok(matched)
}

/// Converts a string to a vector of partial tokens.
fn str_to_partial_tokens<NumericTypes: EvalexprNumericTypes>(
    string: &str,
) -> EvalexprResult<Vec<PartialToken<NumericTypes>>, NumericTypes> {
    let mut result = Vec::new();
    let mut iter = string.chars().peekable();

    while let Some(c) = iter.next() {
        if c == '"' {
            result.push(parse_string_literal(&mut iter)?);
        } else {
            let partial_token = char_to_partial_token(c);

            if let PartialToken::Slash = partial_token {
                if try_skip_comment(&mut iter)? {
                    continue;
                }
            }

            let if_let_successful =
                if let (Some(PartialToken::Literal(last)), PartialToken::Literal(literal)) =
                    (result.last_mut(), &partial_token)
                {
                    last.push_str(literal);
                    true
                } else {
                    false
                };

            if !if_let_successful {
                result.push(partial_token);
            }
        }
    }
    Ok(result)
}

/// Resolves all partial tokens by converting them to complex tokens.
fn partial_tokens_to_tokens<NumericTypes: EvalexprNumericTypes>(
    mut tokens: &[PartialToken<NumericTypes>],
) -> EvalexprResult<Vec<Token<NumericTypes>>, NumericTypes> {
    let mut result = Vec::new();
    while !tokens.is_empty() {
        let first = tokens[0].clone();
        let second = tokens.get(1).cloned();
        let third = tokens.get(2).cloned();
        let mut cutoff = 2;

        result.extend(match first {
            PartialToken::Token(token) => {
                cutoff = 1;
                Some(token)
            },
            PartialToken::Plus => match second {
                Some(PartialToken::Eq) => Some(Token::PlusAssign),
                _ => {
                    cutoff = 1;
                    Some(Token::Plus)
                },
            },
            PartialToken::Minus => match second {
                Some(PartialToken::Eq) => Some(Token::MinusAssign),
                _ => {
                    cutoff = 1;
                    Some(Token::Minus)
                },
            },
            PartialToken::Star => match second {
                Some(PartialToken::Eq) => Some(Token::StarAssign),
                _ => {
                    cutoff = 1;
                    Some(Token::Star)
                },
            },
            PartialToken::Slash => match second {
                Some(PartialToken::Eq) => Some(Token::SlashAssign),
                _ => {
                    cutoff = 1;
                    Some(Token::Slash)
                },
            },
            PartialToken::Percent => match second {
                Some(PartialToken::Eq) => Some(Token::PercentAssign),
                _ => {
                    cutoff = 1;
                    Some(Token::Percent)
                },
            },
            PartialToken::Hat => match second {
                Some(PartialToken::Eq) => Some(Token::HatAssign),
                _ => {
                    cutoff = 1;
                    Some(Token::Hat)
                },
            },
            PartialToken::Literal(literal) => {
                cutoff = 1;
                if let Ok(number) = parse_dec_or_hex::<NumericTypes>(&literal) {
                    Some(Token::Int(number))
                } else if let Ok(number) = literal.parse::<NumericTypes::Float>() {
                    Some(Token::Float(number))
                } else if let Ok(boolean) = literal.parse::<bool>() {
                    Some(Token::Boolean(boolean))
                } else {
                    // If there are two tokens following this one, check if the next one is
                    // a plus or a minus. If so, then attempt to parse all three tokens as a
                    // scientific notation number of the form `<coefficient>e{+,-}<exponent>`,
                    // for example [Literal("10e"), Minus, Literal("3")] => "1e-3".parse().
                    match (second, third) {
                        (Some(second), Some(third))
                            if second == PartialToken::Minus || second == PartialToken::Plus =>
                        {
                            if let Ok(number) = format!("{}{}{}", literal, second, third)
                                .parse::<NumericTypes::Float>()
                            {
                                cutoff = 3;
                                Some(Token::Float(number))
                            } else {
                                Some(Token::Identifier(literal.to_string()))
                            }
                        },
                        _ => Some(Token::Identifier(literal.to_string())),
                    }
                }
            },
            PartialToken::Whitespace => {
                cutoff = 1;
                None
            },
            PartialToken::Eq => match second {
                Some(PartialToken::Eq) => Some(Token::Eq),
                _ => {
                    cutoff = 1;
                    Some(Token::Assign)
                },
            },
            PartialToken::ExclamationMark => match second {
                Some(PartialToken::Eq) => Some(Token::Neq),
                _ => {
                    cutoff = 1;
                    Some(Token::Not)
                },
            },
            PartialToken::Gt => match second {
                Some(PartialToken::Eq) => Some(Token::Geq),
                _ => {
                    cutoff = 1;
                    Some(Token::Gt)
                },
            },
            PartialToken::Lt => match second {
                Some(PartialToken::Eq) => Some(Token::Leq),
                _ => {
                    cutoff = 1;
                    Some(Token::Lt)
                },
            },
            PartialToken::Ampersand => match second {
                Some(PartialToken::Ampersand) => match third {
                    Some(PartialToken::Eq) => {
                        cutoff = 3;
                        Some(Token::AndAssign)
                    },
                    _ => Some(Token::And),
                },
                _ => return Err(EvalexprError::unmatched_partial_token(first, second)),
            },
            PartialToken::VerticalBar => match second {
                Some(PartialToken::VerticalBar) => match third {
                    Some(PartialToken::Eq) => {
                        cutoff = 3;
                        Some(Token::OrAssign)
                    },
                    _ => Some(Token::Or),
                },
                _ => return Err(EvalexprError::unmatched_partial_token(first, second)),
            },
        });

        tokens = &tokens[cutoff..];
    }
    Ok(result)
}

pub(crate) fn tokenize<NumericTypes: EvalexprNumericTypes>(
    string: &str,
) -> EvalexprResult<Vec<Token<NumericTypes>>, NumericTypes> {
    partial_tokens_to_tokens(&str_to_partial_tokens(string)?)
}

fn parse_dec_or_hex<NumericTypes: EvalexprNumericTypes>(
    literal: &str,
) -> Result<NumericTypes::Int, ()> {
    if let Some(literal) = literal.strip_prefix("0x") {
        NumericTypes::Int::from_hex_str(literal)
    } else {
        NumericTypes::Int::from_str(literal).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        token::{char_to_partial_token, tokenize, Token},
        value::numeric_types::default_numeric_types::DefaultNumericTypes,
    };
    use std::fmt::Write;

    #[test]
    fn test_partial_token_display() {
        let chars = vec![
            '+', '-', '*', '/', '%', '^', '(', ')', ',', ';', '=', '!', '>', '<', '&', '|', ' ',
        ];

        for char in chars {
            assert_eq!(
                format!("{}", char),
                format!("{}", char_to_partial_token::<DefaultNumericTypes>(char))
            );
        }
    }

    #[test]
    fn test_token_display() {
        let token_string =
            "+ - * / % ^ == != > < >= <= && || ! ( ) = += -= *= /= %= ^= &&= ||= , ; ";
        let tokens = tokenize::<DefaultNumericTypes>(token_string).unwrap();
        let mut result_string = String::new();

        for token in tokens {
            write!(result_string, "{} ", token).unwrap();
        }

        assert_eq!(token_string, result_string);
    }

    #[test]
    fn test_skip_comment() {
        let token_string =
            "+ - * / % ^ == != > < >= <= && || ! ( ) = += -= *= /= %= ^= &&= ||= , ; ";

        let token_string_with_comments = r"+ - * / % ^ == != > 
            < >= <= && /* inline comment */ || ! ( ) 
            = += -= *= /= %= ^=
            // line comment
            &&= ||= , ; 
            ";

        let tokens = tokenize::<DefaultNumericTypes>(token_string_with_comments).unwrap();
        let mut result_string = String::new();

        for token in tokens {
            write!(result_string, "{} ", token).unwrap();
        }

        assert_eq!(token_string, result_string);
    }

    #[test]
    fn assignment_lhs_is_identifier() {
        let tokens = tokenize::<DefaultNumericTypes>("a = 1").unwrap();
        assert_eq!(
            tokens.as_slice(),
            [
                Token::Identifier("a".to_string()),
                Token::Assign,
                Token::Int(1)
            ]
        );
    }
}
