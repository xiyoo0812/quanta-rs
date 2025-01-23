
#[allow(unused_macros)]

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [::std::os::raw::c_char] as *const ::std::os::raw::c_char
    };
}

#[macro_export]
macro_rules! to_char {
    ($s:expr) => {
        format!("{}\0", $s).as_ptr() as *const ::std::os::raw::c_char
    };
}

#[macro_export]
macro_rules! ternary {
    ($cond:expr, $true_expr:expr, $false_expr:expr) => {
        if $cond { $true_expr } else { $false_expr }
    };
}

#[macro_export]
macro_rules! lua_reg {
    () => {
        lua::luaL_Reg {
            name: std::ptr::null(),
            func: lua::null_function,
        }
    };
    ($name:expr, $func:expr) => {
        lua::luaL_Reg {
            name: cstr!($name),
            func: $func,
        }
    };
}
