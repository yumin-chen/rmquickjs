use crate::*;

pub const fn JS_VALUE_GET_SPECIAL_VALUE(v: JSValue) -> JSValue {
    (v) >> JS_TAG_SPECIAL_BITS
}

pub const fn JS_VALUE_GET_SPECIAL_TAG(v: JSValue) -> JSValue {
    (v) & (1 << JS_TAG_SPECIAL_BITS) - 1
}

pub const fn JS_VALUE_MAKE_SPECIAL(tag: u32, v: JSValue) -> JSValue {
    (tag as u64) | ((v) << JS_TAG_SPECIAL_BITS)
}

pub const fn JS_NULL() -> JSValue {
    JS_VALUE_MAKE_SPECIAL(JS_TAG_NULL, 0)
}

pub const fn JS_UNDEFINED() -> JSValue {
    JS_VALUE_MAKE_SPECIAL(JS_TAG_UNDEFINED, 0)
}

pub const fn JS_UNINITIALIZED() -> JSValue {
    JS_VALUE_MAKE_SPECIAL(JS_TAG_UNINITIALIZED, 0)
}

pub const fn JS_FALSE() -> JSValue {
    JS_VALUE_MAKE_SPECIAL(JS_TAG_BOOL, 0)
}

pub const fn JS_TRUE() -> JSValue {
    JS_VALUE_MAKE_SPECIAL(JS_TAG_BOOL, 1)
}

pub const fn JS_EXCEPTION() -> JSValue {
    JS_VALUE_MAKE_SPECIAL(JS_TAG_EXCEPTION, JS_EX_NORMAL as JSValue)
}

pub const fn JS_IsInt(v: JSValue) -> bool {
    (v & 1) == JS_TAG_INT as u64
}

pub const fn JS_IsPtr(v: JSValue) -> bool {
    (v & (JSW - 1) as u64) == JS_TAG_PTR as u64
}

pub const fn JS_IsBool(v: JSValue) -> bool {
    JS_VALUE_GET_SPECIAL_TAG(v) == JS_TAG_BOOL as u64
}
