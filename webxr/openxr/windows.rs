use winapi::shared::dxgi;
use winapi::shared::dxgiformat;
use winapi::shared::winerror::{DXGI_ERROR_NOT_FOUND, S_OK};
use winapi::Interface;
use wio::com::ComPtr;

pub type OpenXRGraphicsType = D3D11;

pub fn pick_extensions(exts: &mut ExtensionSet) {
    exts.khr_d3d11_enable = true;
}

pub fn surfman_texture_from_swapchain_texture<G: Graphics>(
    image: G::SwapchainImage,
) -> Option<surfman::SurfaceTexture> {
    let image = ComPtr::from_raw(image);
    image.AddRef();
    device.create_surface_texture_from_texture(context, &self.size.to_untyped(), image)
}

fn get_matching_adapter(
    requirements: &Requirements,
) -> Result<ComPtr<dxgi::IDXGIAdapter1>, String> {
    unsafe {
        let mut factory_ptr: *mut dxgi::IDXGIFactory1 = ptr::null_mut();
        let result = dxgi::CreateDXGIFactory1(
            &dxgi::IDXGIFactory1::uuidof(),
            &mut factory_ptr as *mut _ as *mut _,
        );
        assert_eq!(result, S_OK);
        let factory = ComPtr::from_raw(factory_ptr);

        let index = 0;
        loop {
            let mut adapter_ptr = ptr::null_mut();
            let result = factory.EnumAdapters1(index, &mut adapter_ptr);
            if result == DXGI_ERROR_NOT_FOUND {
                return Err("No matching adapter".to_owned());
            }
            assert_eq!(result, S_OK);
            let adapter = ComPtr::from_raw(adapter_ptr);
            let mut adapter_desc = mem::zeroed();
            let result = adapter.GetDesc1(&mut adapter_desc);
            assert_eq!(result, S_OK);
            let adapter_luid = &adapter_desc.AdapterLuid;
            if adapter_luid.LowPart == requirements.adapter_luid.LowPart
                && adapter_luid.HighPart == requirements.adapter_luid.HighPart
            {
                return Ok(adapter);
            }
        }
    }
}

pub fn create_surfman_adapter() -> Option<SurfmanAdapter> {
    let instance = create_instance(false, false).ok()?;
    let system = instance
        .instance
        .system(FormFactor::HEAD_MOUNTED_DISPLAY)
        .ok()?;

    let requirements = D3D11::requirements(&instance.instance, system).ok()?;
    let adapter = get_matching_adapter(&requirements).ok()?;
    Some(SurfmanAdapter::from_dxgi_adapter(adapter.up()))
}

pub fn pick_format(formats: &[dxgiformat::DXGI_FORMAT]) -> dxgiformat::DXGI_FORMAT {
    // TODO: extract the format from surfman's device and pick a matching
    // valid format based on that. For now, assume that eglChooseConfig will
    // gravitate to B8G8R8A8.
    warn!("Available formats: {:?}", formats);
    for format in formats {
        match *format {
            dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM => return *format,
            //dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM => return *format,
            f => {
                warn!("Backend requested unsupported format {:?}", f);
            }
        }
    }

    panic!("No formats supported amongst {:?}", formats);
}

pub fn create_session(
    device: &SurfmanDevice,
    context: Option<&mut surfman::Context>,
    instance: &Instance,
    system: SystemId,
) -> Result<(Session<D3D11>, FrameWaiter, FrameStream<D3D11>), Error> {
    // Get the current surfman device and extract its D3D device. This will ensure
    // that the OpenXR runtime's texture will be shareable with surfman's surfaces.
    let native_device = device.native_device();
    let d3d_device = native_device.d3d11_device;

    // FIXME: we should be using these graphics requirements to drive the actual
    //        d3d device creation, rather than assuming the device that surfman
    //        already created is appropriate. OpenXR returns a validation error
    //        unless we call this method, so we call it and ignore the results
    //        in the short term.
    let _requirements = D3D11::requirements(&instance, system)
        .map_err(|e| Error::BackendSpecific(format!("D3D11::requirements {:?}", e)))?;

    unsafe {
        instance
            .create_session::<D3D11>(system, &SessionCreateInfo { device: d3d_device })
            .map_err(|e| Error::BackendSpecific(format!("Instance::create_session {:?}", e)))
    }
}
