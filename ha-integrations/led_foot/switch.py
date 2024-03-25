from __future__ import annotations

import logging
from typing import Any

from .const import DOMAIN
import homeassistant.components.led_foot.led_foot as led_foot

from homeassistant.components.switch import SwitchEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback

async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    """Set up the switch platform for LED Foot."""
    led_foot_api: led_foot.LedFootApi = hass.data[DOMAIN][entry.entry_id]
    async_add_entities([LedFootRoom(led_foot_api, r) for r in led_foot_api.rooms])

class LedFootRoom(SwitchEntity):
    def __init__(self, led_foot_api: led_foot.LedFootApi, room_id: str):
        self._name = 'Led Foot Room ' + room_id
        self._room_id = room_id
        self._api = led_foot_api

    @property
    def name(self) -> str:
        return self._name

    @property
    def unique_id(self) -> str:
        return 'switch.led_foot.' + self._room_id

    @property
    def is_on(self) -> bool:
        try:
            return self._api.rooms[self._room_id]
        except KeyError:
            return False

    def turn_on(self, **kwargs) -> None:
        try:
            self._api.rooms[self._room_id] = True
        except KeyError:
            pass
        else:
            self._api.push_rooms()

    def turn_off(self, **kwargs) -> None:
        try:
            self._api.rooms[self._room_id] = False
        except KeyError:
            pass
        else:
            self._api.push_rooms()

    def update(self) -> None:
        """Fetch new state data for this switch.

        This is the only method that should fetch new data for Home Assistant.
        """
        self._api.pull_state()