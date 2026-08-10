//! A compiled shader pair, kept as SPIR-V until the renderer needs it.
//!
//! Ancorix never compiles shader source: the bytes come in already compiled,
//! from `glslangValidator`, `slangc`, or anything else that emits SPIR-V (see
//! `include_bytes!`). Nothing here knows or cares which language they started
//! as.

/// The first word of every SPIR-V module, little-endian.
const SPIRV_MAGIC: u32 = 0x0723_0203;

/// A fragment shader, addressed by `ancorix_asset::Handle<Shader>` once
/// registered.
///
/// Only the fragment stage: the engine supplies the vertex half, which is
/// what lets a user shader be one `pixel` function and nothing else.
pub struct Shader {
    fragment: Box<[u8]>,
}

impl Shader {
    /// Returns a [`Shader`] holding a copy of the SPIR-V module.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_shader::Shader;
    ///
    /// // a bare SPIR-V header is enough to be accepted; the driver is what
    /// // ultimately validates the module
    /// let spirv = [0x03, 0x02, 0x23, 0x07, 0, 0, 1, 0];
    /// let shader = Shader::new(&spirv);
    ///
    /// assert_eq!(shader.fragment().len(), 8);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the slice isn't SPIR-V - wrong magic number, or a length
    /// that isn't a whole number of 32-bit words.
    pub fn new(fragment: &[u8]) -> Self {
        check("fragment", fragment);

        Self {
            fragment: fragment.into(),
        }
    }

    /// Returns the fragment module's SPIR-V bytes.
    #[inline]
    pub fn fragment(&self) -> &[u8] {
        &self.fragment
    }
}

// Checked here rather than left to `ancorix_ash`, so that handing over a
// `.glsl` file or a truncated download says so, instead of surfacing as a
// driver error much later.
fn check(stage: &str, spirv: &[u8]) {
    assert!(
        spirv.len() >= 4 && spirv.len().is_multiple_of(4),
        "{stage} shader is {} bytes - SPIR-V is a stream of 32-bit words",
        spirv.len()
    );

    let magic = u32::from_le_bytes([spirv[0], spirv[1], spirv[2], spirv[3]]);
    assert!(
        magic == SPIRV_MAGIC,
        "{stage} shader doesn't start with the SPIR-V magic number - \
         compile it first (glslangValidator -V, slangc -target spirv, ...)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: [u8; 8] = [0x03, 0x02, 0x23, 0x07, 0, 0, 1, 0];

    #[test]
    fn keeps_the_module() {
        assert_eq!(Shader::new(&HEADER).fragment(), HEADER);
    }

    #[test]
    #[should_panic(expected = "magic number")]
    fn rejects_non_spirv() {
        // 12 bytes, so it clears the word-length check and reaches the magic
        Shader::new(b"#version 450");
    }

    #[test]
    #[should_panic(expected = "32-bit words")]
    fn rejects_truncated_words() {
        Shader::new(&HEADER[..5]);
    }
}
