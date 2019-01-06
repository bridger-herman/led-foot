function setup() {
  $('.input-range-container input').on('change', function() {
    $.ajax({
      type: 'POST',
      url: '/api/set-rgbw-' + $('#solid-rgbw-input').serialize(),
    });
  });
}

window.onload = setup;
