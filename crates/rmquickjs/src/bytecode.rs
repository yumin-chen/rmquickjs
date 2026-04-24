/// Checks if the given byte array is a valid MicroQuickJS bytecode.
pub fn is_bytecode(bin: &[u8]) -> bool {
    unsafe { rmquickjs_sys::JS_IsBytecode(bin.as_ptr(), bin.len()) == 1 }
}