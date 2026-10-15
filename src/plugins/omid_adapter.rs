use crate::plugins::plugin_descriptor::PluginDescriptor;
use crate::plugins::plugin_host::AudioPlugin;
use crate::plugins::plugin_parameters::PluginParameter;
use crate::plugins::sdk_ffi::{CrystalAudioBufferC, CrystalPluginVTable};
use omid::OmidPacket;
use std::ffi::CStr;

/// Safe Dynamic Plugin Host wrapping any native C/C++/Rust shared library
pub struct DynamicPluginHost {
    descriptor: PluginDescriptor,
    parameters: Vec<PluginParameter>,
    instance: *mut std::ffi::c_void,
    vtable: CrystalPluginVTable,
    _lib: Option<libloading::Library>,
}

// Safety: The dynamic plugin must adhere to thread-safety guarantees defined in the Crystal-Wave C-ABI
unsafe impl Send for DynamicPluginHost {}
unsafe impl Sync for DynamicPluginHost {}

impl DynamicPluginHost {
    pub fn new_from_vtable(vtable: CrystalPluginVTable, sample_rate: f32) -> Result<Self, String> {
        let desc_c = unsafe { (vtable.get_descriptor)() };
        if desc_c.api_version != crate::plugins::sdk_ffi::CRYSTAL_PLUGIN_API_VERSION {
            return Err(format!(
                "API Version mismatch: expected {}, got {}",
                crate::plugins::sdk_ffi::CRYSTAL_PLUGIN_API_VERSION,
                desc_c.api_version
            ));
        }

        let name = unsafe {
            if desc_c.name.is_null() {
                "Unknown Dynamic Plugin".to_string()
            } else {
                CStr::from_ptr(desc_c.name).to_string_lossy().into_owned()
            }
        };

        let vendor = unsafe {
            if desc_c.vendor.is_null() {
                "Third-Party".to_string()
            } else {
                CStr::from_ptr(desc_c.vendor).to_string_lossy().into_owned()
            }
        };

        let version = unsafe {
            if desc_c.version.is_null() {
                "1.0.0".to_string()
            } else {
                CStr::from_ptr(desc_c.version).to_string_lossy().into_owned()
            }
        };

        let category = crate::plugins::sdk_ffi::category_from_u32(desc_c.category);
        let descriptor = PluginDescriptor {
            unique_id: desc_c.unique_id,
            name,
            vendor,
            version,
            category,
            num_inputs: desc_c.num_inputs as usize,
            num_outputs: desc_c.num_outputs as usize,
            supports_sidechain: desc_c.supports_sidechain,
            latency_samples: desc_c.latency_samples as usize,
        };

        let instance = unsafe { (vtable.create_instance)(sample_rate) };
        if instance.is_null() {
            return Err("create_instance returned null".to_string());
        }

        let mut parameters = Vec::new();
        let param_count = unsafe { (vtable.get_parameter_count)(instance) };
        for i in 0..param_count {
            let p_name = unsafe {
                let ptr = (vtable.get_parameter_name)(instance, i);
                if ptr.is_null() {
                    format!("Param {}", i)
                } else {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            };
            let val = unsafe { (vtable.get_parameter_value)(instance, i) };
            parameters.push(PluginParameter::new(i + 1, p_name, 0.0, 1.0, val, ""));
        }

        Ok(Self {
            descriptor,
            parameters,
            instance,
            vtable,
            _lib: None,
        })
    }

    /// Feeds an 8-byte OMID packet into the plugin
    pub fn dispatch_omid_packet(&mut self, packet: OmidPacket) {
        if let Some(handler) = self.vtable.handle_omid_packet {
            let bytes = packet.to_bytes();
            unsafe {
                handler(self.instance, bytes.as_ptr());
            }
        }
    }

    /// Polls any response OMID packet emitted by the plugin
    pub fn poll_omid_response(&mut self) -> Option<OmidPacket> {
        if let Some(poll_fn) = self.vtable.poll_omid_response {
            let mut buf = [0u8; 8];
            let has_packet = unsafe { poll_fn(self.instance, buf.as_mut_ptr()) };
            if has_packet {
                return Some(OmidPacket::from_bytes(&buf));
            }
        }
        None
    }
}

impl Drop for DynamicPluginHost {
    fn drop(&mut self) {
        if !self.instance.is_null() {
            unsafe {
                (self.vtable.destroy_instance)(self.instance);
            }
            self.instance = std::ptr::null_mut();
        }
    }
}

impl AudioPlugin for DynamicPluginHost {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], sidechain: Option<&[f32]>) {
        let num_samples = left.len().min(right.len()) as u32;
        let mut buffer_c = CrystalAudioBufferC {
            left_channel: left.as_mut_ptr(),
            right_channel: right.as_mut_ptr(),
            sidechain_channel: sidechain.map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            num_samples,
        };

        // Synchronize parameter values between host parameter handles and plugin instance
        for (i, p) in self.parameters.iter_mut().enumerate() {
            unsafe {
                let current_instance_val = (self.vtable.get_parameter_value)(self.instance, i as u32);
                if (current_instance_val - p.get_value()).abs() > 1e-5 {
                    p.set_value(current_instance_val);
                } else {
                    (self.vtable.set_parameter_value)(self.instance, i as u32, p.get_value());
                }
            }
        }

        unsafe {
            (self.vtable.process_audio)(self.instance, &mut buffer_c);
        }
    }

    fn reset(&mut self) {
        unsafe {
            (self.vtable.reset_state)(self.instance);
        }
    }

    fn parameters(&self) -> &[PluginParameter] {
        &self.parameters
    }

    fn latency_samples(&self) -> usize {
        self.descriptor.latency_samples
    }
}
