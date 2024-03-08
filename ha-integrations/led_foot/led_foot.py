''' Integration with LED Foot LED server, written in Rust. Basically mirrors the
HTTP API provided by the Rust server. '''

import json
import requests

LED_FOOT_SERVER_API = 'http://localhost:5000/api/'
DEFAULT_COLOR = (0, 0, 0, 0)

class LedFootState:
    def __init__(self):
        self.current_rgbw = DEFAULT_COLOR
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
    rgbw = requests.get(LED_FOOT_SERVER_API + 'get-color')
    if rgbw.status_code == 200:
        color = rgbw.json()
        return color_dict_to_tuple(color)
    else:
        return DEFAULT_COLOR


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
    ''' Convert a tuple (R, G, B, W) to a dictionary '''
    return {
        'r': color[0],
        'g': color[1],
        'b': color[2],
        'w': color[3],
    }

def color_dict_to_tuple(color: dict) -> tuple:
    ''' Convert a dictionary color to a tuple (R, G, B, W) '''
    return (color['r'], color['g'], color['b'], color['w'])