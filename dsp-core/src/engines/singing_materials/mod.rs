/*
 * o_0 = f_0
 * o_1 = f_1
 * o_2 = f_2
 * o_3 = f_3
 *
 * r_1 = f_1 / f_0
 * r_2 = f_2 / f_0
 * r_3 = f_3 / f_0
 *
 * so we don't even need to map to audio frequencies, we just need to get the ratios
 *
 * - when f_0 is set, set the others by multipliying by their respective ratios
 * - default amplitudes from relative density from mp data
 *   - either parameter per oscillator for amplitude (simpler & more flexible)
 *   - or 1 mixing parameter that does some adjustment on all of them
 * - "temperature" control
 *   - figure out a bose-einstein-like filter?
 *   - or model it somehow: bp with f_c moving down then gradually increasing bandwidth, or something like that
 *   - or make temperature affect the relative amplitudes?
 * -
 */
