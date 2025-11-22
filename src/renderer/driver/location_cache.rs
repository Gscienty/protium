use std::{collections, sync};

use super::{ProgramID, UniformLocationID};

pub(super) struct LocationCache {
    cache: sync::RwLock<
        collections::HashMap<ProgramID, collections::HashMap<String, UniformLocationID>>,
    >,
}

impl LocationCache {
    pub(super) fn new() -> Self {
        Self {
            cache: sync::RwLock::new(collections::HashMap::new()),
        }
    }

    pub(super) fn clear(&self) {
        let Ok(mut cache) = self.cache.write() else {
            return;
        };

        cache.clear();
    }

    pub(super) fn delete_program(&self, program: ProgramID) {
        let Ok(mut cache) = self.cache.write() else {
            return;
        };

        cache.remove(&program);
    }

    pub(super) fn get_location(&self, program: ProgramID, name: &str) -> Option<UniformLocationID> {
        let Ok(cache) = self.cache.read() else {
            return None;
        };

        let program = cache.get(&program)?;

        program.get(name).copied()
    }

    pub(super) fn set_location(&self, program: ProgramID, name: &str, location: UniformLocationID) {
        let Ok(mut cache) = self.cache.write() else {
            return;
        };

        cache
            .entry(program)
            .or_default()
            .insert(name.to_owned(), location);
    }
}
