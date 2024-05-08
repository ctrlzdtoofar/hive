use std::any::{Any, TypeId};
use crate::context::HiveMemory;

pub(crate) trait Journey {

    fn add_rollback<J: Any + Send + Sync>(&mut self, job: J);
    fn get_rollback_job<T: Any + Sync + Send + 'static>(&self) -> Option<&T>;
    fn transactional(&mut self);
    fn rollback_sequence(&mut self) -> Vec<TypeId>;
}

#[derive(Default)]
pub struct HiveJourney {
    pub mem: HiveMemory,
    pub completed: Vec<TypeId>,
    pub transactional: bool
}

impl Journey for HiveJourney {

    fn add_rollback<J: Any + Send + Sync>(&mut self, job: J) {
        self.completed.push(TypeId::of::<J>());
        self.mem.insert(job);
    }

    fn get_rollback_job<T: Any + Sync + Send + 'static>(&self) -> Option<&T> {
        self.mem.get::<T>()
    }

    fn transactional(&mut self) {
        self.transactional = true;
    }

    fn rollback_sequence(&mut self) -> Vec<TypeId> {
        let mut copy = self.completed.clone();
        copy.reverse();
        copy
    }
}

#[macro_export]
macro_rules! create_journey {
    ($mem:expr) => {
        HiveJourney {
            mem: $mem,
            completed: vec![],
            transactional: false,
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::bee::worker::{Job, WorkerError};
    use super::*;

    #[derive(Clone, Default)]
    pub struct UpdatePatientDemo {
        pub name: String,
        pub address: String,
    }

    impl Job for UpdatePatientDemo {

        type Output = String;
        fn execute_job(&self, _: &HiveMemory)  -> Result<Self::Output, WorkerError> {
            Ok("worked".to_string())
        }
    }

    #[test]
    fn can_remember_rollback_job() {
        let put_back = UpdatePatientDemo {
            name: "Jane".to_string(),
            ..Default::default()
        };

        let mut journey = create_journey!(HiveMemory::default());
        journey.add_rollback(put_back.clone());

        let rollback = journey.get_rollback_job::<UpdatePatientDemo>().expect("Should have gotten the rollback job UpdatePatientDemo");
        assert_eq!(rollback.name, put_back.name, "Expected names to match, result {}, actual {}", rollback.name, put_back.name)
    }
}