use crate::parser::ast::Loc;

use lalrpop_util::ParseError;
use std::{iter::Peekable, str::CharIndices};
use unicode_xid::UnicodeXID;

pub type Spanned<Token, Loc, Error> = Result<(Loc, Token, Loc), Error>;

/// Various errors that can happen during lexing
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum CairoLexerError {
    #[error("EndOfFileInString {0}:{1}")]
    EndOfFileInString(usize, usize),
    #[error("EndOfFileInHint {0}:{1}")]
    EndOfFileInHint(usize, usize),
    #[error("EndofFileInHex {0}:{1}")]
    EndofFileInHex(usize, usize),
    #[error("UnrecognisedToken {0}:{1} `{2}`")]
    UnrecognisedToken(usize, usize, String),
    #[error("MissingNumber {0}:{1}")]
    MissingNumber(usize, usize),
    #[error("Unsupported {0}")]
    Unsupported(String),
    #[error("ParserError {0}:{1} `{2}`")]
    ParserError(usize, usize, String),
}

impl<'input> From<ParseError<usize, CairoToken<'input>, CairoLexerError>> for CairoLexerError {
    fn from(err: ParseError<usize, CairoToken<'input>, CairoLexerError>) -> Self {
        match err {
            ParseError::InvalidToken { location } => {
                CairoLexerError::parser_error(Loc(location, location), "invalid token".to_string())
            }
            ParseError::UnrecognizedToken { token: (l, token, r), expected } => {
                CairoLexerError::parser_error(
                    Loc(l, r),
                    format!("unrecognised token `{:?}', expected {}", token, expected.join(", ")),
                )
            }
            ParseError::User { error } => error,
            ParseError::ExtraToken { token } => CairoLexerError::parser_error(
                Loc(token.0, token.2),
                format!("extra token `{}' encountered", token.0),
            ),
            ParseError::UnrecognizedEOF { location, expected } => CairoLexerError::parser_error(
                Loc(location, location),
                format!("unexpected end of file, expecting {}", expected.join(", ")),
            ),
        }
    }
}

impl CairoLexerError {
    pub fn parser_error(pos: Loc, message: String) -> Self {
        Self::ParserError(pos.0, pos.1, message)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CairoToken<'input> {
    Identifier(&'input str),
    StringLiteral(&'input str),
    ShortStringLiteral(&'input str),
    HexNumber(&'input str),
    Comment(&'input str),
    Hint(&'input str),
    Number(&'input str),

    // Punctuation
    Star,
    DoubleStar,
    OpenCurlyBrace,
    CloseCurlyBrace,
    OpenParenthesis,
    CloseParenthesis,
    OpenBracket,
    CloseBracket,
    Underscore,
    Percent,
    Semicolon,
    Point,
    Comma,
    Not,
    Neq,
    At,
    Newline,

    Cast,
    And,
    Equals,
    Add,
    AddAssign,
    DoublePlus,
    Sub,
    Div,
    Assign,
    Colon,
    Const,
    Let,
    Return,
    Ret,
    Func,
    End,
    Local,
    AllocLocals,
    Struct,
    Namespace,
    Felt,
    Member,
    From,
    Import,
    If,
    Else,
    Assert,
    StaticAssert,
    Arrow,
    Builtins,
    Lang,
    Ap,
    Fp,
    As,
    Tempvar,
    Jmp,
    Abs,
    Rel,
    With,
    WithAttr,
    Call,
    Nondet,
    Dw,
}

pub(crate) struct CairoLexer<'input> {
    input: &'input str,
    chars: Peekable<CharIndices<'input>>,
}

impl<'input> CairoLexer<'input> {
    pub fn new(input: &'input str) -> CairoLexer<'input> {
        CairoLexer { chars: input.char_indices().peekable(), input }
    }

    fn string(
        &mut self,
        token_start: usize,
        string_start: usize,
        quote_char: char,
    ) -> Result<(usize, CairoToken<'input>, usize), CairoLexerError> {
        let mut end;

        let mut last_was_escape = false;

        loop {
            if let Some((i, ch)) = self.chars.next() {
                end = i;
                if !last_was_escape {
                    if ch == quote_char {
                        break
                    }
                    last_was_escape = ch == '\\';
                } else {
                    last_was_escape = false;
                }
            } else {
                return Err(CairoLexerError::EndOfFileInString(token_start, self.input.len()))
            }
        }
        if quote_char == '\'' {
            Ok((token_start, CairoToken::ShortStringLiteral(&self.input[string_start..end]), end))
        } else {
            Ok((token_start, CairoToken::StringLiteral(&self.input[string_start..end]), end))
        }
    }

    fn hint(
        &mut self,
        token_start: usize,
        string_start: usize,
    ) -> Result<(usize, CairoToken<'input>, usize), CairoLexerError> {
        let mut end;
        loop {
            match self.chars.next() {
                Some((i, '%')) => {
                    end = i;
                    match self.chars.peek() {
                        Some((_, '}')) => {
                            self.chars.next();
                            break
                        }
                        None => {
                            return Err(CairoLexerError::EndOfFileInHint(
                                token_start,
                                self.input.len(),
                            ))
                        }
                        _ => {}
                    }
                }
                None => return Err(CairoLexerError::EndOfFileInHint(token_start, self.input.len())),
                _ => {}
            }
        }

        Ok((token_start, CairoToken::Hint(&self.input[string_start..end]), end + 1))
    }

    fn keyword(id: &str) -> Option<CairoToken> {
        match id {
            "cast" => Some(CairoToken::Cast),
            "const" => Some(CairoToken::Const),
            "let" => Some(CairoToken::Let),
            "return" => Some(CairoToken::Return),
            "ret" => Some(CairoToken::Ret),
            "func" => Some(CairoToken::Func),
            "end" => Some(CairoToken::End),
            "local" => Some(CairoToken::Local),
            "alloc_locals" => Some(CairoToken::AllocLocals),
            "struct" => Some(CairoToken::Struct),
            "namespace" => Some(CairoToken::Namespace),
            "member" => Some(CairoToken::Member),
            "felt" => Some(CairoToken::Felt),
            "from" => Some(CairoToken::From),
            "import" => Some(CairoToken::Import),
            "if" => Some(CairoToken::If),
            "else" => Some(CairoToken::Else),
            "assert" => Some(CairoToken::Assert),
            "static_assert" => Some(CairoToken::StaticAssert),
            "ap" => Some(CairoToken::Ap),
            "fp" => Some(CairoToken::Fp),
            "%builtins" => Some(CairoToken::Builtins),
            "as" => Some(CairoToken::As),
            "tempvar" => Some(CairoToken::Tempvar),
            "jmp" => Some(CairoToken::Jmp),
            "abs" => Some(CairoToken::Abs),
            "rel" => Some(CairoToken::Rel),
            "with" => Some(CairoToken::With),
            "with_attr" => Some(CairoToken::WithAttr),
            "call" => Some(CairoToken::Call),
            "nondet" => Some(CairoToken::Nondet),
            "dw" => Some(CairoToken::Dw),
            _ => None,
        }
    }

    fn next_token(&mut self) -> Option<Spanned<CairoToken<'input>, usize, CairoLexerError>> {
        loop {
            match self.chars.next() {
                Some((start, ch)) if UnicodeXID::is_xid_start(ch) || ch == '_' => {
                    let end;
                    loop {
                        if let Some((i, ch)) = self.chars.peek() {
                            if !UnicodeXID::is_xid_continue(*ch) && *ch != '$' {
                                end = *i;
                                break
                            }
                            self.chars.next();
                        } else {
                            end = self.input.len();
                            break
                        }
                    }
                    let id = &self.input[start..end];

                    return if let Some(w) = Self::keyword(id) {
                        Some(Ok((start, w, end)))
                    } else {
                        Some(Ok((start, CairoToken::Identifier(id), end)))
                    }
                }
                Some((i, '#')) => {
                    // ignore Comments for now
                    let start = i + 1;
                    let mut end = start;
                    loop {
                        match self.chars.peek() {
                            Some((_, '\r')) | Some((_, '\n')) => break,
                            None => return None,
                            Some((idx, _)) => {
                                end = *idx;
                                self.chars.next();
                            }
                        }
                    }
                    let _end = end;
                    // TODO
                    // return Some(Ok((
                    //     i,
                    //     CairoToken::Comment(&self.input[start..=end]),
                    //     end +1,
                    // )));
                }
                Some((i, '=')) => {
                    return match self.chars.peek() {
                        Some((_, '=')) => {
                            self.chars.next();
                            Some(Ok((i, CairoToken::Equals, i + 2)))
                        }
                        _ => Some(Ok((i, CairoToken::Assign, i + 1))),
                    }
                }
                Some((i, '%')) => {
                    return match self.chars.peek() {
                        Some((_, '{')) => {
                            self.chars.next();
                            Some(self.hint(i, i + 2))
                        }
                        Some((_, 'b')) => {
                            if self.input[i..].starts_with("%builtins") {
                                self.chars.nth(9);
                                Some(Ok((i, CairoToken::Builtins, i + 9)))
                            } else {
                                Some(Ok((i, CairoToken::Percent, i + 1)))
                            }
                        }
                        Some((_, 'l')) => {
                            if self.input[i..].starts_with("%lang") {
                                self.chars.nth(5);
                                Some(Ok((i, CairoToken::Lang, i + 5)))
                            } else {
                                Some(Ok((i, CairoToken::Percent, i + 1)))
                            }
                        }
                        _ => Some(Ok((i, CairoToken::Percent, i + 1))),
                    }
                }
                Some((start, ch)) if ch.is_ascii_digit() => {
                    let mut end = start + 1;
                    if ch == '0' {
                        if let Some((_, 'x')) = self.chars.peek() {
                            // hex number
                            self.chars.next();

                            let mut end = match self.chars.next() {
                                Some((end, ch)) if ch.is_ascii_hexdigit() => end,
                                Some((_, _)) => {
                                    return Some(Err(CairoLexerError::MissingNumber(
                                        start,
                                        start + 1,
                                    )))
                                }
                                None => {
                                    return Some(Err(CairoLexerError::EndofFileInHex(
                                        start,
                                        self.input.len(),
                                    )))
                                }
                            };

                            while let Some((i, ch)) = self.chars.peek() {
                                if !ch.is_ascii_hexdigit() && *ch != '_' {
                                    break
                                }
                                end = *i;
                                self.chars.next();
                            }

                            return Some(Ok((
                                start,
                                CairoToken::HexNumber(&self.input[start..=end]),
                                end + 1,
                            )))
                        }
                    }

                    loop {
                        if let Some((i, ch)) = self.chars.peek().cloned() {
                            if !ch.is_ascii_digit() {
                                break
                            }
                            self.chars.next();
                            end = i;
                        } else {
                            end = self.input.len();
                            break
                        }
                    }
                    return Some(Ok((start, CairoToken::Number(&self.input[start..end]), end + 1)))
                }
                Some((i, '\r' | '\n')) => return Some(Ok((i, CairoToken::Newline, i + 1))),
                Some((i, '(')) => return Some(Ok((i, CairoToken::OpenParenthesis, i + 1))),
                Some((i, ')')) => return Some(Ok((i, CairoToken::CloseParenthesis, i + 1))),
                Some((i, '{')) => return Some(Ok((i, CairoToken::OpenCurlyBrace, i + 1))),
                Some((i, '}')) => return Some(Ok((i, CairoToken::CloseCurlyBrace, i + 1))),
                Some((i, ':')) => return Some(Ok((i, CairoToken::Colon, i + 1))),
                Some((i, '&')) => return Some(Ok((i, CairoToken::And, i + 1))),
                Some((i, ';')) => return Some(Ok((i, CairoToken::Semicolon, i + 1))),
                Some((i, ',')) => return Some(Ok((i, CairoToken::Comma, i + 1))),
                Some((i, '.')) => return Some(Ok((i, CairoToken::Point, i + 1))),
                Some((i, '[')) => return Some(Ok((i, CairoToken::OpenBracket, i + 1))),
                Some((i, ']')) => return Some(Ok((i, CairoToken::CloseBracket, i + 1))),
                Some((i, '!')) => {
                    return match self.chars.peek() {
                        Some((_, '=')) => {
                            self.chars.next();
                            Some(Ok((i, CairoToken::Neq, i + 2)))
                        }
                        _ => Some(Ok((i, CairoToken::Not, i + 1))),
                    }
                }
                Some((i, '-')) => {
                    return match self.chars.peek() {
                        Some((_, '>')) => {
                            self.chars.next();
                            Some(Ok((i, CairoToken::Arrow, i + 2)))
                        }
                        _ => Some(Ok((i, CairoToken::Sub, i + 1))),
                    }
                }
                Some((i, '@')) => return Some(Ok((i, CairoToken::At, i + 1))),
                Some((i, '/')) => return Some(Ok((i, CairoToken::Div, i + 1))),
                Some((i, '*')) => {
                    return match self.chars.peek() {
                        Some((_, '*')) => {
                            self.chars.next();
                            Some(Ok((i, CairoToken::DoubleStar, i + 2)))
                        }
                        _ => Some(Ok((i, CairoToken::Star, i + 1))),
                    }
                }
                Some((i, '+')) => {
                    return match self.chars.peek() {
                        Some((_, '+')) => {
                            self.chars.next();
                            Some(Ok((i, CairoToken::DoublePlus, i + 2)))
                        }
                        Some((_, '=')) => {
                            self.chars.next();
                            Some(Ok((i, CairoToken::AddAssign, i + 2)))
                        }
                        _ => Some(Ok((i, CairoToken::Add, i + 1))),
                    }
                }
                Some((start, quote_char @ ('"' | '\''))) => {
                    return Some(self.string(start, start + 1, quote_char))
                }
                Some((_, ch)) if ch.is_whitespace() => (),
                Some((start, _)) => {
                    let mut end;

                    loop {
                        if let Some((i, ch)) = self.chars.next() {
                            end = i;
                            if ch.is_whitespace() {
                                break
                            }
                        } else {
                            end = self.input.len();
                            break
                        }
                    }

                    return Some(Err(CairoLexerError::UnrecognisedToken(
                        start,
                        end,
                        self.input[start..end].to_owned(),
                    )))
                }
                None => return None,
            }
        }
    }
}

impl<'input> Iterator for CairoLexer<'input> {
    type Item = Spanned<CairoToken<'input>, usize, CairoLexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
