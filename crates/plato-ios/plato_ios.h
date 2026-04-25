// Plato iOS C API Header
// This header defines the C interface for Swift to call into the Rust library
//
// Swift can call these functions using the @convention(c) attribute

#ifndef PLATO_IOS_H
#define PLATO_IOS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque context pointer (Context is defined in Rust)
typedef struct Context Context;

/**
 * Initialize the Plato iOS app
 * 
 * @param width Screen width in pixels
 * @param height Screen height in pixels
 * @param library_path_ptr Pointer to library path UTF-8 string (can be null)
 * @param library_path_len Length of library path string
 * @param settings_path_ptr Pointer to settings path UTF-8 string (can be null)
 * @param settings_path_len Length of settings path string
 * @return true on success, false on failure
 */
bool plato_init(
    uint32_t width,
    uint32_t height,
    const uint8_t* library_path_ptr,
    size_t library_path_len,
    const uint8_t* settings_path_ptr,
    size_t settings_path_len
);

/**
 * Get the global context pointer
 * 
 * @return Pointer to the Context struct, or null if not initialized
 */
Context* plato_get_context(void);

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
 * Render the current view to a caller-provided buffer
 * Should be called on each frame or when rendering is needed
 * 
 * @param buffer_ptr Pointer to pixel buffer (RGBA8888 format)
 * @param len Length of buffer in bytes (must be width * height * 4)
 * @return true on success, false on failure
 */
bool plato_render(uint8_t* buffer_ptr, size_t len);

/**
 * Cleanup resources
 * Should be called when the app is terminating
 */
void plato_deinit(void);

#ifdef __cplusplus
}
#endif

#endif // PLATO_IOS_H
