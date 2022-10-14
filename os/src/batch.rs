//! batch subsystem
///这边是几个app的执行系统
/// 大概思路就是app一开始存在os的数据段上，然后暴露出几个外部地址，
/// 然后根据这些暴露的地址从数据段中读出，并挨个运行
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;
use lazy_static::*;

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    ///这个函数就打印一些调试的数据，以此来确定我们的app管理器正确初始化了
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    //从数据段中加载app到指定位置
    unsafe fn load_app(&self, app_id: usize) {
        //序号超出了，炸了
        if app_id >= self.num_app {
            println!("All applications completed!");

            #[cfg(feature = "board_qemu")]
            use crate::board::QEMUExit;
            #[cfg(feature = "board_qemu")]
            crate::board::QEMU_EXIT_HANDLE.exit_success();

            #[cfg(feature = "board_k210")]
            panic!("All applications completed!");
        }
        println!("[kernel] Loading app_{}", app_id);
        // clear icache，清除指令缓存(担心残余)
        asm!("fence.i");
        // clear app area
        //下面这部分就是简单的内存复制了，这里就不多说
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    //全局app管理器变量，lazy_static!是rust的一个宏，暂时先不管
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app(); //外部指针指向app的数量
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile(); //读出app的数量
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            //接下来分别读出每个app地址
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

/// init batch subsystem
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// run next app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    //获取app管理器以及当前app的索引(最开始+1)
    unsafe {
        app_manager.load_app(current_app);
    }
    //索引+1
    app_manager.move_to_next_app();
    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources

    //下面这段代码才是整个ch2的核心之一
    //看这段代码之前需要首先明确以下几个点：
    /*
     * a. 目前OS处于内核态(因为默认就是内核态)sp指针还指向boot_stack
     * 注意这里传入给__restore的是**内核栈的栈指针**
     *
     */
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(), //在这里初始化sscratch的值
        )) as *const _ as usize);
    }
    //从这里开始就开始使用用户栈，用户栈通过切换到内核态的时候就开始使用内核栈了，boot_stack不再使用
    panic!("Unreachable in batch::run_current_app!");
}
