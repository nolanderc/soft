
macro_rules! impl_elementwise_op {
    ($item:ident {$($field:ident),*}, $trait:ident, $op:ident) => {
        impl $trait<$item> for $item {
            type Output = $item;
            fn $op(self, rhs: $item) -> Self::Output {
                $item {
                    $(
                        $field: $trait::$op(self.$field, rhs.$field),
                    )*
                }
            }
        }
    }
}

macro_rules! impl_scalar_op {
    ($item:ident {$($field:ident),*}, $trait:ident<$scalar:ident>, $op:ident) => {
        impl $trait<$scalar> for $item {
            type Output = $item;
            fn $op(self, rhs: $scalar) -> Self::Output {
                $item {
                    $(
                        $field: $trait::$op(self.$field, rhs),
                    )*
                }
            }
        }
        impl $trait<$item> for $scalar {
            type Output = $item;
            fn $op(self, rhs: $item) -> Self::Output {
                $item {
                    $(
                        $field: $trait::$op(self, rhs.$field),
                    )*
                }
            }
        }
    }
}

