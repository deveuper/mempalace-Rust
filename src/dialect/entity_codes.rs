#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use std::collections::HashMap;

/// Entity code registry for AAAK compression
pub struct EntityCodeRegistry {
    entities: HashMap<String, String>,
    next_code: u32,
}

impl EntityCodeRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_code: 1,
        }
    }

    /// Encode an entity to a short code
    pub fn encode(&mut self, entity: &str) -> String {
        let key = entity.to_lowercase();

        if let Some(code) = self.entities.get(&key) {
            return code.clone();
        }

        // Generate new code
        let code = format!("E{:03}", self.next_code);
        self.next_code += 1;
        self.entities.insert(key, code.clone());

        code
    }

    /// Decode a code back to entity
    pub fn decode(&self, code: &str) -> Option<&String> {
        self.entities.iter().find(|(_, v)| *v == code).map(|(k, _)| k)
    }

    /// Get all entity mappings
    pub fn get_mappings(&self) -> &HashMap<String, String> {
        &self.entities
    }

    /// Get the legend (code -> entity mapping)
    pub fn get_legend(&self) -> HashMap<String, String> {
        self.entities
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect()
    }
}

impl Default for EntityCodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
