use crate::app::App;
use crate::cache::{CpuDecodeCache, GpuTextureCache};
use anyhow::{Context, Result};
use std::sync::Arc;
use winit::window::Window;

pub struct GraphicsResources {
    pub renderer: crate::renderer::Renderer,
    pub estimated_vram_mb: usize,
}

fn fallback_vram_capacity_mb(adapter_info: &wgpu::AdapterInfo) -> usize {
    match adapter_info.backend {
        wgpu::Backend::Vulkan | wgpu::Backend::Dx12 | wgpu::Backend::Metal => {
            match adapter_info.device_type {
                wgpu::DeviceType::DiscreteGpu => 2048,
                wgpu::DeviceType::IntegratedGpu => 1024,
                _ => 512,
            }
        }
        wgpu::Backend::Gl => {
            if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
                2048
            } else {
                512
            }
        }
        wgpu::Backend::BrowserWebGpu => 256,
        _ => 512,
    }
}

#[cfg(target_os = "windows")]
fn query_dxgi_vram_budget_mb(adapter_info: &wgpu::AdapterInfo) -> Option<usize> {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::Foundation::LUID;

    type HResult = i32;

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    impl Guid {
        const fn from_u128(uuid: u128) -> Self {
            let bytes = uuid.to_be_bytes();
            Self {
                data1: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                data2: u16::from_be_bytes([bytes[4], bytes[5]]),
                data3: u16::from_be_bytes([bytes[6], bytes[7]]),
                data4: [
                    bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                    bytes[15],
                ],
            }
        }
    }

    #[repr(C)]
    struct IUnknown {
        lp_vtbl: *const IUnknownVtbl,
    }

    #[repr(C)]
    struct IUnknownVtbl {
        query_interface: unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *const Guid,
            *mut *mut core::ffi::c_void,
        ) -> HResult,
        add_ref: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
        release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    }

    #[repr(C)]
    struct DxgiObjectVtbl {
        base: IUnknownVtbl,
        set_private_data: usize,
        set_private_data_interface: usize,
        get_private_data: usize,
        get_parent: usize,
    }

    #[repr(C)]
    struct DxgiFactoryVtbl {
        base: DxgiObjectVtbl,
        enum_adapters: usize,
        make_window_association: usize,
        get_window_association: usize,
        create_swap_chain: usize,
        create_software_adapter: usize,
    }

    #[repr(C)]
    struct DxgiFactory1Vtbl {
        base: DxgiFactoryVtbl,
        enum_adapters1:
            unsafe extern "system" fn(*mut IDXGIFactory1, u32, *mut *mut IDXGIAdapter1) -> HResult,
        is_current: usize,
    }

    #[repr(C)]
    struct IDXGIFactory1 {
        lp_vtbl: *const DxgiFactory1Vtbl,
    }

    #[repr(C)]
    struct DxgiAdapterVtbl {
        base: DxgiObjectVtbl,
        enum_outputs: usize,
        get_desc: usize,
        check_interface_support: usize,
    }

    #[repr(C)]
    struct DxgiAdapter1Vtbl {
        base: DxgiAdapterVtbl,
        get_desc1: unsafe extern "system" fn(*mut IDXGIAdapter1, *mut DxgiAdapterDesc1) -> HResult,
    }

    #[repr(C)]
    struct IDXGIAdapter1 {
        lp_vtbl: *const DxgiAdapter1Vtbl,
    }

    #[repr(C)]
    struct DxgiAdapter2Vtbl {
        base: DxgiAdapter1Vtbl,
        get_desc2: usize,
    }

    #[repr(C)]
    struct DxgiAdapter3Vtbl {
        base: DxgiAdapter2Vtbl,
        register_hardware_content_protection_teardown_status_event: usize,
        unregister_hardware_content_protection_teardown_status: usize,
        query_video_memory_info: unsafe extern "system" fn(
            *mut IDXGIAdapter3,
            u32,
            i32,
            *mut DxgiQueryVideoMemoryInfo,
        ) -> HResult,
        set_video_memory_reservation: usize,
        register_video_memory_budget_change_notification_event: usize,
        unregister_video_memory_budget_change_notification: usize,
    }

    #[repr(C)]
    struct IDXGIAdapter3 {
        lp_vtbl: *const DxgiAdapter3Vtbl,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct DxgiAdapterDesc1 {
        description: [u16; 128],
        vendor_id: u32,
        device_id: u32,
        sub_sys_id: u32,
        revision: u32,
        dedicated_video_memory: usize,
        dedicated_system_memory: usize,
        shared_system_memory: usize,
        adapter_luid: LUID,
        flags: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct DxgiQueryVideoMemoryInfo {
        budget: u64,
        current_usage: u64,
        available_for_reservation: u64,
        current_reservation: u64,
    }

    const DXGI_ADAPTER_FLAG_SOFTWARE: u32 = 2;
    const DXGI_MEMORY_SEGMENT_GROUP_LOCAL: i32 = 0;

    unsafe extern "system" {
        fn CreateDXGIFactory1(riid: *const Guid, ppfactory: *mut *mut core::ffi::c_void)
        -> HResult;
    }

    unsafe fn desc_matches(desc: &DxgiAdapterDesc1, adapter_info: &wgpu::AdapterInfo) -> bool {
        let is_software = (desc.flags & DXGI_ADAPTER_FLAG_SOFTWARE) != 0;
        !is_software
            && desc.vendor_id == adapter_info.vendor
            && desc.device_id == adapter_info.device
    }

    unsafe fn release_iunknown(ptr: *mut IUnknown) {
        if !ptr.is_null() {
            // SAFETY: `ptr` is a live COM interface pointer obtained from DXGI and released
            // exactly once along this function's control flow.
            unsafe {
                ((*(*ptr).lp_vtbl).release)(ptr as *mut core::ffi::c_void);
            }
        }
    }

    const IID_IDXGIFACTORY1: Guid = Guid::from_u128(0x770aae78_f26f_4dba_a829_253c83d1b387);
    const IID_IDXGIADAPTER3: Guid = Guid::from_u128(0x645967a4_1392_4310_a798_8053ce3e93fd);

    // SAFETY: DXGI returns raw COM pointers here. We null-check them before use, only call
    // methods from the matching vtables, and release every acquired interface before returning.
    unsafe {
        let mut factory = MaybeUninit::<*mut core::ffi::c_void>::zeroed();
        if CreateDXGIFactory1(&IID_IDXGIFACTORY1, factory.as_mut_ptr()) < 0 {
            return None;
        }
        let factory = factory.assume_init() as *mut IDXGIFactory1;
        if factory.is_null() {
            return None;
        }

        let mut adapter_index = 0;
        loop {
            let mut adapter = MaybeUninit::<*mut IDXGIAdapter1>::zeroed();
            let enum_hr = ((*(*factory).lp_vtbl).enum_adapters1)(
                factory,
                adapter_index,
                adapter.as_mut_ptr(),
            );
            if enum_hr < 0 {
                release_iunknown(factory as *mut IUnknown);
                return None;
            }

            let adapter = adapter.assume_init();
            if adapter.is_null() {
                release_iunknown(factory as *mut IUnknown);
                return None;
            }

            let mut desc = MaybeUninit::<DxgiAdapterDesc1>::zeroed();
            let desc_hr = ((*(*adapter).lp_vtbl).get_desc1)(adapter, desc.as_mut_ptr());
            if desc_hr < 0 {
                release_iunknown(adapter as *mut IUnknown);
                adapter_index += 1;
                continue;
            }
            let desc = desc.assume_init();

            if desc_matches(&desc, adapter_info) {
                let mut adapter3 = MaybeUninit::<*mut core::ffi::c_void>::zeroed();
                let query_hr = ((*(*adapter).lp_vtbl).base.base.base.query_interface)(
                    adapter as *mut core::ffi::c_void,
                    &IID_IDXGIADAPTER3,
                    adapter3.as_mut_ptr(),
                );
                if query_hr < 0 {
                    release_iunknown(adapter as *mut IUnknown);
                    release_iunknown(factory as *mut IUnknown);
                    return None;
                }

                let adapter3 = adapter3.assume_init() as *mut IDXGIAdapter3;
                if adapter3.is_null() {
                    release_iunknown(adapter as *mut IUnknown);
                    release_iunknown(factory as *mut IUnknown);
                    return None;
                }

                let mut memory_info = DxgiQueryVideoMemoryInfo::default();
                let memory_hr = ((*(*adapter3).lp_vtbl).query_video_memory_info)(
                    adapter3,
                    0,
                    DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                    &mut memory_info,
                );
                release_iunknown(adapter3 as *mut IUnknown);
                release_iunknown(adapter as *mut IUnknown);
                release_iunknown(factory as *mut IUnknown);
                if memory_hr < 0 {
                    return None;
                }

                let budget_mb = (memory_info.budget / (1024 * 1024)) as usize;
                return Some(budget_mb.max(128));
            }

            release_iunknown(adapter as *mut IUnknown);
            adapter_index += 1;
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn query_dxgi_vram_budget_mb(_adapter_info: &wgpu::AdapterInfo) -> Option<usize> {
    None
}

fn detect_vram_capacity_mb(adapter_info: &wgpu::AdapterInfo) -> usize {
    if let Some(dxgi_budget_mb) = query_dxgi_vram_budget_mb(adapter_info) {
        tracing::info!(
            "[Graphics] DXGI local memory budget detected: {} MB (settings max: {} MB)",
            dxgi_budget_mb,
            ((dxgi_budget_mb / 2).max(64) / 64) * 64
        );
        dxgi_budget_mb
    } else {
        let fallback_mb = fallback_vram_capacity_mb(adapter_info);
        tracing::info!(
            "[Graphics] Falling back to VRAM heuristic: {} MB (settings max: {} MB)",
            fallback_mb,
            ((fallback_mb / 2).max(64) / 64) * 64
        );
        fallback_mb
    }
}

async fn create_graphics_resources(window: Arc<Window>) -> Result<GraphicsResources> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let surface = instance
        .create_surface(window.clone())
        .context("Failed to create surface")?;

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .context("Failed to find an appropriate adapter")?;

    let adapter_info = adapter.get_info();

    let estimated_vram_mb = detect_vram_capacity_mb(&adapter_info);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("HinaView_Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            experimental_features: Default::default(),
            trace: wgpu::Trace::default(),
        })
        .await
        .context("Failed to create device")?;

    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    Ok(GraphicsResources {
        renderer: crate::renderer::Renderer::new(device, queue, surface, config, surface_format),
        estimated_vram_mb,
    })
}

pub async fn rebuild_renderer(window: Arc<Window>) -> Result<GraphicsResources> {
    create_graphics_resources(window).await
}

pub async fn init_app_state(
    window: Arc<Window>,
    location: crate::settings::model::ConfigStorageLocation,
) -> App {
    let graphics = create_graphics_resources(window.clone())
        .await
        .expect("Failed to initialize graphics");
    let size = window.inner_size();

    let (scheduler, upload_queue) =
        crate::pipeline::init_pipeline_with_cache(CpuDecodeCache::new());

    let mut app = App::new(scheduler, upload_queue, location);
    app.set_vram_capacity_mb(graphics.estimated_vram_mb);
    app.texture_manager
        .set_gpu_cache(GpuTextureCache::new_from_vram(graphics.estimated_vram_mb));
    app.window_size = (size.width, size.height);

    app.renderer = Some(graphics.renderer);
    if let Some(renderer) = app.renderer.as_ref() {
        app.toast_renderer = Some(crate::ui_overlay::EguiToastRenderer::new(
            &window,
            &renderer.device,
            renderer.surface_config.format,
        ));
    }

    app
}
