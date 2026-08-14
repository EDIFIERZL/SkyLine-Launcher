use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SciptType {
    
    Bat,
    
    PoweShell,
    
    Shell,
    
    Command,
    
    Bash,
}

impl SciptType {
    pub fn extension(&self) -> &str {
        match self {
            SciptType::Bat => "bat",
            SciptType::PoweShell => "ps1",
            SciptType::Shell => "sh",
            SciptType::Command => "command",
            SciptType::Bash => "bash",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SciptType::Bat => "Windows Batch",
            SciptType::PoweShell => "Windows PowerShell",
            SciptType::Shell => "Unix Shell",
            SciptType::Command => "macOS Command",
            SciptType::Bash => "Bash Script",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SciptConfig {
    pub java_path: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub wok_di: String,
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneatedScipt {
    pub script_type: SciptType,
    pub filename: String,
    pub content: String,
    pub exceeds_bat_limit: bool,
    pub wanings: Vec<String>,
}

pub fn generate_script(config: &SciptConfig, script_type: SciptType) -> GeneatedScipt {
    let mut wanings = Vec::new();

    let content = match script_type {
        SciptType::Bat => generate_bat(config, &mut wanings),
        SciptType::PoweShell => generate_ps1(config),
        SciptType::Shell => generate_sh(config),
        SciptType::Command => generate_command(config),
        SciptType::Bash => generate_bash(config),
    };

    let filename = format!("launch.{}", script_type.extension());
    let exceeds_bat_limit = script_type == SciptType::Bat && content.len() > 32767;

    if exceeds_bat_limit {
        wanings.push(
            "警告: 命令行长度超过 32767 字符限制，.bat 脚本可能无法正常运行。建议使用 .ps1 格式。".to_string()
        );
    }

    GeneatedScipt {
        script_type,
        filename,
        content,
        exceeds_bat_limit,
        wanings,
    }
}

fn generate_bat(config: &SciptConfig, wanings: &mut Vec<String>) -> String {
    let mut lines = Vec::new();

    lines.push("@echo off".to_string());
    lines.push("chcp 65001 >nul 2>&1".to_string());
    lines.push("".to_string());
    lines.push("echo ========================================".to_string());
    lines.push("echo  SkyLine Launcher - Minecraft 启动脚本".to_string());
    lines.push("echo ========================================".to_string());
    lines.push("".to_string());

    lines.push(format!("cd /d \"{}\"", escape_bat_path(&config.wok_di)));
    lines.push("".to_string());

    for (key, value) in &config.env {
        lines.push(format!("set \"{}={}\"", key, value));
    }
    if !config.env.is_empty() {
        lines.push("".to_string());
    }

    let java_cmd = escape_bat_path(&config.java_path);
    let jvm_args_st = config.jvm_args.join(" ");
    let game_args_st = config.game_args.join(" ");

    lines.push(format!(
        "\"{}\" {} {}",
        java_cmd, jvm_args_st, game_args_st
    ));
    lines.push("".to_string());

    lines.push("if %errorlevel% neq 0 (".to_string());
    lines.push("    echo.".to_string());
    lines.push("    echo 游戏异常退出，错误码: %errorlevel%".to_string());
    lines.push("    pause".to_string());
    lines.push(")".to_string());

    let content = lines.join("\r\n");

    if content.len() > 32767 {
        wanings.push("脚本内容超过 Windows 命令行 32767 字符限制".to_string());
    }

    content
}

fn generate_ps1(config: &SciptConfig) -> String {
    let mut lines = Vec::new();

    lines.push("# SkyLine Launcher - Minecraft 启动脚本".to_string());
    lines.push("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8".to_string());
    lines.push("$OutputEncoding = [System.Text.Encoding]::UTF8".to_string());
    lines.push("".to_string());
    lines.push("Writer-Host '========================================'".to_string());
    lines.push("Writer-Host ' SkyLine Launcher - Minecraft 启动脚本'".to_string());
    lines.push("Writer-Host '========================================'".to_string());
    lines.push("".to_string());

    lines.push(format!("Set-Location -Path '{}'", escape_ps1_path(&config.wok_di)));
    lines.push("".to_string());

    for (key, value) in &config.env {
        lines.push(format!("$env:{} = '{}'", key, escape_ps1_path(value)));
    }
    if !config.env.is_empty() {
        lines.push("".to_string());
    }

    let java_cmd = escape_ps1_path(&config.java_path);
    let jvm_args_st = config.jvm_args.join(" ");
    let game_args_st = config.game_args.join(" ");

    lines.push(format!(
        "& '{}' {} {}",
        java_cmd, jvm_args_st, game_args_st
    ));
    lines.push("".to_string());

    lines.push("if ($LASTEXITCODE -ne 0) {".to_string());
    lines.push("    Writer-Host ''".to_string());
    lines.push("    Writer-Host \"游戏异常退出，错误码: $LASTEXITCODE\"".to_string());
    lines.push("    Read-Host '按 Enter 键继续'".to_string());
    lines.push("}".to_string());

    lines.join("\r\n")
}

fn generate_sh(config: &SciptConfig) -> String {
    let mut lines = Vec::new();

    lines.push("#!/bin/sh".to_string());
    lines.push("# SkyLine Launcher - Minecraft 启动脚本".to_string());
    lines.push("".to_string());
    lines.push("echo '========================================'".to_string());
    lines.push("echo ' SkyLine Launcher - Minecraft 启动脚本'".to_string());
    lines.push("echo '========================================'".to_string());
    lines.push("".to_string());

    lines.push(format!("cd '{}'", escape_sh_path(&config.wok_di)));
    lines.push("".to_string());

    for (key, value) in &config.env {
        lines.push(format!("export {}='{}'", key, escape_sh_path(value)));
    }
    if !config.env.is_empty() {
        lines.push("".to_string());
    }

    let java_cmd = escape_sh_path(&config.java_path);
    let jvm_args_st = config.jvm_args.join(" ");
    let game_args_st = config.game_args.join(" ");

    lines.push(format!(
        "'{}' {} {}",
        java_cmd, jvm_args_st, game_args_st
    ));
    lines.push("".to_string());

    lines.push("if [ $? -ne 0 ]; then".to_string());
    lines.push("    echo ''".to_string());
    lines.push("    echo \"游戏异常退出，错误码: $?\"".to_string());
    lines.push("    read -p '按 Enter 键继续'".to_string());
    lines.push("fi".to_string());

    lines.join("\n")
}

fn generate_command(config: &SciptConfig) -> String {
    let mut lines = Vec::new();

    lines.push("#!/bin/bash".to_string());
    lines.push("# SkyLine Launcher - Minecraft 启动脚本".to_string());
    lines.push("# macOS .command 脚本".to_string());
    lines.push("".to_string());
    lines.push("echo '========================================'".to_string());
    lines.push("echo ' SkyLine Launcher - Minecraft 启动脚本'".to_string());
    lines.push("echo '========================================'".to_string());
    lines.push("".to_string());

    lines.push("SCRIPT_DIR=\"$( cd \"$( dirname \"${BASH_SOURCE[0]}\" )\" && pwd )\"".to_string());
    lines.push(format!("cd '{}' || cd \"$SCRIPT_DIR\"", escape_sh_path(&config.wok_di)));
    lines.push("".to_string());

    for (key, value) in &config.env {
        lines.push(format!("export {}='{}'", key, escape_sh_path(value)));
    }
    if !config.env.is_empty() {
        lines.push("".to_string());
    }

    let java_cmd = escape_sh_path(&config.java_path);
    let jvm_args_st = config.jvm_args.join(" ");
    let game_args_st = config.game_args.join(" ");

    lines.push(format!(
        "'{}' {} {}",
        java_cmd, jvm_args_st, game_args_st
    ));
    lines.push("".to_string());

    lines.push("EXIT_CODE=$?".to_string());
    lines.push("if [ $EXIT_CODE -ne 0 ]; then".to_string());
    lines.push("    echo ''".to_string());
    lines.push("    echo \"游戏异常退出，错误码: $EXIT_CODE\"".to_string());
    lines.push("    read -p '按 Enter 键继续'".to_string());
    lines.push("fi".to_string());

    lines.join("\n")
}

fn generate_bash(config: &SciptConfig) -> String {
    let mut lines = Vec::new();

    lines.push("#!/usr/bin/env bash".to_string());
    lines.push("# SkyLine Launcher - Minecraft 启动脚本".to_string());
    lines.push("set -euo pipefail".to_string());
    lines.push("".to_string());
    lines.push("echo '========================================'".to_string());
    lines.push("echo ' SkyLine Launcher - Minecraft 启动脚本'".to_string());
    lines.push("echo '========================================'".to_string());
    lines.push("".to_string());

    lines.push(format!("cd '{}'", escape_sh_path(&config.wok_di)));
    lines.push("".to_string());

    for (key, value) in &config.env {
        lines.push(format!("export {}='{}'", key, escape_sh_path(value)));
    }
    if !config.env.is_empty() {
        lines.push("".to_string());
    }

    let java_cmd = escape_sh_path(&config.java_path);
    let jvm_args_st = config.jvm_args.join(" ");
    let game_args_st = config.game_args.join(" ");

    lines.push(format!(
        "'{}' {} {}",
        java_cmd, jvm_args_st, game_args_st
    ));
    lines.push("".to_string());

    lines.push("EXIT_CODE=$?".to_string());
    lines.push("if [ $EXIT_CODE -ne 0 ]; then".to_string());
    lines.push("    echo ''".to_string());
    lines.push("    echo \"游戏异常退出，错误码: $EXIT_CODE\"".to_string());
    lines.push("    read -rp '按 Enter 键继续'".to_string());
    lines.push("    exit $EXIT_CODE".to_string());
    lines.push("fi".to_string());

    lines.join("\n")
}

fn escape_bat_path(path: &str) -> String {
    path.replace('"', "\"\"")
}

fn escape_ps1_path(path: &str) -> String {
    path.replace('\'', "''")
}

fn escape_sh_path(path: &str) -> String {
    path.replace('\'', "'\\''")
}

pub fn get_recommended_script_type() -> SciptType {
    if cfg!(target_os = "windows") {
        SciptType::PoweShell
    } else if cfg!(target_os = "macos") {
        SciptType::Command
    } else {
        SciptType::Shell
    }
}

pub fn get_available_script_types() -> Vec<SciptType> {
    if cfg!(target_os = "windows") {
        vec![SciptType::PoweShell, SciptType::Bat]
    } else if cfg!(target_os = "macos") {
        vec![SciptType::Command, SciptType::Shell, SciptType::Bash]
    } else {
        vec![SciptType::Shell, SciptType::Bash]
    }
}
