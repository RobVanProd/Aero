#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

_Static_assert(sizeof(size_t) == sizeof(uint64_t),
               "Aero's allocator ABI requires a 64-bit size_t");

typedef union AeroTestHeader {
    max_align_t alignment;
    struct {
        uint64_t size;
        uint64_t magic;
    } metadata;
} AeroTestHeader;

static const uint64_t AERO_TEST_MAGIC = UINT64_C(0xa30b17e5c0deface);

static uint64_t alloc_calls;
static uint64_t realloc_calls;
static uint64_t dealloc_calls;
static uint64_t live_allocations;
static uint64_t size_mismatch_calls;
static uint64_t successful_allocation_events;
static uint64_t fail_after_successes = UINT64_MAX;

static int valid_payload_size(uint64_t size) {
    return size != 0 && size <= (uint64_t)(SIZE_MAX - sizeof(AeroTestHeader));
}

static int allocation_should_fail(void) {
    return successful_allocation_events >= fail_after_successes;
}

static AeroTestHeader *header_for(void *allocation) {
    return ((AeroTestHeader *)allocation) - 1;
}

static int header_matches(AeroTestHeader *header, uint64_t size) {
    return header->metadata.magic == AERO_TEST_MAGIC &&
           header->metadata.size == size;
}

void *aero_alloc(uint64_t size) {
    ++alloc_calls;
    if (!valid_payload_size(size) || allocation_should_fail()) {
        return NULL;
    }

    AeroTestHeader *header =
        (AeroTestHeader *)malloc(sizeof(AeroTestHeader) + (size_t)size);
    if (header == NULL) {
        return NULL;
    }
    header->metadata.size = size;
    header->metadata.magic = AERO_TEST_MAGIC;
    ++successful_allocation_events;
    ++live_allocations;
    return (void *)(header + 1);
}

void *aero_realloc(void *old, uint64_t old_size, uint64_t new_size) {
    ++realloc_calls;
    if (old == NULL || !valid_payload_size(old_size) ||
        !valid_payload_size(new_size)) {
        ++size_mismatch_calls;
        return NULL;
    }

    AeroTestHeader *old_header = header_for(old);
    if (!header_matches(old_header, old_size)) {
        ++size_mismatch_calls;
        return NULL;
    }
    if (allocation_should_fail()) {
        return NULL;
    }

    AeroTestHeader *new_header = (AeroTestHeader *)realloc(
        old_header, sizeof(AeroTestHeader) + (size_t)new_size);
    if (new_header == NULL) {
        return NULL;
    }
    new_header->metadata.size = new_size;
    new_header->metadata.magic = AERO_TEST_MAGIC;
    ++successful_allocation_events;
    return (void *)(new_header + 1);
}

void aero_dealloc(void *allocation, uint64_t size) {
    ++dealloc_calls;
    if (allocation == NULL || !valid_payload_size(size)) {
        ++size_mismatch_calls;
        return;
    }

    AeroTestHeader *header = header_for(allocation);
    if (!header_matches(header, size)) {
        ++size_mismatch_calls;
        return;
    }
    header->metadata.magic = 0;
    free(header);
    --live_allocations;
}

int32_t aero_test_reset(uint64_t requested_fail_after_successes) {
    if (live_allocations != 0) {
        return 0;
    }
    alloc_calls = 0;
    realloc_calls = 0;
    dealloc_calls = 0;
    size_mismatch_calls = 0;
    successful_allocation_events = 0;
    fail_after_successes = requested_fail_after_successes;
    return 1;
}

uint64_t aero_test_alloc_calls(void) { return alloc_calls; }

uint64_t aero_test_realloc_calls(void) { return realloc_calls; }

uint64_t aero_test_dealloc_calls(void) { return dealloc_calls; }

uint64_t aero_test_live_allocations(void) { return live_allocations; }

uint64_t aero_test_size_mismatch_calls(void) { return size_mismatch_calls; }
