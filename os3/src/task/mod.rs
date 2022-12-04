use self::manager::TASK_MANAGER;

mod context;
mod switch;
mod manager;
#[allow(clippy::module_inception)]
mod task;


pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

pub fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended()
}

pub fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}