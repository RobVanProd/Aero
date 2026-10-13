#include <stdint.h>
#include <stdio.h>

#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#include <stdlib.h>
#include <windows.h>
#else
#include <signal.h>
#endif

/* H1E-RUNTIME-001: diagnostic bytes have their own sticky channel state. */
static int32_t aero_stderr_status = 0;
#ifdef _WIN32
static int aero_stderr_mode_initialized = 0;

/* Only these recoverable descriptor operations bypass UCRT's default abort.
   Keep the process-global handler and the caller's thread handler unchanged. */
static void __cdecl aero_stderr_invalid_parameter(const wchar_t *expression,
                                                 const wchar_t *function,
                                                 const wchar_t *file,
                                                 unsigned int line,
                                                 uintptr_t reserved) {
    (void)expression;
    (void)function;
    (void)file;
    (void)line;
    (void)reserved;
}

static int aero_stderr_set_binary_mode(void) {
    _invalid_parameter_handler previous =
        _set_thread_local_invalid_parameter_handler(aero_stderr_invalid_parameter);
    int result = _setmode(_fileno(stderr), _O_BINARY);
    _set_thread_local_invalid_parameter_handler(previous);
    return result;
}

static int aero_stderr_write_and_flush(unsigned char value) {
    _invalid_parameter_handler previous =
        _set_thread_local_invalid_parameter_handler(aero_stderr_invalid_parameter);
    int failed = fputc(value, stderr) == EOF || fflush(stderr) == EOF;
    _set_thread_local_invalid_parameter_handler(previous);
    return failed;
}
#else
static int aero_stderr_signal_initialized = 0;
#endif

int32_t aero_stderr_write_byte(int32_t value) {
    if (aero_stderr_status < 0) {
        return aero_stderr_status;
    }
    if (value < 0 || value > 255) {
        aero_stderr_status = -3;
        return aero_stderr_status;
    }

#ifdef _WIN32
    if (!aero_stderr_mode_initialized) {
        HANDLE stderr_handle = GetStdHandle(STD_ERROR_HANDLE);
        SetLastError(NO_ERROR);
        DWORD stderr_kind = stderr_handle == NULL || stderr_handle == INVALID_HANDLE_VALUE
                                ? FILE_TYPE_UNKNOWN
                                : GetFileType(stderr_handle);
        if (stderr_handle == NULL || stderr_handle == INVALID_HANDLE_VALUE ||
            (stderr_kind == FILE_TYPE_UNKNOWN && GetLastError() != NO_ERROR) ||
            aero_stderr_set_binary_mode() == -1) {
            aero_stderr_status = -2;
            return aero_stderr_status;
        }
        aero_stderr_mode_initialized = 1;
    }
#else
    if (!aero_stderr_signal_initialized) {
        if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) {
            aero_stderr_status = -1;
            return aero_stderr_status;
        }
        aero_stderr_signal_initialized = 1;
    }
#endif

#ifdef _WIN32
    int failed = aero_stderr_write_and_flush((unsigned char)value);
#else
    int failed = fputc((unsigned char)value, stderr) == EOF || fflush(stderr) == EOF;
#endif
    if (failed) {
        aero_stderr_status = -1;
        return aero_stderr_status;
    }
    return 0;
}
