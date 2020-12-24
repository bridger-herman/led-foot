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

function updateSchedule() {
    let schedule = [];
    let scheduleElements = $('.schedule-element');
    scheduleElements.each((_index, element) => {
        let time = parseTime($(element).find('.time').val());
        let days = $(element).find('.days-input').val();
        schedule.push({
            days: days.split(','),
            hour: time.hour,
            minute: time.minute,
            sequence: $(element).find('img').attr('src'),
        });
    });

    $.post({
        url: '/api/set-schedule',
        data: JSON.stringify(schedule),
        contentType: 'application/json; charset=utf-8',
    }).catch((err) => console.log(`Error setting schedule:\n${err}`));
}

function makeScheduleElement(data) {
    return $('<div>', {
        class: 'schedule-element'
    }).append(
        $('<input>', {
            type: 'text',
            class: 'time time-picker',
            value: `${data.hour}:${data.minute}`,
        }).on('change', updateSchedule)
    ).append(
        $('<input>', {
            value: data.days,
            class: 'days-input',
        }).on('change', updateSchedule)
    ).append(
        $('<label>', {text: 'Days'})
    ).append(
        $('<img>', {src: data.sequence})
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
    // let fullHref = window.location.href;
    // let serverNameRegex = new RegExp('http://(.+):.+');
    // let serverName = serverNameRegex.exec(fullHref)[1];

    // let ws = new WebSocket(`ws://${serverName}:9001`);
    // ws.onmessage = (evt) => {
    //     updateSlidersFromJson(JSON.parse(evt.data));
    // };

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

    // Populate all the current color values from the server side
    $.get({
        url: '/api/get-rgbw',
    }).then((data) => updateSlidersFromJson(JSON.parse(data)));

    // Populate the room data
    $.get({
        url: '/api/get-rooms',
    }).then((response) => {
        $('#living-room-check').prop('checked', response['LivingRoom']);
        $('#office-check').prop('checked', response['Office']);
        $('#bedroom-check').prop('checked', response['Bedroom']);
    });

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

    // Populate the schedules
    $.get({
        url: '/api/get-schedule',
    }).then((response) => {
        for (const scheduleEntry of response) {
            $('#schedule')
                .append(makeScheduleElement(scheduleEntry))
        }

        // Once all the schedules are done loading, populate the time pickers
        $('.time-picker').clockTimePicker({
            popupWidthOnDesktop: $(document).width(),
            fonts: {
                fontFamily: 'Arial',
                clockOuterCircleFontSize: 50,
                clockInnerCircleFontSize: 45,
                buttonFontSize: 55,
            },
        });
    });
}

window.onload = setup;