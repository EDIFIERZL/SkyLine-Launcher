<div align="center">
  <img src="public/logo.png" alt="SkyLine Launcher Logo" width="120">
  <h1>SkyLine Launcher</h1>
</div>

<p align="center">
  <b>功能全面的MINECRAFT启动器，让玩游戏更方便</b><br>
  支持Windows / macOS / Linux
  </p>

  <p align="center">
  <img src="https://img.shields.io/badge/SkyLine Launcher-1.0.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%7C%20Linux-lightgrey" alt="Platform">
</p>

---

## 介绍

**SkyLine Launcher** 旨在帮助用户更加方便的游玩和管理游戏，所有功能都在您的本地运行，功能全面，是您的MINECRAFT好帮手

### 游戏

- 全面的游戏下载
- Java路径自动检测
- 支持微软账号 / 本地离线账户 / 第三方皮肤站登录
- 自动优化内存，让游戏体验得到提升

### 资源

- 使用MC百科获取资源图标，资源目前支持Modrinth（em...别问我为何没有CurseForge，因为谷歌账号注册不下来awa）
- 一键导入 / 导出整合包，更加方便的整合包管理
- 默认版本隔离，让资源管理更加方便
- 世界、模组、资源包、光影包、原理图、数据包管理

### 联机

- **陶瓦联机**：更快速、便捷的EasyTier异地组网方案
- **局域网联机**：和朋友一起在家开黑的最佳选择

### 特色功能

- **AI助手**：内置Agnes旗下Agnes-2.5-flash模型，只需API密钥即可免费使用。
  - 游戏崩溃？自动唤醒帮你分析崩溃原因；
  - 可上传图片截屏、视频等文件进行快速分析；
  - 无需使用梯子，国内也能轻松直链。
- **液态玻璃**：灵动的视效，更好的操作反馈（对GPU占用较高，低配设备慎用）
- **自定义背景**：可上传本地图片或视频用于背景
- **音乐播放器**：可选择本地音乐进行播放，无损音质，更好的播放体验（因开发者为未成年，据平台要求无法申请音乐平台API，敬请谅解）

---

## 系统要求

| 平台    | 说明                                       |
| ------- | ------------------------------------------ |
| Windows | Windows 10及以上                           |
| macOS   | （尚不确定支持版本，如无法运行请提交至QQ） |
| Linux   | 64 位主流发行版                            |

## 更新日志

### v1.0.0（2026.8.15）

- SkyLine Launcher第一版发布，支持部分功能

---

## 技术栈

- **主要结构**：使用Tauri (React + Rust) 构建
- **前端**：React + TypeScript + Vite + React Router，使用 Material-UI ， Zustand（stores）状态管理， lucide-react 图标。
- **后端**：Rust + Tauri v2，使用 @tauri-apps/api 进行 IPC，reqwest 进行网络请求，文件处理用 walkdir/zip 。

## 开发

```sh
# 安装依赖
npm install (为了方便你开发直接把 node_modules也放进来了awa)

# 启动热更新开发模式
npm run tauri dev

# 构建
npm run tauri
```

## 项目结构

```sh
# 前端：
src/
├── App.tsx                 # 路由入口，挂载 MaterialProvider
├── main.tsx                # React 根节点
├── App.css / index.css     # 全局样式
├── assets/                 # 静态资源（图标、皮肤等）
├── components/
│   ├── material/           # 封装的 MUI 组件（Button, Input, Card 等）
│   ├── ui/                 # Radix UI 原子组件
│   ├── music/              # 音乐播放器组件
│   ├── InstancePanel.tsx   # 实例侧边栏面板
│   ├── InstanceCard.tsx    # 实例卡片
│   ├── LaunchButton.tsx    # 启动按钮
│   ├── VersionSettingsPanel.tsx
│   ├── DownloadCenter.tsx
│   ├── CrashDialog.tsx
│   ├── SkinViewer3D.tsx / SkinAvatar.tsx
│   ├── ResourceDetail.tsx  # 资源详情下载面板
│   ├── Layout.tsx          # 主布局 + 导航栏
│   └── ...
├── pages/                  # 页面组件（路由对应）
│   ├── Home.tsx            # 首页（实例列表 + 启动）
│   ├── Library.tsx         # 库页面（实例管理）
│   ├── Download.tsx        # 资源下载（模组/资源包/地图等）
│   ├── Account.tsx         # 账号登录
│   ├── Settings.tsx        # 设置
│   ├── Mods.tsx            # 模组管理页
│   ├── ResourcePacks.tsx   # 资源包管理页
│   ├── Schematics.tsx      # 原理图管理页
│   ├── InstanceManagement.tsx  # 实例综合管理（6标签）
│   ├── Multiplayer.tsx     # 多人游戏
│   ├── AiCrash.tsx         # AI 崩溃分析
│   ├── WorldMapPreview.tsx # 世界地图预览
│   ├── ModBrowser.tsx      # 模组浏览器
│   ├── Music.tsx / Help.tsx
│   └── ...
├── stores/                 # Zustand 状态管理
│   ├── authStore.ts
│   ├── instanceStore.ts
│   ├── downloadStore.ts
│   ├── settingsStore.ts
│   ├── musicStore.ts
│   └── notificationStore.ts
├── hooks/
│   ├── useMemoryOptimizer.ts
│   ├── useAuthRefresh.ts
│   └── useKeyboardShortcuts.ts
├── lib/
│   ├── catalog.ts          # 资源目录缓存
│   ├── instanceSort.ts     # 实例排序
│   └── utils.ts
└── types/
    └── index.ts            # 全局 TypeScript 类型

# 后端：
src-tauri/src/
├── lib.rs                  # Tauri 入口，注册所有命令和 State
├── main.rs                 # Windows 桌面入口
├── task.rs                 # 后台任务调度
├── commands/               # IPC 命令模块
│   ├── mod.rs              # pub mod 汇总
│   ├── instance.rs         # 实例 CRUD、文件夹、世界、截图
│   ├── mods.rs             # 模组扫描、开关、检测更新、MC百科搜索
│   ├── modpack.rs          # Modrinth/CurseForge API 下载与安装
│   ├── auth.rs             # 微软/离线/ Authlib 登录
│   ├── launch.rs           # Java 检测、游戏启动/停止进程
│   ├── settings.rs         # 配置读写
│   ├── download.rs         # 下载任务管理
│   ├── crash_ai.rs         # AI 崩溃分析（调用 Agnes API）
│   ├── script.rs           # 启动脚本生成（.bat/.ps1/.sh）
│   ├── music.rs            # 音乐播放
│   ├── terracotta.rs       # Terracotta 模组 IDE 集成
│   ├── memory.rs           # 内存优化
│   └── redstone_online.rs  # 红石联机（已移除命令注册，文件保留）
├── mc/                     # Minecraft 核心逻辑
│   ├── install.rs          # 游戏 & 加载器安装
│   ├── modloader.rs        # Forge/Fabric/Quilt/NeoForge 处理
│   ├── java.rs             # Java 环境检测
│   ├── launch.rs           # JVM 参数构建 & 启动流程
│   ├── process.rs          # 进程管理 & 日志读取
│   ├── world.rs            # 世界扫描/NBT 解析/地图渲染
│   ├── crash.rs            # 崩溃报告解析
│   ├── multiplayer.rs      # 多人服务器 servers.dat 读写
│   ├── region.rs           # 区域文件渲染（地图预览）
│   ├── seedmap.rs          # 种子地图合成
│   ├── auth.rs / authlib.rs # 认证逻辑
│   ├── hardware.rs         # 硬件检测
│   ├── nt_memory.rs        # Windows NT 内存优化
│   ├── mirror.rs           # 镜像源选择
│   ├── proxy.rs            # 代理配置
│   ├── version.rs          # 版本清单解析
│   ├── asset.rs / library.rs
│   ├── logexport.rs / updater.rs / server_status.rs
│   └── mod.rs
├── instance/
│   ├── mod.rs              # Instance / ModLoader / IsolationMode 定义
│   ├── manager.rs          # 实例列表扫描、外部实例识别、去重
│   └── mods.rs             # 模组/资源包/数据包扫描、NBT 解析
├── modpack/
│   ├── mod.rs              # Modrinth/CurseForge API 封装
│   └── modpack.rs          # 整合包导入/导出（MRPACK/CF ZIP/MMC/HMCL）
├── download/
│   ├── mod.rs              # 下载器基础实现
│   ├── multi.rs            # 分片下载器
│   └── manager.rs          # 下载任务队列管理
└── utils/
    ├── mod.rs
    ├── io.rs               # 路径工具（实例目录、配置路径等）
    └── crypto.rs           # SHA1/SHA256/加密

# 根目录文件夹：
skyline-launcher/
├── package.json            # 依赖：React 19, MUI 9, Zustand 5, Tauri 2, Tailwind v4
├── vite.config.ts
├── tsconfig*.json
├── src/                    # 前端
├── src-tauri/              # Rust 后端（Cargo.toml 见上文）
├── dist/                   # 构建产物
└── public/                 # 静态资源
```

## 关于

- **开发者**：EDIFIER_ZL
- **反馈**：QQ群 1025780262
