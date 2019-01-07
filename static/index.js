// Long polling function for checking to see if the current color has been
// updated
function updatePreviewFromServer(color) {
  $.ajax({
    type: 'GET',
    url: '/api/get-rgbw',
    success: function(info) {
      let rgb = `rgb(${info.r}, ${info.g}, ${info.b})`;
      let w = `rgb(${info.w}, ${info.w}, ${info.w})`;
      $('#color-preview').css("background-color", rgb);
      $('#white-preview').css("background-color", w);
      updatePreviewFromServer(info);
    },
    error: function(response) {
      console.log('Error!');
      console.log(response);
    },
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

  // Grab the current color from the server every time it's updated
  updatePreviewFromServer();
}

window.onload = setup;
