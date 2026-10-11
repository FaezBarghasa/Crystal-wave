use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PatternNote {
    pub pitch: u8,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub velocity: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub id: usize,
    pub name: String,
    pub length_ticks: u64,
    pub notes: Vec<PatternNote>,
}

impl Pattern {
    pub fn new(id: usize, name: &str, length_ticks: u64) -> Self {
        Self {
            id,
            name: name.to_string(),
            length_ticks,
            notes: Vec::new(),
        }
    }

    pub fn add_note(&mut self, pitch: u8, start_tick: u64, length_ticks: u64, velocity: u8) {
        self.notes.push(PatternNote {
            pitch,
            start_tick,
            length_ticks,
            velocity,
        });
    }
}

/// A stamped instance of a pattern placed on the arrangement timeline
#[derive(Debug, Clone, PartialEq)]
pub struct PatternInstance {
    pub instance_id: usize,
    pub pattern_id: usize,
    pub track_id: usize,
    pub start_tick: u64,
    pub is_detached: bool, // true = exploded/linear independent clone
}

impl PatternInstance {
    pub fn end_tick(&self, length_ticks: u64) -> u64 {
        self.start_tick + length_ticks
    }
}

/// Hybrid Pattern & Linear Arrangement Timeline Manager
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HybridTimelineManager {
    pub patterns: HashMap<usize, Pattern>,
    pub instances: Vec<PatternInstance>,
    next_pattern_id: usize,
    next_instance_id: usize,
}

impl HybridTimelineManager {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            instances: Vec::new(),
            next_pattern_id: 1,
            next_instance_id: 1,
        }
    }

    pub fn create_pattern(&mut self, name: &str, length_ticks: u64) -> usize {
        let id = self.next_pattern_id;
        self.next_pattern_id += 1;
        self.patterns
            .insert(id, Pattern::new(id, name, length_ticks));
        id
    }

    pub fn stamp_pattern(
        &mut self,
        pattern_id: usize,
        track_id: usize,
        start_tick: u64,
    ) -> Result<usize, &'static str> {
        if !self.patterns.contains_key(&pattern_id) {
            return Err("PatternNotFound");
        }
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;
        self.instances.push(PatternInstance {
            instance_id,
            pattern_id,
            track_id,
            start_tick,
            is_detached: false,
        });
        Ok(instance_id)
    }

    /// "Explodes" or detaches a pattern instance into an independent clone,
    /// allowing linear edits to this specific clip without breaking the original pattern.
    pub fn explode_instance(&mut self, instance_id: usize) -> Result<usize, &'static str> {
        let (pattern_id, original_name) = {
            let inst = self
                .instances
                .iter()
                .find(|i| i.instance_id == instance_id)
                .ok_or("InstanceNotFound")?;
            let pat = self
                .patterns
                .get(&inst.pattern_id)
                .ok_or("PatternNotFound")?;
            (pat.id, pat.name.clone())
        };

        let original_pattern = self.patterns.get(&pattern_id).unwrap().clone();
        let new_pattern_id = self.next_pattern_id;
        self.next_pattern_id += 1;

        let mut cloned_pattern = original_pattern;
        cloned_pattern.id = new_pattern_id;
        cloned_pattern.name = format!("{}_exploded", original_name);

        self.patterns.insert(new_pattern_id, cloned_pattern);

        if let Some(inst) = self
            .instances
            .iter_mut()
            .find(|i| i.instance_id == instance_id)
        {
            inst.pattern_id = new_pattern_id;
            inst.is_detached = true;
        }

        Ok(new_pattern_id)
    }

    /// Counts how many linked (non-detached) instances refer to a given pattern.
    pub fn linked_instance_count(&self, pattern_id: usize) -> usize {
        self.instances
            .iter()
            .filter(|i| i.pattern_id == pattern_id && !i.is_detached)
            .count()
    }
}
