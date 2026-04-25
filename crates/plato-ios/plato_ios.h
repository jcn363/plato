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
 * Handle a touch event from iOS
 * 
 * @param finger_id Unique identifier for the finger
 * @param x Touch X coordinate
 * @param y Touch Y coordinate
 * @param status Touch status: 0=Down, 1=Motion, 2=Up
 */
void plato_handle_touch(
    int32_t finger_id,
    float x,
    float y,
    int32_t status
);

/**
 * Render the current view
 * Should be called on each frame or when rendering is needed
 * 
 * @return true on success, false on failure
 */
bool plato_render(void);

/**
 * Cleanup resources
 * Should be called when the app is terminating
 */
void plato_cleanup(void);

#ifdef __cplusplus
}
#endif

#endif // PLATO_IOS_H
