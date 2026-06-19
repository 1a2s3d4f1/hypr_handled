中文（简体）| [English](README.md)

## Hypr_handled

![hypr_icon](imgs/hypr_handled_path.svg)

注意：Hypr_handled 在 Hyprland 处于运行状态时可用。

Hypr_handled 应用程序包含以下功能。

功能：

* 终端响铃 -> 从混成器接收到特定事件后发出铃声
* 设置Xresources 字体dpi -> 更加当前显示器缩放比例自动计算并设置字体dpi
* 窗口移动 -> 如果窗口进入活动状态，将位于特定特殊工作区的窗口移动到当前工作区

你可以使用`hypr-minimize.sh -m`移动活动窗口从当前工作区到名为`minimized`的特殊工作区。

Hypr_handled 需要`xrdb`用于设置字体DPI和`gsettings`用于获取声音主题。

这个程序使用Rust编写，如果你想在自己的设备上构建，可以用下面的指令：

构建：

```
cargo build --release
```

### 调试

Hypr_handled使用`log4rs` crate作为日志后端，你可以在`/tmp/hypr_handled/logs/`找到日志文件。
