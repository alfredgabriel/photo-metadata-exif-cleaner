import { browser } from '$app/environment';
import { init, register, waitLocale } from 'svelte-i18n';

register('en', () => import('../locales/en.json'));
register('es', () => import('../locales/es.json'));

init({
  fallbackLocale: 'en',
  initialLocale: browser ? window.navigator.language : 'en',
});

export async function load() {
    if (browser) {
        await waitLocale();
    }
}
