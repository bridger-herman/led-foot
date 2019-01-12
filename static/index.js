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

function setup() {
  // Setup ajax POST requests to update RGBW leds
  $('.input-range-container input').on('change', function() {
    let data = {
      r: $('#input-range-red').val(),
      g: $('#input-range-green').val(),
      b: $('#input-range-blue').val(),
      w: $('#input-range-white').val(),
    };
    $.ajax({
      type: 'POST',
      data: JSON.stringify(data, null, '\t'),
      url: '/api/set-rgbw',
    });
  });

  // Populate all the current color values from the server side
  // TODO this is here because Mustache won't recognize JSON for some reason
  $('#input-range-red').val(initialColor.r);
  $('#input-range-green').val(initialColor.g);
  $('#input-range-blue').val(initialColor.b);
  $('#input-range-white').val(initialColor.w);

  // Populate the favorites list
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

window.onload = setup;
