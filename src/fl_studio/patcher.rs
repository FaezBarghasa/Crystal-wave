#[derive(Debug, Clone, PartialEq)]
pub enum PatcherNodeKind {
    FromFL,           // Master audio/MIDI input from host
    ToFL,             // Master audio/MIDI output to host
    GeneratorPlugin,  // Synth
    EffectPlugin,     // Reverb/Delay/Distortion
    Splitter,         // Signal splitter
}

#[derive(Debug, Clone)]
pub struct PatcherNode {
    pub id: usize,
    pub name: String,
    pub kind: PatcherNodeKind,
    pub canvas_x: f32,
    pub canvas_y: f32,
}

#[derive(Debug, Clone)]
pub struct PatcherCable {
    pub from_node: usize,
    pub to_node: usize,
    pub gain: f32, // 0.0 to 1.0 (cable signal attenuation)
}

#[derive(Debug, Clone)]
pub struct MacroMapping {
    pub target_node_id: usize,
    pub parameter_id: u32,
    pub min_value: f32,
    pub max_value: f32,
    pub invert: bool,
}

#[derive(Debug, Clone)]
pub struct PatcherMacroKnob {
    pub label: String,
    pub normalized_value: f32, // 0.0 to 1.0
    pub mappings: Vec<MacroMapping>,
}

pub struct FlPatcher {
    pub nodes: Vec<PatcherNode>,
    pub cables: Vec<PatcherCable>,
    pub macros: Vec<PatcherMacroKnob>,
}

impl Default for FlPatcher {
    fn default() -> Self {
        Self {
            nodes: vec![
                PatcherNode {
                    id: 0,
                    name: "From FL".into(),
                    kind: PatcherNodeKind::FromFL,
                    canvas_x: 50.0,
                    canvas_y: 150.0,
                },
                PatcherNode {
                    id: 1,
                    name: "To FL".into(),
                    kind: PatcherNodeKind::ToFL,
                    canvas_x: 600.0,
                    canvas_y: 150.0,
                },
            ],
            cables: vec![PatcherCable {
                from_node: 0,
                to_node: 1,
                gain: 1.0,
            }],
            macros: Vec::new(),
        }
    }
}

impl FlPatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, name: impl Into<String>, kind: PatcherNodeKind, x: f32, y: f32) -> usize {
        let id = self.nodes.len();
        self.nodes.push(PatcherNode {
            id,
            name: name.into(),
            kind,
            canvas_x: x,
            canvas_y: y,
        });
        id
    }

    pub fn connect(&mut self, from_node: usize, to_node: usize, gain: f32) -> bool {
        if from_node < self.nodes.len() && to_node < self.nodes.len() {
            self.cables.push(PatcherCable {
                from_node,
                to_node,
                gain: gain.clamp(0.0, 2.0),
            });
            true
        } else {
            false
        }
    }

    pub fn add_macro(&mut self, label: impl Into<String>) -> usize {
        let id = self.macros.len();
        self.macros.push(PatcherMacroKnob {
            label: label.into(),
            normalized_value: 0.5,
            mappings: Vec::new(),
        });
        id
    }
}
