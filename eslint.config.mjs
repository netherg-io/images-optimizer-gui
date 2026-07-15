import withNuxt from './.nuxt/eslint.config.mjs';
import stylistic from '@stylistic/eslint-plugin';
import prettier from 'eslint-plugin-prettier/recommended';
import pluginVue from 'eslint-plugin-vue';

export default withNuxt(
  stylistic.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    ignores: [
      'public/*',
      'src-tauri/target/**',
      '.nuxt/**',
      '.output/**',
      'dist/**',
    ],
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      'no-undef': 'off',
      'no-useless-catch': 'off',
      'vue/no-v-html': 'off',
      'vue/prop-name-casing': 'off',
      'vue/multi-word-component-names': 'off',
      '@stylistic/indent': ['error', 2],
    },
  },
  prettier,
);
