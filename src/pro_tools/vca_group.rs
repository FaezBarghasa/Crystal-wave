/// A VCA (Voltage Controlled Amplifier) Master fader that controls
/// assigned slave channels via gain offsets without modifying individual automation.
#[derive(Debug, Clone, PartialEq)]
pub struct VcaMaster {
    pub id: usize,
    pub name: String,
    pub gain_db: f32,
    pub muted: bool,
    pub soloed: bool,
    pub assigned_track_ids: Vec<usize>,
}

impl VcaMaster {
    pub fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            gain_db: 0.0,
            muted: false,
            soloed: false,
            assigned_track_ids: Vec::new(),
        }
    }

    pub fn assign_track(&mut self, track_id: usize) {
        if !self.assigned_track_ids.contains(&track_id) {
            self.assigned_track_ids.push(track_id);
        }
    }

    pub fn unassign_track(&mut self, track_id: usize) {
        self.assigned_track_ids.retain(|&id| id != track_id);
    }
}

/// Manager handling VCA Masters, routing calculations, and console-like behavior.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VcaManager {
    pub masters: Vec<VcaMaster>,
}

impl VcaManager {
    pub fn new() -> Self {
        Self {
            masters: Vec::new(),
        }
    }

    pub fn create_vca(&mut self, name: &str) -> usize {
        let id = self.masters.len();
        self.masters.push(VcaMaster::new(id, name));
        id
    }

    pub fn assign_track_to_vca(
        &mut self,
        vca_id: usize,
        track_id: usize,
    ) -> Result<(), &'static str> {
        if let Some(vca) = self.masters.get_mut(vca_id) {
            vca.assign_track(track_id);
            Ok(())
        } else {
            Err("VcaNotFound: Specified VCA master ID does not exist")
        }
    }

    pub fn set_vca_gain_db(&mut self, vca_id: usize, gain_db: f32) -> Result<(), &'static str> {
        if let Some(vca) = self.masters.get_mut(vca_id) {
            vca.gain_db = gain_db;
            Ok(())
        } else {
            Err("VcaNotFound")
        }
    }

    pub fn set_vca_mute(&mut self, vca_id: usize, muted: bool) -> Result<(), &'static str> {
        if let Some(vca) = self.masters.get_mut(vca_id) {
            vca.muted = muted;
            Ok(())
        } else {
            Err("VcaNotFound")
        }
    }

    pub fn set_vca_solo(&mut self, vca_id: usize, soloed: bool) -> Result<(), &'static str> {
        if let Some(vca) = self.masters.get_mut(vca_id) {
            vca.soloed = soloed;
            Ok(())
        } else {
            Err("VcaNotFound")
        }
    }

    /// Computes the effective linear multiplier for a track by summing all VCA offsets.
    ///
    /// $G_{\text{effective\_db}} = G_{\text{track\_db}} + \sum_{\text{vca}} G_{\text{vca\_db}}$
    ///
    /// Returns the linear gain multiplier: $10^{(G_{\text{effective\_db}} / 20.0)}$
    #[inline]
    pub fn calculate_effective_linear_gain(&self, track_id: usize, track_gain_db: f32) -> f32 {
        let mut total_offset_db = 0.0f32;
        for vca in &self.masters {
            if vca.assigned_track_ids.contains(&track_id) {
                total_offset_db += vca.gain_db;
            }
        }
        let total_db = track_gain_db + total_offset_db;
        10.0f32.powf(total_db / 20.0)
    }

    /// Determines if a track is muted directly or via any assigned VCA master.
    #[inline]
    pub fn is_track_muted(&self, track_id: usize, track_direct_muted: bool) -> bool {
        if track_direct_muted {
            return true;
        }
        for vca in &self.masters {
            if vca.assigned_track_ids.contains(&track_id) && vca.muted {
                return true;
            }
        }
        false
    }

    /// Determines if a track is soloed directly or via an assigned soloed VCA.
    #[inline]
    pub fn is_track_soloed(&self, track_id: usize, track_direct_soloed: bool) -> bool {
        if track_direct_soloed {
            return true;
        }
        for vca in &self.masters {
            if vca.assigned_track_ids.contains(&track_id) && vca.soloed {
                return true;
            }
        }
        false
    }
}
