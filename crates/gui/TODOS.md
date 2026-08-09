# TODOS — 未实现项清单

本文件记录 `crates/gui`(ride-gui,Vulkan 渲染库)当前未实现的功能项。后续开发时读取本文件逐项实现。

## 1. 字体渲染模块 ✅ 已完成

- **位置**: `crates/gui/src/render/`
- **状态**: 已实现(2026-08)
  - `render_font.rs` — 最小 TTF 解析器(cmap/格式4+12、glyf 轮廓、hmtx、name、fvar 变体轴、maxp);支持含每字形 bbox 前缀的字体(实测 MapleMono/JetBrainsMono 均适用),越界安全;TTC 集合感知(from_bytes 自动检测 ttcf 并提取首面),`from_bytes_at` 按偏移解析
  - `render_font_system.rs` — 系统字体发现(标准目录递归扫描 + name 表族名匹配),`RIDE_FONT_FAMILY` 环境变量指定,无默认族,回退内嵌;`load_cjk_font()` — `RIDE_CJK_FONT` 环境变量 + CJK 文件名匹配扫描
  - `render_font_ttc.rs` — TTC 集合解析(`FontCollection`),提取各字体面,`into_first_face()`
  - `render_font_ligature.rs` — 静态码点级连字表(20 条 ASCII 运算符 → Unicode 替换,含 `->`→`→`, `!=`→`≠`, `<=`→`≤`, `===`→`≡` 等)
  - `render_font_gsub.rs` — GSUB 表解析框架(LookupType4 连字提取、LigatureTrie 前缀树匹配,预留标准 `liga` 字体扩展)
  - `render_shape_text.rs` — `TextShaper`(连字替换 + CJK 全角分类),`is_fullwidth()`/`char_width_kind()` 覆盖 15 个 Unicode 区间
  - `render_glyph.rs` — 动态字形图集(ASCII 32..126 预置,`ensure_glyph()` 按需插入任意码点,`commit_if_dirty()` Vulkan 纹理重上传),扫描线填充 + 4x4 超采样
  - `render_text.rs` — 文本布局 + 塑形集成 + 双字体(主字体/CJK 后备) + CJK 双倍步进(全角=2× 半角 cell_width)
  - `TexturedVertex` + 纹理采样着色器
- **待办**: 无

## 2. 编辑器区域连字支持 ✅ 已完成

- **位置**: `crates/gui/src/render/`(渲染层)/ `crates/gui/src/widgets/` + `examples/demo.rs`(编辑器端)
- **状态**: 已完成(2026-08)
  - ✅ **GSUB/calt 引擎**(`render_font_gsub.rs`):Coverage(fmt1/2)、LookupType1 Single / 3 Alternate / 4 Ligature / 6 Chain Context(fmt1/fmt3 + 嵌套 sublookup);`apply_gsub` 按 ccmp→calt→liga→rlig 应用;全程越界安全(不 panic)
  - ✅ **TextShaper 集成**(`render_shape_text.rs`):码点→glyph_id→GSUB 字形序列替换;静态连字表 + GSUB 双机制;无 GSUB 字体回退静态表
  - ✅ **图集按 glyph_id 索引**(`render_glyph.rs`):`(font_index, glyph_id)` 键,支持连字替换字形与双字体
  - ✅ **编辑器实时输入**(`widgets_editor.rs` + `render_editor.rs` + `RenderLoop` 事件回调):键入/退格/回车/方向键实时编辑,连字每帧重新塑形
- **说明**:MapleMono 的 `->`/`!=` 等经典运算符连字不在默认 calt(在样式集 ss01-ss11),由静态连字表提供;GSUB 引擎对字体真实的 calt/liga 规则生效

## 3. 编辑器区域中英文等宽混排支持 ✅ 已完成

- **位置**: `crates/gui/src/render/`(渲染层)/ `crates/gui/src/widgets/` + `examples/demo.rs`(编辑器端)
- **状态**: 已完成(2026-08)
  - ✅ `is_fullwidth()` / `char_width_kind()`(15 个 Unicode 区间),全角=2× 半角 cell_width
  - ✅ `TextShaper::column_to_x()` / `x_to_column()`(半角列 ↔ 像素,全角按 2 列)
  - ✅ `EditorBuffer::caret_x()` / `column_at_x()`(光标/选区坐标换算)
  - ✅ `EditorView`(行号、CJK 双宽文本、按网格定位的光标)
  - ✅ `TextRenderer` 双字体(主/CJK 后备)+ 网格对齐 advance

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

## 11. 编译警告清理:GSUB 框架死代码(12 个警告) ✅ 已完成

- **状态**: 已完成(2026-08)——GSUB 引擎已真正接入塑形管线,全部 12 个死代码警告消除
  - ✅ `render_font_gsub.rs` 的 `parse_gsub`/`parse_lookup_list`/`parse_feature_list` 与常量由 `TextShaper` 调用
  - ✅ `render_font.rs` 的 `gsub` 字段 / `gsub_table()` / `raw_data()` / `has_gsub()` 由引擎使用
  - ✅ `cargo check --all-targets` 恢复**零警告**
- **遗留说明**:LigatureTrie / LigatureEntry / build_ligature_trie 保留(测试覆盖),作为额外连字数据结构;如需精简可后续移除

## 12. 潜在 Bug:某些字形轮廓坐标异常巨大导致栅格化死循环

- **位置**: `crates/gui/src/render/render_glyph.rs`(栅格化)与 `render_font.rs`(glyf 解析)
- **现象**:运行 demo 时窗口不弹出(首帧永不完成)。定位:主线程卡在 `point_inside`(栅格化某字形),坐标高达 x=-640/y=655。
- **根因**:**MapleMono 的 U+2192(箭头 →,glyph 8594)等字形轮廓解析出 ±656px 的巨大坐标**(bbox 1312×1312),使 `rasterize_outline` 的逐像素 × 16 超采样循环量级爆炸(可能数亿次点测试)→ 近乎死循环。
- **已加保护**:`render_glyph.rs` 增加 `MAX_GLYPH_SIZE = 256` bbox 上限,超限跳过该字形 → 首帧完成,窗口正常。**副作用**:`->`→`→`、`!=`→`≠` 等连字字符字形被跳过不显示。
- **待排查**:为何 glyph 8594(U+2192)在 `glyph_outline` 中产生 ±656px 坐标(怀疑与每字形 8 字节 bbox 前缀解析或复合字形有关);`glyph_outline_with_shift` 的 shift-8 路径返回了"垃圾但非 None"的结果,未触发 shift-0 回退。修复方向:
  - 解析出超界轮廓时判定解析失败,回退标准布局(shift-0)
  - 或对轮廓坐标合理性做校验(超出 units_per_em×k 视为异常)
- **验证**:修复后 `->`/`!=` 连字字形应能正常渲染(当前被跳过)
