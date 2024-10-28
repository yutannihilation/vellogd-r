
#include <stdint.h>
#include <Rinternals.h>
#include <R_ext/Parse.h>

#include "rust/api.h"

static uintptr_t TAGGED_POINTER_MASK = (uintptr_t)1;

SEXP handle_result(SEXP res_) {
    uintptr_t res = (uintptr_t)res_;

    // An error is indicated by tag.
    if ((res & TAGGED_POINTER_MASK) == 1) {
        // Remove tag
        SEXP res_aligned = (SEXP)(res & ~TAGGED_POINTER_MASK);

        // Currently, there are two types of error cases:
        //
        //   1. Error from Rust code
        //   2. Error from R's C API, which is caught by R_UnwindProtect()
        //
        if (TYPEOF(res_aligned) == CHARSXP) {
            // In case 1, the result is an error message that can be passed to
            // Rf_errorcall() directly.
            Rf_errorcall(R_NilValue, "%s", CHAR(res_aligned));
        } else {
            // In case 2, the result is the token to restart the
            // cleanup process on R's side.
            R_ContinueUnwind(res_aligned);
        }
    }

    return (SEXP)res;
}

SEXP savvy_vellogd_impl__impl(SEXP c_arg__filename, SEXP c_arg__width, SEXP c_arg__height) {
    SEXP res = savvy_vellogd_impl__ffi(c_arg__filename, c_arg__width, c_arg__height);
    return handle_result(res);
}

SEXP savvy_save_as_png__impl(SEXP c_arg__filename) {
    SEXP res = savvy_save_as_png__ffi(c_arg__filename);
    return handle_result(res);
}

SEXP savvy_vellogd_with_server_impl__impl(SEXP c_arg__filename, SEXP c_arg__width, SEXP c_arg__height, SEXP c_arg__server) {
    SEXP res = savvy_vellogd_with_server_impl__ffi(c_arg__filename, c_arg__width, c_arg__height, c_arg__server);
    return handle_result(res);
}

SEXP savvy_debuggd__impl(void) {
    SEXP res = savvy_debuggd__ffi();
    return handle_result(res);
}

SEXP savvy_do_tracing__impl(SEXP c_arg__expr) {
    SEXP res = savvy_do_tracing__ffi(c_arg__expr);
    return handle_result(res);
}


static const R_CallMethodDef CallEntries[] = {
    {"savvy_vellogd_impl__impl", (DL_FUNC) &savvy_vellogd_impl__impl, 3},
    {"savvy_save_as_png__impl", (DL_FUNC) &savvy_save_as_png__impl, 1},
    {"savvy_vellogd_with_server_impl__impl", (DL_FUNC) &savvy_vellogd_with_server_impl__impl, 4},
    {"savvy_debuggd__impl", (DL_FUNC) &savvy_debuggd__impl, 0},
    {"savvy_do_tracing__impl", (DL_FUNC) &savvy_do_tracing__impl, 1},
    {NULL, NULL, 0}
};

void R_init_vellogd(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);

    // Functions for initialzation, if any.

}
