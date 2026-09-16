#include <stdint.h>
#include <stdio.h>

#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#include <windows.h>
#else
#include <signal.h>
#endif

/* H1E-RUNTIME-001: diagnostic bytes have their own sticky channel state. */
static int32_t aero_stderr_status = 0;
#ifdef _WIN32
static int aero_stderr_mode_initialized = 0;
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
            _setmode(_fileno(stderr), _O_BINARY) == -1) {
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

    if (fputc((unsigned char)value, stderr) == EOF || fflush(stderr) == EOF) {
        aero_stderr_status = -1;
        return aero_stderr_status;
    }
    return 0;
}
