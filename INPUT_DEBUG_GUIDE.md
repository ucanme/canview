# 调试文本输入问题

## 测试步骤

1. 运行程序：
```bash
.\target\release\view.exe
```

2. 切换到 Library Management 视图

3. 点击 "Add Library" 按钮

4. 尝试输入字符（先测试英文 `test`，再测试中文 `测试`）

5. 观察控制台输出

## 需要查看的日志

### 英文输入时应该看到：
```
Global handler - Key: 't', focused: Some("new_library_input")
🔍 DEBUG: key='t', len=1, chars=['t']
   first_char=Some('t'), is_ascii=Some(true), is_control=Some(false)
   is_control_key=false, ime_composing=false
✓ Inserted 't' (len=1) at position 0, result: 't', cursor: 1
```

### 中文输入时应该看到：
**情况1：如果通过IME正常输入**
```
Global handler - Key: '测', focused: Some("new_library_input")
🔍 DEBUG: key='测', len=3, chars=['测']
   first_char=Some('测'), is_ascii=Some(false), is_control=Some(false)
   is_control_key=false, ime_composing=false
✓ Inserted '测' (len=3) at position 0, result: '测', cursor: 1
```

**情况2：如果正在IME组合中**
```
Global handler - Key: 'c', focused: Some("new_library_input")
→ IME composing, passing key: 'c'
```

## 关键调试信息

每次按键都会输出：
1. **key**: 按键的实际内容
2. **len**: 字符串的字节长度（中文字符通常是3字节）
3. **chars**: 包含的所有字符
4. **is_ascii**: 是否是ASCII字符
5. **is_control**: 是否是控制键
6. **is_control_key**: 验证逻辑是否认为是控制键
7. **ime_composing**: IME是否正在组合中

## 如果看不到这些日志

说明全局键盘处理器没有被触发。可能是：
- 焦点没有正确设置
- input_id 不匹配

## 请提供

1. 点击 "Add Library" 后的完整日志
2. 输入英文 `test` 时的完整日志
3. **输入中文 `测试` 时的完整日志**（最重要）
4. 是否看到光标？
5. 输入框是否有蓝色边框（表示聚焦）？
6. 中文输入法是否正常？在其他应用中能正常输入中文吗？
