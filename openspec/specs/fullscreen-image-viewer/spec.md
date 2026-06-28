### Requirement: E key opens fullscreen image viewer for image files
当用户在目录面板或预览面板按下 `E` 键且当前选中文件为图片时，系统 SHALL 打开全屏图片查看器。当文件为非图片时，系统 SHALL 打开文本全屏编辑器（保持现有行为）。

#### Scenario: E key on image file from directory panel
- **WHEN** 用户在目录面板选中一个 .jpg 文件并按下 `E`
- **THEN** 系统打开全屏图片查看器，显示该图片的原图

#### Scenario: E key on text file from directory panel
- **WHEN** 用户在目录面板选中一个 .txt 文件并按下 `E`
- **THEN** 系统打开全屏文本编辑器（现有行为不变）

#### Scenario: E key on image file from preview panel
- **WHEN** 用户在预览面板查看一个 .png 文件并按下 `E`
- **THEN** 系统打开全屏图片查看器

### Requirement: Fullscreen viewer displays original image fit to screen
全屏查看器 SHALL 以黑色背景全屏显示，默认将图片缩放至适应屏幕大小（fit-to-screen），图片完整可见且不变形。

#### Scenario: Landscape image on landscape screen
- **WHEN** 用户打开一张 3840×2160 的图片，屏幕分辨率为 1920×1080
- **THEN** 图片以 0.5x 比例显示，完整适应屏幕

#### Scenario: Portrait image on landscape screen
- **WHEN** 用户打开一张 1000×3000 的图片
- **THEN** 图片以高度为基准缩放，完整适应屏幕

### Requirement: j/k keys navigate between images in directory
用户 SHALL 能够使用 `j`（下一张）和 `k`（上一张）在当前目录的图片文件列表中循环切换。非图片文件 SHALL 被跳过。

#### Scenario: Navigate to next image
- **WHEN** 用户在全屏查看器中按下 `j`
- **THEN** 显示切换到目录中下一张图片（循环到第一张）

#### Scenario: Navigate to previous image
- **WHEN** 用户在全屏查看器中按下 `k`
- **THEN** 显示切换到目录中上一张图片（循环到最后一张）

#### Scenario: Only image files in navigation list
- **WHEN** 目录中有 photo.jpg、readme.txt、icon.png
- **THEN** `j/k` 只在 photo.jpg 和 icon.png 之间切换，跳过 readme.txt

### Requirement: h/l keys zoom out and zoom in
用户 SHALL 能够使用 `h` 缩小图片、`l` 放大图片。缩放以图片中心为锚点。最小缩放比例为 0.1，无最大限制。

#### Scenario: Zoom in
- **WHEN** 用户按下 `l`
- **THEN** 图片放大 0.25 倍（scale += 0.25）

#### Scenario: Zoom out
- **WHEN** 用户按下 `h`
- **THEN** 图片缩小 0.25 倍（scale -= 0.25）

#### Scenario: Minimum zoom limit
- **WHEN** 用户持续按 `h` 缩小到 scale = 0.1 后再次按 `h`
- **THEN** scale 保持 0.1 不变

### Requirement: Ctrl+j/k/h/l keys pan the image
用户 SHALL 能够使用 `Ctrl+j`（下移）、`Ctrl+k`（上移）、`Ctrl+h`（右移）、`Ctrl+l`（左移）平移图片焦点，每次移动 100px。

#### Scenario: Pan image down
- **WHEN** 用户按下 `Ctrl+j`
- **THEN** 图片向下平移 100px

#### Scenario: Pan image left
- **WHEN** 用户按下 `Ctrl+l`
- **THEN** 图片向左平移 100px

### Requirement: Esc/q closes the viewer
用户 SHALL 能够使用 `Esc` 或 `q` 关闭全屏查看器，返回三列布局，焦点恢复到打开前的面板。

#### Scenario: Close with Escape
- **WHEN** 用户在全屏查看器中按下 `Esc`
- **THEN** 查看器关闭，焦点恢复到之前活动的列

#### Scenario: Close with q
- **WHEN** 用户在全屏查看器中按下 `q`
- **THEN** 查看器关闭，焦点恢复到之前活动的列

### Requirement: Image info footer
全屏查看器底部 SHALL 显示当前图片的文件名、原始尺寸、文件大小和在图片列表中的位置（如 2/8）。

#### Scenario: Footer shows image metadata
- **WHEN** 用户打开一张 3840×2160、2.4MB 的 photo.jpg，目录中共有 8 张图片，当前是第 2 张
- **THEN** 底部显示 "photo.jpg  3840×2160  2.4MB  2/8"
