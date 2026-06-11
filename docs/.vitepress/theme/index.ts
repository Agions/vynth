import DefaultTheme from 'vitepress/theme';
import './styles.css';

export default {
  ...DefaultTheme,
  enhanceApp({ app }) {
    // 可以在这里注册全局组件
  },
};
