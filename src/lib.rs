use std::{
    ffi::{c_char, c_void},
    fs,
    str::FromStr,
};

use ngx::core;
use ngx::ffi::{
    ngx_array_push, ngx_command_t, ngx_conf_t, ngx_http_handler_pt, ngx_http_module_t,
    ngx_http_phases_NGX_HTTP_ACCESS_PHASE, ngx_int_t, ngx_module_t, ngx_str_t, ngx_uint_t,
    NGX_CONF_TAKE1, NGX_HTTP_LOC_CONF, NGX_HTTP_LOC_CONF_OFFSET, NGX_HTTP_MODULE, NGX_LOG_EMERG,
};
use ngx::http::{HTTPStatus, HttpModule, Merge, MergeConfigError, Request};
use ngx::http::{HttpModuleLocationConf, HttpModuleMainConf, NgxHttpCoreModule};
use ngx::{http_request_handler, ngx_conf_log_error, ngx_string};

use nom::Finish;

use biscuit_auth::{AuthorizerBuilder, Biscuit, PublicKey};

mod parser;

struct Module;

impl HttpModule for Module {
    fn module() -> &'static ngx_module_t {
        unsafe { &*::core::ptr::addr_of!(ngx_http_auth_biscuit_module) }
    }

    unsafe extern "C" fn postconfiguration(cf: *mut ngx_conf_t) -> ngx_int_t {
        // SAFETY: this function is called with non-NULL cf always
        let cf = unsafe { &mut *cf };
        let cmcf = NgxHttpCoreModule::main_conf_mut(cf).expect("http core main conf");

        let h = unsafe {
            ngx_array_push(
                &mut cmcf.phases[ngx_http_phases_NGX_HTTP_ACCESS_PHASE as usize].handlers,
            ) as *mut ngx_http_handler_pt
        };
        if h.is_null() {
            return core::Status::NGX_ERROR.into();
        }
        // set an Access phase handler
        unsafe { *h = Some(biscuit_access_handler) };
        core::Status::NGX_OK.into()
    }
}

#[derive(Debug, Default)]
struct ModuleConfig {
    root: Option<PublicKey>,
    authorizer: Option<AuthorizerBuilder>,
}

unsafe impl HttpModuleLocationConf for Module {
    type LocationConf = ModuleConfig;
}

static mut NGX_HTTP_AUTH_BISCUIT_COMMANDS: [ngx_command_t; 3] = [
    ngx_command_t {
        name: ngx_string!("auth_biscuit_public_key"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_auth_biscuit_public_key),
        conf: NGX_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t {
        name: ngx_string!("auth_biscuit_authorizer_file"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_http_auth_biscuit_authorizer_file),
        conf: NGX_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_command_t::empty(),
];

static NGX_HTTP_AUTH_BISCUIT_MODULE_CTX: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration),
    postconfiguration: Some(Module::postconfiguration),
    create_main_conf: None,
    init_main_conf: None,
    create_srv_conf: None,
    merge_srv_conf: None,
    create_loc_conf: Some(Module::create_loc_conf),
    merge_loc_conf: Some(Module::merge_loc_conf),
};

// Generate the `ngx_modules` table with exported modules.
ngx::ngx_modules!(ngx_http_auth_biscuit_module);

#[used]
#[allow(non_upper_case_globals)]
pub static mut ngx_http_auth_biscuit_module: ngx_module_t = ngx_module_t {
    ctx: std::ptr::addr_of!(NGX_HTTP_AUTH_BISCUIT_MODULE_CTX) as _,
    commands: unsafe { &NGX_HTTP_AUTH_BISCUIT_COMMANDS[0] as *const _ as *mut _ },
    type_: NGX_HTTP_MODULE as _,
    ..ngx_module_t::default()
};

impl Merge for ModuleConfig {
    fn merge(&mut self, _prev: &ModuleConfig) -> Result<(), MergeConfigError> {
        Ok(())
    }
}

http_request_handler!(biscuit_access_handler, |request: &mut Request| {
    let co = Module::location_conf(request).expect("module config is none");

    let (root, authorizer) = match (co.root.as_ref(), co.authorizer.as_ref()) {
        // Neither `auth_biscuit_public_key` nor `auth_biscuit_authorizer_file` have been provided
        // we'll assume this is not enabled for this location.
        (None, None) => return core::Status::NGX_DECLINED,
        (Some(root), Some(authorizer)) => (root, authorizer),
        // If either are missing, module is not configured correctly
        _ => {
            // TODO: probably worth to log something?
            //ngx_conf_log_error!(NGX_LOG_EMERG, cf, "auth_biscuit is enabled but not fully configured");
            return HTTPStatus::FORBIDDEN.into();
        }
    };

    let mut token = None;
    for (name, value) in request.headers_in_iterator() {
        if let Ok(name) = name.to_str()
            && name.to_lowercase() == "authorization"
            && let Ok(value) = http::HeaderValue::from_bytes(value.as_bytes())
        {
            let Ok((_, value)) = parser::bearer_token(value.as_bytes()).finish() else {
                todo!()
            };
            token = Some(value.to_vec());
        }
    }

    let Some(token) = token else {
        return HTTPStatus::FORBIDDEN.into();
    };

    let Ok(token) = Biscuit::from_base64(&token, root) else {
        return HTTPStatus::FORBIDDEN.into();
    };

    let authorizer = authorizer.clone().time();

    let Ok(mut authorizer) = authorizer.build(&token) else {
        return HTTPStatus::FORBIDDEN.into();
    };

    match authorizer.authorize() {
        Ok(_) => {}
        Err(_) => return HTTPStatus::FORBIDDEN.into(),
    };

    core::Status::NGX_OK
});

extern "C" fn ngx_http_auth_biscuit_public_key(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);

        if conf.root.is_some() {
            ngx_conf_log_error!(
                NGX_LOG_EMERG,
                cf,
                "`auth_biscuit_public_key` does not support duplicates"
            );
            return ngx::core::NGX_CONF_ERROR;
        }

        let args: &[ngx_str_t] = (*(*cf).args).as_slice();

        let Ok(val) = args[1].to_str() else {
            ngx_conf_log_error!(
                NGX_LOG_EMERG,
                cf,
                "`auth_biscuit_public_key` argument is not utf-8 encoded"
            );
            return ngx::core::NGX_CONF_ERROR;
        };

        let Ok(public_key) = PublicKey::from_str(val) else {
            ngx_conf_log_error!(
                NGX_LOG_EMERG,
                cf,
                "`auth_biscuit_public_key` value appears invalid"
            );
            return ngx::core::NGX_CONF_ERROR;
        };

        conf.root = Some(public_key);
    };

    ngx::core::NGX_CONF_OK
}

extern "C" fn ngx_http_auth_biscuit_authorizer_file(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);

        if conf.authorizer.is_some() {
            ngx_conf_log_error!(
                NGX_LOG_EMERG,
                cf,
                "`auth_biscuit_authorizer_file` does not support duplicates"
            );
            return ngx::core::NGX_CONF_ERROR;
        }

        let args: &[ngx_str_t] = (*(*cf).args).as_slice();

        let Ok(val) = args[1].to_str() else {
            ngx_conf_log_error!(
                NGX_LOG_EMERG,
                cf,
                "`auth_biscuit_authorizer_file` argument is not utf-8 encoded"
            );
            return ngx::core::NGX_CONF_ERROR;
        };

        let code_rule = match fs::read_to_string(val) {
            Ok(content) => content,
            Err(_e) => {
                ngx_conf_log_error!(
                    NGX_LOG_EMERG,
                    cf,
                    "`auth_biscuit_authorizer_file` failed to read content"
                );
                return ngx::core::NGX_CONF_ERROR;
            }
        };

        let authorizer = AuthorizerBuilder::new()
            .code(&code_rule)
            .expect("should parse correctly");

        conf.authorizer = Some(authorizer)
    };

    ngx::core::NGX_CONF_OK
}
