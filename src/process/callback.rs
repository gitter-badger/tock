use core::nonzero::NonZero;
use process;

#[derive(Clone,Copy)]
pub struct AppId {
    idx: usize
}

impl AppId {
    pub unsafe fn new(idx: usize) -> AppId {
        AppId {idx: idx}
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}


pub struct Callback {
    process_ptr: AppId,
    appdata: usize,
    fn_ptr: NonZero<*mut ()>
}

impl Callback {
    pub unsafe fn new(appid: AppId, appdata: usize, fn_ptr: *mut ()) -> Callback {
        Callback {
            process_ptr: appid,
            appdata: appdata,
            fn_ptr: NonZero::new(fn_ptr)
        }
    }

    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        process::schedule(process::Callback{
            r0: r0,
            r1: r1,
            r2: r2,
            r3: self.appdata,
            pc: *self.fn_ptr as usize
        }, self.process_ptr)
    }
}

