import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'Synerix',
  description: 'AI-native coding terminal that thinks, writes, reviews, and fixes code',
  lang: 'zh-CN',
  
  // GitHub Pages 子路径部署
  base: '/synerix/',

  themeConfig: {
    // 导航栏
    nav: [
      { text: '首页', link: '/' },
      { text: '指南', link: '/guide/getting-started' },
      { text: 'API', link: '/api/overview' },
      { text: '更新日志', link: '/changelog' },
    ],
    
    // 侧边栏
    sidebar: {
      '/guide/': [
        {
          text: '指南',
          items: [
            { text: '快速开始', link: '/guide/getting-started' },
            { text: '安装', link: '/guide/installation' },
            { text: '配置', link: '/guide/configuration' },
            { text: '使用模式', link: '/guide/modes' },
            { text: '故障排除', link: '/guide/troubleshooting' },
          ],
        },
      ],
      '/api/': [
        {
          text: 'API 文档',
          items: [
            { text: '概览', link: '/api/overview' },
            { text: '命令', link: '/api/commands' },
            { text: '配置', link: '/api/config' },
            { text: '插件', link: '/api/plugins' },
          ],
        },
      ],
    },
    
    // 社交链接
    socialLinks: [
      { icon: 'github', link: 'https://github.com/Agions/synerix' },
      { icon: 'gitlab', link: 'https://gitee.com/Agions/synerix' },
    ],
    
    // 编辑链接
    editLink: {
      pattern: 'https://github.com/Agions/synerix/edit/main/docs/:path',
      text: '编辑此页面',
    },
    
    // 搜索
    search: {
      provider: 'local',
    },
    
    // 页脚
    footer: {
      message: '使用 VitePress 构建',
      copyright: '© 2024 Agions. MIT License',
    },
  },
  
  // Markdown 配置
  markdown: {
    lineNumbers: true,
  },
});
