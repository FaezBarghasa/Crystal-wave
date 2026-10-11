#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalPort {
    pub id: usize,
    pub name: String,
    pub direction: PortDirection,
    pub hardware_channel: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareInsert {
    pub id: usize,
    pub name: String,
    pub send_port_id: usize,
    pub return_port_id: usize,
    pub delay_compensation_samples: usize,
    pub active: bool,
}

impl HardwareInsert {
    pub fn new(
        id: usize,
        name: &str,
        send_port_id: usize,
        return_port_id: usize,
        delay_compensation_samples: usize,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            send_port_id,
            return_port_id,
            delay_compensation_samples,
            active: true,
        }
    }
}

/// Explicit I/O routing matrix and Hardware Insert delay calibration manager
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IoMatrix {
    pub physical_ports: Vec<PhysicalPort>,
    pub hardware_inserts: Vec<HardwareInsert>,
}

impl IoMatrix {
    pub fn new() -> Self {
        Self {
            physical_ports: Vec::new(),
            hardware_inserts: Vec::new(),
        }
    }

    pub fn add_port(
        &mut self,
        name: &str,
        direction: PortDirection,
        hardware_channel: usize,
    ) -> usize {
        let id = self.physical_ports.len();
        self.physical_ports.push(PhysicalPort {
            id,
            name: name.to_string(),
            direction,
            hardware_channel,
        });
        id
    }

    pub fn create_hardware_insert(
        &mut self,
        name: &str,
        send_port_id: usize,
        return_port_id: usize,
    ) -> Result<usize, &'static str> {
        let has_send = self
            .physical_ports
            .iter()
            .any(|p| p.id == send_port_id && p.direction == PortDirection::Output);
        let has_return = self
            .physical_ports
            .iter()
            .any(|p| p.id == return_port_id && p.direction == PortDirection::Input);

        if !has_send || !has_return {
            return Err(
                "InvalidPorts: Send port must be an Output and Return port must be an Input",
            );
        }

        let id = self.hardware_inserts.len();
        self.hardware_inserts.push(HardwareInsert::new(
            id,
            name,
            send_port_id,
            return_port_id,
            0,
        ));
        Ok(id)
    }

    /// Measures loopback ping latency between a transmitted test impulse and returned signal.
    /// Returns the exact round-trip delay in samples.
    pub fn calculate_ping_latency(
        send_impulse: &[f32],
        received_signal: &[f32],
        detection_threshold: f32,
    ) -> Option<usize> {
        if send_impulse.is_empty() || received_signal.is_empty() {
            return None;
        }

        // Find the index of the send pulse peak
        let send_peak_idx = send_impulse
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.abs()
                    .partial_cmp(&b.1.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)?;

        // Find the first index in the received signal that crosses the detection threshold
        let received_peak_idx = received_signal
            .iter()
            .enumerate()
            .position(|(_, &sample)| sample.abs() >= detection_threshold)?;

        if received_peak_idx >= send_peak_idx {
            Some(received_peak_idx - send_peak_idx)
        } else {
            None
        }
    }

    /// Calibrates and applies ping delay compensation to a hardware insert.
    pub fn calibrate_insert_delay(
        &mut self,
        insert_id: usize,
        measured_delay_samples: usize,
    ) -> Result<(), &'static str> {
        if let Some(insert) = self.hardware_inserts.get_mut(insert_id) {
            insert.delay_compensation_samples = measured_delay_samples;
            Ok(())
        } else {
            Err("InsertNotFound")
        }
    }
}
