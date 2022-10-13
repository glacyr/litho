use logos::Logos;

/// Represents a kind of token that appears in a GraphQL document and forms the
/// basis of our [Logos] parser.
///
/// ```rbf
/// SourceCharacter ::= U+0009 | U+000A | U+000D | U+0020-U+FFFF
/// ```
///
/// GraphQL documents are expressed as a sequence of Unicode code points
/// (informally referred to as _"characters"_ through most of this
/// specification). However, with few exceptions, most of GraphQL is expressed
/// only in the original non-control ASCII range so as to be as widely
/// compatible with as many existing tools, languages and serialization formats
/// as possible and avoid display issues in text editors and source control.
///
/// __Note:__ Non-ASCII Unicode characters may appear freely within
/// _StringValue_ and _Comment_ portions of GraphQL.
///
/// _Source: [Sec. 2.1 Source Text](https://spec.graphql.org/October2021/#sec-Unicode)_
#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// The "Byte Order Mark" is a special Unicode character which may appear at
    /// the beginning of a file containing Unicode which programs may use to
    /// determine the fact that the text stream is Unicode, what endianness the
    /// text stream is in, and which of several Unicode encodings to interpret.
    ///
    /// _Source: [Sec. 2.1.1 Unicode](https://spec.graphql.org/October2021/#sec-Unicode)_
    #[token("\u{feff}", logos::skip)]
    ByteOrderMark,

    /// White space is used to improve legibility of source code text and act as
    /// a separation between tokens, and any amount of white space may appear
    /// before or after any token. White space between tokens is not significant
    /// to the semantic meaning of a GraphQL Document, however white space
    /// characters may appear within a `String` or `Comment` token.
    ///
    /// Note: GraphQL intentionally does not consider Unicode "Zs" category
    /// characters as white-space, avoiding misinterpretation by text editors
    /// and source control tools.
    ///
    /// _Source: [Sec. 2.1.2 White Space](https://spec.graphql.org/October2021/#sec-White-Space)_
    #[token("\u{0009}", logos::skip)]
    #[token("\u{0020}", logos::skip)]
    WhiteSpace,

    /// ### 2.1.3 Line Terminators
    /// Like white space, line terminators are used to improve the legibility of
    /// source text and separate lexical tokens, any amount may appear before or
    /// after any other token and have no significance to the semantic meaning
    /// of a GraphQL Document. Line terminators are not found within any other
    /// token.
    ///
    /// Note: Any error reporting which provides the line number in the source
    /// of the offending syntax should use the preceding amount of
    /// `LineTerminator` to produce the line number.
    ///
    /// _Source: [Sec. 2.1.3 Line Terminators](https://spec.graphql.org/October2021/#sec-Line-Terminators)_
    #[token("\u{000a}", logos::skip)]
    #[token("\u{000d}", logos::skip)]
    #[token("\u{000d}\u{000a}", logos::skip)]
    LineTerminator,

    /// GraphQL source documents may contain single-line comments, starting with
    /// the # marker.
    ///
    /// A comment can contain any Unicode code point in `SourceCharacter` except
    /// `LineTerminator` so a comment always consists of all code points
    /// starting with the # character up to but not including the
    /// `LineTerminator` (or end of the source).
    ///
    /// Comments are `Ignored` like white space and may appear after any token,
    /// or before a `LineTerminator`, and have no significance to the semantic
    /// meaning of a GraphQL Document.
    ///
    /// _Source: [Sec. 2.1.4 Comments](https://spec.graphql.org/October2021/#sec-Comments)_
    #[regex("#[\u{0009}\u{0020}-\u{ffff}]*", logos::skip)]
    Comment,

    /// Similar to white space and line terminators, commas (`,`) are used to
    /// improve the legibility of source text and separate lexical tokens but
    /// are otherwise syntactically and semantically insignificant within
    /// GraphQL Documents.
    ///
    /// Non-significant comma characters ensure that the absence or presence of
    /// a comma does not meaningfully alter the interpreted syntax of the
    /// document, as this can be a common user-error in other languages. It also
    /// allows for the stylistic use of either trailing commas or line
    /// terminators as list delimiters which are both often desired for
    /// legibility and maintainability of source code.
    ///
    /// _Source: [Sec. 2.1.5 Insignificant Commas](https://spec.graphql.org/October2021/#sec-Insignificant-Commas)_
    #[token(",", logos::skip)]
    Comma,

    /// GraphQL documents include punctuation in order to describe structure.
    /// GraphQL is a data description language and not a programming language,
    /// therefore GraphQL lacks the punctuation often used to describe
    /// mathematical expressions.
    ///
    /// _Source: [Sec. 2.1.8 Punctuators](https://spec.graphql.org/October2021/#sec-Punctuators)_
    #[token("!")]
    #[token("$")]
    #[token("&")]
    #[token("(")]
    #[token(")")]
    #[token("...")]
    #[token(":")]
    #[token("=")]
    #[token("@")]
    #[token("[")]
    #[token("]")]
    #[token("{")]
    #[token("|")]
    #[token("}")]
    Punctuator,

    /// GraphQL Documents are full of named things: operations, fields,
    /// arguments, types, directives, fragments, and variables. All names must
    /// follow the same grammatical form.
    ///
    /// Names in GraphQL are case-sensitive. That is to say `name`, `Name`, and
    /// `NAME` all refer to different names. Underscores are significant, which
    /// means `other_name` and `othername` are two different names.
    ///
    /// A `Name` must not be followed by a `NameContinue`. In other words, a
    /// `Name` token is always the longest possible valid sequence. The source
    /// characters `a1` cannot be interpreted as two tokens since `a` is
    /// followed by the `NameContinue` `1`.
    ///
    /// Note: Names in GraphQL are limited to the Latin ASCII subset of
    /// `SourceCharacter` in order to support interoperation with as many other
    /// systems as possible.
    ///
    /// ## Reserved Names
    /// Any _Name_ within a GraphQL type system must not start with underscores
    /// `"__"` unless it is part of the introspection system as defined by this
    /// specification.
    ///
    /// _Source: [Sec. 2.1.9 Names](https://spec.graphql.org/October2021/#sec-Names)_
    #[regex("[A-Za-z_][A-Za-z0-9_]*")]
    Name,

    /// An `IntValue` is specified without a decimal point or exponent but may
    /// be negative (ex. `-123`). It must not have any leading `0`.
    ///
    /// An `IntValue` must not be followed by a `Digit`. In other words, an
    /// `IntValue` token is always the longest possible valid sequence. The
    /// source characters `12` cannot be interpreted as two tokens since `1` is
    /// followed by the `Digit` 2. This also means the source `00` is invalid
    /// since it can neither be interpreted as a single token nor two `0`
    /// tokens.
    ///
    /// An `IntValue` must not be followed by a `.` or `NameStart`. If either
    /// `.` or `ExponentIndicator` follows then the token must be interpreted as
    /// a possible `FloatValue`. No other `NameStart` character can follow. For
    /// example the sequences `0x123` and `123L` have no valid lexical
    /// interpretations.
    ///
    /// _Source: [Sec. 2.9.1 Int Value](https://spec.graphql.org/October2021/#sec-Int-Value)_
    #[regex("-?(0|[1-9][0-9]*)")]
    IntValue,

    /// A `FloatValue` includes either a decimal point (ex. `1.0`) or an
    /// exponent (ex. `1e50`) or both (ex. `6.0221413e23`) and may be negative.
    /// Like `IntValue`, it also must not have any leading `0`.
    ///
    /// A `FloatValue` must not be followed by a `Digit`. In other words, a
    /// `FloatValue` token is always the longest possible valid sequence. The
    /// source characters `1.23` cannot be interpreted as two tokens since `1.2`
    /// followed by the `Digit` `3`.
    ///
    /// A `FloatValue` must not be followed by a`.`. For example, the sequence
    /// `1.23.4` cannot be interpreted as two tokens (`1.2`, `3.4`).
    ///
    /// A `FloatValue` must not be followed by a `NameStart`. For example the
    /// sequence `0x1.2p3` has no valid lexical representation.
    ///
    /// Note: The numeric literals `IntValue` and `FloatValue` both restrict
    /// being immediately followed by a letter (or other `NameStart`) to reduce
    /// confusion or unexpected behavior since GraphQL only supports decimal
    /// numbers.
    ///
    /// _Source: [Sec. 2.9.2 Float Value](https://spec.graphql.org/October2021/#sec-Float-Value)_
    #[regex("-?(0|[1-9][0-9]*)\\.[0-9]+")]
    #[regex("-?(0|[1-9][0-9]*)(\\.[0-9]+)?[eE][\\+\\-]?[0-9]+", priority = 3)]
    FloatValue,

    /// Strings are sequences of characters wrapped in quotation marks (U+0022).
    /// (ex. `"Hello World"`). White space and other otherwise-ignored
    /// characters are significant within a string value.
    ///
    /// The empty string `""` must not be followed by another `"` otherwise it
    /// would be interpreted as the beginning of a black string. As an example,
    /// the source `""""""` can only be interpreted as a single empty block
    /// string and not three empty strings.
    ///
    /// Non-ASCII Unicode characters are allowed within single-quoted strings.
    /// Since `SourceCharacter` must not contain some ASCII control characters,
    /// escape sequences must be used to represent these characters. The `\, "`
    /// characters also must be escaped. All other escape sequences are
    /// optional.
    ///
    /// _Source: [Sec. 2.9.4 String Value](https://spec.graphql.org/October2021/#sec-String-Value)_
    #[regex(r#""([^"\\]|\\"|\\\\)*""#)]
    #[regex("\"\"\"(?:[^\"\"\"])*\"\"\"")]
    StringValue,

    /// Any unrecognized token ends up being consumed by this rule.
    #[error]
    #[regex("-?(0|[1-9][0-9]*)[0-9A-Za-z_]+")]
    #[regex("-?(0|[1-9][0-9]*)\\.[0-9]+[A-Za-z_][0-9A-Za-z]*", priority = 2)]
    #[regex("-?(0|[1-9][0-9]*)(\\.[0-9]+)?[eE][\\+\\-]?[0-9]+[A-Za-z_][0-9A-Za-z]*")]
    Error,
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use super::TokenKind;

    #[track_caller]
    fn test_equals(source: &str, rhs: &[TokenKind]) {
        assert_eq!(TokenKind::lexer(source).collect::<Vec<_>>(), rhs);
    }

    #[test]
    fn test_unicode_byte_order_mark() {
        test_equals(" \u{feff} ", &[]);
    }

    #[test]
    fn test_whitespace() {
        test_equals(" \u{0009} ", &[]);
        test_equals(" \u{0020} ", &[]);
        test_equals(" other name ", &[TokenKind::Name, TokenKind::Name]);
    }

    #[test]
    fn test_line_terminators() {
        test_equals(" \u{000a} ", &[]);
        test_equals(" \u{000d}\u{000a} ", &[]);
        test_equals(" \u{000d} ", &[]);
    }

    #[test]
    fn test_comments() {
        test_equals(" # Hello World! ", &[]);
        test_equals(" # Hello World!\rhello ", &[TokenKind::Name]);
        test_equals(" # Hello World!\r\nhello ", &[TokenKind::Name]);
        test_equals(" # Hello World!\nhello ", &[TokenKind::Name]);
    }

    #[test]
    fn test_insignificant_commas() {
        test_equals("foo, bar", &[TokenKind::Name, TokenKind::Name]);
    }

    #[test]
    fn test_name() {
        test_equals(" name ", &[TokenKind::Name]);
        test_equals(" Name ", &[TokenKind::Name]);
        test_equals(" NAME ", &[TokenKind::Name]);
        test_equals(" other_name ", &[TokenKind::Name]);
        test_equals(" othername ", &[TokenKind::Name]);
    }

    #[test]
    fn test_int_value() {
        test_equals(" 01 ", &[TokenKind::Error]);
        test_equals(" 1 ", &[TokenKind::IntValue]);
        test_equals(" 1L ", &[TokenKind::Error]);
    }

    #[test]
    fn test_float_value() {
        test_equals(" 1.0 ", &[TokenKind::FloatValue]);
        test_equals(" 1e50 ", &[TokenKind::FloatValue]);
        test_equals(" 6.0221413e23 ", &[TokenKind::FloatValue]);
        test_equals(" 0.0 ", &[TokenKind::FloatValue]);
    }

    #[test]
    fn test_string_value() {
        test_equals(
            " \"foo\" \"bar\" ",
            &[TokenKind::StringValue, TokenKind::StringValue],
        );
        test_equals(" \"foo \\\" \\\"bar\" ", &[TokenKind::StringValue]);
        test_equals(
            " \"foo \\\\\" \"bar\" ",
            &[TokenKind::StringValue, TokenKind::StringValue],
        );
        test_equals(" \"\"\"block\"\"\" ", &[TokenKind::StringValue]);
    }
}
