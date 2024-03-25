"""Platform for light integration."""
from __future__ import annotations

import logging
from typing import Any

from .const import DOMAIN
import homeassistant.components.led_foot.led_foot as led_foot

# Import the device class from the component that you want to support
from homeassistant.components.light import (ATTR_BRIGHTNESS, PLATFORM_SCHEMA,
                                            LightEntity)
from homeassistant.config_entries import ConfigEntry
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
    def __init__(self, led_foot_api: led_foot.LedFootApi) -> None:
        self._name: str = "Led Foot"
        self._api: led_foot.LedFootApi = led_foot_api

    @property
    def name(self) -> str:
        return self._name

    @property
    def unique_id(self) -> str:
        return 'light.led_foot.rgbw'

    @property
    def supported_color_modes(self) -> set[ColorMode] | None:
        return {ColorMode.BRIGHTNESS, ColorMode.ONOFF, ColorMode.RGBW}

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

    def turn_on(self, rgbw_color=None, brightness=None, **kwargs) -> None:
        if rgbw_color is None and brightness is None:
            self._api.current_rgbw = led_foot.DEFAULT_ON_COLOR
        elif rgbw_color is None:
            self._api.current_rgbw = tuple([brightness] * 4)
        else:
            self._api.current_rgbw = rgbw_color
        self._api.push_rgbw()

    def turn_off(self, **kwargs) -> None:
        self._api.current_rgbw = led_foot.DEFAULT_OFF_COLOR
        self._api.push_rgbw()

    def update(self) -> None:
        """Fetch new state data for this light.

        This is the only method that should fetch new data for Home Assistant.
        """
        self._api.pull_state()
