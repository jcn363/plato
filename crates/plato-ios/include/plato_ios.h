// Plato iOS C API Header
// This header defines the C interface for Swift to call into the iOS-specific Rust library
//
// Swift can call these functions using the @convention(c) attribute
// These functions are implemented in crates/plato-ios/src/lib.rs

#ifndef PLATO_IOS_H
#define PLATO_IOS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Initialize the Plato core
 * 
 * @param width Screen width in pixels
 * @param height Screen height in pixels
 * @return true on success, false on failure
 */
bool plato_init(uint32_t width, uint32_t height);

/**
 * Render the current view to a caller-provided buffer
 * 
 * @param buffer_ptr Pointer to pixel buffer (RGBA8888 format)
 * @param len Length of buffer in bytes (must be width * height * 4)
 * @return true on success, false on failure
 */
bool plato_render(uint8_t* buffer_ptr, size_t len);

/**
 * Handle touch down event
 * 
 * @param id Touch finger identifier
 * @param x Touch X coordinate in pixels
 * @param y Touch Y coordinate in pixels
 */
void plato_touch_down(int32_t id, int32_t x, int32_t y);

/**
 * Handle touch move event
 * 
 * @param id Touch finger identifier
 * @param x Touch X coordinate in pixels
 * @param y Touch Y coordinate in pixels
 */
void plato_touch_move(int32_t id, int32_t x, int32_t y);

/**
 * Handle touch up event
 * 
 * @param id Touch finger identifier
 * @param x Touch X coordinate in pixels
 * @param y Touch Y coordinate in pixels
 */
void plato_touch_up(int32_t id, int32_t x, int32_t y);

/**
 * Resize the framebuffer to new dimensions
 * 
 * @param width New screen width in pixels
 * @param height New screen height in pixels
 * @return true on success, false on failure
 */
bool plato_resize(uint32_t width, uint32_t height);

/**
 * Cleanup resources
 * Should be called when the app is terminating
 */
void plato_deinit(void);

#ifdef __cplusplus
}
#endif

#endif // PLATO_IOS_H
