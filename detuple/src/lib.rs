pub trait FromNested<T> {
    fn from_nested(nested: T) -> Self;
}

macro_rules! nest {
        ($a:tt) => { $a };
        ($a:tt $b:ident) => { ($a, $b) };
        ($a:tt $b:ident $($c:ident)*) => { nest!(($a, $b) $($c)*) };
    }

macro_rules! nested {
        ($a:ident $b:ident $($ident:ident)*) => {
            nested!(@ $a $b ; $($ident)*);
        };
        (@ $($ident:ident)* ; $next:ident $($rest:ident)*) => {
            nested!(@ $($ident)* ;);
            nested!(@ $($ident)* $next ; $($rest)*);
        };
        (@ $($ident:ident)* ;) => {
            #[allow(non_snake_case)]
            impl<$($ident,)*> FromNested<nest!($($ident)*)> for ($($ident,)*) {
                fn from_nested(nested: nest!($($ident)*)) -> Self {
                    let nest!($($ident)*) = nested;
                    ($($ident,)*)
                }
            }
        }
    }

nested!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);

pub trait Flatten<T> {
    type Flat;

    fn flatten(self) -> Self::Flat;
}

impl<T, U> Flatten<T> for Option<U>
where
    T: FromNested<U>,
{
    type Flat = Option<T>;

    fn flatten(self) -> Option<T> {
        self.map(T::from_nested)
    }
}

#[cfg(test)]
mod tests {
    use super::Flatten;

    #[test]
    fn test_option() {
        let (x, y, z) = Some(1).zip(Some(2)).zip(Some(3)).flatten().unwrap();
        let _ = (x, y, z);
    }
}
