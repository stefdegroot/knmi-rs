
This repository is under active development. The system will not yet work based on the latest commited code.

#### Running the system

```bash
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

Eccodes install:
https://gist.github.com/MHBalsmeier/a01ad4e07ecf467c90fad2ac7719844a

```
export PKG_CONFIG_PATH=/usr/src/eccodes/lib/pkgconfig
export LD_LIBRARY_PATH=/usr/src/eccodes/lib
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