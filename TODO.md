# TODO

- [x] Get a `<canvas>` element
- [x] Draw the board pins
- [x] Draw a random path
  - [x] Simulate the choice of a random path
- [x] Draw all paths in transparency
- [x] Draw the new path in red
- [x] Draw the histogram of the probabilities
- [x] Make it possible to clear the canvas with a button/keyboard key
- [x] Make a slider to adjust the speed of the animation
- [x] Take the default value of the slider at the beginning
- [x] Allow doing multiple paths in one frame
  - [x] Separate the drawing code from the update code to separate FPS
        capping from animation speed
- [ ] Make the `update()` function not dependent on `requestAnimationFrame()`
      to go faster?
- [ ] Draw normal law on top of histogram
- [ ] Show total paths count
- [ ] Show probability for each histogram bar
- [ ] Animate drawing segment-by-segment
- [ ] Refactor
  - [ ] Make a function that returns the position of a pin from its "coordinates"

# BUGS

- [ ] When unfocusing the browser tab and coming back after some time, the
      animation is accelerated until probably the timer is back to under 1/FPS.
      Setting the frame timer back to zero instead of decrementing it by the
      elapsed time could probably fix this, although then frames would not be
      the same length, but I don't think this is a problem.
