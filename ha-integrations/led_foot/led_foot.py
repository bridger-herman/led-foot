''' Integration with LED Foot LED server, written in Rust. Basically mirrors the
HTTP API provided by the Rust server. '''

import json
import requests

LED_FOOT_SERVER_API = 'http://localhost:5000/api/'
DEFAULT_OFF_COLOR = (0, 0, 0, 0)
DEFAULT_ON_COLOR = (255, 75, 0, 255)


class LedFootState:
    def __init__(self):
        self.current_rgbw = DEFAULT_OFF_COLOR
        self.current_sequence = None

    def pull(self):
        self.current_rgbw = get_rgbw()
        self.current_sequence = get_sequence()

    def push(self):
        set_rgbw(*self.current_rgbw)
        set_sequence(self.current_sequence)

    # async def check_connection(self):
    #     '''check connection to Led Foot server'''
    #     resp = requests.get(LED_FOOT_SERVER_API)
    #     if resp.status_code != 200:
    #         raise Exception('Unable to connect to Led Foot server API: ' + resp.text)



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

def get_sequence():
    pass

def set_sequence(seq_name: str):
    pass


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