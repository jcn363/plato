use crate::font::freetype_sys::*;
use anyhow::{bail, Error};
use std::path::Path;
use std::ptr;

pub struct Library {
    lib: *mut FtLibrary,
}

impl Library {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let mut lib: *mut FtLibrary = ptr::null_mut();
            let result = FT_Init_FreeType(&mut lib);
            if result != FT_ERR_OK {
                bail!("FreeType initialization failed: {}", result);
            }
            Ok(Library { lib })
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut FtLibrary {
        self.lib
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        if !self.lib.is_null() {
            unsafe { FT_Done_FreeType(self.lib) };
        }
    }
}

pub struct Face {
    face: *mut FtFace,
    lib: *mut FtLibrary,
}

impl Face {
    pub fn from_path(lib: &Library, path: &Path, index: i32) -> Result<Self, Error> {
        let path_cstr = std::ffi::CString::new(
            path.to_str()
                .ok_or_else(|| anyhow::format_err!("invalid path"))?,
        )?;

        unsafe {
            let mut face: *mut FtFace = ptr::null_mut();
            let result = FT_New_Face(
                lib.as_ptr(),
                path_cstr.as_ptr(),
                index as libc::c_long,
                &mut face,
            );
            if result != FT_ERR_OK {
                bail!("Failed to load font {}: {}", path.display(), result);
            }
            Ok(Face {
                face,
                lib: lib.as_ptr(),
            })
        }
    }

    pub fn from_memory(lib: &Library, data: &[u8], index: i32) -> Result<Self, Error> {
        unsafe {
            let mut face: *mut FtFace = ptr::null_mut();
            let result = FT_New_Memory_Face(
                lib.as_ptr(),
                data.as_ptr(),
                data.len() as libc::c_long,
                index as libc::c_long,
                &mut face,
            );
            if result != FT_ERR_OK {
                bail!("Failed to load font from memory: {}", result);
            }
            Ok(Face {
                face,
                lib: lib.as_ptr(),
            })
        }
    }

    pub fn set_char_size(
        &self,
        width: i32,
        height: i32,
        horz_resolution: u32,
        vert_resolution: u32,
    ) -> Result<(), Error> {
        unsafe {
            let result = FT_Set_Char_Size(
                self.face,
                width as FtF26Dot6,
                height as FtF26Dot6,
                horz_resolution,
                vert_resolution,
            );
            if result != FT_ERR_OK {
                bail!("Failed to set char size: {}", result);
            }
            Ok(())
        }
    }

    pub fn set_pixel_sizes(&self, width: u32, height: u32) -> Result<(), Error> {
        unsafe {
            let result = FT_Set_Pixel_Sizes(self.face, width, height);
            if result != FT_ERR_OK {
                bail!("Failed to set pixel sizes: {}", result);
            }
            Ok(())
        }
    }

    pub fn load_glyph(&self, glyph_index: u32, flags: i32) -> Result<(), Error> {
        unsafe {
            let result = FT_Load_Glyph(self.face, glyph_index, flags);
            if result != FT_ERR_OK {
                bail!("Failed to load glyph {}: {}", glyph_index, result);
            }
            Ok(())
        }
    }

    pub fn load_char(&self, char_code: u32, flags: i32) -> Result<(), Error> {
        unsafe {
            let result = FT_Load_Char(self.face, char_code as libc::c_ulong, flags);
            if result != FT_ERR_OK {
                bail!("Failed to load char {}: {}", char_code, result);
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_char_index(&self, char_code: u32) -> u32 {
        unsafe { FT_Get_Char_Index(self.face, char_code as libc::c_ulong) }
    }

    #[inline]
    pub fn glyph(&self) -> &FtGlyphSlot {
        unsafe { &*(*self.face).glyph }
    }

    #[inline]
    pub fn face_ptr(&self) -> *mut FtFace {
        self.face
    }

    #[inline]
    pub fn size(&self) -> &FtSize {
        unsafe { &*(*self.face).size }
    }

    pub fn family_name(&self) -> Option<String> {
        unsafe {
            let name = (*self.face).family_name;
            if name.is_null() {
                None
            } else {
                std::ffi::CStr::from_ptr(name)
                    .to_str()
                    .map(|s| s.to_string())
                    .ok()
            }
        }
    }

    pub fn style_name(&self) -> Option<String> {
        unsafe {
            let name = (*self.face).style_name;
            if name.is_null() {
                None
            } else {
                std::ffi::CStr::from_ptr(name)
                    .to_str()
                    .map(|s| s.to_string())
                    .ok()
            }
        }
    }

    #[inline]
    pub fn num_glyphs(&self) -> i32 {
        unsafe { (*self.face).num_glyphs as i32 }
    }

    #[inline]
    pub fn units_per_em(&self) -> u16 {
        unsafe { (*self.face).units_per_em }
    }

    #[inline]
    pub fn ascender(&self) -> i16 {
        unsafe { (*self.face).ascender }
    }

    #[inline]
    pub fn descender(&self) -> i16 {
        unsafe { (*self.face).descender }
    }

    #[inline]
    pub fn x_ppem(&self) -> u16 {
        unsafe { (*(*self.face).size).metrics.x_ppem as u16 }
    }

    #[inline]
    pub fn ascender_scaled(&self) -> i32 {
        unsafe { (*(*self.face).size).metrics.ascender as i32 / 64 }
    }

    #[inline]
    pub fn descender_scaled(&self) -> i32 {
        unsafe { (*(*self.face).size).metrics.descender as i32 / 64 }
    }

    #[inline]
    pub fn height_scaled(&self) -> i32 {
        unsafe { (*(*self.face).size).metrics.height as i32 / 64 }
    }

    #[inline]
    pub fn bbox(&self) -> FtBBox {
        unsafe { (*self.face).bbox }
    }

    pub fn get_mm_var(&self) -> Result<MmVar, Error> {
        unsafe {
            let mut varia: *mut FtMmVar = ptr::null_mut();
            let result = FT_Get_MM_Var(self.face, &mut varia);
            if result != FT_ERR_OK {
                bail!("Failed to get MM variable: {}", result);
            }
            Ok(MmVar {
                mmvar: varia,
                lib: self.lib,
            })
        }
    }

    pub fn set_var_design_coordinates(&self, coords: &[i32]) -> Result<(), Error> {
        unsafe {
            let result = FT_Set_Var_Design_Coordinates(
                self.face,
                coords.len() as u32,
                coords.as_ptr() as *const FtFixed,
            );
            if result != FT_ERR_OK {
                bail!("Failed to set design coordinates: {}", result);
            }
            Ok(())
        }
    }

    pub fn get_sfnt_name_count(&self) -> u32 {
        unsafe { FT_Get_Sfnt_Name_Count(self.face) }
    }

    pub fn get_sfnt_name(&self, index: u32) -> Option<FtSfntName> {
        unsafe {
            let mut name: FtSfntName = std::mem::zeroed();
            let result = FT_Get_Sfnt_Name(self.face, index, &mut name);
            if result != FT_ERR_OK {
                return None;
            }
            Some(name)
        }
    }
}

impl Drop for Face {
    fn drop(&mut self) {
        if !self.face.is_null() {
            unsafe { FT_Done_Face(self.face) };
        }
    }
}

pub struct MmVar {
    mmvar: *mut FtMmVar,
    lib: *mut FtLibrary,
}

impl MmVar {
    pub fn num_axis(&self) -> u32 {
        unsafe { (*self.mmvar).num_axis }
    }

    pub fn num_namedstyles(&self) -> u32 {
        unsafe { (*self.mmvar).num_namedstyles }
    }

    pub fn axis(&self) -> &[FtVarAxis] {
        unsafe {
            let num_axis = (*self.mmvar).num_axis;
            let axis_ptr = (*self.mmvar).axis;
            std::slice::from_raw_parts(axis_ptr, num_axis as usize)
        }
    }

    pub fn namedstyle(&self) -> &[FtNamedStyle] {
        unsafe {
            let num_namedstyles = (*self.mmvar).num_namedstyles;
            let style_ptr = (*self.mmvar).namedstyle;
            std::slice::from_raw_parts(style_ptr, num_namedstyles as usize)
        }
    }
}

impl Drop for MmVar {
    fn drop(&mut self) {
        if !self.mmvar.is_null() {
            unsafe { FT_Done_MM_Var(self.lib, self.mmvar) };
        }
    }
}
