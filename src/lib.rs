use std::rc::Rc;
use std::cell::Cell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    // 创建画板
    let canvas = document
        .create_element("canvas")? //创建 canvas
        .dyn_into::<web_sys::HtmlCanvasElement>() // 转换为 CanvasElement
        .unwrap();

    // 获取画板根节点
    let drawer_element = document.query_selector("#drawer").unwrap();

    // 挂在 Canvas 到 DOM
    drawer_element.unwrap().append_child(&canvas)?;

    // 设置画板宽高，样式
    canvas.set_width(800);
    canvas.set_height(600);
    canvas.style().set_property("border", "1px solid blue")?;

    // 获取 Canvas Context
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    // 设置画笔属性
    context.set_stroke_style(&JsValue::from_str("pink"));
    context.set_line_width(2f64);

    let context = Rc::new(context);

    // 判断是否正在画
    let drawing = Rc::new(Cell::new(false));
    {
        // 按住鼠标逻辑
        //因为闭包中需要调用 context 和 drawing，，所以这里需要 Rc clone 一下
        let context = Rc::clone(&context);
        let drawing = Rc::clone(&drawing);
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            drawing.set(true);
            context.begin_path();
            context.move_to(event.offset_x() as f64, event.offset_y() as f64);
        }) as Box<dyn FnMut(_)>);
        // 监听 mousedown
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        // 鼠标移动逻辑
        let context = Rc::clone(&context);
        let drawing = Rc::clone(&drawing);
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if drawing.get() {
                // 画线
                context.line_to(event.offset_x() as f64, event.offset_y() as f64);
                context.stroke();
                context.begin_path();
                context.move_to(event.offset_x() as f64, event.offset_y() as f64)
            }
        }) as Box<dyn FnMut(_)>);
        // 监听 mousemove
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        // 放开鼠标逻辑
        let context = Rc::clone(&context);
        let drawing = Rc::clone(&drawing);
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            drawing.set(false);
            context.line_to(event.offset_x() as f64, event.offset_y() as f64);
            context.stroke();
        }) as Box<dyn FnMut(_)>);
        // 监听 mouseup
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    Ok(())
}
