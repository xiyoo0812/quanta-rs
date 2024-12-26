
#[allow(unused_macros)]
 macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [::std::os::raw::c_char] as *const ::std::os::raw::c_char
    };
}

#[allow(unused_macros)]
macro_rules! ternary {
    ($cond:expr, $true_expr:expr, $false_expr:expr) => {
        if $cond { $true_expr } else { $false_expr }
    };
}
