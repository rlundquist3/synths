#include "plugin.hpp"
#include "../../target/cxxbridge/dsp/src/lib.rs.h"

struct EnvelopeGenerator : Module
{
	enum ParamId
	{
		ATTACK_PARAM,
		DECAY_PARAM,
		SUSTAIN_PARAM,
		RELEASE_PARAM,
		PARAMS_LEN
	};
	enum InputId
	{
		GATE_INPUT,
		INPUTS_LEN
	};
	enum OutputId
	{
		OUT_OUTPUT,
		OUTPUTS_LEN
	};
	enum LightId
	{
		LIGHTS_LEN
	};

	::rust::Box<Envelope> envelope;

	EnvelopeGenerator() : envelope(new_envelope())
	{
		config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
		configParam(ATTACK_PARAM, 0.f, 1.f, 0.f, "attack");
		configParam(DECAY_PARAM, 0.f, 1.f, 0.f, "decay");
		configParam(SUSTAIN_PARAM, 0.f, 1.f, 0.f, "sustain");
		configParam(RELEASE_PARAM, 0.f, 1.f, 0.f, "release");
		configInput(GATE_INPUT, "");
		configOutput(OUT_OUTPUT, "");
	}

	void process(const ProcessArgs &args) override
	{
		float attackParam = params[ATTACK_PARAM].getValue();
		float decayParam = params[DECAY_PARAM].getValue();
		float sustainParam = params[SUSTAIN_PARAM].getValue();
		float releaseParam = params[RELEASE_PARAM].getValue();

		float gateInput = inputs[GATE_INPUT].getVoltage();

		float output = 0.f;
		if (gateInput == 0.f)
		{
			if (!envelope->get_releasing() && !envelope->get_release_complete())
			{
				envelope->set_releasing();
			}
			if (!envelope->get_release_complete())
			{
				output = envelope->next();
			}
		}
		else
		{
			output = envelope->next();
		}

				outputs[OUT_OUTPUT].setVoltage(output);
	}
};

struct EnvelopeWidget : ModuleWidget
{
	EnvelopeWidget(EnvelopeGenerator *module)
	{
		setModule(module);
		setPanel(createPanel(asset::plugin(pluginInstance, "res/Envelope.svg")));

		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

		addParam(createParam<VCVSlider>(mm2px(Vec(3.628, 34.654)), module, EnvelopeGenerator::ATTACK_PARAM));
		addParam(createParam<VCVSlider>(mm2px(Vec(12.436, 34.654)), module, EnvelopeGenerator::DECAY_PARAM));
		addParam(createParam<VCVSlider>(mm2px(Vec(21.052, 34.654)), module, EnvelopeGenerator::SUSTAIN_PARAM));
		addParam(createParam<VCVSlider>(mm2px(Vec(30.508, 34.654)), module, EnvelopeGenerator::RELEASE_PARAM));

		addInput(createInputCentered<PJ301MPort>(mm2px(Vec(12.264, 107.938)), module, EnvelopeGenerator::GATE_INPUT));

		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(29.9, 107.937)), module, EnvelopeGenerator::OUT_OUTPUT));
	}
};

Model *modelEnvelope = createModel<EnvelopeGenerator, EnvelopeWidget>("Envelope");