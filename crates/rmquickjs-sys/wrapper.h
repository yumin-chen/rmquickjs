#include <stddef.h>
#include <stdint.h>

#include "mquickjs/mquickjs.h"

extern const JSSTDLibraryDef js_stdlib;

typedef JSValue (*JSHostCallback)(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv, JSValue params);
void JS_SetHostCallback(JSHostCallback callback);
void* JS_GetContextOpaque(JSContext *ctx);
