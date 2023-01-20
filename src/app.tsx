// this will cause esbuild to also create src/bundle.css
import './app.css';

import * as React from 'react';
import * as ReactDOM from 'react-dom/client';

const App = () => {
  // Declare a new state variable, which we'll call "count" 
  const [submittedForm, setSubmittedForm] = React.useState(null);

  const submitForm = (e: SubmitEvent) => {
    setSubmittedForm({
      host: e.target.host.value,
      token: e.target.token.value,
    });
    e.preventDefault();
  }

  let stepFour = null;

  if (submittedForm) {
    const url =`${window.location.href}feed?host=${submittedForm.host}&token=${submittedForm.token}`;
    stepFour = <li className="green">
      Subscribe to the following URL in your feed reader. Anybody who knows this URL can read your bookmarks!

      <form className="pure-form">
        <fieldset>
          <input type="text" className="pure-input-1" readOnly value={url} />
        </fieldset>
      </form>
    </li>;
  }

  return <ol>
        <li>Go to <code>yourinstance.example/settings/applications</code></li>
        <li>Create a new application with permission to read bookmarks</li>
        <li>Paste the <i>Access Token</i> back into this form:

            <form className="pure-form pure-form-aligned" onSubmit={submitForm}>
                <fieldset>
                    <div className="pure-control-group">
                    <label htmlFor="host">Your instance</label>
                    <input type="text" id="host" name="host" placeholder="yourinstance.example" pattern="[a-zA-Z0-9.]+" />
                    </div>

                    <div className="pure-control-group">
                    <label htmlFor="token">The token you copied</label>
                    <input type="password" id="token" name="token" />
                    </div>

                    <div className="pure-controls">
                    <input type="submit" className="pure-button pure-button-primary" value="Get RSS feed" />
                    </div>
                </fieldset>
            </form>
        </li>
        {stepFour}
    </ol>
};

const root = ReactDOM.createRoot(document.getElementById('app-root'));
root.render(<App />);
