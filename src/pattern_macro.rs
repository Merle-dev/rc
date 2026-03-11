#[macro_export]
macro_rules! pattern {
    // 1. Unit Variants: e.g., Token::A
    ($tokens:expr, $enum_name:ident :: $variant:ident) => {
        $tokens.split_first()
            .ok_or("Tokenstream ended too early".to_owned())
            .and_then(|(first, rest)| {
                if let $enum_name::$variant = first {
                    Ok((rest, ()))
                } else {
                    Err(format!("[{}:{}] Expected {}, but found {:?}",file!(), line!(), stringify!($variant), first))
                }
            })
    };

    // 2. Tuple Variants: e.g., Token::C(x)
    ($tokens:expr, $enum_name:ident :: $variant:ident ($binding:ident)) => {
        $tokens.split_first()
            .ok_or("Tokenstream ended too early".to_owned())
            .and_then(|(first, rest)| {
                if let $enum_name::$variant($binding) = first {
                    Ok((rest, $binding))
                } else {
                    Err(format!("[{}:{}] Expected {}, but found {:?}",file!(), line!(), stringify!($variant), first))

                }
            })
    };

    // 3. Struct Variants: e.g., Token::D { d1, d2 }
    ($tokens:expr, $enum_name:ident :: $variant:ident { $($field:ident),* }) => {
        $tokens.split_first()
            .ok_or("Tokenstream ended too early".to_owned())
            .and_then(|(first, rest)| {
                if let $enum_name::$variant { $($field),* } = first {
                    // We clone the fields here because the slice holds references,
                    // but the user's main function expects owned types (u8, String)
                    // for the struct variant.
                    Ok((rest, ($($field.clone()),*)))
                } else {
                    Err(format!("Expected {}, but found {:?}   [{}:{}]", stringify!($variant), first,file!(), line!(), ))
                }
            })
    };
}
