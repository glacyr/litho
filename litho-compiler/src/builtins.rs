pub const fn builtins() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "litho://std.litho.dev/directives.graphql",
            include_str!("../std/directives.graphql"),
        ),
        (
            "litho://std.litho.dev/introspection.graphql",
            include_str!("../std/introspection.graphql"),
        ),
        (
            "litho://std.litho.dev/litho.graphql",
            include_str!("../std/litho.graphql"),
        ),
        (
            "litho://std.litho.dev/scalars.graphql",
            include_str!("../std/scalars.graphql"),
        ),
    ]
}
