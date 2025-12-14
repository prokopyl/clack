use super::*;

impl PluginAsAuv2Factory<'_> {
    #[inline]
    pub fn auv2_info(&self, index: u32) -> Option<PluginInfoAsAUv2> {
        let get_info = self.0.get().get_auv2_info?;
        let mut info = PluginInfoAsAUv2::empty();

        // SAFETY:
        // - This type guarantees the factory pointer is valid
        // - We got the function from the same instance the pointer points to
        // - The info buf pointer comes from a &mut reference to the above local, so it is guaranteed to be valid
        if unsafe { get_info(self.0.as_ptr(), index, info.as_raw_mut()) } {
            Some(info)
        } else {
            None
        }
    }
}
