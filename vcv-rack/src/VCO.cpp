#include "plugin.hpp"
#include "../../target/cxxbridge/dsp/src/lib.rs.h"

struct VCO : Module
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
		SAW_OUTPUT,
		SQUARE_OUTPUT,
		TRIANGLE_OUTPUT,
		OUTPUTS_LEN
	};
	enum LightId
	{
		LIGHTS_LEN
	};

	::rust::Box<Oscillator> osc;

	VCO() : osc(new_oscillator())
	{
		config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
		configParam(PITCH_PARAM, -0.5f, 0.5f, 0.f, "");
		configInput(PITCH_INPUT, "");
		configOutput(SINE_OUTPUT, "Sine");
		configOutput(SAW_OUTPUT, "Sawtooth");
		configOutput(SQUARE_OUTPUT, "Square");
		configOutput(TRIANGLE_OUTPUT, "Triangle");
	}

	void process(const ProcessArgs &args) override
	{
		float pitchParam = params[PITCH_PARAM].getValue();
		float vOctInput = inputs[PITCH_INPUT].getVoltage();
		float pitch = pitchParam + vOctInput;
		osc->set_freq(dsp::FREQ_C4 * std::pow(2.f, pitch));

		SamplesByWaveform s = osc->next_sample_by_waveform();
		outputs[SINE_OUTPUT].setVoltage(s.sine * 5.f);
		outputs[SAW_OUTPUT].setVoltage(s.sawtooth * 5.f);
		outputs[SQUARE_OUTPUT].setVoltage(s.square * 5.f);
		outputs[TRIANGLE_OUTPUT].setVoltage(s.triangle * 5.f);
	}
};

struct VCOWidget : ModuleWidget
{
	VCOWidget(VCO *module)
	{
		setModule(module);
		setPanel(createPanel(asset::plugin(pluginInstance, "res/VCO.svg")));

		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

		addParam(createParamCentered<RoundBlackKnob>(mm2px(Vec(20.248, 46.39)), module, VCO::PITCH_PARAM));

		addInput(createInputCentered<PJ301MPort>(mm2px(Vec(20.139, 71.548)), module, VCO::PITCH_INPUT));

		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(10.35, 96.749)), module, VCO::SINE_OUTPUT));
		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(31.252, 96.749)), module, VCO::SAW_OUTPUT));
		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(10.35, 116.977)), module, VCO::SQUARE_OUTPUT));
		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(31.252, 116.977)), module, VCO::TRIANGLE_OUTPUT));
	}
};

Model *modelVCO = createModel<VCO, VCOWidget>("VCO");