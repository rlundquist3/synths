#include "plugin.hpp"
#include "../../target/cxxbridge/dsp/src/lib.rs.h"

struct SineOscillator : Module
{
	enum ParamId
	{
		PITCH_PARAM,
		PARAMS_LEN
	};
	enum InputId
	{
		PITCH_INPUT,
		INPUTS_LEN
	};
	enum OutputId
	{
		SINE_OUTPUT,
		OUTPUTS_LEN
	};
	enum LightId
	{
		BLINK_LIGHT,
		LIGHTS_LEN
	};

	::rust::Box<Oscillator> osc;

	SineOscillator() : osc(new_oscillator())
	{
		config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
		configParam(PITCH_PARAM, -0.5f, 0.5f, 0.f, "");
		configInput(PITCH_INPUT, "");
		configOutput(SINE_OUTPUT, "");
	}

	// void onSampleRateChange(const SampleRateChangeEvent &e) override
	// {
	// 	osc->set_sample_rate(e.sampleRate);
	// }

	void process(const ProcessArgs &args) override
	{
		float pitchParam = params[PITCH_PARAM].getValue();
		float vOctInput = inputs[PITCH_INPUT].getVoltage();
		float pitch = pitchParam + vOctInput;
		osc->set_freq(dsp::FREQ_C4 * std::pow(2.f, pitch));

		float nextSample = osc->next_sample();
		outputs[SINE_OUTPUT].setVoltage(nextSample * 5.0);
	}
};

struct SineOscillatorWidget : ModuleWidget
{
	SineOscillatorWidget(SineOscillator *module)
	{
		setModule(module);
		setPanel(createPanel(asset::plugin(pluginInstance, "res/SineOscillator.svg")));

		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

		addParam(createParamCentered<RoundBlackKnob>(mm2px(Vec(15.24, 46.063)), module, SineOscillator::PITCH_PARAM));

		addInput(createInputCentered<PJ301MPort>(mm2px(Vec(15.24, 77.478)), module, SineOscillator::PITCH_INPUT));

		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(15.24, 108.713)), module, SineOscillator::SINE_OUTPUT));

		addChild(createLightCentered<MediumLight<RedLight>>(mm2px(Vec(15.24, 25.81)), module, SineOscillator::BLINK_LIGHT));
	}
};

Model *modelSineOscillator = createModel<SineOscillator, SineOscillatorWidget>("SineOscillator");