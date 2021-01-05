const DAYS_OF_WEEK = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const WEMO_COMMANDS = [undefined, 'on', 'off', 'toggle'];
const ROOM_ICON_MAP = {
    'bedroom': 'night_shelter',
    'office': 'keyboard',
    'living_room': 'weekend',
};
const WEMO_ICON_MAP = {
    'Insight': 'wb_incandescent',
    'Mini': 'radio',
};

// Relies on the
//
// ```
// ./led-foot-sequences/<sequence_name>.png
// ```
//
// convention
function stripName(sequencePath) {
    let nameRegex = /.\/led-foot-sequences\/([a-zA-Z\-]+).png/;
    let match = sequencePath.match(nameRegex);
    return match[1];
}

function loadSequence(name) {
    let data = {'name': name};
    $.post({
        url: '/api/set-sequence',
        data: JSON.stringify(data),
        contentType: 'application/json; charset=utf-8',
    }).catch((err) => console.log(`Error setting sequence:\n${err}`));
}

function parseTime(timeStr) {
    let parts = timeStr.split(':');
    return {
        hour: parts[0],
        minute: parts[1],
    };
}

function updateSchedule(currentlyEdited, $origSchedEl) {
    let schedule = [];
    let scheduleElements = $('.schedule-element');
    scheduleElements.each((_index, element) => {
        // Skip the one that we've modified and add the modified values instead
        if ($(element).is($origSchedEl)) {
            schedule.push(currentlyEdited);
            return;
        }
        let time = parseTime($(element).find('.time').text());

        let days = $(element).find('.days').text();

        let $rooms = $(element).find('.room-icon');
        let rooms = {};
        $rooms.each((_i, el) => {
            let roomName = $(el).data('room');
            let status = $(el).data('status');
            rooms[roomName] = status;
        });

        let $wemos = $(element).find('.wemo-icon');
        let wemos = {};
        $wemos.each((_i, el) => {
            let wemoName = $(el).data('wemo');
            let cmd = $(el).data('command');
            wemos[wemoName] = cmd;
        });

        schedule.push({
            days: days.split(','),
            hour: time.hour,
            minute: time.minute,
            sequence: $(element).find('img').attr('src'),
            rooms,
            wemos,
        });
    });

    $.post({
        url: '/api/set-schedule',
        data: JSON.stringify(schedule),
        contentType: 'application/json; charset=utf-8',
    }).catch((err) => console.error(err));
}

function makeScheduleEditor(data, $scheduleElement) {
    let $sed = $('#schedule-editor-content');
    $sed.empty();
    $sed.data('scheduleElement', $scheduleElement);

    // This only works on FF, Chrome, and Edge.
    let $timeInput = $('<input>', {
        type: 'time',
        val: `${data.hour}:${data.minute}`,
    });
    $sed.append($('<div>', {
        class: 'schedule-input-row',
    }).append($timeInput));

    let $daysInput = $('<div>', {
        class: 'schedule-input-row',
    });
    for (const day of DAYS_OF_WEEK) {
        $daysInput.append(
            $('<span>', {
                class: 'day-of-week-input'
            }).append(
                $('<input>', {
                    id: `checkbox-${day}`,
                    type: 'checkbox',
                    prop: {checked: data.days.indexOf(day) >= 0},
                })
            ).append(
                $('<label>', {
                    for: `checkbox-${day}`,
                    text: day,
                })
            )
        )
    }
    $sed.append($daysInput);

    let $roomsInput = $('<div>', {
        class: 'schedule-input-row',
    });
    for (let room in ROOM_ICON_MAP) {
        $roomsInput.append(
            $('<span>', {
                class: 'room-input'
            }).append(
                $('<input>', {
                    type: 'checkbox',
                    id: `room-checkbox-${room}`,
                    prop: {checked: data.rooms[room]}
                })
            ).append(
                $('<label>', {
                    class: 'material-icons',
                    for: `room-checkbox-${room}`,
                    text: ROOM_ICON_MAP[room],
                })
            )
        )
    }
    $sed.append($roomsInput);

    let $wemosInput = $('<div>', {
        class: 'schedule-input-row',
    });
    for (let wemo in WEMO_ICON_MAP) {
        let $wemo = $('<span>', {
            class: 'wemo-input'
        }).append(
            $('<label>', {
                class: 'material-icons',
                for: `wemo-command-${wemo}`,
                text: WEMO_ICON_MAP[wemo],
            })
        );
        let $select = $('<select>', {
            id: `wemo-command-${wemo}`,
        });
        for (const cmd of WEMO_COMMANDS) {
            $select.append($('<option>', {
                val: cmd,
                text: cmd,
            }));
        }
        $select.val(data.wemos[wemo]);
        $wemo.append($select);
        $wemosInput.append($wemo);
    }
    $sed.append($wemosInput);

    let sequencePaths = [];
    $('#favorite-list>li>div').each((_i, el) => {
        sequencePaths.push($(el).data('sequencePath'));
    });

    let $sequencesInput = $('<div>', {
        class: 'schedule-input-row',
    });
    let $sequencesSelect = $('<select>', {
        id: 'sequence-selector',
    });
    $sequencesSelect.append(
        $('<option>', {val: undefined})
    );
    for (let seq of sequencePaths) {
        $sequencesSelect.append(
            $('<option>', {
                val: seq,
                text: stripName(seq),
            })
        )
    }
    $sequencesSelect.val(data.sequence);
    $sequencesInput.append($sequencesSelect);
    $sed.append($sequencesInput);
}

function makeScheduleElement(data) {
    let $el = $('<div>', {
        class: 'schedule-element'
    }).append(
        $('<button>', {
            class: 'material-icons',
                text: 'create' 
        }).on('click', (evt) => {
            let $sched = $(evt.target).parents('.schedule-element');
            $('#schedule-editor').dialog('open');
            makeScheduleEditor(data, $sched);
        })
    ).append(
        $('<p>', {
            class: 'time',
            text: `${data.hour.padStart(2)}:${data.minute.padStart(2)}`,
        })
    ).append(
        $('<p>', {
            class: 'days',
            text: data.days
        })
    );

    if (data.rooms) {
        let $roomIcons = $('<p>');
        for (let room in data.rooms) {
            let on = data.rooms[room];
            $roomIcons.append($('<span>', {
                class: 'room-icon material-icons',
                text: ROOM_ICON_MAP[room],
                css: {
                    color: on ? '#eee' : '#444',
                },
            }).data({
                room,
                status: on,
            }));
        }
        $el.append($roomIcons);
    }

    if (data.wemos) {
        let $wemoIcons = $('<p>');
        for (let wemo in data.wemos) {
            $wemoIcons.append(
                $('<span>').append(
                    $('<span>', {
                        class: 'wemo-icon material-icons',
                        text: WEMO_ICON_MAP[wemo],
                    }).data({
                        wemo,
                        command: data.wemos[wemo],
                    })
                ).append(
                    $('<span>', {
                        class: 'wemo-command',
                        text: `: ${data.wemos[wemo]}`,
                    })
                )
            );
        }
        $el.append($wemoIcons);
    }

    if (data.sequence) {
        $el.append($('<img>', {src: data.sequence}));
    }

    return $el;
}

function updateSlidersFromJson(data) {
    $('#input-range-red').val(data.r);
    $('#input-range-green').val(data.g);
    $('#input-range-blue').val(data.b);
    $('#input-range-white').val(data.w);
}

function setupNav() {
    let $buttons = $('nav>button');
    $buttons.each((i, el) => {
        $(el).on('click', (evt) => {
            $('nav>button').removeClass('active');
            $(evt.target).addClass('active');

            // delete `-button` to find id of article
            let id = evt.target.id.slice(0, evt.target.id.indexOf('-button'));
            $('article').css('display', 'none');
            $('#' + id).css('display', 'block');

            getLatestState();
        })
    });
}

function getLatestState() {
    // Populate all the current color values from the server side
    $.get({
        url: '/api/get-rgbw',
    }).then((data) => updateSlidersFromJson(data));

    // Populate the room data
    $.get({
        url: '/api/get-rooms',
    }).then((response) => {
        $('#living-room-check').prop('checked', response['living_room']);
        $('#office-check').prop('checked', response['office']);
        $('#bedroom-check').prop('checked', response['bedroom']);
    });
}

function getLatestSchedule() {
    $('#schedule').empty();

    // Populate the schedules
    $.get({
        url: '/api/get-schedule',
    }).then((response) => {
        for (const scheduleEntry of response) {
            $('#schedule')
                .append(makeScheduleElement(scheduleEntry))
        }
    });
}

function setup() {
    // Start a WebSocket for updating the sliders in realtime
    // let fullHref = window.location.href;
    // let serverNameRegex = new RegExp('http://(.+):.+');
    // let serverName = serverNameRegex.exec(fullHref)[1];

    // let ws = new WebSocket(`ws://${serverName}:9001`);
    // ws.onmessage = (evt) => {
    //     updateSlidersFromJson(JSON.parse(evt.data));
    // };

    let $schedEditor = $('<div>', {
        id: 'schedule-editor',
        title: 'Schedule Editor',
    }).dialog({
        height: $('body').height() * 0.9,
        width: $('body').width() * 0.9,
        draggable: false,
        resizable: false,
        autoOpen: false,
        buttons: [
            {
                text: 'Cancel',
                click: function() {
                    $(this).dialog('close');
                }
            },
            {
                text: 'Save',
                click: function() {
                    let $origSchedEl = $(this).children('#schedule-editor-content').data('scheduleElement');

                    let time = $(this).find('input[type="time"]').val();
                    let parsedTime = parseTime(time);

                    let days = [];
                    for (const day of DAYS_OF_WEEK) {
                        if ($(this).find(`input[type="checkbox"]#checkbox-${day}`).prop('checked')) {
                            days.push(day);
                        }
                    }

                    let rooms = {};
                    for (const room in ROOM_ICON_MAP) {
                        let checked = $(this).find(`input[type="checkbox"]#room-checkbox-${room}`).prop('checked');
                        rooms[room] = checked;
                    }

                    let wemos = {};
                    for (const wemo in WEMO_ICON_MAP) {
                        let option = $(this).find(`select#wemo-command-${wemo}`).val();
                        if (option) {
                            wemos[wemo] = option;
                        }
                    }

                    let sequence = $(this).find('select#sequence-selector').val();

                    updateSchedule({
                        hour: parsedTime.hour,
                        minute: parsedTime.minute,
                        days,
                        sequence,
                        rooms,
                        wemos,
                    }, $origSchedEl);
                    getLatestSchedule();
                    $(this).dialog('close');
                }
            },
        ]
    }).append($('<div>', {id: 'schedule-editor-content'}));

    setupNav();
    $('#favorites-button').trigger('click');

    // Set up WeMo commands
    $('.wemo-button').on('click', (evt) => {
        let button = evt.target.closest('button');
        let wemo = button.id;
        let command = 'toggle';
        let data = {};
        data[wemo] = command;
        $(button).prop('disabled', true);
        $.post({
            data: JSON.stringify(data),
            url: '/api/wemo',
            contentType: 'application/json; charset=utf-8',
        }).then(() => {
            $(button).prop('disabled', false);
        });
    })

    // Setup ajax POST requests to update RGBW leds
    $('.input-range-container input').on('change', function() {
        let data = {
            r: parseFloat($('#input-range-red').val()),
            g: parseFloat($('#input-range-green').val()),
            b: parseFloat($('#input-range-blue').val()),
            w: parseFloat($('#input-range-white').val()),
        };
        $.post({
            data: JSON.stringify(data),
            url: '/api/set-rgbw',
            contentType: 'application/json; charset=utf-8',
        });
    });

    // Setup ajax POST requests to update active rooms
    $('#rooms input').on('change', function() {
        let data = {
            living_room: $('#living-room-check').prop('checked'),
            office: $('#office-check').prop('checked'),
            bedroom: $('#bedroom-check').prop('checked'),
        };
        $.post({
            data: JSON.stringify(data),
            url: '/api/set-rooms',
            contentType: 'application/json; charset=utf-8',
        });
    });

    getLatestState();

    $.get({
        url: '/api/get-sequences',
    }).then((allSequences) => {
        for (let s of allSequences) {
            $('#favorite-list').append(
                $('<li>').append(
                    $('<div>', {
                        class: 'favorite-thumb',
                        data: {
                            sequencePath: s,
                        },
                    }).on('click', (evt) => {
                        let text =
                            $(evt.target)
                            .parent('.favorite-thumb')
                            .data('sequencePath');
                        loadSequence(text);
                    })
                        .append($('<img>', {attr: {src: s}}))
                        .append($('<p>', {text: stripName(s)}))
                )
            );
        }
    });

    getLatestSchedule();
}

window.onload = setup;