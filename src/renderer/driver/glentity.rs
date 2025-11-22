use std::{collections, sync};

pub(super) struct GLEntityMap<T>
where
    T: Eq,
{
    data: sync::RwLock<collections::HashMap<usize, T>>,
    last_id: sync::Mutex<usize>,
}

impl<T> GLEntityMap<T>
where
    T: Eq,
{
    pub(super) fn new() -> Self {
        Self {
            data: sync::RwLock::new(collections::HashMap::new()),
            last_id: sync::Mutex::new(1),
        }
    }

    pub(super) fn create(&self, value: T) -> usize {
        let id = self.generate_id();
        if id == 0 {
            return 0;
        }

        let Ok(mut data) = self.data.write() else {
            return 0;
        };
        data.insert(id, value);

        id
    }

    pub(super) fn remove(&self, key: usize) -> Option<T> {
        let Ok(mut data) = self.data.write() else {
            return None;
        };

        data.remove(&key)
    }

    pub(super) fn get_or_create(&self, value: T) -> usize {
        if let Some(id) = self.get_entity_id(&value) {
            id
        } else {
            self.create(value)
        }
    }

    pub(super) fn process_entity<F>(&self, id: usize, cb: F) -> bool
    where
        F: FnOnce(&T),
    {
        let Ok(data) = self.data.read() else {
            return false;
        };

        let Some(value) = data.get(&id) else {
            return false;
        };
        cb(value);

        true
    }

    pub(super) fn get_entity_id(&self, value: &T) -> Option<usize> {
        let Ok(data) = self.data.read() else {
            return None;
        };

        data.iter()
            .find(|(_, stored_value)| stored_value.eq(&value))
            .map(|(&index, _)| index)
    }

    fn generate_id(&self) -> usize {
        let Ok(mut last_id_guard) = self.last_id.lock() else {
            return 0;
        };

        let id = *last_id_guard;
        *last_id_guard += 1;

        id
    }
}
