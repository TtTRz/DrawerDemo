# 使用 Rust + WebAssembly 实现画板

## 环境
* Rust: 1.52.0-nightly
* Node: 15.5.1

*上述环境的搭建过程就不再赘述了，还没有的童鞋可以百度一下*

## 开始
### 工程配置部分
#### 1. 初始化工程并用 `IDE` 打开

```shell
cargo new drawer --lib
cd drawer
npm init -y
code . 
```

#### 2. 配置 Cargo.toml
这里主要是安装所需要的 `Rust` 包

#### 3. 配置 npm package
使用 `webpack` 集成 `wasm-pack-plugin`，省去手动编译的繁琐步骤

#### 4. 配置 webpack
在根目录下创建 `webpack.config.js`
**配置完之后在命令行输入 `npm i` 安装依赖**
#### 5. 配置网页入口文件
在根目录下创建 `index.html` 和 `index.js`

### Rust 部分
#### 1. 清除干净 `src/lib.rs` 中的内容

#### 2. 导入所需要的库

```Rust
use std::rc::Rc;
use std::cell::Cell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
```
#### 3. 编写入口函数
```Rust
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
  // Main Logic
  Ok(())
}
```
到这里可以在命令行运行 `npm run dev` 看能不能跑起来。

如果报错的话，检查下前面的配置有没有遗漏的地方。

如果没有红色的报错提示，可以继续往下。

```#[wasm_bindgen(start)]``` 这个方法会在实例化 `WebAssembly 模块` 的时候调用，在这里我们希望浏览器加载完就开始运行我们的代码。

#### 4. 创建 Canvas 画板
获取 `Document` 对象。

```Rust
// 获取 Document 对象
let document = web_sys::window().unwrap().document().unwrap();
```

创建 `Canvas` 元素并挂载到指定的元素节点上
```Rust
// 创建画板
let canvas = document
    .create_element("canvas")? //创建 canvas
    .dyn_into::<web_sys::HtmlCanvasElement>() // 转换为 CanvasElement
    .unwrap();

// 获取自定义画板元素节点
let drawer_element = document.query_selector("#drawer").unwrap();

// 挂在 Canvas 到元素节点
drawer_element.unwrap().append_child(&canvas)?;
```

设置 **Canvas Context** 相关的属性

```Rust
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

let context = Rc::new(context); // 需要重复使用，所以 Rc 智能指针包一下
```

#### 5. 画笔逻辑

**创建完画板后，我们先来思考下整个画图的逻辑**
* 当我们点住鼠标的时候，开始记录我们在画板上的哪个点

* 当我们点住鼠标并且移动鼠标的时候，开始跟随我们移动的轨迹绘制

* 当我们的放开鼠标的时候，绘制结束

**可以看到，整个画图逻辑主要就分为这三部分逻辑**

那接下来就是分别来实现这三部分逻辑了。

#### 6. 按下画笔

在写落下画笔的逻辑之前，还需要一个变量来记录画笔当前的状态，即他是否正在按着鼠标。

```Rust
// 判断是否正在画
let drawing = Rc::new(Cell::new(false));
```

监听浏览器的 `onmosuedown` 事件，当鼠标按下的时候即按下画笔的时候，将画画状态设置为 `true`，并记录当前的点。

**注意一下代码段要用括号包起来，创建一个新的作用域**
```Rust
{   // <------- 这里的括号是必须的
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
}   // <------- 这里的括号是必须的
```

#### 7. 移动画笔

监听浏览器的 `onmosuemove` 事件，当鼠标出现移动的时候，首先判断是否正为画画状态，是的话就开始绘制路径。

```Rust
{   // <------- 这里的括号是必须的
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
}   // <------- 这里的括号是必须的
```

#### 8. 放开画笔

监听浏览器的 `onmouseup` 事件，当不再按住鼠标的时候，停止绘制逻辑。

```Rust
{   // <------- 这里的括号是必须的
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
}   // <------- 这里的括号是必须的
```

#### 9. 看看效果

到了这里基本就实现完了，可以去浏览器看看效果

在已经在命令行 `npm run dev` 的并且没有报错的情况下，打开浏览器并输入 `localhost:8080` 即可看到效果。

可以用鼠标在这个画板随意画画～～

### 总结 

得益于 `WebAssembly` 的特性，这个画板会比使用 `JavaScript` 编写的画板性能要好。

所以针对某些特定的场景和需求，可以考虑将其进行封装，直接在 `Web` 中进行调用。

### 参考资料

web-sys 文档: https://docs.rs/web-sys/0.3.47/web_sys/struct.CanvasRenderingContext2d.html

create-dev-s-offline-page-with-rust-and-webassembly(Sendil Kumar N):https://dev.to/sendilkumarn/create-dev-s-offline-page-with-rust-and-webassembly-21gn
