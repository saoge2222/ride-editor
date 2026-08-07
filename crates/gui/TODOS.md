# TODOS — 未实现项清单

本文件记录 `crates/gui`(ride-gui,Vulkan 渲染库)当前未实现的功能项。后续开发时读取本文件逐项实现。

## 1. 字体渲染模块

- **位置**: `crates/gui/src/render/`
- **现状**: 未生成;本阶段仅实现纯几何图形(矩形/线段/圆)
- **目标**: 渲染字体、字形图集、文本行布局
- **实现要点**:
  - 自写最小 TTF 解析器(`render_font.rs`),解析 `MapleMono-TTF` 静态字体(无外部字体库依赖)
  - 字形图集(`render_glyph.rs`):将解析出的字形栅格化到纹理图集,复用 `render_texture.rs` 的纹理基础设施
  - 文本行布局与字形 quad 绘制(`render_text.rs`),基于现有 `PushConstants` 的像素坐标变换
  - 字体文件位于 `crates/gui-workbench/fonts/` 与 `crates/gui/src/fonts/`(include_bytes! 相对路径嵌入)
  - 远期:变体字体 `SourceHanSans-VF.ttf.ttc`(fvar/gvar 表)解析

## 2. 编辑器区域连字支持

- **位置**: `crates/gui-workbench`(编辑器前端)
- **现状**: 未实现
- **实现要点**:
  - 依据 `crates/gui-workbench/fonts/MapleMono-TTF/config.json` 做连字(shaping)处理
  - 字形组合映射与绘制偏移修正

## 3. 编辑器区域中英文等宽混排支持

- **位置**: `crates/gui-workbench`(编辑器前端)
- **现状**: 未实现
- **实现要点**:
  - 中文字符宽度按 2 个半角单位对齐的网格布局
  - 光标/选区坐标换算需兼容双宽字符

## 4. 组件/布局/事件系统完整实现

- **位置**: `crates/gui/src/component/`
- **现状**: 骨架(类型与接口已定义,`Container` 仅简单 arrange)
- **实现要点**:
  - 完整布局引擎:`component_layout.rs` 中 `Axis`/`Alignment`/`Layout` 的 Box/Flex 布局算法
  - 组件树遍历与命中测试(`Rect::contains`)
  - 事件分发:将 winit 事件(见 `component_event.rs`)转换为 `ComponentEvent` 并分发到组件树
  - 样式系统完善(`component_style.rs`)

## 5. 内置组件库实现

- **位置**: `crates/gui/src/widgets/`
- **现状**: 仅定义(Button/Window/FileTree/List/EditorBuffer/TextInput 结构体)
- **实现要点**: 基于 `component` 层渲染与事件接口,实现各组件实际绘制与交互

## 6. 后端异步通信接入

- **位置**: `crates/gui/src/backend/`
- **现状**: 仅类型定义(LSP/语法高亮/文件管理/Git 事件枚举)
- **实现要点**:
  - `backend_worker.rs` 后台线程 + 事件桥接,将事件投递到 UI 线程渲染
  - 文件管理:对接 `crates/ride-editor/filemng.rs` 的 ride-fm IPC(JSON 行协议)
  - LSP/语法树解析高亮服务的通道接入

## 7. 剪贴板共享完整集成

- **位置**: `crates/gui/src/vulkano_base/vulkano_base_clipboard.rs`
- **现状**: 骨架(接口存在,get_text 返回空字符串,set_text 空操作)
- **原因**: vulkano 0.34 必须搭配 winit 0.28,而 winit 0.28 无剪贴板 API
- **实现要点**: 接入平台剪贴板(如 x11/wl-clipboard),并接入 winit 0.29 的 `clipboard_text`/`set_clipboard_text`(需评估与 vulkano 0.34 的 raw-window-handle 0.5 兼容性)

## 8. ride-editor 接入 Vulkan GUI

- **位置**: `crates/ride-editor/main.rs`
- **现状**: 占位程序(仅启动 ride-fm 并打印当前目录)
- **实现要点**:
  - 将 `ride_gui` 作为依赖加入根 `Cargo.toml`
  - 用 `RenderLoop` + `RenderPipelineContext` 搭建编辑器主界面
  - 接入 `filemng`/`keyboard_monitor` 逻辑与 Vulkan 渲染

## 9. 恢复 filemng.rs 被注释 API

- **位置**: `crates/ride-editor/filemng.rs`
- **现状**: `FileEntry`/`OpenFile` 结构体与 10 个方法被 `/* */` 注释(占位 main 触发 dead-code 警告所致)
- **实现要点**: 在接入 Vulkan GUI(第 8 项)恢复这些 API 时取消注释并恢复 `apply_state` 的全量状态同步
