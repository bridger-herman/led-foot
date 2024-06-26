"""Platform for light integration."""
from __future__ import annotations

import logging
from typing import Any

from .const import DOMAIN
from .led_foot import LedFootApi, DEFAULT_ON_COLOR, DEFAULT_OFF_COLOR

# Import the device class from the component that you want to support
from homeassistant.components.light import LightEntity, LightEntityFeature
from homeassistant.config_entries import ConfigEntry
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.components.light import ColorMode

_LOGGER = logging.getLogger(__name__)

async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    """Set up the light platform for LED Foot."""
    led_foot_api = hass.data[DOMAIN][entry.entry_id]
    async_add_entities([LedFootRgbw(led_foot_api)])


class LedFootRgbw(LightEntity):
    def __init__(self, led_foot_api: LedFootApi) -> None:
        self._name: str = "Led Foot"
        self._api: LedFootApi = led_foot_api

    @property
    def name(self) -> str:
        return self._name

    @property
    def unique_id(self) -> str:
        return 'light.led_foot.rgbw'

    @property
    def device_info(self) -> DeviceInfo:
        """Return the device info."""
        return DeviceInfo(
            identifiers={
                (DOMAIN, self.unique_id)
            },
            name=self.name,
            manufacturer='Kind Digits',
            model='LED Foot RGBW',
            sw_version='0.0.1',
        )

    @property
    def supported_color_modes(self) -> set[ColorMode] | None:
        return {ColorMode.BRIGHTNESS, ColorMode.ONOFF, ColorMode.RGBW}

    @property
    def supported_features(self) -> LightEntityFeature:
        return LightEntityFeature.EFFECT

    @property
    def color_mode(self) -> ColorMode | None:
        return ColorMode.RGBW

    @property
    def is_on(self) -> bool | None:
        return sum(self._api.current_rgbw) > 0

    @property
    def brightness(self) -> int | None:
        r, g, b, w = self._api.current_rgbw
        return round(((0.2126*r + 0.7152*g + 0.0722*b) + w) / 2.0)


    @property
    def rgbw_color(self) -> tuple[int, int, int, int] | None:
        return self._api.current_rgbw

    @property
    def effect(self) -> str | None:
        return self._api.current_sequence or 'None'

    @property
    def effect_list(self) -> list[str] | None:
        return self._api.sequence_list

    def turn_on(self, rgbw_color=None, brightness=None, effect=None, **kwargs) -> None:
        if effect is None:
            if rgbw_color is None and brightness is None:
                self._api.current_rgbw = DEFAULT_ON_COLOR
            elif rgbw_color is None:
                # if rgbw color isn't supplied but brightness is, assume setting brightness
                scale_factor = brightness / self.brightness
                self._api.current_rgbw = tuple([scale_factor * channel for channel in self._api.current_rgbw])
            else:
                self._api.current_rgbw = rgbw_color

            self._api.current_sequence = None
            self._api.push_sequence()
            self._api.push_rgbw()
        else:
            self._api.current_sequence = effect
            self._api.push_sequence()

    def turn_off(self, **kwargs) -> None:
        self._api.current_rgbw = DEFAULT_OFF_COLOR
        self._api.current_sequence = None
        self._api.push_rgbw()
        self._api.push_sequence()

    def update(self) -> None:
        """Fetch new state data for this light.

        This is the only method that should fetch new data for Home Assistant.
        """
        self._api.pull_state()
