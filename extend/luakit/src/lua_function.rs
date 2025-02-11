#![allow(non_snake_case)]
#![allow(unused_macros)]
#![allow(dead_code)]

use libc::c_int as int;
use libc::c_char as char;
use libc::c_void as void;

use std::any::Any;
use std::{ mem, ptr };
use std::marker::PhantomData;

use lua::{ cstr, to_char, lua_State };
use crate::lua_stack::lua_push_userdata;
use crate::lua_stack::{ LuaRead, LuaPush, LuaPushFn, LuaPushFnMut, LuaPushLuaFn, LuaPushLuaFnMut};

pub struct FuncWrapper<F, P, R> {
    pub function: F,
    pub marker: PhantomData<(P, R)>,
}

pub struct ClassFuncWrapper<F, O, P, R> {
    pub function: F,
    pub marker: PhantomData<(O, P, R)>,
}

pub struct ClassMemberWrapper<O, M> {
    pub offset: usize,
    pub marker: PhantomData<(O, M)>,
}

pub trait FuncAdapter<P> {
    type Output;
    fn call(&mut self, params: P) -> Self::Output;
}

pub trait FuncLuaAdapter<P> {
    type Output;
    fn call(&mut self, L: *mut lua_State, params: P) -> Self::Output;
}

pub trait ClassFuncAdapter<O, P> {
    type Output;
    fn call(&mut self, obj: &O, params: P) -> Self::Output;
}

pub trait ClassLuaFuncAdapter<O, P> {
    type Output;
    fn call(&mut self, obj: &O, L: *mut lua_State, params: P) -> Self::Output;
}

pub trait ClassFuncMutAdapter<O, P> {
    type Output;
    fn call(&mut self, obj: &mut O, params: P) -> Self::Output;
}

pub trait ClassLuaFuncMutAdapter<O, P> {
    type Output;
    fn call(&mut self, obj: &mut O, L: *mut lua_State, params: P) -> Self::Output;
}

pub trait ClassMemberAdapter<O, M> {
    fn get(&mut self, obj: &mut O) -> M;
    fn set(&mut self, obj: &mut O, params: M);
}

impl<O, M> ClassMemberAdapter<O, M> for ClassMemberWrapper<O, M> where M: Clone {
    fn get(&mut self, obj: &mut O) -> M {
        unsafe {
            let ptr = (obj as *mut O as *mut u8).add(self.offset) as *mut M;
            ptr.read().clone()
        }
    }
    fn set(&mut self, obj: &mut O, params: M) {
        unsafe {
            let ptr = (obj as *mut O as *mut u8).add(self.offset) as *mut M;
            ptr.write(params);
        }
    }
}

macro_rules! push_lua_wraper_address {
    ($fn:ty, $L:expr, $w:expr) => {
        //这里压入的是warper的第一个成员(function,offset)的指针，其实就是warper的地址
        let lua_func = lua::lua_newuserdata($L, mem::size_of::<$fn>() as libc::size_t);
        let func_ptr: *mut $fn = std::mem::transmute(lua_func);
        ptr::write(func_ptr, $w);
    }
}

macro_rules! push_global_function {
    ($fn:ty, $L:ident, $f:expr, $wraper:expr) => {{
        push_lua_wraper_address!($fn, $L, $f);
        lua::lua_pushcclosure($L, $wraper, 1);
        1
    }}
}

macro_rules! push_class_function {
    ($fn:ty, $L:ident, $f:expr, $wraper:expr) => {{
        lua::lua_newtable($L);
        push_lua_wraper_address!($fn, $L, $f);
        lua::lua_setfield($L, -2, cstr!("__wrapper"));
        lua::lua_pushcfunction($L, |cL| -> i32 {
            lua::lua_pushcclosure(cL, $wraper, 2);
            1
        });
        lua::lua_setfield($L, -2, cstr!("__getter"));
        1
    }}
}

macro_rules! lua_return_adapter {
    ($L:expr, $value:expr) => {{
        let vref = &$value as &dyn Any;
        if let Some(i) = vref.downcast_ref::<int>() {
            return *i;
        }
        $value.native_to_lua($L)
    }}
}

macro_rules! lua_read_upvalueindex {
    ($L:expr, $idx:expr) => {{
        let obj_raw: *mut void = lua::lua_touserdata($L, lua::lua_upvalueindex($idx));
        mem::transmute(obj_raw)
    }}
}

macro_rules! lua_read_function_args {
    ($L:expr) => {{
        let argc = lua::lua_gettop($L);
        let args = match LuaRead::lua_to_native($L, -argc) {
            Some(a) => a,
            _ => {
                let err_msg = format!("wrong parameter for call function arguments is {}", argc);
                lua::luaL_error($L, err_msg.as_ptr() as * const char);
            }
        };
        args
    }}
}

impl<O, M> LuaPush for ClassMemberWrapper<O, M> where M: LuaPush + LuaRead + Clone + 'static {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe {
            lua::lua_newtable(L);
            push_lua_wraper_address!(usize, L, self.offset);
            lua::lua_setfield(L, -2, cstr!("__wrapper"));
            lua::lua_pushcfunction(L, |cL: *mut lua_State| -> i32 {
                lua::lua_pushcclosure(cL, lua_getter_adapter::<Self, _, M>, 2);
                lua::lua_call(cL, 0, 1);
                1
            });
            lua::lua_setfield(L, -2, cstr!("__getter"));
            lua::lua_pushcfunction(L, |cL| -> i32 {
                lua::lua_pushvalue(cL, 1);
                lua::lua_pushvalue(cL, 2);
                lua::lua_pushcclosure(cL, lua_setter_adapter::<Self, _, M>, 2);
                lua::lua_pushvalue(cL, 3);
                lua::lua_call(cL, 1, 0);
                0
            });
            lua::lua_setfield(L, -2, cstr!("__setter"));
            1
        }
    }
}

macro_rules! lua_adapter {
    ($F:ty, $L:ident) => {{
        let data: &mut $F = lua_read_upvalueindex!($L, 1);
        let args = lua_read_function_args!($L);
        let ret_value = data.call(args);
        ret_value.native_to_lua($L)
    }}
}

macro_rules! lua_lua_adapter {
    ($F:ty, $L:ident) => {{
        let data: &mut $F = lua_read_upvalueindex!($L, 1);
        let args = lua_read_function_args!($L);
        let ret_value = data.call($L, args);
        lua_return_adapter!($L, ret_value)
    }}
}

macro_rules! lua_class_adapter {
    ($O:ty, $F:ty, $L:ident) => {{
        let data: &mut $F = lua_read_upvalueindex!($L, 1);
        let obj: &mut $O = lua_read_upvalueindex!($L, 2);
        let args = lua_read_function_args!($L);
        let ret_value = data.call(obj, args);
        ret_value.native_to_lua($L)
    }}
}

macro_rules! lua_classlua_adapter {
    ($O:ty, $F:ty, $L:expr) => {{
        let data: &mut $F = lua_read_upvalueindex!($L, 1);
        let obj: &mut $O = lua_read_upvalueindex!($L, 2);
        let args = lua_read_function_args!($L);
        let ret_value = data.call(obj, $L, args);
        lua_return_adapter!($L, ret_value)
    }}
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

        impl<Z, R $(,$p)*> FuncLuaAdapter<($($p,)*)> for FuncWrapper<Z, ($($p,)*), R> where Z: FnMut(*mut lua_State, $($p),*) -> R {
            type Output = R;
            fn call(&mut self, L: *mut lua_State, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)(L, $($p),*)
            }
        }

        impl<O, Z, R $(,$p)*> ClassFuncAdapter<O, ($($p,)*)> for ClassFuncWrapper<Z, O, ($($p,)*), R> where Z: FnMut(&O, $($p),*) -> R {
            type Output = R;
            fn call(&mut self, obj: &O, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)(obj, $($p),*)
            }
        }

        impl<O, Z, R $(,$p)*> ClassFuncMutAdapter<O, ($($p,)*)> for ClassFuncWrapper<Z, O, ($($p,)*), R> where Z: FnMut(&mut O, $($p),*) -> R {
            type Output = R;
            fn call(&mut self, obj: &mut O, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)(obj, $($p),*)
            }
        }

        impl<O, Z, R $(,$p)*> ClassLuaFuncAdapter<O, ($($p,)*)> for ClassFuncWrapper<Z, O, ($($p,)*), R>
            where Z: FnMut(&O, *mut lua_State, $($p),*) -> R {
            type Output = R;
            fn call(&mut self, obj: &O, L: *mut lua_State, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)(obj, L, $($p),*)
            }
        }

        impl<O, Z, R $(,$p)*> ClassLuaFuncMutAdapter<O, ($($p,)*)> for ClassFuncWrapper<Z, O, ($($p,)*), R>
            where Z: FnMut(&mut O, *mut lua_State, $($p),*) -> R {
            type Output = R;
            fn call(&mut self, obj: &mut O, L: *mut lua_State, params: ($($p,)*)) -> Self::Output {
                let ($($p,)*) = params;
                (self.function)(obj, L, $($p),*)
            }
        }

        impl<Z, R $(,$p: 'static)*> LuaPushFn for FuncWrapper<Z, ($($p,)*), R>
            where Z: FnMut($($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { push_global_function!(Z, L, self.function, lua_adapter::<Self, _, R>) }
            }
        }

        impl<Z, R $(,$p: 'static)*> LuaPushLuaFn for FuncWrapper<Z, ($($p,)*), R>
            where Z: FnMut(*mut lua_State, $($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { push_global_function!(Z, L, self.function, lua_lua_adapter::<Self, _, R>) }
            }
        }

        impl<O, Z, R $(,$p: 'static)*> LuaPushFn for ClassFuncWrapper<Z, O, ($($p,)*), R>
            where Z: FnMut(&O, $($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { push_class_function!(Z, L, self.function, lua_class_adapter::<Self, _, _, R>) }
            }
        }

        impl<O, Z, R $(,$p: 'static)*> LuaPushFnMut for ClassFuncWrapper<Z, O, ($($p,)*), R>
            where Z: FnMut(&mut O, $($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { push_class_function!(Z, L, self.function, lua_classmut_adapter::<Self, _, _, R>) }
            }
        }

        impl<O, Z, R $(,$p: 'static)*> LuaPushLuaFn for ClassFuncWrapper<Z, O, ($($p,)*), R>
            where Z: FnMut(&O, *mut lua_State, $($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { push_class_function!(Z, L, self.function, lua_classlua_adapter::<Self, _, _, R>) }
            }
        }

        impl<O, Z, R $(,$p: 'static)*> LuaPushLuaFnMut for ClassFuncWrapper<Z, O, ($($p,)*), R>
            where Z: FnMut(&mut O, *mut lua_State, $($p),*) -> R, ($($p,)*): LuaRead, R: LuaPush + 'static {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { push_class_function!(Z, L, self.function, lua_classluamut_adapter::<Self, _, _, R>) }
            }
        }
    )
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

fn lua_getter_adapter<T, O, M>(L: *mut lua_State) -> int where T: ClassMemberAdapter<O, M>, M: LuaPush + Clone {
    unsafe {
        let data: &mut T = lua_read_upvalueindex!(L, 1);
        let obj: &mut O = lua_read_upvalueindex!(L, 2);
        let ret_value = data.get(obj);
        ret_value.native_to_lua(L)
    }
}

fn lua_setter_adapter<T, O, M>(L: *mut lua_State) -> int where T: ClassMemberAdapter<O, M>, M: LuaPush + LuaRead + 'static {
    unsafe {
        let data: &mut T = lua_read_upvalueindex!(L, 1);
        let obj: &mut O = lua_read_upvalueindex!(L, 2);
        let args = lua_read_function_args!(L);
        data.set(obj, args);
        0
    }
}

fn lua_adapter<T, P, R>(L: *mut lua_State) -> int where T: FuncAdapter<P, Output = R>, P: LuaRead + 'static, R: LuaPush + 'static {
    unsafe { lua_adapter!(T, L) }
}

fn lua_lua_adapter<T, P, R>(L: *mut lua_State) -> int where T: FuncLuaAdapter<P, Output = R>, P: LuaRead + 'static, R: LuaPush + 'static {
    unsafe { lua_lua_adapter!(T, L) }
}

fn lua_class_adapter<T, O, P, R>(L: *mut lua_State) -> int
    where T: ClassFuncAdapter<O, P, Output = R>, P: LuaRead + 'static, R: LuaPush + 'static {
    unsafe { lua_class_adapter!(O, T, L) }
}

fn lua_classmut_adapter<T, O, P, R>(L: *mut lua_State) -> int
    where T: ClassFuncMutAdapter<O, P, Output = R>, P: LuaRead + 'static, R: LuaPush + 'static {
    unsafe { lua_class_adapter!(O, T, L) }
}

fn lua_classlua_adapter<T, O, P, R>(L: *mut lua_State) -> int
    where T: ClassLuaFuncAdapter<O, P, Output = R>, P: LuaRead + 'static, R: LuaPush + 'static {
    unsafe { lua_classlua_adapter!(O, T, L) }
}

fn lua_classluamut_adapter<T, O, P, R>(L: *mut lua_State) -> int
    where T: ClassLuaFuncMutAdapter<O, P, Output = R>, P: LuaRead + 'static, R: LuaPush + 'static {
    unsafe { lua_classlua_adapter!(O, T, L) }
}

//get global function
//-------------------------------------------------------------------------------
pub fn get_global_function(L: *mut lua_State, func: &str) -> bool {
    unsafe {
        lua::lua_getglobal(L, to_char!(func));
        return lua::lua_isfunction(L, -1);
    }
}

pub fn get_table_function(L: *mut lua_State, table: &str, func: &str) -> bool {
    unsafe {
        lua::lua_getglobal(L, to_char!(table));
        if !lua::lua_istable(L, -1) {
            return false;
        }
        lua::lua_getfield(L, -1, to_char!(func));
        lua::lua_remove(L, -2);
        return lua::lua_isfunction(L, -1);
    }
}

pub fn get_object_function<T>(L: *mut lua_State, obj: &mut T, func: &str) -> bool {
    unsafe {
        lua_push_userdata(L, obj as *mut T);
        if !lua::lua_istable(L, -1) {
            return false;
        }
        lua::lua_getfield(L, -1, to_char!(func));
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
            let error_msg = lua::to_utf8(lua::lua_tostring(L, -1));
            lua::lua_pop(L, 2);  // 弹出错误信息和 traceback
            return Err(error_msg);
        }
        lua::lua_remove(L, -(ret_count + 1));  // 移除 'traceback'
    }
    return Ok(true);
}

pub fn call_global_function(L: *mut lua_State, func: &str, arg_count: int, ret_count: int) -> Result<bool, String> {
    if !get_global_function(L, func) {
        return Err("function not found".to_string());
    }
    return lua_call_function(L, arg_count, ret_count);
}

pub fn call_table_function(L: *mut lua_State, table: &str, func: &str, arg_count: int, ret_count: int) -> Result<bool, String> {
    if !get_table_function(L, table, func) {
        return Err("function not found".to_string());
    }
    return lua_call_function(L, arg_count, ret_count);
}

#[macro_export]
macro_rules! set_function {
    ($lua:expr, $fname:expr, $func:expr) => (
        let wrapper = luakit::FuncWrapper { function: $func, marker: std::marker::PhantomData };
        wrapper.native_to_lua($lua.L());
        let wref = luakit::Reference::new($lua.L());
        $lua.set($fname, wref);
    )
}

#[macro_export]
macro_rules! lua_set_function {
    ($name:ident, $($p:ident),*) => (
        pub fn $name<Z, R $(, $p)*>(&mut self, fname: &str, f: Z)
            where Z: FnMut($($p),*) -> R, FuncWrapper<Z, ($($p,)*), R>: LuaPush {
            let wrapper = FuncWrapper { function: f, marker: std::marker::PhantomData };
            self.set(fname, wrapper);
        }
    )
}

#[macro_export]
macro_rules! call_function_impl {
    ($L:expr, $nret:expr $(, $arg:expr)*) => {{
        let mut argc = 0;
        $(argc += $arg.native_to_lua($L);)*
        let res = luakit::lua_call_function($L, argc, $nret);
        match res {
            Err(e) => Err(e),
            Ok(_) => {
                let mut ret = Vec::<luakit::Reference>::new();
                for i in 0..$nret {
                    ret.push(luakit::LuaRead::lua_to_native($L, -($nret - i)).unwrap());
                }
                lua::lua_pop($L, $nret);
                Ok(ret)
            }
        }
    }}
}

#[macro_export]
macro_rules! call {
    ($lua:expr, $func:expr, $nret:expr $(, $arg:expr)*) => {{
        if !$lua.get_function($func) {
            Err("function not found".to_string())
        } else {
            let L = $lua.L();
            luakit::call_function_impl!(L, $nret $(, $arg)*)
        }
    }}
}

#[macro_export]
macro_rules! table_call {
    ($lua:expr, $table:expr, $func:expr, $nret:expr $(, $arg:expr)*) => {{
        let L = $lua.L();
        if !$luakit::get_table_function(L, $table, $func) {
            Err("table function not found".to_string())
        } else {
            luakit::call_function_impl!(L, $nret $(, $arg)*)
        }
    }}
}

#[macro_export]
macro_rules! object_call {
    ($lua:expr, $obj:expr, $func:expr, $nret:expr $(, $arg:expr)*) => {{
        let L = $lua.L();
        if !luakit::get_object_function(L, $obj, $func) {
            Err("obj function not found".to_string())
        } else {
            luakit::call_function_impl!(L, $nret $(, $arg)*)
        }
    }}
}

#[macro_export]
macro_rules! call_function_inner {
    ($L:expr, $nret:expr $(, $arg:expr)*) => {{
        let mut argc = 0;
        $(argc += $arg.native_to_lua($L);)*
        let res = lua_call_function($L, argc, $nret);
        match res {
            Err(e) => Err(e),
            Ok(_) => {
                let mut ret = Vec::<Reference>::new();
                for i in 0..$nret {
                    ret.push(LuaRead::lua_to_native($L, -($nret - i)).unwrap());
                }
                lua::lua_pop($L, $nret);
                Ok(ret)
            }
        }
    }}
}

#[macro_export]
macro_rules! call_lua_warper {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub fn $name<$($p),*>(&mut self, func: &str, retc: i32, $($p : $p, )*) -> Result<Vec<Reference>, String> where $($p : LuaPush),* {
            if !self.get_function(func) {
                return Err("function not found".to_string());
            }
            call_function_inner!(self.m_L, retc $(, $p)*)
        }
    };
}

#[macro_export]
macro_rules! call_table_warper {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub fn $name<$($p),*>(&mut self, table: &str, func: &str, retc: i32 $(,$p : $p)*) -> Result<Vec<Reference>, String> where $($p : LuaPush),* {
            if !get_table_function(self.m_L, table, func) {
                return Err("function not found".to_string());
            }
            call_function_inner!(self.m_L, retc $(, $p)*)
        }
    };
}

#[macro_export]
macro_rules! call_object_warper {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub fn $name<T, $($p),*>(&mut self, obj: &mut T, func: &str, retc: i32 $(,$p : $p)*) -> Result<Vec<Reference>, String> where $($p : LuaPush),* {
            if !get_object_function(self.m_L, obj, func) {
                return Err("function not found".to_string());
            }
            call_function_inner!(self.m_L, retc $(, $p)*)
        }
    };
}

#[macro_export]
macro_rules! variadic_return {
    ($lua:ident, $($p:expr),+) => {{
        let mut argc = 0;
        $(argc += $p.native_to_lua($lua);)+
        argc
    }}
}

#[macro_export]
macro_rules! vector_return {
    ($L:expr, $vec:expr) => {
        unsafe {
            lua::lua_createtable($L, $vec.len() as i32, 0);
            for (i, item) in $vec.into_iter().enumerate() {
                item.native_to_lua($L);
                lua::lua_rawseti($L, -2, (i + 1) as i32);
            }
            1
        }
    }
}
