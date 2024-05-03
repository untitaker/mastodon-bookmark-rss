<script lang="ts">
  export let launchLogin;
  export let feedUrlPromise;

  let baseFeedUrl;

  function submitLoginForm(e) {
    launchLogin(e.target.host.value);
    e.preventDefault();
  }

  function changeClient(e) {
    const client = e.target.value;
    const result = document.querySelector('input');
    if(!baseFeedUrl) baseFeedUrl = result.value;
    result.value = `${client === 'none' ? baseFeedUrl : `${baseFeedUrl}&client=${client}`}`;
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
    <div>
      <p class="green">Subscribe to the following URL in your feed reader. Anybody who knows this
      URL can read your bookmarks!</p>
      <form class="pure-form">
        <fieldset>
          <input type="text" class="pure-input-1" readOnly value={feedUrl} />
        </fieldset>
        <fieldset>
          <label>Optional: Add an "Open In" link to bookmarks
            <select on:change={changeClient}>
              <option value="none" selected>None</option>
              <option value="host">Your Mastodon Host</option>
              <option value="elk">Elk</option>
              <option value="elkcanary">Elk Canary</option>
              <option value="phanpy">Phanpy</option>
              <option value="phanpydev">Phanpy Dev</option>
              <option value="trunks">Trunks</option>
              <option value="ivory">Ivory</option>
            </select>
          </label>
        </fieldset>
      </form>
    </div>
  {/if}
{/await}
