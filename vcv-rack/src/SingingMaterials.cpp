#include "plugin.hpp"
#include "../../target/cxxbridge/dsp/src/lib.rs.h"

struct SingingMaterialsModule : Module
{
	enum ParamId
	{
		PITCH_PARAM,
		WAVEFORM_PARAM,
		PARAMS_LEN
	};
	enum InputId
	{
		PITCH_INPUT,
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

	::rust::Box<SingingMaterials> engine;

	SingingMaterialsModule() : engine(new_singing_materials())
	{
		config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
		configParam(PITCH_PARAM, -0.5f, 0.5f, 0.f, "");
		configParam(WAVEFORM_PARAM, 0.f, 3.f, 0.f, "");
		configInput(PITCH_INPUT, "");
		configOutput(OUT_OUTPUT, "");
	}

	static const std::vector<std::string> &materialLabels()
	{
		static const std::vector<std::string> labels = {"Diamond (athermal)", "Diamond (300K)", "Calcium Carbonate (athermal)", "Stishovite (athermal)"};
		return labels;
	}

	size_t material = 0;

	size_t getMaterial()
	{
		return material;
	}

	void setMaterial(size_t m)
	{
		material = m;
		engine->set_material(m);
	}

	// Save/load the material with the patch, since it isn't a param.
	json_t *dataToJson() override
	{
		json_t *rootJ = json_object();
		json_object_set_new(rootJ, "material", json_integer(material));
		return rootJ;
	}

	void dataFromJson(json_t *rootJ) override
	{
		json_t *materialJ = json_object_get(rootJ, "material");
		if (materialJ)
			setMaterial(json_integer_value(materialJ));
	}

	void process(const ProcessArgs &args) override
	{
		float pitchParam = params[PITCH_PARAM].getValue();
		float vOctInput = inputs[PITCH_INPUT].getVoltage();
		float pitch = pitchParam + vOctInput;
		engine->set_freq(dsp::FREQ_C4 * std::pow(2.f, pitch));

		float waveformParam = params[WAVEFORM_PARAM].getValue();
		engine->set_waveform(waveformParam);

		float sample = engine->next_sample();
		outputs[OUT_OUTPUT].setVoltage(sample * 5.f);
	}

	void onSampleRateChange(const SampleRateChangeEvent &e)
	{
		engine->set_sample_rate(e.sampleRate);
	}
};

struct MaterialChoice : LedDisplayChoice
{
	SingingMaterialsModule *module = nullptr;

	void step() override
	{
		const auto &labels = SingingMaterialsModule::materialLabels();
		size_t m = module ? module->getMaterial() : 0;
		text = m < labels.size() ? labels[m] : "";
		LedDisplayChoice::step();
	}

	void onAction(const ActionEvent &e) override
	{
		if (!module)
			return;
		Menu *menu = createMenu();
		menu->addChild(createMenuLabel("Material"));
		const auto &labels = SingingMaterialsModule::materialLabels();
		for (size_t i = 0; i < labels.size(); i++)
		{
			menu->addChild(createCheckMenuItem(labels[i], "", [=]()
											   { return module->getMaterial() == i; }, [=]()
											   { module->setMaterial(i); }));
		}
	}
};

struct SingingMaterialsWidget : ModuleWidget
{
	SingingMaterialsWidget(SingingMaterialsModule *module)
	{
		setModule(module);
		setPanel(createPanel(asset::plugin(pluginInstance, "res/SingingMaterials.svg")));

		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

		addParam(createParamCentered<RoundBlackKnob>(mm2px(Vec(10.721, 97.899)), module, SingingMaterialsModule::PITCH_PARAM));
		addParam(createParamCentered<RoundBlackSnapKnob>(mm2px(Vec(29.637, 97.802)), module, SingingMaterialsModule::WAVEFORM_PARAM));

		addInput(createInputCentered<PJ301MPort>(mm2px(Vec(10.721, 116.601)), module, SingingMaterialsModule::PITCH_INPUT));

		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(31.252, 116.601)), module, SingingMaterialsModule::OUT_OUTPUT));

		LedDisplay *display = createWidget<LedDisplay>(mm2px(Vec(2.0, 20.0)));
		display->box.size = mm2px(Vec(36.64, 8.0));
		addChild(display);

		MaterialChoice *materialChoice = createWidget<MaterialChoice>(Vec(0, 0));
		materialChoice->box.size = display->box.size;
		materialChoice->module = module;
		display->addChild(materialChoice);
	}
};

Model *modelSingingMaterials = createModel<SingingMaterialsModule, SingingMaterialsWidget>("SingingMaterials");