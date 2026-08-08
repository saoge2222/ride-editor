# TODOS — 未实现项清单

本文件记录 `crates/gui`(ride-gui,Vulkan 渲染库)当前未实现的功能项。后续开发时读取本文件逐项实现。

## 1. 字体渲染模块 ✅ 已完成

- **位置**: `crates/gui/src/render/`
- **状态**: 已实现(2026-08)
  - `render_font.rs` — 最小 TTF 解析器(cmap/格式4、glyf 轮廓、hmtx、name 表);支持含每字形 bbox 前缀的字体(实测 MapleMono/JetBrainsMono 均适用),越界安全,复合字形返回空
  - `render_font_system.rs` — 系统字体发现(标准目录递归扫描 + name 表族名匹配),`RIDE_FONT_FAMILY` 环境变量指定,无默认族,回退内嵌
  - `render_glyph.rs` — 字形图集(ASCII 32..126 预置,扫描线填充 + 4x4 超采样),`GlyphAtlas`
  - `render_text.rs` — 文本布局 + 字形 quad,`TextRenderer`(纹理管线 + descriptor set)
  - `TexturedVertex` + 纹理采样着色器
- **待办**: 变体字体 `SourceHanSans-VF.ttc`(fvar/gvar)解析;非 ASCII 字符按需加入图集

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

## 4. 组件/布局/事件系统完整实现 ✅ 已完成

- **位置**: `crates/gui/src/component/`
- **状态**: 已实现(2026-08)
  - `component_layout.rs` — 完整 Flex 布局引擎(`Constraints`/`Size`/`Flex`,主轴排列 + 交叉轴对齐 + gap/padding),`Child` 持有组件
  - `component_definition.rs` / `component_container.rs` — `Component` trait(layout/arrange/draw/handle_event)、`Container` 递归布局 + 命中测试事件分发,`draw` 桥接 render `DrawList`
  - `component_event.rs` — `EventTranslator`(winit `WindowEvent` → `ComponentEvent`,含光标位置跟踪)
  - `component_style.rs` — 默认主题样式
- **待办**: 组件渲染与文本混排(在容器内绘制文本)、更细的命中/焦点管理

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
- **原因**: vulkano 0.35 搭配 winit 0.30,而 winit 0.30 无内置剪贴板 API(0.29 曾有 clipboard_text,0.30 已移除)
- **实现要点**: 接入平台剪贴板库(如 arboard)或平台 API(x11/wl-clipboard),需评估是否违反"仅 Vulkan/Vulkano 依赖"约束

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

## 10. 潜在 Bug:sRGB 色彩空间导致颜色偏亮

- **位置**: `crates/gui/src/vulkano_base/vulkano_base_surface.rs` 的 `pick_format`
- **现状**: `pick_format` 取 `formats.first()`,在 llvmpipe 上首个表面格式为 sRGB(如 `B8G8R8A8_SRGB`)。渲染到 sRGB 附件时 Vulkan 把片元着色器输出当**线性值**做 sRGB 编码 → 所有颜色变亮/发灰。
- **证据**(demo.png 像素级验证,linear→sRGB 换算与观测值逐一相等):
  - 窗背景 `[0.14,0.14,0.17]` → 显示为 `#696973`
  - 紫面板 `[0.55,0.35,0.75]` → `#C4A0E1`;蓝面板 → `#89B3E1`;绿面板 → `#95DABB`;绿圆 → `#95EDAA`
- **影响**: 内容/字形/布局均正确,仅颜色偏离设定值(非渲染失败)
- **候选修复**(二选一):
  - `pick_format` 优先选非 sRGB 格式(`B8G8R8A8_UNORM`),使颜色与设定一致
  - 或保留 sRGB 格式,将颜色值按线性空间提供(着色器侧调整)
