
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
sources = [
    "ForecastNetherlands",
    "ForecastEurope",
    "RealTimeObservations",
]

[knmi.open_data_api]
token = ""

[knmi.notification_service]
url = "mqtt.dataplatform.knmi.nl"
port = 433
token = ""
```

KNMI data sources can be indiviually enabled to load only the weather data you require.

To obtain access to the KNMI data platform, you can register an account in the [KNMI Developer Portal](https://developer.dataplatform.knmi.nl/register/).

# Models

#### Arome

[Arome documentation (NL)](https://www.knmidata.nl/open-data/harmonie)