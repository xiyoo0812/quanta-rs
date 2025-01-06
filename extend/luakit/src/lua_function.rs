#![allow(non_snake_case)]
#![allow(unused_macros)]
#![allow(dead_code)]

use libc::c_int as int;
use libc::c_char as char;

use std::{ mem, ptr };
use std::marker::PhantomData;

use lua::{ cstr, lua_State };
use crate::lua_stack::{ LuaPush, LuaRead };

pub struct FuncWrapper<F, P, R> {
    pub function: F,
    pub marker: PhantomData<(P, R)>,
}

pub trait FuncAdapter<P> {
    type Output;
    fn call(&mut self, params: P) -> Self::Output;
}

macro_rules! impl_func_adapter {
    ($($p:ident),*) => (
        impl<Z, R $(,$p)*> FuncAdapter<($($p,)*)> for FuncWrapper<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R {
            type Output = R;
            fn call(&mut self, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)($($p),*)
            }
        }

        impl<Z, R $(,$p: 'static)*> LuaPush for FuncWrapper<Z, ($($p,)*), R> where Z: FnMut($($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe {
                    let lua_data = lua::lua_newuserdata(L, mem::size_of::<Z>() as libc::size_t);
                    let lua_data: *mut Z = std::mem::transmute(lua_data);
                    ptr::write(lua_data, self.function);
                    let wrapper = lua_adapter::<Self, _, R>;
                    lua::lua_pushcclosure(L, wrapper, 1);
                    1
                }
            }
        }
    )
}

fn lua_adapter<T, P, R>(L: *mut lua_State) -> int where T: FuncAdapter<P, Output = R>, P: LuaRead + 'static, R: LuaPush {
    unsafe {
        let data_raw = lua::lua_touserdata(L, lua::lua_upvalueindex(1));
        let data: &mut T = mem::transmute(data_raw);
        let argc = lua::lua_gettop(L);
        let args = match LuaRead::lua_to_native(L, -argc) {
            Some(a) => a,
            _ => {
                let err_msg = format!("wrong parameter for call function arguments is {}", argc);
                lua::luaL_error(L, err_msg.as_ptr() as * const char);
            }
        };
    
        let ret_value = data.call(args);
        ret_value.native_to_lua(L)
    }
}

impl_func_adapter!();
impl_func_adapter!(A);
impl_func_adapter!(A, B);
impl_func_adapter!(A, B, C);
impl_func_adapter!(A, B, C, D);
impl_func_adapter!(A, B, C, D, E);
impl_func_adapter!(A, B, C, D, E, F);
impl_func_adapter!(A, B, C, D, E, F, G);
impl_func_adapter!(A, B, C, D, E, F, G, H);
impl_func_adapter!(A, B, C, D, E, F, G, H, I);
impl_func_adapter!(A, B, C, D, E, F, G, H, I, J);

#[macro_export]
macro_rules! lua_wrapper_function_impl {
    ($name:ident, $($p:ident),*) => (
        pub fn $name<Z, R $(, $p)*>(&mut self, fname: *const char, f: Z) 
            where Z: FnMut($($p),*) -> R, FuncWrapper<Z, ($($p,)*), R>: LuaPush {
            let wrapper = FuncWrapper { function: f, marker: PhantomData };
            self.set(fname, wrapper);
        }
    )
}


//get global function
//-------------------------------------------------------------------------------
pub fn get_global_function(L: *mut lua_State, func: *const char) -> bool {
    unsafe {
        lua::lua_getglobal(L, func);
        return lua::lua_isfunction(L, -1);
    }
}

pub fn get_table_function(L: *mut lua_State, table: *const char, func: *const char) -> bool {
    unsafe {
        lua::lua_getglobal(L, table);
        if !lua::lua_istable(L, -1) {
            return false;
        }
        lua::lua_getfield(L, -1, func);
        lua::lua_remove(L, -2);
        return lua::lua_isfunction(L, -1);
    }
}

//call function
//-------------------------------------------------------------------------------
pub fn lua_call_function(L: *mut lua_State, arg_count: int, ret_count: int) -> Result<bool, String> {
    unsafe {
        let func_idx = lua::lua_gettop(L) - arg_count;
        if func_idx <= 0 || !lua::lua_isfunction(L, func_idx) {
            return Err("function not found".to_string());
        }
        // 获取 Lua 调试信息（可选）
        lua::lua_getglobal(L, cstr!("debug"));
        lua::lua_getfield(L, -1, cstr!("traceback"));
        lua::lua_remove(L, -2);  // 移除 'debug'
    
        lua::lua_insert(L, func_idx);
        if lua::lua_pcall(L, arg_count, ret_count, func_idx) != 0 {
            let error_msg = lua::lua_tostring(L, -1).unwrap();
            lua::lua_pop(L, 2);  // 弹出错误信息和 traceback
            return Err(error_msg.to_string());
        }
        lua::lua_remove(L, -(ret_count + 1));  // 移除 'traceback'
    }
    return Ok(true);
}

pub fn call_global_function(L: *mut lua_State, func: *const char, arg_count: int, ret_count: int) -> Result<bool, String> {
    if !get_global_function(L, func) {
        return Err("function not found".to_string());
    }
    return lua_call_function(L, arg_count, ret_count);
}

#[macro_export]
macro_rules! lua_call_function_impl {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub fn $name<$($p),*>(&mut self, func: *const char, retc: i32, $($p : $p, )*) -> Result<Vec<Reference>, String> where $($p : LuaPush),* {
            if !self.get_function(func) {
                return Err("function not found".to_string());
            }
            let mut argc = 0;
            $(argc += $p.native_to_lua(self.m_L); )*
            let res = lua_call_function(self.m_L, argc, retc);
            match res {
                Err(e) => return Err(e),
                Ok(_) => {
                    let mut ret = Vec::new();
                    for i in 0..retc {
                        ret.push(LuaRead::lua_to_native(self.m_L, -(retc - i)).unwrap());
                    }
                    lua::lua_pop(self.m_L, retc);
                    return Ok(ret);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! table_call_function_impl {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub fn $name<$($p),*>(&mut self, table: *const char, func: *const char, retc: i32, $($p : $p, )*) -> Result<Vec<Reference>, String> where $($p : LuaPush),* {
            if !get_table_function(self.m_L, table, func) {
                return Err("function not found".to_string());
            }
            let mut argc = 0;
            $(argc += $p.native_to_lua(self.m_L); )*
            let res = lua_call_function(self.m_L, argc, retc);
            match res {
                Err(e) => return Err(e),
                Ok(_) => {
                    let mut ret = Vec::new();
                    for i in 0..retc {
                        ret.push(LuaRead::lua_to_native(self.m_L, -(retc - i)).unwrap());
                    }
                    lua::lua_pop(self.m_L, retc);
                    return Ok(ret);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! variadic_return_impl {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub fn $name<$($p),*>(L: *mut lua_State, $($p : $p, )*) -> i32 where $($p : LuaPush),* {
            let mut argc = 0;
            $(argc += $p.native_to_lua(L);)*
            return argc
        }
    }
}

variadic_return_impl!(variadic_return1, A);
variadic_return_impl!(variadic_return2, A, B);
variadic_return_impl!(variadic_return3, A, B, C);
variadic_return_impl!(variadic_return4, A, B, C, D);
variadic_return_impl!(variadic_return5, A, B, C, D, E);
variadic_return_impl!(variadic_return6, A, B, C, D, E, F);
variadic_return_impl!(variadic_return7, A, B, C, D, E, F, G);
variadic_return_impl!(variadic_return8, A, B, C, D, E, F, G, H);
variadic_return_impl!(variadic_return9, A, B, C, D, E, F, G, H, I);
variadic_return_impl!(variadic_return10, A, B, C, D, E, F, G, H, I, J);
