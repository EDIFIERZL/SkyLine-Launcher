@echo off
chcp 65001 >nul
cd /d "%~dp0"

if "%~1"=="" (
  echo.
  echo SkyLine Launcher 启动脚本
  echo.
  echo 用法：run-skyline [dev^|build^|tauri^|tauri-build^|preview^|lint]
  echo.
  echo 命令：
  echo   dev          启动 Vite 开发服务器 ^(对应 package.json 的 dev^)
  echo   build        构建前端生产包 ^(对应 package.json 的 build^)
  echo   tauri        启动 Tauri 开发模式 ^(对应 package.json 的 tauri dev^)
  echo   tauri-build  构建 Tauri 应用安装包 ^(对应 package.json 的 tauri build^)
  echo   preview      预览构建后的前端 ^(对应 package.json 的 preview^)
  echo   lint         运行代码检查 ^(对应 package.json 的 lint^)
  echo.
  pause
  exit /b 1
)

set "cmd=%~1"

if /i "%cmd%"=="dev" (
  npm run dev
  exit /b %errorlevel%
)

if /i "%cmd%"=="build" (
  npm run build
  exit /b %errorlevel%
)

if /i "%cmd%"=="tauri" (
  npm run tauri dev
  exit /b %errorlevel%
)

if /i "%cmd%"=="tauri-build" (
  npm run tauri build
  exit /b %errorlevel%
)

if /i "%cmd%"=="preview" (
  npm run preview
  exit /b %errorlevel%
)

if /i "%cmd%"=="lint" (
  npm run lint
  exit /b %errorlevel%
)

echo.
echo 错误：未知命令 "%cmd%"
echo 请运行 run-skyline.bat 查看可用命令
pause
exit /b 1
