# crates/lsp 待办任务

## Debug Feature（未完成）

### 1. 日志输出
- [ ] stderr 日志模块：服务器运行日志（启动信息、错误记录、消息收发摘要）
- [ ] `window/logMessage` 通知：服务器 → 客户端推送日志消息（type: 1=Error ~ 4=Log）
- [ ] `window/showMessage` 通知：服务器 → 客户端弹窗提示

### 2. LSP Trace 追踪
- [ ] `$/setTrace` 通知处理：客户端设置 trace 级别（off / messages / verbose）
- [ ] `$/logTrace` 通知发送：服务器按 trace 级别输出追踪日志
- [ ] 消息追踪：verbose 级别下记录每帧 Request / Response / Notification 摘要（method、id、耗时）

### 3. 遥测与诊断
- [ ] `telemetry/event` 通知发送：遥测事件上报
- [ ] 挂起请求诊断：出站请求超时检测与告警日志

### 4. 补全功能调试
- [ ] completion resolve 增强：completionItem/resolve 返回真实 detail / documentation
- [ ] `$/progress` 实际发送：补全等长耗时操作的进度通知（当前仅注册 no-op 处理）

## 其他待办
- [ ] 文档同步：Incremental 的 UTF-16 偏移换算（当前简化为字符偏移）
- [ ] `workspace/didChangeWatchedFiles` 注册后的实际文件监听（当前仅动态注册演示）
