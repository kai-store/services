// Implementation of the Ads SDK api.

(function _AdsSdk(global) {
    global.KaiAdsSdk = {
        _onready: null,
        _onexit: null,

        set onready(val) {
            this._onready = val;
        },

        set onexit(val) {
            this._onexit = val;
        },

        init() {
            console.log("AdsSdk init");
            global.onmessage = (event) => {
                if (event.data === "ad-frame-ready" && this._onready) {
                    let { content, target_frame } = this._onready();
                    let frame = document.getElementById(target_frame);
                    frame.contentWindow.postMessage({ content }, "*");
                } else if (event.data === "ad-frame-exit" && this._onexit) {
                    this._onexit();
                }
            }
        }
    }
}(window));