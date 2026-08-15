#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

_Static_assert(sizeof(size_t) == sizeof(uint64_t),
               "Aero's allocator ABI requires a 64-bit size_t");

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
