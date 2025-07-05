use alloc::collections::vec_deque::VecDeque;

use crate::scheduler::process::Process;

pub mod process;

pub struct Scheduler{
    current_process: Option<Process>,
    queue: VecDeque<Process>,
}

impl Scheduler{
    pub fn new() -> Self{
        Scheduler { current_process: None, queue: VecDeque::new() }
    }

    pub fn add_to_queue(&mut self, process: Process){
        self.queue.push_back(process);
    }

    pub unsafe fn end_current_process(&mut self){
        self.current_process = None
    }

    pub unsafe fn next_process(&mut self){
        let new_process = self.queue.pop_front();
        self.current_process = new_process;
    }

    pub fn resume_current_process(&self){
        if let Some(process) = &self.current_process{
            unsafe { process.execute_process(); };
        }
    }

    pub fn get_current_process(&self) -> Option<&Process>{
        self.current_process.as_ref()
    }
}