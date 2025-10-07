use crate::TigerReadable;

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: TigerReadable),+> TigerReadable for ($($name,)+)
        {
            fn read_ds_endian<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, endian: crate::Endian) -> crate::Result<Self> {
                Ok(($($name::read_ds_endian(reader, endian)?,)+))
            }

            

            const SIZE: usize = 0 $(+ $name::SIZE)+;
        }
    };
}

tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
