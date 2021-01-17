use std::ffi::c_void;
use std::os::raw::c_char;

use std::io::Write;

use std::ffi::CStr;
use std::ffi::CString;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use libloading::Library;
use libloading::Symbol;

use tempfile::NamedTempFile;

static real_NPP_Stream_As_File: Lazy<Mutex<Option<NPP_StreamAsFileProcPtr>>> = Lazy::new(|| Mutex::new(None));
static real_NPP_NewStream: Lazy<Mutex<Option<NPP_NewStreamProcPtr>>> = Lazy::new(|| Mutex::new(None));
static real_NPN_GetUrlNotify: Lazy<Mutex<Option<NPN_GetUrlNotifyProcPtr>>> = Lazy::new(|| Mutex::new(None));

static temp_files: Lazy<Mutex<Vec<NamedTempFile>>> = Lazy::new(|| Mutex::new(vec![]));

static hypercosm: Lazy<Library> = Lazy::new(|| Library::new("xnphypercosm.dll").unwrap());
static real_NP_GetEntryPoints: Lazy<Symbol<unsafe extern "stdcall" fn(&mut NPPluginFuncs) -> NPError>> = Lazy::new(|| unsafe { hypercosm.get(b"NP_GetEntryPoints\0").unwrap() });
static real_NP_Initialize: Lazy<Symbol<unsafe extern "stdcall" fn(&mut NPNetscapeFuncs) -> NPError>> = Lazy::new(|| unsafe { hypercosm.get(b"NP_Initialize\0").unwrap() });
static real_NP_Shutdown: Lazy<Symbol<unsafe extern "stdcall" fn() -> NPError>> = Lazy::new(|| unsafe { hypercosm.get(b"NP_Shutdown\0").unwrap() });


type NPError = i16;

type NPP_StreamAsFileProcPtr = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char);
type NPP_NewStreamProcPtr = unsafe extern "C" fn(*mut c_void, *mut c_char, &mut NPStream, u8, &mut u16) -> NPError;

type NPN_GetUrlNotifyProcPtr = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, *mut c_void) -> NPError;

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
pub struct NPNetscapeFuncs {
    size: u16,
    version: u16,
    geturl: *mut c_void,
    posturl: *mut c_void,
    requestread: *mut c_void,
    newstream: *mut c_void,
    write: *mut c_void,
    destroystream: *mut c_void,
    status: *mut c_void,
    uagent: *mut c_void,
    memalloc: *mut c_void,
    memfree: *mut c_void,
    memflush: *mut c_void,
    reloadplugins: *mut c_void,
    getJavaEnv: *mut c_void,
    getJavaPeer: *mut c_void,
    geturlnotify: NPN_GetUrlNotifyProcPtr,
    posturlnotify: *mut c_void,
    getvalue: *mut c_void,
    setvalue: *mut c_void,
    invalidaterect: *mut c_void,
    invalidateregion: *mut c_void,
    forceredraw: *mut c_void,
    getstringidentifier: *mut c_void,
    getstringidentifiers: *mut c_void,
    getintidentifier: *mut c_void,
    identifierisstring: *mut c_void,
    utf8fromidentifier: *mut c_void,
    intfromidentifier: *mut c_void,
    createobject: *mut c_void,
    retainobject: *mut c_void,
    releaseobject: *mut c_void,
    invoke: *mut c_void,
    invokeDefault: *mut c_void,
    evaluate: *mut c_void,
    getproperty: *mut c_void,
    setproperty: *mut c_void,
    removeproperty: *mut c_void,
    hasproperty: *mut c_void,
    hasmethod: *mut c_void,
    releasevariantvalue: *mut c_void,
    setexception: *mut c_void,
    pushpopupsenabledstate: *mut c_void,
    poppopupsenabledstate: *mut c_void,
    enumerate: *mut c_void,
    pluginthreadasynccall: *mut c_void,
    construct: *mut c_void,
    getvalueforurl: *mut c_void,
    setvalueforurl: *mut c_void,
    getauthenticationinfo: *mut c_void,
    scheduletimer: *mut c_void,
    unscheduletimer: *mut c_void,
    popupcontextmenu: *mut c_void,
    convertpoint: *mut c_void,
    handleevent: *mut c_void,
    unfocusinstance: *mut c_void,
    urlredirectresponse: *mut c_void,
    initasyncsurface: *mut c_void,
    finalizeasyncsurface: *mut c_void,
    setcurrentasyncsurface: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
struct NPStream {
    pdata: *mut c_void,
    ndata: *mut c_void,
    url: *const c_char,
    end: u32,
    lastmodified: u32,
    notifyData: *mut c_void,
    headers: *const c_char
}

extern "C" fn NPP_NewStream(npp: *mut c_void, mime: *mut c_char, stream: &mut NPStream, seekable: u8, mode: &mut u16) -> NPError {
    let mut file = std::fs::OpenOptions::new().append(true).create(true).open("C:\\hypercosmlog.txt").unwrap();
    unsafe {
        let url = CStr::from_ptr(stream.url);
        let mime_cstr = CStr::from_ptr(mime);
        writeln!(&mut file, "Stream MIME type: {:?}", mime_cstr).unwrap();
        writeln!(&mut file, "Stream URL: {:?}", url).unwrap();
        writeln!(&mut file, "Stream notify: {:#?}", stream.notifyData);
        
        let real_NPP_NewStream_func = real_NPP_NewStream.lock().unwrap().unwrap();
        let result = if url.to_str().unwrap() == "http://highprogrammer.com/alan/hypercosm/applets/airhockey/" {
            1 // NPERR_GENERIC_ERROR
        } else {
            real_NPP_NewStream_func(npp, mime, stream, seekable, mode)
        };
        writeln!(&mut file, "Stream Mode: {:?}", *mode).unwrap();
        result
    }
}

extern "C" fn NPP_StreamAsFile(npp: *mut c_void, stream: *mut c_void, fname: *const c_char) {
    unsafe {
        let mut temp_files_lock = temp_files.lock().unwrap();
        let temp = NamedTempFile::new().unwrap();
        let fname = std::str::from_utf8(CStr::from_ptr(fname).to_bytes()).unwrap(); // TODO: ENCODING PROBLEMS? PATH WITH NON-ENGLISH CHARS???
        std::fs::copy(fname, temp.path()).unwrap();
        let given_name = CString::new(temp.path().to_str().unwrap()).unwrap();
        let real_NPP_Stream_As_File_func = real_NPP_Stream_As_File.lock().unwrap().unwrap();
        real_NPP_Stream_As_File_func(npp, stream, given_name.as_ptr());
        std::mem::forget(given_name); // TODO: Memory leak. Store globally?
        temp_files_lock.push(temp);
    }
}

#[no_mangle]
pub extern "stdcall" fn NP_GetEntryPoints(plugin_funcs: &mut NPPluginFuncs) -> NPError {
    unsafe {
        let result = real_NP_GetEntryPoints(plugin_funcs);
        if result != 0 {
            return result;
        }

        let mut stream_as_file_lock = real_NPP_Stream_As_File.lock().unwrap();
        *stream_as_file_lock = Some(plugin_funcs.asfile);
        plugin_funcs.asfile = NPP_StreamAsFile;

        let mut newstream_lock = real_NPP_NewStream.lock().unwrap();
        *newstream_lock = Some(plugin_funcs.newstream);
        //plugin_funcs.newstream = NPP_NewStream;
        return 0;
    }
}

pub extern "C" fn NPN_GetUrlNotify(instance: *mut c_void, url: *const c_char, window: *const c_char, notify_data: *mut c_void) -> NPError {
    unsafe {
        let mut file = std::fs::OpenOptions::new().append(true).create(true).open("C:\\hypercosmlog.txt").unwrap();
        writeln!(&mut file, "NPN_GetUrlNotify on {}", CStr::from_ptr(url).to_str().unwrap());
        let lock = real_NPN_GetUrlNotify.lock().unwrap();
        let real_NPN_GetUrlNotify_func = lock.unwrap();
        real_NPN_GetUrlNotify_func(instance, url, window, notify_data)
    }
}

#[no_mangle]
pub extern "stdcall" fn NP_Initialize(funcs: &mut NPNetscapeFuncs) -> NPError {
    unsafe {
        {
            let mut urlnotify_lock = real_NPN_GetUrlNotify.lock().unwrap();
            *urlnotify_lock = Some(funcs.geturlnotify);
            //funcs.geturlnotify = NPN_GetUrlNotify;
        }
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