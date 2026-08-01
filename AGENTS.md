<!-- 项目AGENTS.md文件
   - @author saoge2010@163.com
   - @date   2026-8-1 
   - @note   仅用于学习,非正式项目
   -->
# ride-editor项目AGENTS.md文件

## 项目概览
- 项目名称:ride-editor文本编辑器(`github.com/saoge2222/ride-editor`)
- 项目语言:Rust(version 2024)
- 编译目标环境:GNU/Linux, macOS, Windows等一般家用桌面环境
- 项目内容:一个基于Vim键盘绑定的,使用slint作为框架的,功能完善强大的GUI文本编辑器

## 项目依赖及环境
- 编译器:cargo(`cargo 1.96.1`)
 - 调试构建:`cargo build`,输出文件目录:`/target/debug/editor-learn`
 - 发布构建:`cargo build --release`,输出文件目录`target/release/editor_learn`
 - 直接运行:`cargo run`
 - 代码检查:`cargo check`
 - Lint检查:`cargo clippy`
 - 格式化:`cargo fmt`
- 依赖框架:
 1. `slint`
  - 安装方式:在Cargo.toml添加:
```toml
[package]
name = "editor_learn"
version = "0.1.0"
edition = "2024"

[dependencies]
slint = "1.17.1"
[build-dependencies]
slint-build = "1.17.1"
```
  随后运行:`cargo add slint`以安装相关依赖. 
 > [!CAUTION]
 > 当项目引入`.slint` UI定义文件时,需要:
 > 1. 在`Cargo.toml`中添加`[build-dependencies]`下的`slint-build`
 > 2. 创建`build.rs`调用`slint_build::compile()`编译`.slint`文件 

## 代码规范

## 注释规范
- 在每一个函数,方法,结构体等代码模块加入模块级别注释,要求:
 - 使用markdown语法,使用`//!`注释语法
 - 模块内容包含:
  - 代码要素的简要功能职责描述
  - 代码要素的参数,返回结果(如果有),代码实现的行为等必要说明,不需要展开过多语言细节
  - 可能发生的错误/Panic场景(如果有)
  - 对于顶层API,提供使用示例
  - 使用中文,英文两套注释
- 对于单行注释,仅在较关键代码行添加:
 - 使用`//`注释语法
 - 简要解释代码内容即可
 - 使用英文注释

## 文档修改要求
- 不允许对以下文件进行修改:
 - `/target`目录下所有内容(`cargo`编译输出)
 - `/project_docs`目录下所有内容(开发备用文档)
 - `/.vscode`目录下所有内容(编辑器配置文件)
- 必须在每次修改之前展示修改改动diff情况,标出每行的删除添加情况,并询问用户修改请求
- 不允许作出提示命令之外的要求以实现所谓"优化"行为,若有较大必要性修改须询问用户修改请求

## Git输出要求
- 每一次代码提交应当在信息前添加以下tags:
 - `backend`:修改后端文件源代码
 - `frontend`:修改前端文件源代码
 - `buildcfg`:修改有关编译构建的代码
 - `chores`:Git文件,Markdown文档修改
 - `agents`:AGENTS.md修改
- 分支管理要求:
 - 在`main`分支上提交正式版本发布代码,在`dev`分支合并后要求添加版本tag,但无需删除`dev`分支
 - 在`dev`分支上提交开发代码
 
