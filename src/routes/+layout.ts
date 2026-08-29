import { browser } from '\/environment';
import { init, register } from 'svelte-i18n';

register('en', () => import('../locales/en.json'));
register('es', () => import('../locales/es.json'));

init({
  fallbackLocale: 'en',
  initialLocale: browser ? window.navigator.language : 'en',
});
