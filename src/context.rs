use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::{Debug, Formatter};
use fnv::FnvHashMap;

#[derive(Default)]
pub struct HiveMemory(FnvHashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl HiveMemory {

    pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
        self.0.insert(TypeId::of::<D>(), Box::new(data));
    }

    pub fn get<T: Any + Sync + Send + 'static>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>())
            .and_then(|box_any| box_any.downcast_ref::<T>())
    }
}

impl Debug for HiveMemory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Data").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a test type
    #[derive(Clone)]
    pub struct AppState {
        pub encoding_key: String,
        pub decoding_key: String,
        pub api_key: String,
        pub api_secret: String,
    }

    impl Default for AppState {
        fn default() -> Self {
            AppState {
                encoding_key: String::from("default_encoding_key"),
                decoding_key: String::from("default_decoding_key"),
                api_key: String::from("default_api_key"),
                api_secret: String::from("default_api_secret"),
            }
        }
    }

    #[derive(Clone, Default)]
    pub struct DbPool {
        pub pretend_pool: String,
    }

    #[test]
    fn can_manage_data() {
        let mut mem = HiveMemory::default();
        let app_state = AppState::default();
        let db_pool = DbPool::default();

        mem.insert(app_state.clone());
        mem.insert(db_pool.clone());

        let hm_app_state = mem.get::<AppState>().expect("Failed to get app state as expected");
        let hm_db_pool = mem.get::<DbPool>().expect("Failed to get db pool as expected");
        let hm_app_state_2 = mem.get::<AppState>().expect("Failed to get app state as expected");

        assert_eq!(hm_app_state_2.api_key,
                   app_state.api_key,
                   "Api key should have matched, expected {}, actual {}",
                   app_state.api_key,
                   hm_app_state_2.api_key);

        let from_context = mem.get::<DbPool>()
            .expect("Failed to get db pool type as expected");
        assert_eq!(from_context.pretend_pool,
                   db_pool.pretend_pool,
                   "Api key should have matched, expected {}, actual {}",
                   db_pool.pretend_pool,
                   from_context.pretend_pool);
    }
}
