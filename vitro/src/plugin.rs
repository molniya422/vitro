// ── 插件系统(boa_engine,QuickJS 精神:记事本可编辑)──
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use super::*;  // 引用 main.rs 的 app()/App/DockItem 等
struct PluginHost {
    plugins: Vec<PluginInstance>,
}

struct PluginInstance {
    name: String,
    bitmap_idx: usize,   // dock 里的索引
    timer_ms: u32,
}

static mut PLUGIN_HOST: Option<PluginHost> = None;

// 加载插件代码,返回 docklet 位图索引
pub(crate) unsafe fn load_plugin(name: &str, code: &str) -> std::result::Result<String, String> {
    use boa_engine::{Context, Source, JsValue, JsResult, NativeFunction};
    use boa_engine::object::builtins::{JsFunction, JsArrayBuffer};

    // 注意:APP 尚未初始化(插件加载发生在主结构体赋值前),这里不得触碰 app()
    let mut ctx = Context::default();

    // 注册 api 对象:set(html), every(ms)
    // 注意:原型阶段 api.set 只做「返回要显示的文本」——每 tick 我们调 getDisplay()
    // 正式版用通道把 set() 直接投到 UI 线程
    let display_val: JsValue = JsValue::from("");
    ctx.register_global_property("api_set_result", display_val, boa_engine::property::Attribute::all()).ok();
    // api 对象:注册全局 fn 再 eval 赋值(boa 0.17 无点号注册)
    ctx.register_global_callable("api_set", 1, NativeFunction::from_fn_ptr(api_set)).ok();
    ctx.register_global_callable("api_every", 2, NativeFunction::from_fn_ptr(api_every)).ok();
    ctx.register_global_callable("api_log", 1, NativeFunction::from_fn_ptr(api_log)).ok();
    let _ = ctx.eval(Source::from_bytes(
        b"var api = { set: api_set, every: api_every, log: api_log };"));

    // 执行插件代码(顶层代码可以立即调 api.set / api.every)
    let src = Source::from_bytes(code.as_bytes());
    ctx.eval(src).map_err(|e| format!("JS error: {}", e))?;

    Ok(get_plugin_text())
}

static mut PLUGIN_TEXT: Option<String> = None;

unsafe fn plugin_host() -> &'static mut PluginHost {
    if PLUGIN_HOST.is_none() { PLUGIN_HOST = Some(PluginHost { plugins: Vec::new() }); }
    PLUGIN_HOST.as_mut().unwrap()
}

fn get_plugin_text() -> String {
    unsafe { PLUGIN_TEXT.clone().unwrap_or_default() }
}

// ── JS 绑定:api.set ──
fn api_set(
    _this: &boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    _ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    if let Some(v) = args.first() {
        let s = v.to_string(_ctx)?.to_std_string_escaped();
        unsafe { PLUGIN_TEXT = Some(s); }
    }
    Ok(boa_engine::JsValue::undefined())
}

fn api_every(
    _this: &boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    _ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    // 原型:记录间隔(正式版:SetTimer 挂到宿主,周期执行插件回调)
    Ok(boa_engine::JsValue::undefined())
}

fn api_log(
    _this: &boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    _ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    if let Some(v) = args.first() {
        let s = v.to_string(_ctx)?.to_std_string_escaped();
        println!("[plugin] {}", s);
    }
    Ok(boa_engine::JsValue::undefined())
}

// 插件文本格式(懒创建,因为 DWrite 已在启动后 drop)
static mut PLUGIN_FMT: Option<IDWriteTextFormat> = None;
pub(crate) unsafe fn plugin_fmt() -> &'static IDWriteTextFormat {
    if PLUGIN_FMT.is_none() {
        let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
        let fmt = dw.CreateTextFormat(
            w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL, 18.0, w!("zh-CN"),
        ).unwrap();
        let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
        let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
        PLUGIN_FMT = Some(fmt);
    }
    PLUGIN_FMT.as_ref().unwrap()
}

// 徽章小字号(插件磁贴底部药丸用)
static mut PLUGIN_FMT_SM: Option<IDWriteTextFormat> = None;
pub(crate) unsafe fn plugin_fmt_sm() -> &'static IDWriteTextFormat {
    if PLUGIN_FMT_SM.is_none() {
        let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
        let fmt = dw.CreateTextFormat(
            w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL, 11.5, w!("zh-CN"),
        ).unwrap();
        let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
        let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
        PLUGIN_FMT_SM = Some(fmt);
    }
    PLUGIN_FMT_SM.as_ref().unwrap()
}
