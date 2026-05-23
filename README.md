# waybar-niri-workspaces

一个用于 [Waybar](https://github.com/Alexays/Waybar) 的 CFFI 模块，在 Waybar 中显示 [Niri](https://github.com/YaLTeR/niri) 窗口管理器的工作区和窗口列表。
基于 [niri_window_buttons](https://github.com/adelmonte/niri_window_buttons)修改，在此表示感谢！

![demo](image.png)

## 功能

- **工作区展示**：显示所有工作区，每个工作区以标签形式呈现
- **窗口列表**：在工作区内显示当前打开的窗口，附带应用图标
- **点击交互**：
  - 点击工作区切换至该工作区
  - 点击窗口按钮聚焦对应窗口
- **实时同步**：通过 Niri IPC 实时监听窗口和工作区变化
- **高度可配置**：支持丰富的自定义配置选项

## 依赖

- [Rust](https://www.rust-lang.org/) >= 1.85 (用于编译)
- [Waybar](https://github.com/Alexays/Waybar) (支持 CFFI 模块)
- [Niri](https://github.com/YaLTeR/niri) 窗口管理器

## 构建

```bash
cargo build --release
```

编译完成后，动态库位于 `target/release/libwaybar_niri_workspaces.so`。

## 安装

将编译好的 `.so` 文件复制到你希望的位置，例如：

```bash
mkdir -p ~/.config/waybar/modules
cp target/release/libwaybar_niri_workspaces.so ~/.config/waybar/modules/
```

## Waybar 配置

在 Waybar 配置文件（通常是 `~/.config/waybar/config.jsonc`）中添加模块：

```jsonc
{
  "modules-center": ["cffi/niri_workspaces"],

  "cffi/niri_workspaces": {
    "module_path": "/home/用户名/.config/waybar/modules/libwaybar_niri_workspaces.so",
    "only_current_workspace": false, // 是否只显示当前工作区
    "icon_size": 20,                 // 窗口图标大小（像素）
    "ignore_rules": [                // 忽略特定窗口（可选）
      { "app_id": "firefox" }
    ]
  }
}
```

### 配置选项

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `module_path` | string | 必填 | `.so` 文件的绝对路径 |
| `only_current_workspace` | boolean | `false` | 是否仅显示当前工作区 |
| `icon_size` | integer | `24` | 应用图标大小（像素） |
| `ignore_rules` | array | `[]` | 忽略特定窗口的规则列表 |

#### `ignore_rules` 规则字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `app_id` | string | 匹配窗口的 app_id |
| `title` | string | 精确匹配窗口标题 |
| `title_contains` | string | 窗口标题包含指定字符串 |
| `title_regex` | string | 窗口标题匹配正则表达式 |
| `workspace` | integer | 匹配指定工作区 ID |

## 样式配置

在 Waybar 的 `style.css` 中添加以下样式来自定义外观：

```css
/************************************************************/
/* cffi/niri_workspaces 模块样式 */

/* 主容器 */
.niri-workspaces {
}

/* 按钮基础样式 */
.niri-workspaces button {
    background-color: transparent;
    border: none;
    margin: 0;
    padding: 2px;
    transition: background-color 200ms ease;
}

/* 按钮悬停 */
.niri-workspaces button:hover {
    background-color: rgba(255, 255, 255, 0.35);
}

/* 当前聚焦的窗口按钮（代码中使用的是 .focused 不是 .active） */
.niri-workspaces button.focused {
    background-color: rgba(13, 13, 13, 0.5);
}

/* workspace 容器 */
.niri-workspaces .workspace {
    background-color: rgba(191, 110, 23, 0.7);
    padding: 0;
    margin: 0;
}

/* 当前活跃的 workspace */
.niri-workspaces .workspace.active {
    background-color: rgba(128, 160, 150, 0.95);
}

/* workspace 序号标签 */
.niri-workspaces .workspace-label {
    color: rgba(253, 255, 254, 1);
    font-weight: bold;
    font-size: 13px;
    margin-right: 4px;
}
```

### CSS 类名说明

| 类名 | 作用目标 | 说明 |
|------|----------|------|
| `.niri-workspaces` | 主容器 | 模块根元素 |
| `.workspace` | 工作区容器 | 每个工作区的外部盒子 |
| `.workspace.active` | 活跃工作区 | 当前可见的工作区 |
| `.workspace-label` | 工作区标签 | 显示工作区序号的文本 |
| `button` | 窗口按钮 | 每个打开窗口对应的按钮 |
| `button.focused` | 聚焦窗口按钮 | 当前输入焦点的窗口 |
| `button:hover` | 悬停状态 | 鼠标悬停在按钮上时 |

## 技术说明

- 本模块使用 Rust 编写，通过 `waybar-cffi` 与 Waybar 交互
- 使用 GTK3 渲染 UI
- 通过 `niri-ipc` 与 Niri 通信，监听窗口事件
- 应用图标通过系统图标主题解析（支持 `IconTheme` 回退机制）

## 许可证

GPL-3.0 license 
