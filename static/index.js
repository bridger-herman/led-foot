function loadSequence(name) {
  let data = {'name': name};
  console.log(JSON.stringify(data, null, '\t'));
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
    $.ajax({
      type: 'POST',
      url: '/api/set-rgbw-' + $('#solid-rgbw-input').serialize(),
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
    $('#favorite-list')
      .append(
        $('<li/>')
          .append(
            $('<button/>', {
              class: 'sequence-loader',
              text: allSequences[sequence],
              on: {
                click: function(event) {
                  loadSequence(event.target.innerText);
                }
              }
            })
          )
      );
  }
}

window.onload = setup;
