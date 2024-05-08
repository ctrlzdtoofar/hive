use thiserror::Error;
use crate::bee::journey::HiveJourney;
use crate::context::HiveMemory;

#[derive(Error, Debug)]
pub enum WorkerError {
   #[error("database error: {0}")]
   DatabaseError(#[from] diesel::result::Error),

   #[error("network error: {0}")]
   NetworkError(#[from] std::io::Error),

   #[error("invalid input: {0}")]
   InvalidInput(String),

   #[error("permission denied")]
   PermissionDenied,

   #[error("internal error: {0}")]
   InternalError(String),

   #[error("job timeout exceeded")]
   Timeout,
}
pub trait Job {
   type Output;

   fn execute_job(&self, mem: &HiveMemory) -> Result<Self::Output, WorkerError>;
}

pub trait Worker<J, R> {
   type Output;

   fn execute_job(&self, journey: &mut HiveJourney, mem: &HiveMemory, job: J, rollback: R) -> Result<Self::Output, WorkerError>;
}

#[cfg(test)]
mod tests {
   use crate::bee::journey::{Journey, HiveJourney};
   use crate::bee::worker::Job;
   use crate::create_journey;
   use super::*;

   struct ProcessPayment {
      id: String,
      amount: i32,
   }

   struct ResponseData {
      balance: i32,
   }

   impl Job for ProcessPayment  {
      type Output = ResponseData;

      fn execute_job(&self, _: &HiveMemory) -> Result<Self::Output, WorkerError> {
         Ok(ResponseData {
            balance: 0
         })
      }
   }

   struct AthenaWorker();
   impl Worker<ProcessPayment, ProcessPayment> for AthenaWorker {
      type Output = ResponseData;

      fn execute_job(&self,
                     journey: &mut HiveJourney,
                     mem: &HiveMemory,
                     job: ProcessPayment,
                     rollback: ProcessPayment) -> Result<Self::Output, WorkerError> {

         match job.execute_job(mem) {
            Ok(resp) => {

               journey.add_rollback(rollback);
               Ok(resp)
            }
            Err(err) => {
               Err(err)
            }
         }
      }
   }

   #[test]
   fn can_run_job() {

      let payment = ProcessPayment {
         id: "123".to_string(),
         amount: 499
      };

      let refund = ProcessPayment {
         id: "123".to_string(),
         amount: -499
      };

      let memory = HiveMemory::default();
      let mut journey = create_journey!(HiveMemory::default());
      let worker = AthenaWorker();

      let resp = worker.execute_job(&mut journey, &memory, payment, refund).expect("athena worker should run w/o err");

      assert_eq!(resp.balance, 0); // per impl

   }
}
