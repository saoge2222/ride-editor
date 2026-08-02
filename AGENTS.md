<!-- 项目AGENTS.md文件
   - @author saoge2010@163.com
   - @date   2026-8-1 
   - @note   仅用于学习,非正式项目
   -->
# AGENTS.md

## 项目概览
- 项目名称:ride-editor文本编辑器(`https://github.com/saoge2222/ride-editor`)
- 项目语言:Rust(version 2024)
- 编译目标环境:WSL
- 项目内容:一个基于Vim键盘绑定的,使用slint作为框架的,功能完善强大的GUI文本编辑器

## 项目依赖及环境
- 编译器:cargo(`cargo 1.96.1`)
  - 调试构建:`cargo build`,输出文件目录:`/target/debug/editor-learn`
  - 发布构建:`cargo build --release`,输出文件目录`target/release/editor_learn`
  - 直接运行:`cargo run`
  - 代码检查:`cargo check`
- 依赖框架:`slint`
  - 安装方式:在Cargo.toml添加以下内容,并运行`cargo add slint`以安装相关依赖. 
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
- 项目代码入口:`/src`

## 代码规范
- 保证代码书写风格一致
- 不要添加任何`unsafe fn`等不安全函数
- 避免一切运行时潜在的未定义行为
- 涉及魔术值的代码严禁使用硬编码代替
- 变量,函数/方法,结构体,Traits等命名遵循Rust命名规则
  - 类型级使用驼峰命名法(UpperCamelCase),值级使用蛇形命名法(snake_case)
  - 更多内容参见:`https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md`

## 注释规范
- 在每一个函数,方法,结构体等代码模块加入模块级别注释,要求:
  - 使用markdown语法,使用`//!`注释语法
  - 模块内容包含:
    - 代码要素的简要功能职责描述
    - 代码要素的参数,返回结果(如果有),代码实现的行为等必要说明,不需要展开过多细节
    - 可能发生的错误/Panic场景(如果有)
    - 对于顶层API,提供使用示例
    - 使用中文,英文两套注释
- 对于单行注释,仅在较关键代码行添加:
  - 使用`//`注释语法
  - 简要解释代码内容即可
  - 使用英文注释

## 高风险操作注意
- 严禁大规模修改项目文件结构
- 不允许对以下文件进行修改:
  - `/target`目录下所有内容(`cargo`编译输出)
  - `/project_docs`目录下所有内容(开发备用文档)
  - `/.vscode`目录下所有内容(编辑器配置文件)
- 必须在每次修改之前展示修改改动diff情况(即标出每行的删除添加情况),并向用户确认
- 不允许作出提示命令之外的要求以实现所谓"优化"行为,若有较大必要性修改须向用户确认
- 涉及大批量文件修改,文件移除,项目构建相关配置修改,依赖安装/升级/切换等内容,请向用户确认
- 在每一次执行shell命令时请向用户确认

## Git提交要求
- 在每一次执行提交命令时请向用户确认
- 以下文件严禁提交
  - `/.vscode`下所有内容(编辑器配置文件)
  - `/project_docs`下所有文件(开发备用文档)
  - `/target`下所有文件(二进制文件输出)
  - `/src/temp`下所有文件(测试代码)
- 每一次代码提交应当在信息前添加以下tags:
  - `dev-backend`:后端文件源代码新内容
  - `dev-frontend`:前端文件源代码新内容开发
  - `fix`:bug修复
  - `buildcfg`:修改有关编译构建的代码,如修改`Cargo.toml`
  - `chores`:Git文件,Markdown文档修改
  - `agents`:AGENTS.md修改
- 分支管理要求:
  - 在`main`分支上提交正式版本发布代码
  - 在`dev`分支上提交开发代码
    - 当可以发行一个版本阶段之后即可将`dev`分支合并到`main`分支上,并添加版本tag
  - 对于Pull Request,则严禁作出任何行为
- 提交信息示例: 
```
dev-backend: add entry point func
dev-frontend: add windows of the editor
fix: fixed ui bugs
buildcfg: modified Cargo.toml
chores: modified README.md of the whole project
agents: modified AGENTS.md of the whole project
0.0.1: version release 
```