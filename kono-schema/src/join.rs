use super::{Item, Schema};

macro_rules! impl_tuple {
    ( $first:ident $($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<$first, $($name),+> Schema for ($first, $($name),+)
        where
            $first: Schema,
            $($name: Schema,)+
        {
			fn schema(&self) -> Vec<Item> {
				let (ref $first, $(ref $name),*) = self;

				vec![$first.schema(), $($name.schema()),*].into_iter().flatten().collect()
			}
		}
	}
}

impl_tuple!(A B);
impl_tuple!(A B C);
impl_tuple!(A B C D);
impl_tuple!(A B C D E);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J K L);
