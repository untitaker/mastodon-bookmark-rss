<script lang="ts">
  export let launchLogin;
  export let feedUrlPromise;

  function submitLoginForm(e) {
    launchLogin(e.target.host.value);
    e.preventDefault();
  }
</script>

{#await feedUrlPromise}
  <div>loading...</div>
{:then feedUrl}
  {#if !feedUrl}
    <form class="pure-form pure-form-stacked" on:submit={submitLoginForm}>
      <fieldset>
        <label for="host">Your instance</label>
        <input
          type="text"
          id="host"
          class="pure-input-1"
          required
          name="host"
          placeholder="e.g. mastodon.social"
          pattern="[a-zA-Z0-9.:\\-]+"
          title="Something that looks like a hostname"
        />
      </fieldset>

      <input
        type="submit"
        class="pure-button pure-button-primary"
        value="Get RSS feed"
      />
    </form>
  {:else}
    <div class="green">
      Subscribe to the following URL in your feed reader. Anybody who knows this
      URL can read your bookmarks!
      <form class="pure-form">
        <fieldset>
          <input type="text" class="pure-input-1" readOnly value={feedUrl} />
        </fieldset>
      </form>
    </div>
  {/if}
{/await}
