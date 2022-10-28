pub trait DiagnosticInfo<S> {
    fn code(&self) -> &'static str;
    fn message(&self) -> &'static str;
    fn span(&self) -> S;
    fn labels(&self) -> Vec<(S, String)>;
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

            pub fn code(&self) -> &'static str {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.code(),)*
                }
            }

            pub fn message(&self) -> &'static str {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.message(),)*
                }
            }

            pub fn span(&self) -> S {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.span(),)*
                }
            }

            pub fn labels(&self) -> Vec<(S, String)> {
                match self {
                    $(Diagnostic::$name(diagnostic) => diagnostic.labels(),)*
                }
            }
        }

        $(
            #[doc = concat!("(", stringify!($code), ") ", $message)]
            #[derive(Clone, Debug)]
            pub struct $name<S> where S: Copy {
                $($(pub $var: String,)*)?
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
