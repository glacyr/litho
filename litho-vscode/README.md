# Litho for Visual Studio Code

Litho for VSCode is a small extension that dramatically improves language
support for developers working with GraphQL in VSCode. Litho implements the full
2021 GraphQL spec, comes with a fault-tolerant parser, intelligent auto-complete
and code assistance and requires zero configuration.

## Getting Started

### Installation
Simply install Litho from the Visual Studio marketplace to get started!

### Usage

Litho will automatically index all GraphQL files it can find in your workspace,
except for any files included in your `.gitignore`.

### Support

If you have any questions, feedback or suggestions, don't hesitate to contact us
through [chat on our website](https://litho.dev/).

## Features

Litho comes with a bunch of useful features that you need to write effective
GraphQL.

<table>
<tr>
<td width="50%" valign="top">

![Screenshot for Intelligent Auto-Complete](https://litho.dev/screenshots/png/autocomplete@3x.png)

#### Intelligent Auto-Complete
Litho provides context-aware, detailed completions that will help you write
GraphQL type-systems, queries, and mutations in no time.

</td>
<td width="50%" valign="top">

![Screenshot for Spec Compliant](https://litho.dev/screenshots/png/insightful-error-messages@3x.png)

#### Spec Compliant
Litho implements the full 2021 GraphQL spec, catching more than 120 different
types of hard-to-find bugs before your code hits production.

</td>
</tr>
<tr>
<td width="50%" valign="top">

![Screenshot for Zero-Configuration](https://litho.dev/screenshots/png/goto-definition@3x.png)

#### Zero-Configuration
Litho automatically indexes all GraphQL files it can find in your workspace and
starts providing intelligent assistance in a fraction of a second.

</td>
<td width="50%" valign="top">

![Screenshot for Fault-Tolerant Parsing](https://litho.dev/screenshots/png/recoverable-parser@3x.png)

#### Fault-Tolerant Parsing
Litho can automatically diagnose and correct syntax errors, and continue to give
accurate code completions, bringing you the best GraphQL development experience
on the market.

</td>
</tr>
<tr>
<td width="50%" valign="top">

![Screenshot for Incremental Compilation](https://litho.dev/screenshots/png/incremental-compilation@3x.png)

#### Incremental Compilation
Litho is written in Rust and uses incremental compilation to minimize our CPU
and memory footprint on your machine and to give accurate code assistance in
single-digit milliseconds.

</td>
<td width="50%" valign="top">

![Screenshot for Documentation-Driven](https://litho.dev/screenshots/png/documentation@3x.png)

#### Documentation-Driven
Litho parses all of your existing Markdown-formatted GraphQL documentation and
shows it in a neat tooltip alongside type inference information whenever you
hover over a symbol.

</td>
</tr>
<tr>
<td width="50%" valign="top">

![Screenshot for Transitive Type Checking](https://litho.dev/screenshots/png/transitive-type-checking@3x.png)

#### Transitive Type Checking
Litho transitively checks fragment definitions for missing variable definitions,
type mismatches in variable usages, and unmergeable selection sets.

</td>
<td width="50%" valign="top">

![Screenshot for Auto-Import Remote Schemas](https://litho.dev/screenshots/png/auto-import@3x.png)

#### Auto-Import Remote Schemas
Litho can import schemas from remote URLs and automatically refresh them to
integrate the latest changes.

</td>
</tr>
<tr>
<td width="50%" valign="top">

![Screenshot for Code Actions](https://litho.dev/screenshots/png/code-actions@3x.png)

#### Code Actions
Litho automatically suggests potential fixes to common problems such as missing
fields, variables, or arguments.

</td>
<td width="50%" valign="top">

![Screenshot for CodeLens](https://litho.dev/screenshots/png/codelens@3x.png)

#### CodeLens
Litho indexes your entire schema and provides CodeLens functionality that lets
you easily track down type usage and fragment includes.

</td>
</tr>
<tr>
<td width="50%" valign="top">

![Screenshot for Built-in Linter](https://litho.dev/screenshots/png/linter@3x.png)

#### Built-in Linter
Litho warns early about potential issues with your schema, such as unusued
types, duplicate selections, or ambiguous formatting.

</td>
<td width="50%" valign="top">

![Screenshot for Built-in Linter](https://litho.dev/screenshots/png/semantic-tokens@3x.png)

#### Semantic Syntax Highlighting
Litho uses its language server to provide context-aware syntax highlighting,
assigning different colors to identical tokens based on semantic contextual
differences.

</td>
</tr>
</table>

# Acknowledgements

Portions of this software may utilize the following copyrighted material, the
use of which is hereby acknowledged:

```
Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT (1):
    wasi

Apache-2.0 OR BSL-1.0 (1):
    ryu

Apache-2.0 OR MIT (76):
    async-trait, auto_impl, beef, bitflags, bstr, cfg-if, crossbeam-utils, fnv,
    form_urlencoded, futures, futures-channel, futures-core, futures-executor,
    futures-io, futures-macro, futures-sink, futures-task, futures-util,
    hashbrown, hermit-abi, httparse, idna, indexmap, itoa, lazy_static, libc,
    lock_api, log, logos, logos-derive, minimal-lexical, multimap, num_cpus,
    once_cell, parking_lot, parking_lot_core, paste, percent-encoding,
    pin-project, pin-project-internal, pin-project-lite, pin-utils,
    proc-macro-error, proc-macro-error-attr, proc-macro2, quote, regex,
    regex-syntax, rust-analyzer, scopeguard, serde, serde_derive, serde_json,
    serde_repr, signal-hook-registry, smallvec, smol_str, socket2, syn,
    thread_local, tower-lsp, tower-lsp-macros, unicode-bidi, unicode-ident,
    unicode-normalization, unindent, url, winapi, winapi-i686-pc-windows-gnu,
    winapi-x86_64-pc-windows-gnu, windows-sys, windows_aarch64_msvc,
    windows_i686_gnu, windows_i686_msvc, windows_x86_64_gnu,
    windows_x86_64_msvc, yansi
    
Apache-2.0 OR MIT OR Zlib (2):
    tinyvec, tinyvec_macros

MIT (18):
    ariadne, bytes, dashmap, line-col, lsp-types, matches, mio, nom,
    redox_syscall, slab, tokio, tokio-macros, tokio-util, tower, tower-layer,
    tower-service, tracing, tracing-core

MIT OR Unlicense (7):
    aho-corasick, globset, ignore, memchr, same-file, walkdir, winapi-util
```
