# Home Assistant Integrations

This folder contains integrations for [Home Assistant](https://homeassistant.io).

## Installation Instructions

On Linux, the easiest way to install this custom component for HA is just to
copy the component folder to the `<config>/custom_components` folder
(unfortunately, linking the folder doesn't seem to work):

```
# if the custom_components folder doesn't exist yet:
mkdir <path to ha config>

cp -r <path to this repo>/ha-integrations/led_foot <path to ha config>/custom_components
```


If something doesn't work right, check the log at `<ha config>/home-assistant.log`.

Note that after you make any changes, you'll need to restart Home Assistant.

For example, if running the HA container in Docker, the command would be

```
docker restart homeassistant
```