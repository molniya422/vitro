// gpu.rs — D3D11 → D2D → DComp 管线(Glassboard windows/setup.rs 思路:资源集中一次创建,结构体持有即存活)
// v3 关键决策:交换链固定 WIN_W×WIN_H 一次成型,永不 Resize —— 布局自适应全部发生在绘制层
// (玻璃药丸居中画,两侧透明;DComp 对透明像素天然点击穿透)→ 彻底消灭 ResizeBuffers bug 类
use windows::core::*;
use windows::Win32::Foundation::{HWND, HMODULE};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory2, IDXGIDevice, IDXGISwapChain1, IDXGISurface};

use crate::state::{WIN_H, WIN_W};

pub(crate) struct Gpu {
    pub dc: ID2D1DeviceContext,
    pub rt: ID2D1RenderTarget,
    pub sc: IDXGISwapChain1,
    // 保活字段(RAII:结构体存活 = 资源存活;不 pub 防误触)
    _target: ID2D1Bitmap1,
    _d3d: ID3D11Device,
    _imctx: ID3D11DeviceContext,
    _dxgi: IDXGIDevice,
    _factory: IDXGIFactory2,
    _d2d_device: ID2D1Device,
    _comp: IDCompositionDevice,
    _comp_target: IDCompositionTarget,
    _visual: IDCompositionVisual,
}

pub(crate) unsafe fn init(hwnd: HWND) -> Result<Gpu> {
    let mut d3d = None;
    let mut imctx = None;
    D3D11CreateDevice(None, D3D_DRIVER_TYPE_HARDWARE, HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, None, D3D11_SDK_VERSION,
        Some(&mut d3d), None, Some(&mut imctx))?;
    let d3d = d3d.unwrap();
    let imctx = imctx.unwrap();
    let dxgi: IDXGIDevice = d3d.cast()?;
    let factory: IDXGIFactory2 = CreateDXGIFactory1::<IDXGIFactory1>()?.cast()?;
    let d2d_factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
    let d2d_device = d2d_factory.CreateDevice(&dxgi)?;
    let dc = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

    let sc = factory.CreateSwapChainForComposition(&d3d, &DXGI_SWAP_CHAIN_DESC1 {
        Width: WIN_W as u32, Height: WIN_H as u32,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM, Stereo: false.into(),
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT, BufferCount: 2,
        Scaling: DXGI_SCALING_STRETCH, SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED, Flags: 0,
    }, None)?;

    let buffer: IDXGISurface = sc.GetBuffer(0)?;
    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
        dpiX: 96.0, dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        colorContext: std::mem::ManuallyDrop::new(None),
    };
    let target = dc.CreateBitmapFromDxgiSurface(&buffer, Some(std::ptr::from_ref(&props)))?;
    dc.SetTarget(&target.cast::<ID2D1Image>()?);

    let comp: IDCompositionDevice = DCompositionCreateDevice(&dxgi)?;
    let comp_target = comp.CreateTargetForHwnd(hwnd, true)?;
    let visual = comp.CreateVisual()?;
    visual.SetContent(&sc)?;
    comp_target.SetRoot(&visual)?;
    comp.Commit()?;

    let rt: ID2D1RenderTarget = dc.cast()?;
    Ok(Gpu { dc, rt, sc, _target: target, _d3d: d3d, _imctx: imctx, _dxgi: dxgi,
        _factory: factory, _d2d_device: d2d_device, _comp: comp,
        _comp_target: comp_target, _visual: visual })
}
