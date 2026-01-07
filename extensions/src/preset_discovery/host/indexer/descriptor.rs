use crate::preset_discovery::indexer::{IndexerImpl, IndexerWrapper, IndexerWrapperError};
use crate::preset_discovery::preset_data::{FileType, LocationInfo, Soundpack};
use clack_common::utils::ClapVersion;
use clack_host::prelude::HostInfo;
use clap_sys::factory::preset_discovery::{
    clap_preset_discovery_filetype, clap_preset_discovery_indexer, clap_preset_discovery_location,
    clap_preset_discovery_soundpack,
};
use std::ffi::c_void;
use std::pin::Pin;

pub struct RawIndexerDescriptor {
    raw: clap_preset_discovery_indexer,
    _host_info: HostInfo,
}

impl RawIndexerDescriptor {
    pub fn new<I: IndexerImpl>(
        host_info: HostInfo,
        wrapper: Pin<&mut IndexerWrapper<I>>,
    ) -> Pin<Box<Self>> {
        Box::pin(Self {
            raw: clap_preset_discovery_indexer {
                clap_version: ClapVersion::CURRENT.to_raw(),
                indexer_data: wrapper.as_raw_mut(),
                name: host_info.name().as_ptr(),
                vendor: host_info.vendor().as_ptr(),
                url: host_info.url().as_ptr(),
                version: host_info.version().as_ptr(),
                get_extension: Some(get_extension::<I>),
                declare_filetype: Some(declare_filetype::<I>),
                declare_location: Some(declare_location::<I>),
                declare_soundpack: Some(declare_soundpack::<I>),
            },
            _host_info: host_info,
        })
    }

    pub fn as_raw_mut(self: Pin<&mut Self>) -> *mut clap_preset_discovery_indexer {
        // SAFETY: This method does not move anything out, it just gets the pointer
        let s = unsafe { self.get_unchecked_mut() };
        &mut s.raw
    }
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_extension<I: IndexerImpl>(
    indexer: *const clap_preset_discovery_indexer,
    identifier: *const std::os::raw::c_char,
) -> *const c_void {
    todo!()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn declare_filetype<I: IndexerImpl>(
    indexer: *const clap_preset_discovery_indexer,
    filetype: *const clap_preset_discovery_filetype,
) -> bool {
    IndexerWrapper::<I>::handle(indexer, |indexer| {
        let filetype = FileType::from_raw_ptr(filetype)
            .ok_or(IndexerWrapperError::InvalidParameter("Invalid FileType"))?;

        indexer.declare_filetype(filetype);
        Ok(())
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn declare_location<I: IndexerImpl>(
    indexer: *const clap_preset_discovery_indexer,
    location: *const clap_preset_discovery_location,
) -> bool {
    IndexerWrapper::<I>::handle(indexer, |indexer| {
        let location = LocationInfo::from_raw_ptr(location)
            .ok_or(IndexerWrapperError::InvalidParameter("Invalid Location"))?;

        indexer.declare_location(location);
        Ok(())
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn declare_soundpack<I: IndexerImpl>(
    indexer: *const clap_preset_discovery_indexer,
    soundpack: *const clap_preset_discovery_soundpack,
) -> bool {
    IndexerWrapper::<I>::handle(indexer, |indexer| {
        let filetype = Soundpack::from_raw_ptr(soundpack)
            .ok_or(IndexerWrapperError::InvalidParameter("Invalid Soundpack"))?;

        indexer.declare_soundpack(filetype);
        Ok(())
    })
    .is_some()
}
