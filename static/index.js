// Relies on the
//
// ```
// <color/gradient>_<name>_<duration?>_<repeat?>.png
// ```
//
// convention
function stripName(sequencePath) {
    let firstUnderscore = sequencePath.indexOf('_');
    let secondUnderscore = sequencePath.indexOf('_', firstUnderscore + 1);
    let dotIndex = sequencePath.lastIndexOf('.');
    let lastIndex = secondUnderscore >= 0 ? secondUnderscore : dotIndex;
    return sequencePath.slice(firstUnderscore + 1, lastIndex);
}

function loadSequence(name) {
    let data = {'name': name};
    $.ajax({
        type: 'POST',
        url: '/api/set-sequence',
        data: JSON.stringify(data, null, '\t'),
        error: function(err) {
            console.log('error setting sequence');
            console.log(err);
        }
    });
}

function updateSchedule() {
    let schedule = [];
    let scheduleElements = $('.schedule-element');
    scheduleElements.each(function (index, element) {
        let hour = $(element).find('.hour-input').val();
        let minute = $(element).find('.minute-input').val();
        let days = $(element).find('.days-input').val();
        schedule.push({
            days: days.split(','),
            hour: hour,
            minute: minute,
            sequence: $(element).find('img').attr('src'),
        });
    });

    $.ajax({
        type: 'POST',
        url: '/api/set-schedule',
        data: JSON.stringify(schedule, null, '\t'),
        error: function(err) {
            console.log('error setting schedule');
            console.log(err);
        }
    });
}

function makeScheduleElement(data) {
    return $('<div/>', {
        class: 'schedule-element'
    }).append(
        $('<input/>', {
            class: 'hour-input',
            value: data.hour,
            type: 'number',
            min: 0, max: 23,
            on: {
                change: updateSchedule
            }
        })
    ).append(
        $('<label/>', {text: 'Hour'})
    ).append(
        $('<input/>', {
            class: 'minute-input',
            value: data.minute,
            type: 'number',
            min: 0,
            max: 59,
            on: {
                change: updateSchedule
            }
        })
    ).append(
        $('<label/>', {text: 'Minute'})
    ).append(
        $('<input/>', {
            value: data.days,
            class: 'days-input',
            on: {
                change: updateSchedule
            }
        })
    ).append(
        $('<label/>', {text: 'Days'})
    ).append(
        $('<img/>', {src: data.sequence})
    );
}

function updateSlidersFromJson(data) {
    $('#input-range-red').val(data.r);
    $('#input-range-green').val(data.g);
    $('#input-range-blue').val(data.b);
    $('#input-range-white').val(data.w);
}

function setup() {
    // Start a WebSocket for updating the sliders in realtime
    let fullHref = window.location.href;
    let serverNameRegex = new RegExp('http://(.+):.+');
    let serverName = serverNameRegex.exec(fullHref)[1];

    let ws = new WebSocket(`ws://${serverName}:9001`);
    ws.onmessage = (evt) => {
        updateSlidersFromJson(JSON.parse(evt.data));
    };

    // Setup ajax POST requests to update RGBW leds
    $('.input-range-container input').on('change', function() {
        let data = {
            r: parseFloat($('#input-range-red').val()),
            g: parseFloat($('#input-range-green').val()),
            b: parseFloat($('#input-range-blue').val()),
            w: parseFloat($('#input-range-white').val()),
        };
        $.ajax({
            type: 'POST',
            data: JSON.stringify(data, null, '\t'),
            url: '/api/set-rgbw',
        });
    });

    // Setup ajax POST requests to update active rooms
    $('#rooms input').on('change', function() {
        let data = {
            LivingRoom: $('#living-room-check').prop('checked'),
            Office: $('#office-check').prop('checked'),
            Bedroom: $('#bedroom-check').prop('checked'),
        };
        $.ajax({
            type: 'POST',
            data: JSON.stringify(data, null, '\t'),
            url: '/api/set-rooms',
        });
    });

    // Populate all the current color values from the server side
    $.ajax({
        type: 'GET',
        url: '/api/get-rgbw',
        success: updateSlidersFromJson,
    });


    // Populate the room data
    $.ajax({
        type: 'GET',
        url: '/api/get-rooms',
        success: (response) => {
            console.log(response);
        },
    });

    $.ajax({
        type: 'GET',
        url: '/api/get-sequences',
        success: (allSequences) => {
            for (let sequence in allSequences) {
                let s = allSequences[sequence];
                $('#favorite-list')
                    .append(
                        $('<li/>')
                        .append(
                            $('<div/>', {
                                class: 'favorite-thumb',
                                attr: {
                                    sequencePath: s,
                                },
                                on: {
                                    click: function(event) {
                                        let text =
                                            $(event.target)
                                            .parent('.favorite-thumb')
                                            .attr('sequencePath');
                                        loadSequence(text);
                                    }
                                }
                            })
                            .append($('<img/>', {attr: {src: s}}))
                            .append($('<p/>', {text: stripName(s)}))
                        )
                    );
            }
        }
    });

    // Populate the favorites list
    // Populate the schedules
    $.ajax({
        type: 'GET',
        url: '/api/get-schedule',
        success: function(response) {
            for (let i in response) {
                $('#schedule')
                    .append(makeScheduleElement(response[i]))
            }
        },
    });
}

window.onload = setup;

