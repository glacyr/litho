# Rules

| Name                   | Recognizer   | Parser                                             |
| :--------------------- | :----------- | :------------------------------------------------- |
| `alt(... n)`           | `alt(... n)` | `alt(... n(r))`                                    |
| `opt(a)`               | `a`          | `nom::multi::opt(a).(r)`                           |
| `many0(a)`             | `a`          | `nom::multi::many0(skip_unrecognized(a).(a \| r))` |
| `many1(a)`             | `a`          | `nom::multi::many1(skip_unrecognized(a).(a \| r))` |
| `and(a, b)`            | `a`          | `a(b \| r) & skip_unrecognized(b).(r)`             |
| `and_recognize(a, b)`  | `a \| b`     | `a(b \| r) & skip_unrecognized(b).(r)`             |
| `and_recover(a, b)`    | `a`          | `a(b \| r) & opt(b).(r)`                           |
| `recover(a)`           | `a`          | `map(opt(a).(r), Recoverable)`                     |
| `map(a, f)`            | `a`          | `a(r)`                                             |
| `skip_unrecognized(a)` | `a`          | `extend_rest(many_till(next, a(r) \| r))`          |

# Rules

- Every non-terminal consists of a first