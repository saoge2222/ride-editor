# TODOS

## 未解决 Bug

### B1. 光标不渲染（最高优先级）
- **现象**：textbox 聚焦后（键盘输入正常、on_key_down 事件生效），光标（Line/Block）完全不渲染；截图像素级扫描 0 像素 60A5FA（caret_color），文本渲染正常。
- **已验证事实**：
  - 键盘输入链路正常：点击 → `on_mouse_down` → `window.focus(&focus_handle)` → handle 入焦点栈 → `on_key_down` 分发 → `handle_key` 执行 → text_changed 事件触发。
  - 但 `textbox.focused` 恒为 false（临时 println 调试输出：`TBOX focus=false key=a`）。
  - `focused` 只由 `window.on_focus_in(&handle, ...)`（textbox.rs:532）设置，该回调从未触发。
- **根因分析**：gpui 的 `on_focus_in(handle)` 监听的是「窗口获得焦点 + handle 在焦点栈」的事件。测试序列（XSetInputFocus 先行、点击后置）导致窗口焦点事件先于 handle 入栈发生 → 回调不触发。**需确认真实用户操作顺序（先点窗口后键入）下是否也会触发失败**——若 gpui 的 focus 事件只在窗口 FocusIn/FocusOut 时 dispatch，则「窗口已 active 后再 window.focus(handle)」的场景 focused 永远不更新，属于组件逻辑缺陷。
- **修复方向**：
  1. 在 `on_mouse_down` 中直接置 `self.focused = true`（点击即聚焦，不依赖窗口级 FocusIn 事件），并补充 `focus_out` 逻辑；
  2. 或订阅 gpui 的 `dispatch_focus`（Focusable 的 focus_handle 回调）替代/补充 on_focus_in；
  3. 修复后需在 X11 下重新验证光标渲染 + 闪烁。
- **影响范围**：textbox.rs 光标渲染条件（L706、L773）；editor.rs 的 render_caret 同样依赖 focused（阶段 5 同源问题，需一并验证）。

### B2. 多行文本框行高/光标纵向定位疑点
- **现象**：notes-box（line_count=3）内 9 行文本滚动后可见行间距在截图中约 24-30px，与 `LINE_HEIGHT_MULTIPLIER`(1.5) × text_size(14) = 21px 不符；光标 `top = line_ix × caret_height` 的纵向位置需复核。
- **状态**：未确认是真 bug 还是截图分析误差（光标未渲染导致无法直接测光标位置）。待 B1 修复后复测。

## 临时修改需恢复（验证完成后）

### T1. textbox.rs 调试残留
- `handle_key` 开头临时 `println!("TBOX focus={} ...")`（调试输出，需删除）。
- 光标渲染处 `with_blink_animation` 被临时替换为直接 `child(rendered_caret)`（B1 排查用，需恢复闪烁动画）。

### T2. textbox_demo.rs 初始文本
- `set_text("first line\nsecond line")` 被临时改为 9 行文本（验证垂直滚动用，需还原）。

## 未完成验证（Task #29 阻塞项）

### V1. textbox_demo 运行时验证
- 多行块状光标（半透明覆盖、字符可读）+ 闪烁；
- 多行连续输入超视口后光标始终可见（垂直滚动跟随）；
- 单行 title 框横向滚动不受影响。
- **阻塞于 B1**。

### V2. editor_demo 运行时验证（阶段 5，从未验证）
- NORMAL 模式块状光标覆盖文本、字符可读；
- `i` 切换 INSERT 线形光标渲染于块状光标左缘；
- h/j/k/l 移动 + 拖尾 trail + EaseOut 动画；`MovePreset::Teleport` 直跳；
- Blink/Fade 闪烁预设（CaretAnimationConfig 绑定）；
- `v` 进入 VISUAL 模式（状态栏显示）、`escape` 返回。
- 同源问题：editor 的 focused/caret 渲染依赖 on_focus_in，**疑似同样不触发**（B1 修复后验证）。

### V3. pane_demo 运行时验证（阶段 6，从未验证）
- 2×2 网格排布、Center 对齐；
- 行列/offset 定位、越界项不渲染；
- 点击 "Hide Title" 切换标题 textbox 显隐；
- 外部 "Hide/Show Pane" 按钮 set_visible/is_visible；
- on_item_click / on_item_hover 事件回调。

## 环境问题（非代码 bug，需记录）

### E1. Wayland 后端在 WSLg 上不可用
- gpui 0.2.2 Wayland 客户端在 WSLg 上 panic：`wayland/client.rs:151:51 UnsupportedVersion`。
- **规避**：`env -u WAYLAND_DISPLAY cargo run --example <demo>` 强制 X11 后端（DISPLAY=:0 可用）。

### E2. X11 焦点/窗口验证流程
- Weston（Xwayland）控制 X 焦点；点击合成事件（XTEST）在无焦点时会被丢弃。
- **可用流程**：启动 → `xwininfo -root -tree` 取窗口 → `xsetfocus <wid>`（/tmp/xsetfocus）→ 点击目标区域 → 输入。
- 窗口位置每次启动漂移，须动态解析（xwininfo 行尾 `+X+Y` 为绝对位置）。
- 窗口随 demo 进程退出而销毁；timeout 需 ≥300s 并一次命令内完成全部验证。

### E3. 用户侧物理键盘干扰
- 用户通过 WSLg 在 Windows 侧对 demo 窗口打字会混入输入（日志中反复出现非脚本注入的字符流）。
- 验证时优先采用临时初始文本/一次性输入序列，避免依赖长时间键入。
