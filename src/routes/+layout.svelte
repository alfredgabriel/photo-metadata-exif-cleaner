<script>
  import { isLoading } from 'svelte-i18n';
</script>
{#if $isLoading}
  <div class="loader">Loading...</div>
{:else}
  <slot />
{/if}
<style>
  .loader {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    color: #a1a1aa;
    background-color: #09090b;
    font-family: sans-serif;
  }
</style>
