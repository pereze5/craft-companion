<script lang="ts">
  export let title: string;
  export let open = false;
</script>

<section>
  <button class="toggle" on:click={() => (open = !open)}>
    {open ? "▼" : "▶"} {title}
  </button>

  {#if open}
    <div class="content">
      <slot />
    </div>
  {/if}
</section>

<style>
  .toggle {
    width: 100%;
    text-align: left;
    font-weight: 700;
  }

  .content {
    margin-top: 0.75rem;
  }
</style>