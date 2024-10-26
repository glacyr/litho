# Rules

| Name            | Recognizer                                    | Parser                                                                                                           |
| :-------------- | :-------------------------------------------- | :--------------------------------------------------------------------------------------------------------------- |
| `alt`           | `alt(self.*.recognizer())`                    | `alt(self.*.parser(recovery_point))`                                                                             |
| `opt`           | `self.0.recognizer()`                         | `extend_rest(many_till(next, self.0.parser(recovery_point).map(Some).or(recovery_point.recognizer().map(None)))` |
| `many0`         | `self.0.recognizer()`                         | `nom::multi::many0(self.0.parser(recovery_point.or(self.0)))`                                                    |
| `many1`         | `self.0.recognizer()`                         | `nom::multi::many1(self.0.parser(recovery_point.or(self.0)))`                                                    |
| `and`           | `self.0.recognizer()`                         | `self.0.parser(recovery_point.or(self.1)).and(self.1.parser(recovery_point))`                                    |
| `and_recognize` | `self.0.recognizer().or(self.1.recognizer())` | `and(self.0, self.1).parser(recovery_point)`                                                                     |
| `and_recover`   | `self.0.recognizer()`                         | `self.0.parser(recovery_point.or(self.1)).and(opt(self.1).parser(recovery_point)).map(Recoverable)`              |
| `recover`       | `self.0.recognizer()`                         | `opt(self.0).parser(recovery_point).map(Recoverable)`                                                            |
| `map`           | `self.0.recognizer()`                         | `self.0.parser(recovery_point).map(_)`                                                                           |
