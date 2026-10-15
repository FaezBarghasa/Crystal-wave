use omid::OmidPacket;

/// High-Level Idiomatic Rust Trait for Crystal-Wave Plugins
pub trait CrystalPlugin: Send + 'static {
    fn name(&self) -> &'static str;
    fn vendor(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> u32; // 0..=11
    fn unique_id(&self) -> u32;
    fn latency_samples(&self) -> usize { 0 }
    fn supports_sidechain(&self) -> bool { false }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], sidechain: Option<&[f32]>);
    fn reset(&mut self) {}

    fn parameter_count(&self) -> usize { 0 }
    fn parameter_name(&self, _index: usize) -> &'static str { "" }
    fn get_parameter(&self, _index: usize) -> f32 { 0.0 }
    fn set_parameter(&mut self, _index: usize, _val: f32) {}

    // Omid Protocol Hook
    fn on_omid_packet(&mut self, _packet: OmidPacket) {}
    fn poll_omid_response(&mut self) -> Option<OmidPacket> { None }
}

/// Helper macro to generate exported C-ABI vtable for any struct implementing `CrystalPlugin`
#[macro_export]
macro_rules! export_crystal_plugin {
    ($plugin_type:ty) => {
        static PLUGIN_NAME: std::sync::LazyLock<std::ffi::CString> = std::sync::LazyLock::new(|| {
            std::ffi::CString::new("Crystal Dynamic Plugin").unwrap()
        });

        pub struct PluginInstanceWrapper {
            pub inner: $plugin_type,
            pub sample_rate: f32,
        }

        pub unsafe extern "C" fn crystal_get_descriptor() -> $crate::plugins::sdk_ffi::CrystalPluginDescriptorC {
            // Temporary instance to inspect static properties
            $crate::plugins::sdk_ffi::CrystalPluginDescriptorC {
                api_version: $crate::plugins::sdk_ffi::CRYSTAL_PLUGIN_API_VERSION,
                unique_id: 8888,
                name: b"Crystal Plugin\0".as_ptr() as *const std::ffi::c_char,
                vendor: b"PureRust\0".as_ptr() as *const std::ffi::c_char,
                version: b"1.0.0\0".as_ptr() as *const std::ffi::c_char,
                category: 0,
                num_inputs: 2,
                num_outputs: 2,
                supports_sidechain: false,
                latency_samples: 0,
            }
        }

        pub unsafe extern "C" fn crystal_create_instance(sample_rate: f32) -> *mut std::ffi::c_void {
            let instance = Box::new(PluginInstanceWrapper {
                inner: <$plugin_type>::default(),
                sample_rate,
            });
            Box::into_raw(instance) as *mut std::ffi::c_void
        }

        pub unsafe extern "C" fn crystal_destroy_instance(instance: *mut std::ffi::c_void) {
            if !instance.is_null() {
                drop(Box::from_raw(instance as *mut PluginInstanceWrapper));
            }
        }

        pub unsafe extern "C" fn crystal_process_audio(
            instance: *mut std::ffi::c_void,
            buffer: *mut $crate::plugins::sdk_ffi::CrystalAudioBufferC,
        ) {
            if instance.is_null() || buffer.is_null() {
                return;
            }
            let wrapper = &mut *(instance as *mut PluginInstanceWrapper);
            let b = &mut *buffer;
            let len = b.num_samples as usize;
            let left = std::slice::from_raw_parts_mut(b.left_channel, len);
            let right = std::slice::from_raw_parts_mut(b.right_channel, len);
            let sidechain = if b.sidechain_channel.is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(b.sidechain_channel, len))
            };
            wrapper.inner.process(left, right, sidechain);
        }

        pub unsafe extern "C" fn crystal_reset_state(instance: *mut std::ffi::c_void) {
            if !instance.is_null() {
                let wrapper = &mut *(instance as *mut PluginInstanceWrapper);
                wrapper.inner.reset();
            }
        }

        pub unsafe extern "C" fn crystal_get_parameter_count(instance: *mut std::ffi::c_void) -> u32 {
            if instance.is_null() {
                0
            } else {
                let wrapper = &*(instance as *mut PluginInstanceWrapper);
                wrapper.inner.parameter_count() as u32
            }
        }

        pub unsafe extern "C" fn crystal_get_parameter_value(
            instance: *mut std::ffi::c_void,
            param_index: u32,
        ) -> f32 {
            if instance.is_null() {
                0.0
            } else {
                let wrapper = &*(instance as *mut PluginInstanceWrapper);
                wrapper.inner.get_parameter(param_index as usize)
            }
        }

        pub unsafe extern "C" fn crystal_set_parameter_value(
            instance: *mut std::ffi::c_void,
            param_index: u32,
            val: f32,
        ) {
            if !instance.is_null() {
                let wrapper = &mut *(instance as *mut PluginInstanceWrapper);
                wrapper.inner.set_parameter(param_index as usize, val);
            }
        }

        pub unsafe extern "C" fn crystal_get_parameter_name(
            instance: *mut std::ffi::c_void,
            param_index: u32,
        ) -> *const std::ffi::c_char {
            if instance.is_null() {
                std::ptr::null()
            } else {
                let wrapper = &*(instance as *mut PluginInstanceWrapper);
                let name = wrapper.inner.parameter_name(param_index as usize);
                // Note: name should be null-terminated in production
                name.as_ptr() as *const std::ffi::c_char
            }
        }

        pub unsafe extern "C" fn crystal_handle_omid(
            instance: *mut std::ffi::c_void,
            packet_bytes: *const u8,
        ) {
            if !instance.is_null() && !packet_bytes.is_null() {
                let wrapper = &mut *(instance as *mut PluginInstanceWrapper);
                let slice = std::slice::from_raw_parts(packet_bytes, 8);
                let mut buf = [0u8; 8];
                buf.copy_from_slice(slice);
                wrapper.inner.on_omid_packet(omid::OmidPacket::from_bytes(&buf));
            }
        }

        pub unsafe extern "C" fn crystal_poll_omid(
            instance: *mut std::ffi::c_void,
            out_bytes: *mut u8,
        ) -> bool {
            if !instance.is_null() && !out_bytes.is_null() {
                let wrapper = &mut *(instance as *mut PluginInstanceWrapper);
                if let Some(pkt) = wrapper.inner.poll_omid_response() {
                    let b = pkt.to_bytes();
                    std::ptr::copy_nonoverlapping(b.as_ptr(), out_bytes, 8);
                    return true;
                }
            }
            false
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn crystal_plugin_get_vtable() -> $crate::plugins::sdk_ffi::CrystalPluginVTable {
            $crate::plugins::sdk_ffi::CrystalPluginVTable {
                get_descriptor: crystal_get_descriptor,
                create_instance: crystal_create_instance,
                destroy_instance: crystal_destroy_instance,
                process_audio: crystal_process_audio,
                reset_state: crystal_reset_state,
                get_parameter_count: crystal_get_parameter_count,
                get_parameter_value: crystal_get_parameter_value,
                set_parameter_value: crystal_set_parameter_value,
                get_parameter_name: crystal_get_parameter_name,
                handle_omid_packet: Some(crystal_handle_omid),
                poll_omid_response: Some(crystal_poll_omid),
            }
        }
    };
}
