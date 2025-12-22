/*
 * Typhon Runtime Library - Minimal Stub Implementation
 *
 * This file provides minimal stub implementations of runtime functions
 * to enable linking and testing of the Typhon compiler pipeline.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

/* Type definitions for Typhon runtime objects */
typedef struct {
    int64_t value;
    /* TODO: include
     * - Reference count
     * - Type tag
     * - Additional metadata
     */
} TyphonInt;

/*
 * typhon_int_new - Create a new integer object
 *
 * STUB: Just returns the value directly as an int64_t.
 *
 * TODO:
 * - Allocate TyphonInt object on heap
 * - Initialize reference count
 * - Set type tag
 * - Return pointer to object
 */
int64_t typhon_int_new(int64_t value) {
    return value;
}

/*
 * typhon_add - Add two integer objects
 *
 * STUB: Just adds the values directly.
 *
 * TODO:
 * - Check object types
 * - Handle overflow
 * - Allocate result object
 * - Manage reference counts
 */
int64_t typhon_add(int64_t a, int64_t b) {
    return a + b;
}

/*
 * typhon_print - Print an integer value
 *
 * STUB: Simple printf wrapper.
 *
 * TODO:
 * - Check object type
 * - Call appropriate __str__ method
 * - Handle different types (int, str, float, etc.)
 */
void typhon_print(int64_t value) {
    printf("%lld\n", (long long)value);
}
