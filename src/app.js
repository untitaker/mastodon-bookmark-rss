// this will cause esbuild to also create src/bundle.css
import "./app.css";

const OAUTH_CLIENT_NAME = "Mastodon Bookmark RSS";
const OAUTH_SCOPES = "read:bookmarks";
const SERVICE_BASE_URL = `${window.location.origin}${window.location.pathname}`;

const launchLogin = async (baseUrl) => {
  window.localStorage.setItem("baseUrl", baseUrl);

  const params = new FormData();
  params.append("client_name", OAUTH_CLIENT_NAME);
  params.append("website", SERVICE_BASE_URL);
  params.append("scopes", OAUTH_SCOPES);
  params.append("redirect_uris", SERVICE_BASE_URL);

  const response = await fetch(`https://${baseUrl}/api/v1/apps`, {
    method: "POST",
    mode: "cors",
    body: params,
  });
  const json = await response.json();
  const clientId = json.client_id;
  const clientSecret = json.client_secret;
  window.localStorage.setItem("clientId", clientId);
  window.localStorage.setItem("clientSecret", clientSecret);

  const url = `https://${baseUrl}/oauth/authorize?scope=${OAUTH_SCOPES}&response_type=code&redirect_uri=${SERVICE_BASE_URL}&client_id=${clientId}&client_secret=${clientSecret}`;
  window.location.href = url;
};

const getFeedUrl = async () => {
  const queryParams = new URL(document.location.toString()).searchParams;
  const authCode = queryParams.get("code");
  if (!authCode) {
    return false;
  }

  const baseUrl = window.localStorage.getItem("baseUrl");
  const clientId = window.localStorage.getItem("clientId");
  const clientSecret = window.localStorage.getItem("clientSecret");

  if (!baseUrl || !clientId || !clientSecret) {
    return false;
  }

  // prune local storage and reset query paramters so that hard refresh of the
  // page works and we don't store data we don't need to store.
  window.localStorage.removeItem("baseUrl");
  window.localStorage.removeItem("clientId");
  window.localStorage.removeItem("clientSecret");
  history.replaceState({}, "", "/");

  const params = new FormData();
  params.append("client_id", clientId);
  params.append("client_secret", clientSecret);
  params.append("grant_type", "authorization_code");
  params.append("code", authCode);
  params.append("redirect_uri", SERVICE_BASE_URL);

  const response = await fetch(`https://${baseUrl}/oauth/token`, {
    method: "POST",
    mode: "cors",
    body: params,
  });
  const json = await response.json();

  const accessToken = json.access_token;
  if (!accessToken) {
    return false;
  }

  return `${SERVICE_BASE_URL}feed?host=${baseUrl}&token=${accessToken}`;
};

(async () => {
  const appRoot = document.getElementById("app-root");
  appRoot.innerText = "loading...";
  const feedUrl = await getFeedUrl();
  if (!feedUrl) {
    appRoot.innerHTML = `
    <form class="pure-form pure-form-stacked" onsubmit="submitLoginForm()">
      <fieldset>
        <label for="host">Your instance</label>
        <input
          type="text"
          id="host"
          class="pure-input-1"
          required
          name="host"
          placeholder="e.g. mastodon.social"
          title="Something that looks like a hostname"
        />
      </fieldset>

      <input
        type="submit"
        class="pure-button pure-button-primary"
        value="Get RSS feed"
      />
    </form>
    `;

    document.querySelector("form").onsubmit = (e) => {
      launchLogin(e.target.host.value);
      e.preventDefault();
    };
  } else {
    appRoot.innerHTML = `
    <div>
      <p class="green">Subscribe to the following URL in your feed reader. Anybody who knows this
      URL can read your bookmarks!</p>
      <form class="pure-form pure-form-stacked">
        <fieldset>
          <input type="text" class="pure-input-1" readOnly />
          <label for="form-client-link">Optional: Add an "Open In" link to bookmarks</label>
          <select onchange="changeClient()">
            <option value="none" selected>None</option>
            <option value="host">Your Mastodon Host</option>
            <option value="elk">Elk</option>
            <option value="elkcanary">Elk Canary</option>
            <option value="phanpy">Phanpy</option>
            <option value="phanpydev">Phanpy Dev</option>
            <option value="trunks">Trunks</option>
            <option value="ivory">Ivory</option>
          </select>
        </fieldset>
      </form>
    </div>
    `;

    const changeClient = (client) => {
      const result = document.querySelector("input");
      result.value = `${
        client === "none" ? feedUrl : `${feedUrl}&client=${client}`
      }`;
    };

    document.querySelector("select").onchange = (e) => {
      changeClient(e.target.value);
    };

    changeClient("none");
  }
})();
