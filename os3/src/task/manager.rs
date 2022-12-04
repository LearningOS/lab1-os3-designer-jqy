use lazy_static::*;
use crate::config::MAX_APP_NUM;
use crate::sync::UPSafeCell;
use crate::loader::{get_num_app, init_app_cx};
use super::context::TaskContext;
use super::task::{TaskControlBlock, TaskStatus};
use super::switch::__switch;

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            // init_app_cx(i) 往内核栈上压入了第 i 个 task 的 TrapContext (包含第 i 个任务的指令启动地址和应用栈指针)
            // TaskContext::goto_restore 将 1 步骤中压入内核栈的数据载入对应的寄存器, 从而实现了
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaslManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            }
        }
    };
}

impl TaskManager {
    pub fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let first_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, first_task_cx_ptr);
        }
        panic!("unreachable in run_first_task");
    }

    pub fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].task_status = TaskStatus::Ready;
    }

    pub fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].task_status = TaskStatus::Exited;
    }

    pub fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        ((current + 1) .. (current + self.num_app + 1))
        .map(|v| v % id.num_app)
        .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    pub fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // 返回用户态
        } else {
            panic!("All applications completed!");
        }
        
    }
}