/// Trait implemented by all different diagnostic types that Litho uses.
pub trait DiagnosticInfo<S> {
    /// Returns the code of this diagnostic. This is usually a letter (e.g. `E`)
    /// followed by 4 digits that represent the error number.
    fn code(&self) -> &'static str;

    /// Returns the message of this diagnostic.
    fn message(&self) -> &'static str;

    /// Returns the primary span that triggered this diagnostic.
    fn span(&self) -> S;

    /// Returns additional labels for this diagnostic: pairs of spans and
    /// explanations.
    fn labels(&self) -> Vec<(S, String)>;

    /// Returns a boolean that indicates if this diagnostic is deprecated.
    /// Deprecated diagnostics are no longer returned but still part of the docs
    /// for historic purposes.
    fn is_deprecated(&self) -> bool;
}

macro_rules! deprecated {
    (@deprecated) => {
        true
    };
    () => {
        false
    };
}

macro_rules! diagnostics {
    ($(
        $code:ident => $name:ident @ $span:ident $(+ $($var:ident),*)? {
            $message:literal,
            $(
                $label:literal @ $label_span:ident
            ),*
        } $(@$directive:ident)?
    ),*) => {
        /// Enum that contains all possible diagnostics that Litho can return.
        #[derive(Clone, Debug)]
        pub enum Diagnostic<S> where S: Copy {
            $(
                #[doc = concat!("(", stringify!($code), ") ", $message)]
                $name($name<S>),
            )*
        }

        impl<S> Diagnostic<S> where S: Copy {
            paste::paste! {
                $(
                    #[doc = concat!("(", stringify!($code), ") ", $message)]
                    pub fn [<$name:snake>](
                        $($($var: String,)*)?
                        $(
                            $label_span: S,
                        )*
                    ) -> Diagnostic<S> {
                        Diagnostic::$name($name {
                            $($($var,)*)?
                            $(
                                $label_span,
                            )*
                        })
                    }
                )*
            }

            /// Returns the code of this diagnostic. This is usually a letter (e.g. `E`)
            /// followed by 4 digits that represent the error number.
            pub fn code(&self) -> &'static str {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.code(),)*
                }
            }

            /// Returns the message of this diagnostic.
            pub fn message(&self) -> &'static str {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.message(),)*
                }
            }

            /// Returns the primary span that triggered this diagnostic.
            pub fn span(&self) -> S {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.span(),)*
                }
            }

            /// Returns additional labels for this diagnostic: pairs of spans and
            /// explanations.
            pub fn labels(&self) -> Vec<(S, String)> {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.labels(),)*
                }
            }
        }

        $(
            #[allow(rustdoc::bare_urls)]
            #[doc = concat!("(", stringify!($code), ") ", $message)]
            #[derive(Clone, Debug)]
            pub struct $name<S> where S: Copy {
                $($(
                    #[doc = concat!("Value of `{", stringify!($var), "}` that is referenced in the message and/or one of the labels.")]
                    pub $var: String,
                )*)?
                $(
                    #[doc = $label]
                    pub $label_span: S,
                )*
            }

            impl<S> DiagnosticInfo<S> for $name<S> where S: Copy {
                fn code(&self) -> &'static str {
                    stringify!($code)
                }

                fn message(&self) -> &'static str {
                    $message
                }

                fn span(&self) -> S {
                    self.$span
                }

                fn labels(&self) -> Vec<(S, String)> {
                    let $name { $($($var,)*)? $($label_span,)* } = &self;

                    vec![
                        $(
                            (
                                *$label_span,
                                format!($label),
                            ),
                        )*
                    ]
                }

                fn is_deprecated(&self) -> bool {
                    deprecated!($(@$directive)?)
                }
            }
        )*
    };
}
