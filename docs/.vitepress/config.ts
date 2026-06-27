import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'Synerix',
  description: 'AI-native coding terminal — think, write, review, and fix code in the terminal',
  lang: 'en-US',
  base: '/synerix/',
  lastUpdated: true,

  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'API', link: '/api/overview' },
      { text: 'Changelog', link: '/changelog' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Configuration', link: '/guide/configuration' },
            { text: 'Coding Modes', link: '/guide/modes' },
            { text: 'Troubleshooting', link: '/guide/troubleshooting' },
            { text: 'Contributing', link: '/guide/contributing' },
          ],
        },
      ],
      '/api/': [
        {
          text: 'API Reference',
          items: [
            { text: 'Overview', link: '/api/overview' },
            { text: 'Commands', link: '/api/commands' },
            { text: 'Configuration', link: '/api/config' },
            { text: 'Plugins', link: '/api/plugins' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/Agions/synerix' },
      { icon: 'gitlab', link: 'https://gitee.com/Agions/synerix' },
    ],

    editLink: {
      pattern: 'https://github.com/Agions/synerix/edit/main/docs/:path',
      text: 'Edit this page',
    },

    search: {
      provider: 'local',
    },

    footer: {
      message: 'Powered by VitePress',
      copyright: '© 2026 Agions. MIT License',
    },
  },
});
