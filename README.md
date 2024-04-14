
#### Running the system

```bash
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

#### Config

The server requires a toml config file in the root folder with the following properties.

```toml
[server]
port = 3000

[knmi]
open_data_api_token  = ""
notification_service_token  = ""
```

# Models

#### Arome

[Arome documentation (NL)](https://www.knmidata.nl/open-data/harmonie)