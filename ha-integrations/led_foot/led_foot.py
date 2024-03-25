''' Integration with LED Foot LED server, written in Rust. Basically mirrors the
HTTP API provided by the Rust server. '''

import json
import requests

LED_FOOT_SERVER_API = 'http://localhost:5000/api/'
DEFAULT_OFF_COLOR = (0, 0, 0, 0)
DEFAULT_ON_COLOR = (255, 75, 0, 255)
DEFAULT_ROOMS = ['living_room', 'office', 'bedroom']
DEFAULT_ROOM_STATE = True

class LedFootApi:
    def __init__(self):
        self.current_rgbw = DEFAULT_OFF_COLOR
        self.rooms = {r: DEFAULT_ROOM_STATE for r in DEFAULT_ROOMS}
        self.sequence_list = []
        self.current_sequence = None

    def pull_state(self):
        self.current_rgbw = LedFootApi.get_rgbw()
        self.rooms = LedFootApi.get_rooms()
        self.sequence_list = LedFootApi.list_sequences()
        self.current_sequence = LedFootApi.get_sequence()

    def push_rgbw(self):
        LedFootApi.set_rgbw(*self.current_rgbw)

    def push_sequence(self):
        LedFootApi.set_sequence(self.current_sequence)

    def push_rooms(self):
        LedFootApi.set_rooms(self.rooms)

    def check_connection() -> bool:
        '''check connection to Led Foot server'''
        resp = requests.get(LED_FOOT_SERVER_API)
        if resp.status_code != 200:
            raise Exception('Unable to connect to Led Foot server API: ' + resp.text)
        return True



    def get_rgbw():
        # use get-color-future because HASS performs an update() immediately after
        # changing state, and the LED Foot is still in a transition then.
        rgbw = requests.get(LED_FOOT_SERVER_API + 'get-color-future')
        if rgbw.status_code == 200:
            color = rgbw.json()
            return color_dict_to_tuple(color)
        else:
            return DEFAULT_OFF_COLOR


    def set_rgbw(r: float, g: float, b: float, w: float):
        color_json = json.dumps(color_tuple_to_dict((r, g, b, w)))
        status = requests.post(
            LED_FOOT_SERVER_API + 'set-color',
            color_json,
            headers={'Content-type': 'application/json'}
        )

    def get_rooms() -> dict:
        rooms = requests.get(LED_FOOT_SERVER_API + 'get-rooms')
        if rooms.status_code == 200:
            return rooms.json()
        else:
            return {}


    def set_rooms(current_rooms: dict):
        rooms_json = json.dumps(current_rooms)
        status = requests.post(
            LED_FOOT_SERVER_API + 'set-rooms',
            rooms_json,
            headers={'Content-type': 'application/json'}
        )


    def list_sequences() -> list[str]:
        resp = requests.get(LED_FOOT_SERVER_API + 'list-sequences')
        if resp.status_code == 200:
            return resp.text.split('\n')
        else:
            return []

    def get_sequence() -> str | None:
        resp = requests.get(LED_FOOT_SERVER_API + 'get-sequence')
        if resp.status_code == 200:
            return resp.text
        else:
            return None

    def set_sequence(seq_name: str):
        status = requests.post(
            LED_FOOT_SERVER_API + 'set-sequence',
            seq_name
        )


def color_tuple_to_dict(color: tuple) -> dict:
    ''' Convert a HASS tuple (R, G, B, W [0-255]) to a dictionary {r: g: b: w: [0-1]}'''
    return {
        'r': color[0] / 255,
        'g': color[1] / 255,
        'b': color[2] / 255,
        'w': color[3] / 255,
    }

def color_dict_to_tuple(color: dict) -> tuple:
    ''' Convert a dictionary color {r: g: b: w: [0-1]} to a tuple (R, G, B, W [0-255]) '''
    return (round(color['r'] * 255), round(color['g'] * 255), round(color['b'] * 255), round(color['w'] * 255))