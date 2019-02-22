use libc::c_void;
use libc::c_char;
use libc::size_t;

#[link(name = "model_runner")]
extern {
    pub fn getModelRunnerInstance(model_path: *const c_char) -> *const c_void;
    pub fn deleteModelRunnerInstance(ptr: *const c_void);
    pub fn modelRunnerInfer(ptr: *const c_void, input: *const *const c_char, input_n: size_t, result: *mut *const *const c_char, result_n: *mut size_t, max_len: size_t);
}