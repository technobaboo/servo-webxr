use log::warn;
use openxr::{
    opengles::SessionCreateInfo, ExtensionSet, FrameStream, FrameWaiter, Graphics, Instance,
    OpenGlEs, Session, SystemId,
};
use sparkle::gl;
use surfman::connection::NativeConnection;
use webxr_api::Error;

use crate::egl::types::EGLContext;

pub type OpenXRGraphicsType = OpenGlEs;

pub fn pick_extensions(exts: &mut ExtensionSet) {
    exts.khr_opengl_es_enable = true;
    exts.mndx_egl_enable = true;
}

pub fn pick_format(formats: &[u32]) -> u32 {
    // TODO: extract the format from surfman's device and pick a matching
    // valid format based on that. For now, assume that eglChooseConfig will
    // gravitate to B8G8R8A8.
    warn!("Available formats: {:?}", formats);
    for format in formats {
        match *format {
            gl::ffi_gl::RGBA8 => return *format,
            //dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM => return *format,
            f => {
                warn!("Backend requested unsupported format {:?}", f);
            }
        }
    }

    panic!("No formats supported amongst {:?}", formats);
}

pub fn create_session(
    device: &surfman::Device,
    egl_context: EGLContext,
    instance: &Instance,
    system: SystemId,
) -> Result<(Session<OpenGlEs>, FrameWaiter, FrameStream<OpenGlEs>), Error> {
    let _requirements = OpenGlEs::requirements(&instance, system)
        .map_err(|e| Error::BackendSpecific(format!("OpenGlEs::requirements {:?}", e)))?;

    unsafe {
        let session_create_info = SessionCreateInfo::Egl {
            display: device.connection().native_connection().egl_display() as *mut _,
            config: std::ptr::null_mut(),
            // context: device.native_context(context).egl_context() as usize as *mut _,
            context: egl_context as *mut _,
            get_proc_address: surfman::platform::generic::get_proc_address_raw(),
        };
        instance
            .create_session::<OpenGlEs>(system, &session_create_info)
            .map_err(|e| Error::BackendSpecific(format!("Instance::create_session {:?}", e)))
    }
}
