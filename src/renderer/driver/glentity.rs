use std::{cell, collections, sync};

pub(super) struct GLEntityMap<T>
where
    T: Eq,
{
    data: cell::RefCell<collections::HashMap<usize, T>>,
    last_id: sync::atomic::AtomicUsize,
}

impl<T> GLEntityMap<T>
where
    T: Eq,
{
    pub(super) fn new() -> Self {
        Self {
            data: cell::RefCell::new(collections::HashMap::new()),
            last_id: sync::atomic::AtomicUsize::new(1),
        }
    }

    pub(super) fn create(&self, value: T) -> usize {
        let id = self.generate_id();
        if id == 0 {
            return 0;
        }

        let Ok(mut data) = self.data.try_borrow_mut() else {
            return 0;
        };
        data.insert(id, value);

        id
    }

    pub(super) fn remove(&self, key: usize) -> Option<T> {
        let Ok(mut data) = self.data.try_borrow_mut() else {
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
        let Ok(data) = self.data.try_borrow() else {
            return false;
        };

        let Some(value) = data.get(&id) else {
            return false;
        };
        cb(value);

        true
    }

    pub(super) fn get_entity_id(&self, value: &T) -> Option<usize> {
        let Ok(data) = self.data.try_borrow() else {
            return None;
        };

        data.iter()
            .find(|(_, stored_value)| stored_value.eq(&value))
            .map(|(&index, _)| index)
    }

    fn generate_id(&self) -> usize {
        self.last_id.fetch_add(1, sync::atomic::Ordering::SeqCst)
    }
}
