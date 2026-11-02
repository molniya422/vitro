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
use std::cell::RefCell;
use std::rc::Rc;
// 加载插件代码,返回 docklet 位图索引
pub(crate) unsafe fn load_plugin(name: &str, code: &str) -> std::result::Result<String, String> {
    use boa_engine::{Context, Source, JsValue, NativeFunction};

    // 注意:APP 尚未初始化(插件加载发生在主结构体赋值前),这里不得触碰 app()
    let output = Rc::new(RefCell::new(String::new()));
    let mut ctx = Context::default();

    // 注册 api 对象:set(html), every(ms)
    // 注意:原型阶段 api.set 只做「返回要显示的文本」——每 tick 我们调 getDisplay()
    // 正式版用通道把 set() 直接投到 UI 线程
    let display_val: JsValue = JsValue::from("");
    ctx.register_global_property("api_set_result", display_val, boa_engine::property::Attribute::all()).ok();
    // api 对象:注册全局 fn 再 eval 赋值(boa 0.17 无点号注册)
    let set_output = Rc::clone(&output);
    let set = unsafe {
        NativeFunction::from_closure(move |_this, args, context| {
            if let Some(value) = args.first() {
                *set_output.borrow_mut() = value.to_string(context)?.to_std_string_escaped();
            }
            Ok(JsValue::undefined())
        })
    };
    let every = unsafe {
        NativeFunction::from_closure(|_this, _args, _context| Ok(JsValue::undefined()))
    };
    ctx.register_global_callable("api_set", 1, set).ok();
    ctx.register_global_callable("api_every", 2, every).ok();
    ctx.register_global_callable("api_log", 1, NativeFunction::from_fn_ptr(api_log)).ok();
    let _ = ctx.eval(Source::from_bytes(
        b"var api = { set: api_set, every: api_every, log: api_log };"));

    // 执行插件代码(顶层代码可以立即调 api.set / api.every)
    let src = Source::from_bytes(code.as_bytes());
    ctx.eval(src).map_err(|e| format!("JS error: {}", e))?;

    let text = output.borrow().clone();
    Ok(text)
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

pub(crate) struct TextFormats {
    pub(crate) plugin: IDWriteTextFormat,
    pub(crate) plugin_small: IDWriteTextFormat,
}

pub(crate) unsafe fn create_text_formats() -> Result<TextFormats> {
    let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
    let plugin = dw.CreateTextFormat(
        w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL, 18.0, w!("zh-CN"),
    )?;
    let _ = plugin.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    let _ = plugin.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

    let plugin_small = dw.CreateTextFormat(
        w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL, 11.5, w!("zh-CN"),
    )?;
    let _ = plugin_small.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
    let _ = plugin_small.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

    Ok(TextFormats { plugin, plugin_small })
}

#[cfg(test)]
mod tests {
    use super::load_plugin;

    #[test]
    fn plugin_display_text_is_scoped_to_each_load() {
        let first = unsafe { load_plugin("first", "api.set('one')") }.unwrap();
        let second = unsafe { load_plugin("second", "") }.unwrap();
        assert_eq!(first, "one");
        assert_eq!(second, "");
    }
}
