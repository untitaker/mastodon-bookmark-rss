// this will cause esbuild to also create src/bundle.css
import "./app.css";

import * as React from "react";
import * as ReactDOM from "react-dom/client";

const OAUTH_CLIENT_NAME = "Mastodon Bookmark RSS";
const OAUTH_SCOPES = "read:bookmarks";
const SERVICE_BASE_URL = `${window.location.origin}${window.location.pathname}`;

const launchLogin = async (baseUrl: string) => {
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

type AppProps = { feedUrlPromise: Promise<string | false> };

const App = ({ feedUrlPromise }: AppProps) => {
  const [feedUrl, setFeedUrl] = React.useState(null);
  feedUrlPromise.then((url) => setFeedUrl(url));

  if (feedUrl === null) {
    return <div>loading...</div>;
  } else if (feedUrl === false) {
    const submitLoginForm = (e: React.FormEvent<HTMLFormElement>) => {
      launchLogin(e.target.host.value);
      e.preventDefault();
    };

    return (
      <form className="pure-form pure-form-aligned" onSubmit={submitLoginForm}>
        <fieldset>
          <div className="pure-control-group">
            <label htmlFor="host">Your instance</label>
            <input
              type="text"
              id="host"
              required
              name="host"
              placeholder="yourinstance.example"
              pattern="[a-zA-Z0-9.]+"
              title="Something that looks like a hostname"
            />
          </div>

          <div className="pure-controls">
            <input
              type="submit"
              className="pure-button pure-button-primary"
              value="Get RSS feed"
            />
          </div>
        </fieldset>
      </form>
    );
  } else {
    return (
      <div className="green">
        Subscribe to the following URL in your feed reader. Anybody who knows
        this URL can read your bookmarks!
        <form className="pure-form">
          <fieldset>
            <input
              type="text"
              className="pure-input-1"
              readOnly
              value={feedUrl}
            />
          </fieldset>
        </form>
      </div>
    );
  }
};

const root = ReactDOM.createRoot(document.getElementById("app-root"));
root.render(<App feedUrlPromise={getFeedUrl()} />);
