#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#include <windows.h>
#else
#include <signal.h>
#endif

_Static_assert(sizeof(size_t) == sizeof(uint64_t),
               "Aero's allocator ABI requires a 64-bit size_t");

static int32_t aero_stdin_status = 0;
static int32_t aero_stdout_status = 0;
#ifdef _WIN32
static int aero_stdin_mode_initialized = 0;
static int aero_stdout_mode_initialized = 0;
#else
static int aero_stdout_signal_initialized = 0;
#endif

int32_t aero_stdin_read_byte(void) {
    if (aero_stdin_status < 0) {
        return aero_stdin_status;
    }

#ifdef _WIN32
    if (!aero_stdin_mode_initialized) {
        HANDLE stdin_handle = GetStdHandle(STD_INPUT_HANDLE);
        SetLastError(NO_ERROR);
        DWORD stdin_kind = stdin_handle == NULL || stdin_handle == INVALID_HANDLE_VALUE
                               ? FILE_TYPE_UNKNOWN
                               : GetFileType(stdin_handle);
        if (stdin_handle == NULL || stdin_handle == INVALID_HANDLE_VALUE ||
            (stdin_kind == FILE_TYPE_UNKNOWN && GetLastError() != NO_ERROR) ||
            _setmode(_fileno(stdin), _O_BINARY) == -1) {
            aero_stdin_status = -2;
            return aero_stdin_status;
        }
        aero_stdin_mode_initialized = 1;
    }
#endif

    int byte = fgetc(stdin);
    if (byte != EOF) {
        return (int32_t)byte;
    }
    aero_stdin_status = feof(stdin) ? -1 : -2;
    return aero_stdin_status;
}

int32_t aero_stdout_write_byte(int32_t value) {
    if (aero_stdout_status < 0) {
        return aero_stdout_status;
    }
    if (value < 0 || value > 255) {
        aero_stdout_status = -3;
        return aero_stdout_status;
    }

#ifdef _WIN32
    if (!aero_stdout_mode_initialized) {
        HANDLE stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
        SetLastError(NO_ERROR);
        DWORD stdout_kind = stdout_handle == NULL || stdout_handle == INVALID_HANDLE_VALUE
                                ? FILE_TYPE_UNKNOWN
                                : GetFileType(stdout_handle);
        if (stdout_handle == NULL || stdout_handle == INVALID_HANDLE_VALUE ||
            (stdout_kind == FILE_TYPE_UNKNOWN && GetLastError() != NO_ERROR) ||
            _setmode(_fileno(stdout), _O_BINARY) == -1) {
            aero_stdout_status = -2;
            return aero_stdout_status;
        }
        aero_stdout_mode_initialized = 1;
    }
#else
    if (!aero_stdout_signal_initialized) {
        if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) {
            aero_stdout_status = -1;
            return aero_stdout_status;
        }
        aero_stdout_signal_initialized = 1;
    }
#endif

    if (fputc((unsigned char)value, stdout) == EOF || fflush(stdout) == EOF) {
        aero_stdout_status = -1;
        return aero_stdout_status;
    }
    return 0;
}

void *aero_alloc(uint64_t size) {
    if (size == 0) {
        return NULL;
    }
    return malloc((size_t)size);
}

void *aero_realloc(void *old, uint64_t old_size, uint64_t new_size) {
    if (old == NULL || old_size == 0 || new_size == 0) {
        return NULL;
    }
    (void)old_size;
    return realloc(old, (size_t)new_size);
}

void aero_dealloc(void *allocation, uint64_t size) {
    if (allocation == NULL || size == 0) {
        return;
    }
    (void)size;
    free(allocation);
}
