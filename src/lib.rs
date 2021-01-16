use std::ffi::c_void;
use std::os::raw::c_char;

use std::ffi::CStr;
use std::ffi::CString;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use libloading::Library;
use libloading::Symbol;

use tempfile::NamedTempFile;

static real_NPP_Stream_As_File: Lazy<Mutex<Option<NPP_StreamAsFileProcPtr>>> = Lazy::new(|| Mutex::new(None));

static temp_files: Lazy<Mutex<Vec<NamedTempFile>>> = Lazy::new(|| Mutex::new(vec![]));

static hypercosm: Lazy<Library> = Lazy::new(|| Library::new("xnphypercosm.dll").unwrap());
static real_NP_GetEntryPoints: Lazy<Symbol<unsafe extern "stdcall" fn(&mut NPPluginFuncs) -> NPError>> = Lazy::new(|| unsafe { hypercosm.get(b"NP_GetEntryPoints\0").unwrap() });
static real_NP_Initialize: Lazy<Symbol<unsafe extern "stdcall" fn(*mut c_void) -> NPError>> = Lazy::new(|| unsafe { hypercosm.get(b"NP_Initialize\0").unwrap() });
static real_NP_Shutdown: Lazy<Symbol<unsafe extern "stdcall" fn() -> NPError>> = Lazy::new(|| unsafe { hypercosm.get(b"NP_Shutdown\0").unwrap() });


type NPError = i16;

type NPP_StreamAsFileProcPtr = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char);
type NPP_NewStreamProcPtr = unsafe extern "C" fn(*mut c_void, *mut c_char, &mut NPStream, u8, &mut u16) -> NPError;

#[repr(C)]
pub struct NPPluginFuncs {
    size: u16,
    version: u16,
    newp: *mut c_void,
    destroy: *mut c_void,
    setwindow: *mut c_void,
    newstream: NPP_NewStreamProcPtr,
    destroystream: *mut c_void,
    asfile: NPP_StreamAsFileProcPtr,
    writeready: *mut c_void,
    write: *mut c_void,
    print: *mut c_void,
    event: *mut c_void,
    urlnotify: *mut c_void,
    javaClass: *mut c_void,
    getvalue: *mut c_void,
    setvalue: *mut c_void,
    gotfocus: *mut c_void,
    lostfocus: *mut c_void,
    urlredirectnotify: *mut c_void,
    clearsitedata: *mut c_void,
    getsiteswithdata: *mut c_void,
    didComposite: *mut c_void,

}

#[repr(C)]
struct NPStream {
    pdata: *mut c_void,
    ndata: *mut c_void,
    url: *const c_char,
    end: u32,
    lastmodified: u32,
    notifyData: *mut c_void,
    headers: *const c_char
}

extern "C" fn NPP_StreamAsFile(npp: *mut c_void, stream: *mut c_void, fname: *const c_char) {
    unsafe {
        let mut lock = temp_files.lock().unwrap();
        let temp = NamedTempFile::new().unwrap();
        let fname = std::str::from_utf8(CStr::from_ptr(fname).to_bytes()).unwrap(); // TODO: ENCODING PROBLEMS? PATH WITH NON-ENGLISH CHARS???
        std::fs::copy(fname, temp.path()).unwrap();
        let given_name = CString::new(temp.path().to_str().unwrap()).unwrap();
        let real_NPP_Stream_As_File_func = real_NPP_Stream_As_File.lock().unwrap().unwrap();
        real_NPP_Stream_As_File_func(npp, stream, given_name.as_ptr());
        std::mem::forget(given_name); // TODO: Memory leak. Store globally?
        lock.push(temp);
    }
}

#[no_mangle]
pub extern "stdcall" fn NP_GetEntryPoints(plugin_funcs: &mut NPPluginFuncs) -> NPError {
    unsafe {
        let result = real_NP_GetEntryPoints(plugin_funcs);
        if result != 0 {
            return result;
        }
        let mut lock = real_NPP_Stream_As_File.lock().unwrap();
        *lock = Some(plugin_funcs.asfile);
        plugin_funcs.asfile = NPP_StreamAsFile;
        return 0;
    }
}

#[no_mangle]
pub extern "stdcall" fn NP_Initialize(funcs: *mut c_void) -> NPError {
    unsafe {
        real_NP_Initialize(funcs)
    }
}

#[no_mangle]
pub extern "stdcall" fn NP_Shutdown() -> NPError {
    unsafe {
        let result = real_NP_Shutdown();
        let mut lock = temp_files.lock().unwrap();
        lock.clear();
        result
    }
}