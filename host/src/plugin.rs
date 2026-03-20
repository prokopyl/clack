//! Core types and traits to instantiate and interact with CLAP plugins.

use crate::prelude::*;
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::Arc;

mod error;
mod handle;
pub(crate) mod instance;

pub use error::PluginInstanceError;
pub use handle::*;
use instance::*;

pub use clack_common::plugin::*;

/// A plugin instance.
///
/// This type is generic over `H`, the type of the host callback handlers, which the plugin can
/// call use directly when its own methods are called by the host.
pub struct PluginInstance<H: HostHandlers> {
    pub(crate) inner: ManuallyDrop<Arc<PluginInstanceInner<H>>>,
    _no_send: PhantomData<*const ()>,
}

impl<H: HostHandlers> PluginInstance<H> {
    /// Creates a new plugin instance from a given [`PluginEntry`].
    ///
    /// The specific plugin to be instantiated is identified by the given `plugin_id`.
    ///
    /// This method takes two closures, here named `FS` and `FH`, which will respectively create
    /// the [`Shared`](HostHandlers::Shared) and [`MainThread`](HostHandlers::MainThread) callback
    /// handler instances associated with this plugin instance.
    ///
    /// The [`MainThread`](HostHandlers::MainThread)-producing callback takes a long-lived reference
    /// to its [`Shared`](HostHandlers::Shared) counterpart, so that it can be accessed at any time
    /// from the main thread if needed.
    ///
    /// The [`Shared`](HostHandlers::Shared)-producing callback takes a unit reference `&()`. It is
    /// not used for anything, but is necessary for proper lifetime binding.
    ///
    /// This method also takes a reference to a [`HostInfo`], giving the plugin some metadata about
    /// the host itself.
    ///
    /// # Errors
    ///
    /// If for some reason the given [`PluginEntry`] does not expose a [`PluginFactory`](crate::factory::plugin::PluginFactory),
    /// then this will return [`PluginInstanceError::MissingPluginFactory`].
    ///
    /// If the `PluginFactory`'s `create_plugin` function pointer is NULL,
    /// this will return [`PluginInstanceError::NullFactoryCreatePluginFunction`].
    ///
    /// If the given `plugin_id` does not exist, or if more generally the
    /// `PluginFactory`'s `create_plugin` function returned NULL,
    /// then this will return [`PluginInstanceError::PluginNotFound`].
    ///
    /// Otherwise, if the plugin instantiation implementation failed, it will return
    /// [`PluginInstanceError::InstantiationFailed`].
    ///
    /// # Example
    ///
    /// ```
    /// use clack_host::prelude::*;
    ///
    /// struct MyHost;
    ///
    /// struct MyHostMainThread { /* ... */ }
    ///
    /// impl HostHandlers for MyHost {
    ///     type Shared<'a> = /* ... */
    /// # ();
    ///     type AudioProcessor<'a> = /* ... */
    /// # ();
    ///     type MainThread<'a> = MyHostMainThread;
    /// }
    ///
    /// impl<'a> MainThreadHandler<'a> for MyHostMainThread {
    ///     fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
    ///         // Called whn the plugin has been fully initialized.
    ///     }
    /// }
    ///
    /// # mod diva { include!("./entry/diva_stub.rs"); }
    ///
    /// let entry = /* ... */
    /// # PluginEntry::load_from_clack::<diva::Entry>(c"").unwrap();
    /// let host_info = HostInfo::new("Foo DAW", "Foo Inc.", "https://example.com", "0.0.1").unwrap();
    ///
    /// let instance = PluginInstance::<MyHost>::new(
    ///     |_|(),
    ///     |_| MyHostMainThread {},
    ///     &entry,
    ///     c"com.audio.the-plugin",
    ///     &host_info
    /// );
    ///
    /// ```
    pub fn new<FS, FH>(
        shared: FS,
        main_thread: FH,
        entry: &PluginEntry,
        plugin_id: &CStr,
        host: &HostInfo,
    ) -> Result<Self, PluginInstanceError>
    where
        FS: for<'b> FnOnce(&'b ()) -> <H as HostHandlers>::Shared<'b>,
        FH: for<'b> FnOnce(
            &'b <H as HostHandlers>::Shared<'b>,
        ) -> <H as HostHandlers>::MainThread<'b>,
    {
        let inner = PluginInstanceInner::<H>::instantiate(
            shared,
            main_thread,
            entry,
            plugin_id,
            host.clone(),
        )?;

        Ok(Self {
            inner: ManuallyDrop::new(inner),
            _no_send: PhantomData,
        })
    }

    /// Activates the plugin instance, preparing it to process audio and events according to the given [`PluginAudioConfiguration`].
    ///
    /// This method takes a closure `FA`, which will create
    /// the [`AudioProcessor`](HostHandlers::AudioProcessor) callback
    /// handler instance associated with this plugin instance.
    ///
    /// This closure takes a long-lived reference to the [`Shared`](HostHandlers::Shared) handler
    /// so that it can be accessed at any time from the audio thread / audio-processing context if needed.
    ///
    /// This closure also provides a temporary, exclusive reference to the [`MainThread`](HostHandlers::MainThread) handler.
    ///
    /// # Errors
    ///
    /// This will return [`PluginInstanceError::AlreadyActivatedPlugin`] if the plugin has already
    /// been activated.
    ///
    /// If the plugin instance's `activate` function pointer is NULL,
    /// this will return [`PluginInstanceError::NullActivateFunction`].
    ///
    /// Otherwise, if the plugin instance's `activate` function implementation failed for any reason,
    /// this will return [`PluginInstanceError::ActivationFailed`].
    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<StoppedPluginAudioProcessor<H>, PluginInstanceError>
    where
        FA: for<'a> FnOnce(
            &'a <H as HostHandlers>::Shared<'a>,
            &mut <H as HostHandlers>::MainThread<'a>,
        ) -> <H as HostHandlers>::AudioProcessor<'a>,
    {
        let wrapper =
            Arc::get_mut(&mut self.inner).ok_or(PluginInstanceError::AlreadyActivatedPlugin)?;
        wrapper.activate(audio_processor, configuration)?;

        Ok(StoppedPluginAudioProcessor::new(Arc::clone(&self.inner)))
    }

    /// De-activates the plugin instance, freeing its processing-related resources, and allows it
    /// to be re-[activated](Self::activate) with a different audio configuration.
    ///
    /// This consumes the [`StoppedPluginAudioProcessor`] handle created by [`activate`](Self::activate),
    /// in order to ensure it cannot be used anymore.
    ///
    /// For a version of this method that does not require the [`StoppedPluginAudioProcessor`] handle,
    /// see [`try_deactivate`](Self::try_deactivate).
    ///
    /// # Panics
    ///
    /// This will panic if the provided `processor` handle did not come from this plugin instance.
    #[inline]
    pub fn deactivate(&mut self, processor: StoppedPluginAudioProcessor<H>) {
        self.deactivate_with(processor, |_, _| ())
    }

    /// Attempts to de-activate a plugin instance, which [`PluginAudioProcessor`] handle has been
    /// dropped.
    ///
    /// This is an alternative to [`deactivate`](Self::deactivate) which does not require the associated
    /// [`PluginAudioProcessorHandle`] to be passed. However, this method is fallible, and will
    /// return an error if the [`PluginAudioProcessorHandle`] has not been dropped.
    ///
    /// This may be useful if it is more practical for you to drop the [`PluginAudioProcessorHandle`]
    /// (e.g. in another thread) compared to sending it back to the main thread to use with
    /// [`deactivate`](Self::deactivate).
    ///
    /// However, if you can easily access or retrieve [`PluginAudioProcessorHandle`] back to the main
    /// thread, you should use [`deactivate`](Self::deactivate) instead.
    ///
    /// # Errors
    ///
    /// This method will return [`PluginInstanceError::StillActivatedPlugin`] if the associated
    /// [`PluginAudioProcessorHandle`] has not been dropped.
    #[inline]
    pub fn try_deactivate(&mut self) -> Result<(), PluginInstanceError> {
        self.try_deactivate_with(|_, _| ())
    }

    /// De-activates the plugin instance, freeing its processing-related resources, and allows it
    /// to be re-[activated](Self::activate) with a different audio configuration.
    ///
    /// This consumes the [`StoppedPluginAudioProcessor`] handle created by [`activate`](Self::activate),
    /// in order to ensure it cannot be used anymore.
    ///
    /// This is equivalent to [`Self::deactivate`], except this method also takes a closure type `D`,
    /// which allows to customize the destruction process of the [`AudioProcessor`](HostHandlers::AudioProcessor) callback
    /// handler instance that was created during [`activate`](Self::activate).
    ///
    /// This may be useful to e.g. reuse or reprocess components or allocations the [`AudioProcessor`](HostHandlers::AudioProcessor) callback
    /// handler instance owned, either by giving it to the [`MainThread`](HostHandlers::MainThread) handler
    /// (which is also provided by a temporary, exclusive reference), or by returning it.
    ///
    /// This function will return whatever the destructor closure returned.
    ///
    /// # Panics
    ///
    /// This will panic if the provided `processor` handle did not come from this plugin instance.
    pub fn deactivate_with<T, D>(
        &mut self,
        processor: StoppedPluginAudioProcessor<H>,
        drop_with: D,
    ) -> T
    where
        D: for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'s>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    {
        if !Arc::ptr_eq(&self.inner, &processor.inner) {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        self.try_deactivate_with(drop_with).unwrap()
    }

    /// Attempts to de-activate a plugin instance, which [`PluginAudioProcessor`] handle has been
    /// dropped.
    ///
    /// This is equivalent to [`Self::try_deactivate`], except this method also takes a closure type `D`,
    /// which allows to customize the destruction process of the [`AudioProcessor`](HostHandlers::AudioProcessor) callback
    /// handler instance that was created during [`activate`](Self::activate).
    ///
    /// This is an alternative to [`deactivate_with`](Self::deactivate_with) which does not require the associated
    /// [`PluginAudioProcessorHandle`] to be passed. However, this method is fallible, and will
    /// return an error if the [`PluginAudioProcessorHandle`] has not been dropped.
    ///
    /// This may be useful if it is more practical for you to drop the [`PluginAudioProcessorHandle`]
    /// (e.g. in another thread) compared to sending it back to the main thread to use with
    /// [`deactivate_with`](Self::deactivate_with).
    ///
    /// However, if you can easily access or retrieve [`PluginAudioProcessorHandle`] back to the main
    /// thread, you should use [`deactivate_with`](Self::deactivate_with) instead.
    ///
    /// # Errors
    ///
    /// This method will return [`PluginInstanceError::StillActivatedPlugin`] if the associated
    /// [`PluginAudioProcessorHandle`] has not been dropped.
    pub fn try_deactivate_with<T, D>(&mut self, drop_with: D) -> Result<T, PluginInstanceError>
    where
        D: for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'s>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    {
        let wrapper =
            Arc::get_mut(&mut self.inner).ok_or(PluginInstanceError::StillActivatedPlugin)?;

        wrapper.deactivate_with(drop_with)
    }

    /// Calls the plugin's `on_main_thread` callback.
    ///
    /// This is usually done in response to the plugin calling [`SharedHandler::request_callback`]
    /// earlier from a different thread, allowing it to "wake up" its main thread after e.g. another
    /// of the plugin's threads sent a message to it.
    #[inline]
    pub fn call_on_main_thread_callback(&mut self) {
        // SAFETY: this is done on the main thread, and the &mut reference guarantees no aliasing
        unsafe { self.inner.on_main_thread() }
    }

    /// Returns a shared, read-only reference to the raw, C-FFI compatible representation
    /// of thsi plugin instance.
    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        self.inner.raw_instance()
    }

    /// Returns the [`PluginEntry`] this plugin instance was loaded from.
    #[inline]
    pub fn entry(&self) -> &PluginEntry {
        self.inner.entry()
    }

    /// Returns `true` if the current plugin instance has been activated.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Access the [`SharedHandler`] instance associated to this plugin instance using the given callback.
    ///
    /// The callback's return value `R` is returned directly by this method.
    ///
    /// Accessing the [`HostHandlers`] types can only be done with accessor callbacks because this
    /// type is self-referential: both the [`HostHandlers`] and the plugin's instance data hold
    /// references to each other, and both are owned by this type.
    #[inline]
    pub fn access_shared_handler<'s, R>(
        &'s self,
        access: impl for<'a> FnOnce(&'s <H as HostHandlers>::Shared<'a>) -> R,
    ) -> R {
        access(self.inner.wrapper().shared())
    }

    /// Access a shared reference to [`MainThreadHandler`] instance associated to this plugin instance using the given callback.
    ///
    /// The callback's return value `R` is returned directly by this method.
    ///
    /// Accessing the [`HostHandlers`] types can only be done with accessor callbacks because this
    /// type is self-referential: both the [`HostHandlers`] and the plugin's instance data hold
    /// references to each other, and both are owned by this type.
    #[inline]
    pub fn access_handler<'s, R>(
        &'s self,
        access: impl for<'a> FnOnce(&'s <H as HostHandlers>::MainThread<'a>) -> R,
    ) -> R {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { access(self.inner.wrapper().main_thread().as_ref()) }
    }

    /// Access an exclusive `&mut` reference to [`MainThreadHandler`] instance associated to this plugin instance using the given callback.
    ///
    /// The callback's return value `R` is returned directly by this method.
    ///
    /// Accessing the [`HostHandlers`] types can only be done with accessor callbacks because this
    /// type is self-referential: both the [`HostHandlers`] and the plugin's instance data hold
    /// references to each other, and both are owned by this type.
    #[inline]
    pub fn access_handler_mut<'s, R>(
        &'s mut self,
        access: impl for<'a> FnOnce(&'s mut <H as HostHandlers>::MainThread<'a>) -> R,
    ) -> R {
        // SAFETY: we take &mut self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { access(self.inner.wrapper().main_thread().as_mut()) }
    }

    /// Returns a thread-safe handle to the plugin.
    ///
    /// Unlike the [`PluginMainThreadHandle`] returned by [`plugin_handle`](Self::plugin_handle),
    /// this handle can be cloned and sent to other threads.
    ///
    /// However, it has fewer capabilities, and cannot be used for operations that have to be
    /// performed on the main-thread (e.g. GUI).
    #[inline]
    pub fn plugin_shared_handle(&self) -> PluginSharedHandle<'_> {
        self.inner.plugin_shared()
    }

    /// Returns a main-thread handle to the plugin.
    #[inline]
    pub fn plugin_handle(&mut self) -> PluginMainThreadHandle<'_> {
        // SAFETY: this type can only exist on the main thread.
        unsafe { PluginMainThreadHandle::new(self.inner.raw_instance().into()) }
    }

    /// Returns a main-thread handle to the plugin, that can only exist while the plugin is in the 'inactive' state.
    ///
    /// Calling this method also prevents the plugin instance to be activated as long as the handle exists.
    ///
    /// If the plugin is actually activated, this returns `None`.
    #[inline]
    pub fn inactive_plugin_handle(&mut self) -> Option<InactivePluginMainThreadHandle<'_>> {
        if self.inner.is_active() {
            return None;
        }

        // SAFETY: this type can only exist on the main thread.
        // We also checked above that the plugin isn't in the active state, and the handle keeps
        // this instance mutably borrowed, so it is not possible to activate the plugin while the
        // handle still exists.
        Some(unsafe { InactivePluginMainThreadHandle::new(self.inner.raw_instance().into()) })
    }
}

impl<H: HostHandlers> Drop for PluginInstance<H> {
    fn drop(&mut self) {
        // Only drop our Arc if we are the sole owner.
        // This leaks the plugin instance, but prevents accidentally transferring ownership to the
        // audio thread if the audio processor handle is still around somewhere.
        if Arc::get_mut(&mut self.inner).is_some() {
            // SAFETY: We can only call this once (as we're in Drop), and we never use the inner
            // value again afterward.
            unsafe { ManuallyDrop::drop(&mut self.inner) }
        };
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(PluginInstance<()>: Send, Sync);
}
