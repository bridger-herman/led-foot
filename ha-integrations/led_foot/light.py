"""Platform for light integration."""
from __future__ import annotations

import logging
from typing import Any

# import awesomelights
import homeassistant.components.led_foot.led_foot as led_foot
import voluptuous as vol

# Import the device class from the component that you want to support
import homeassistant.helpers.config_validation as cv
from homeassistant.components.light import (ATTR_BRIGHTNESS, PLATFORM_SCHEMA,
                                            LightEntity)
from homeassistant.config_entries import ConfigEntry
from homeassistant.const import CONF_HOST, CONF_PASSWORD, CONF_USERNAME
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.helpers.typing import ConfigType, DiscoveryInfoType
from homeassistant.components.light import ColorMode

_LOGGER = logging.getLogger(__name__)

# Validation of the user's configuration
PLATFORM_SCHEMA = PLATFORM_SCHEMA.extend({
    vol.Required(CONF_HOST): cv.string,
    vol.Optional(CONF_USERNAME, default='admin'): cv.string,
    vol.Optional(CONF_PASSWORD): cv.string,
})


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    """Set up the light platform for LEDBLE."""
    async_add_entities([LedFoot()])


class LedFoot(LightEntity):
    """Representation of an Awesome Light."""

    def __init__(self) -> None:
        """Initialize an AwesomeLight."""
        self._name = "Led Foot"
        self._state = led_foot.LedFootState()

    @property
    def name(self) -> str:
        return self._name

    @property
    def supported_color_modes(self) -> set[ColorMode] | None:
        return set(ColorMode.RGBW)

    @property
    def color_mode(self) -> ColorMode | None:
        return ColorMode.RGBW

    @property
    def is_on(self) -> bool | None:
        return sum(self._state.current_rgbw) > 0

    @property
    def brightness(self) -> int | None:
        return 1

    @property
    def hs_color(self) -> tuple[float, float] | None:
        return (0.5, 0.5)

    @property
    def rgbw_color(self) -> tuple[int, int, int, int] | None:
        return self._state.current_rgbw

    def turn_on(self) -> None:
        self._state.current_rgbw = (1, 1, 1, 1)
        self._state.push()

    def turn_off(self) -> None:
        self._state.current_rgbw = (0, 0, 0, 0)
        self._state.push()

    def update(self) -> None:
        """Fetch new state data for this light.

        This is the only method that should fetch new data for Home Assistant.
        """
        self._state.pull()
