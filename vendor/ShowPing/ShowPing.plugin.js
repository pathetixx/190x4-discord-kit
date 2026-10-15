/**
 * @name ShowPing
 * @description Displays your live ping
 * @version 2.6.13
 * @author nicola02nb
 * @invite hFuY8DfDGK
 * @authorLink https://github.com/nicola02nb
 * @source https://github.com/nicola02nb/BetterDiscord-Stuff/tree/main/Plugins/ShowPing
*/
/*
 * Modified by 190x4 (pathetixx), 2026-10: Discord minifies CSS-module export
 * keys, so class modules are looked up by class values (byClassNames/getClasses)
 * instead of Filters.byKeys. Original: nicola02nb/BetterDiscord-Stuff, GPL-3.0.
 * This modified version is distributed under the same license (see LICENSE).
 */
const config = {
    changelog: [
        //{ title: "New Features", type: "added", items: [""] },
        //{ title: "Bug Fix", type: "fixed", items: [""] },
        //{ title: "Improvements", type: "improved", items: [""] },
        //{ title: "On-going", type: "progress", items: [""] }
    ],
    settings: [
        {
            type: "switch",
            id: "hideKrispButton",
            name: "Hide krisp button",
            note: "If enabled, hides krisp button near disconnect channel button",
            value: true
        },
    ]
};
function getSetting(key) {
    return config.settings.reduce((found, setting) => found ? found : (setting.id === key ? setting : setting.settings?.find(s => s.id === key)), undefined)
}

const { Webpack, React, Data, DOM, Patcher, UI } = BdApi;
const { Filters } = Webpack;

// Discord minifies CSS-module export keys (ei:"labelWrapper__06d62"), so modules are
// looked up by class values and remapped back to readable names.
function byClassNames(...names) {
    return m => {
        if (!m || typeof m !== "object") return false;
        const tokens = new Set();
        for (const v of Object.values(m)) {
            if (typeof v === "string") v.split(" ").forEach(c => tokens.add(c.replace(/_+[0-9a-z]{5,6}$/i, "")));
        }
        return names.every(n => tokens.has(n));
    };
}
function getClasses(...names) {
    const mod = Webpack.getModule(byClassNames(...names));
    if (!mod) return null;
    const out = {};
    for (const v of Object.values(mod)) {
        if (typeof v !== "string") continue;
        const name = v.split(" ")[0].replace(/_+[0-9a-z]{5,6}$/i, "");
        if (!(name in out)) out[name] = v;
    }
    return out;
}

const DiscordModules = Webpack.getModule(Filters.byKeys("subscribe", "dispatch"), { searchExports: true });
const RTCConnectionStore = Webpack.getModule(Filters.byStoreName("RTCConnectionStore"));
const textMdMedium = Webpack.getModule(m => m?.["text-md/medium"] && !m.avatar && !m.active && !m.h1);
const ConnectionStatus = Webpack.getModule(Filters.bySource("hasVideo", "hasConnectedChannel", "textVariant"));

const wrappedTypes = new WeakMap();

function PingElement() {
    const [ping, setPing] = React.useState(() => RTCConnectionStore?.getLastPing());
    React.useEffect(() => {
        const update = () => setPing(RTCConnectionStore?.getLastPing());
        DiscordModules?.subscribe("RTC_CONNECTION_PING", update);
        return () => DiscordModules?.unsubscribe("RTC_CONNECTION_PING", update);
    }, []);
    return React.createElement("div", {
        className: `${textMdMedium?.["text-md/medium"] ?? ""} pingDisplay`,
        children: " " + (ping !== undefined && ping !== null ? `${Math.round(ping)} ms` : "N/A")
    });
}

module.exports = class ShowPing {
    constructor(meta) {
        this.meta = meta;

        this.settings = new Proxy({}, {
            get: (_target, key) => {
                return Data.load(this.meta.name, key) ?? getSetting(key)?.value;
            },
            set: (_target, key, value) => {
                Data.save(this.meta.name, key, value);
                getSetting(key).value = value;
                return true;
            }
        });
    }

    initSettings(settings = config.settings) {
        settings.forEach(setting => {
            if (setting.settings) {
                this.initSettings(setting.settings);
            } else if (setting.id) {
                this.settings[setting.id] = Data.load(this.meta.name, setting.id) ?? setting.value;
            }
        });
    }

    getSettingsPanel() {
        return UI.buildSettingsPanel({
            settings: config.settings,
            onChange: (category, id, value) => {
                if (id === "hideKrispButton") {
                    this.displayKrispButton(!value);
                }
                this.settings[id] = value;
            },
        });
    }

    showChangelog() {
        const savedVersion = Data.load(this.meta.name, "version");
        if (savedVersion !== this.meta.version) {
            if (config.changelog.length > 0) {
                UI.showChangelogModal({
                    title: this.meta.name,
                    subtitle: this.meta.version,
                    changes: config.changelog
                });
            }
            Data.save(this.meta.name, "version", this.meta.version);
        }
    }

    wrapLabelType(originalType) {
        let wrapped = wrappedTypes.get(originalType);
        if (!wrapped) {
            wrapped = (props) => {
                const rendered = originalType(props);
                if (rendered?.props) rendered.props.children = [rendered.props.children, React.createElement(PingElement)];
                return rendered;
            };
            wrappedTypes.set(originalType, wrapped);
        }
        return wrapped;
    }

    start() {
        this.initSettings();
        this.showChangelog();

        const missing = [["Dispatcher", DiscordModules], ["RTCConnectionStore", RTCConnectionStore], ["ConnectionStatus", ConnectionStatus]]
            .filter(([, m]) => !m).map(([n]) => n);
        if (missing.length) {
            UI.showToast(`${this.meta.name}: Discord modules not found (${missing.join(", ")})`, { type: "error" });
            return;
        }

        const status = getClasses("labelWrapper", "rtcConnectionStatusConnected");
        const hoverRoll = getClasses("hoverRoll", "hovered", "default");
        this.voiceButtons = getClasses("voiceButtonsContainer");

        const css = [".pingDisplay {min-width: min-content;}"];
        if (status && hoverRoll) css.push(`.${status.rtcConnectionStatusConnected.split(" ")[0]} .${hoverRoll["default"].split(" ")[0]} {display: flex !important;}`);
        if (status) css.push(`.${status.labelWrapper} > button {width: 100%; display: inline;}`);
        if (hoverRoll) css.push(`.${hoverRoll["default"].split(" ")[0]} > div, .${hoverRoll["hovered"].split(" ")[0]} > div {text-overflow: ellipsis; overflow: hidden;}`);
        if (this.voiceButtons) css.push(`.${this.voiceButtons.voiceButtonsContainer} {margin-left: 2px !important;}`);
        DOM.addStyle(this.meta.name, css.join("\n"));

        const exportKey = Object.keys(ConnectionStatus).find(k => typeof ConnectionStatus[k] === "function" && ConnectionStatus[k].toString().includes("childrenAsSubtitle")) ?? "A";

        Patcher.after(this.meta.name, ConnectionStatus, exportKey, (_, args, ret) => {
            try {
                const popout = ret?.props?.children?.[0]?.props?.children;
                const renderFunc = popout?.props?.children;
                if (typeof renderFunc !== "function") return ret;

                popout.props.children = (e) => {
                    const element = renderFunc(e);
                    try {
                        const wrapper = element?.props?.children?.[1];
                        const wrapperChildren = wrapper?.props?.children;
                        if (!Array.isArray(wrapperChildren)) return element;
                        const label = wrapperChildren[0]?.props?.children;
                        if (typeof label?.type === "function") {
                            label.type = this.wrapLabelType(label.type);
                        } else if (wrapperChildren[0] == null) {
                            // Larger-target RTC panel experiment: Discord renders the status text elsewhere.
                            wrapperChildren[0] = React.createElement(PingElement);
                        }
                    } catch (err) {
                        console.error(`[${this.meta.name}] Error in inner patch`, err);
                    }
                    return element;
                };
            } catch (err) {
                console.error(`[${this.meta.name}] Error in patch ConnectionStatus`, err);
            }
            return ret;
        });

        if (this.settings.hideKrispButton) {
            this.displayKrispButton(false);
        }
    }

    stop() {
        Patcher.unpatchAll(this.meta.name);
        this.displayKrispButton(true);
        DOM.removeStyle(this.meta.name);
    }

    displayKrispButton(show) {
        if (show || !this.voiceButtons) {
            DOM.removeStyle(this.meta.name + "-hidekrispStyle");
        } else {
            DOM.addStyle(this.meta.name + "-hidekrispStyle", `.${this.voiceButtons.voiceButtonsContainer} > button[aria-label$="Krisp"] {display: none;}`);
        }
    }
};
