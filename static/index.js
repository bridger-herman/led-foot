function longPollColor() {

}

function setup() {
  // Setup ajax POST requests to update RGBW leds
  $('.input-range-container input').on('change', function() {
    $.ajax({
      type: 'POST',
      url: '/api/set-rgbw-' + $('#solid-rgbw-input').serialize(),
    });
  });

  // Grab the current color from the server every time it's updated
  $.ajax({
    type: 'GET',
    url: '/api/get-rgbw',
    success: function() {
      console.log('Success!');
    },
    error: function(response) {
      console.log('Error!');
      console.log(response);
    },
  });
}

window.onload = setup;
