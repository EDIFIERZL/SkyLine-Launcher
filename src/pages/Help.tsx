import { useState } from 'react'
import { Box, Typography, Card } from '../components/material'
import { HelpCircle, BookOpen, ExternalLink, Keyboard, Download, Settings, Puzzle, Shield } from 'lucide-react'

interface HelpSection {
  id: string
  icon: typeof HelpCircle
  title: string
  content: string
  links?: { label: string; url: string }[]
}

const HELP_SECTIONS: HelpSection[] = [
  {
    id: 'quick-start',
    icon: BookOpen,
    title: '快速开始',
    content: '1. 在「账户」页面登录（支持离线、微软、Authlib 等方式）\n2. 在「资源」页面下载 Minecraft 版本\n3. 回到首页点击「启动」即可开始游戏',
  },
  {
    id: 'java',
    icon: Settings,
    title: 'Java 管理',
    content: '启动器会自动检测系统中已安装的 Java。如果需要手动指定，可以在设置页面的 Java 管理部分添加自定义 Java 路径。\n\n推荐版本：\n• Minecraft 1.16 及以下：Java 8\n• Minecraft 1.17-1.20.4：Java 17\n• Minecraft 1.20.5+：Java 21',
  },
  {
    id: 'mods',
    icon: Puzzle,
    title: '模组管理',
    content: '在实例的「模组管理」页面可以：\n• 启用/禁用模组（点击开关）\n• 删除模组（点击删除按钮）\n• 检查模组更新\n• 搜索和筛选模组\n\n模组文件放在实例目录的 mods 文件夹中。',
  },
  {
    id: 'download',
    icon: Download,
    title: '下载资源',
    content: '在「资源」页面可以下载：\n• Minecraft 版本\n• 模组（Forge/Fabric/NeoForge/Quilt）\n• 整合包\n• 资源包\n• 光影包\n\n支持 CurseForge 和 Modrinth 两个平台。',
  },
  {
    id: 'account',
    icon: Shield,
    title: '账户登录',
    content: '支持以下登录方式：\n• 离线模式：无需正版账号\n• Microsoft：正版微软账号登录\n• Authlib Injector：第三方认证服务器（如 littleskin）\n• 统一通行证（Nide）：国内认证服务\n\n多个账户可以在账户页面管理并快速切换。',
  },
  {
    id: 'shortcuts',
    icon: Keyboard,
    title: '快捷键',
    content: '• Ctrl+1：跳转首页\n• Ctrl+2：跳转资源页\n• Ctrl+3：跳转设置页',
  },
]

const EXTERNAL_LINKS = [
  { label: 'Minecraft 官网', url: 'https://www.minecraft.net' },
  { label: 'Forge 官网', url: 'https://files.minecraftforge.net' },
  { label: 'Fabric 官网', url: 'https://fabricmc.net' },
  { label: 'Modrinth', url: 'https://modrinth.com' },
  { label: 'CurseForge', url: 'https://www.curseforge.com' },
  { label: 'MCBBS', url: 'https://www.mcbbs.net' },
]

export function Help() {
  const [expandedId, setExpandedId] = useState<string | null>('quick-start')

  return (
    <Box className="max-w-3xl space-y-6">
      <Box>
        <Typography variant="h5" className="flex items-center gap-2">
          <HelpCircle className="w-6 h-6 text-[var(--accent-color)]" /> 帮助中心
        </Typography>
        <Typography variant="body2" color="text.secondary">
          常见问题解答和使用指南
        </Typography>
      </Box>

      <Box className="space-y-3">
        {HELP_SECTIONS.map((section) => {
          const Icon = section.icon
          const isExpanded = expandedId === section.id
          return (
            <Card key={section.id} className="overflow-hidden">
              <button
                className="w-full flex items-center gap-3 p-4 text-left hover:bg-surface-50 dark:hover:bg-surface-800 transition-colors"
                onClick={() => setExpandedId(isExpanded ? null : section.id)}
              >
                <Icon className="w-5 h-5 text-[var(--accent-color)] shrink-0" />
                <Typography variant="subtitle1" className="flex-1">{section.title}</Typography>
                <span className={`transform transition-transform ${isExpanded ? 'rotate-180' : ''}`}>
                  ▼
                </span>
              </button>
              {isExpanded && (
                <Box className="px-4 pb-4 pt-0">
                  <Typography variant="body2" color="text.secondary" className="whitespace-pre-line">
                    {section.content}
                  </Typography>
                  {section.links && (
                    <Box className="flex flex-wrap gap-2 mt-3">
                      {section.links.map((link) => (
                        <a
                          key={link.url}
                          href={link.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-[var(--accent-color)] hover:underline text-sm flex items-center gap-1"
                        >
                          {link.label} <ExternalLink className="w-3 h-3" />
                        </a>
                      ))}
                    </Box>
                  )}
                </Box>
              )}
            </Card>
          )
        })}
      </Box>

      <Card>
        <Box className="p-4 space-y-3">
          <Typography variant="subtitle1">相关链接</Typography>
          <Box className="flex flex-wrap gap-2">
            {EXTERNAL_LINKS.map((link) => (
              <a
                key={link.url}
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                className="px-3 py-1.5 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 text-sm transition-colors flex items-center gap-1.5"
              >
                {link.label}
                <ExternalLink className="w-3 h-3 opacity-50" />
              </a>
            ))}
          </Box>
        </Box>
      </Card>

      <Card>
        <Box className="p-4">
          <Typography variant="subtitle1" className="mb-2">关于 SkyLine Launcher</Typography>
          <Typography variant="body2" color="text.secondary">
            SkyLine Launcher 是一个现代化的 Minecraft 启动器，支持多种登录方式、模组管理、整合包导入导出等功能。
          </Typography>
        </Box>
      </Card>
    </Box>
  )
}
