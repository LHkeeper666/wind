## 1. 修复 :%s 替换预览高亮

- [x] 1.1 在 PreviewEditor.svelte 的 sMatchField update 函数中，去掉共享 regex 对象的 `g` 标志
- [x] 1.2 非 global 分支改用 `line.text.match(regex)` 替代 `regex.exec(line.text)`
- [x] 1.3 global 分支在每行循环内新建 `new RegExp(pattern, 'gi')`，确保 lastIndex 不跨行污染
- [x] 1.4 验证：打开含多行 "md" 的文件，进入编辑模式，输入 `:%s/md/`，确认所有行均高亮

## 2. 修复 $ 等数字符号键的 vim key 映射

- [x] 2.1 在 PreviewEditor.svelte 的 `codeToVimKey()` 中，为 `Digit` 分支添加 shift 字符映射表
- [x] 2.2 在 FullscreenEditor.svelte 的 `codeToVimKey()` 中，做相同修改
- [x] 2.3 验证：在 normal 模式下按 `$` 确认光标跳到行尾，按 `%` 确认括号跳转

## 3. 修复 visual 模式 : 命令范围

- [x] 3.1 在 PreviewEditor.svelte 的 `handleOverlayKeydown()` 中，`:` 分支检测 `vimState.visualMode`，若为 true 则预填 `'<,'>`
- [x] 3.2 在 FullscreenEditor.svelte 的 `handleOverlayKeydown()` 中，做相同修改
- [x] 3.3 sMatchField 范围修复：`'<,'>` 使用选区行范围而非全文高亮

## 4. hlsearch 高亮样式修复

- [x] 4.1 .cm-searchMatch CSS 加 `!important` 防止被 oneDark 覆盖
- [ ] 4.2 验证：执行 `:%s/old/new/` 后 old 高亮为橙色(非蓝色)，`:noh` 清除高亮(经 Vim.handleEx fallback)

## 5. 最终验证

- [x] 5.1 运行 `npx svelte-check` 确认无 TypeScript 错误
- [x] 5.2 运行 `cargo check` 确认 Rust 端无回归
- [ ] 5.3 手动测试全部修复的验证场景
